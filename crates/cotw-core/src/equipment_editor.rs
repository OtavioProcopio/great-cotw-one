//! Editor de equipamentos (Premium, spec `equipment-editor`): entregar e liberar itens, ajustar
//! quantidades e o dinheiro do perfil, sempre no mesmo tamanho de arquivo.
//!
//! Tudo o que muda mora em arrays de tamanho fixo do `thp_player_profile_adf` (posse, quantidades,
//! liberados e estatísticas de armas) e no dinheiro; nada é criado nem movido dentro do arquivo.
//! Os conjuntos de equipamento (loadouts) usam listas de tamanho variável e por isso ficam de fora.

use std::fs;
use std::path::Path;

use cotw_formats::adf::Adf;
use cotw_formats::envelope::SaveEnvelope;
use serde::Deserialize;

use crate::equipment::{self, Kind, hash_of};
use crate::transaction::{FileChange, ItemChange, Verifier};

const PROFILE: &str = "thp_player_profile_adf";

/// Maior dinheiro que o app grava (o jogo mostra até 8 dígitos sem problema; fica bem abaixo).
pub const MAX_CASH: u64 = 10_000_000;
/// Maior quantidade de munição que o app grava por item.
pub const MAX_AMMO: u32 = 999;
/// Maior quantidade de consumíveis, iscas e abrigos por item.
pub const MAX_STACK: u32 = 99;

/// Erros do editor de equipamentos.
#[derive(Debug, thiserror::Error)]
pub enum EquipmentError {
    /// Não foi possível ler o perfil.
    #[error("não foi possível ler o perfil do jogador")]
    Read,
    /// O perfil não tem a estrutura esperada.
    #[error("o perfil do jogador não pôde ser interpretado")]
    Parse,
    /// O item não está no catálogo do jogo.
    #[error("o item {0} não existe no catálogo")]
    UnknownItem(String),
    /// O item pedido precisa já estar na posse do jogador.
    #[error("o jogador não tem o item {0}")]
    NotOwned(String),
    /// Quantidade fora do que o app grava para esse tipo de item.
    #[error("quantidade fora do limite")]
    InvalidAmount,
    /// Não há vaga livre nas listas do perfil.
    #[error("não há vaga livre no perfil")]
    NoSlot,
    /// Dinheiro acima do que o app grava.
    #[error("dinheiro acima do limite")]
    CashTooHigh,
    /// Nenhuma mudança altera alguma coisa.
    #[error("nada a mudar")]
    NothingToChange,
    /// Falha ao montar o arquivo novo.
    #[error("não foi possível montar o arquivo novo")]
    Build,
}

impl EquipmentError {
    /// Código estável para a interface traduzir.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Parse => "equipmentParse",
            Self::UnknownItem(_) => "equipmentUnknownItem",
            Self::NotOwned(_) => "equipmentNotOwned",
            Self::InvalidAmount => "equipmentInvalidAmount",
            Self::NoSlot => "equipmentNoSlot",
            Self::CashTooHigh => "equipmentCashTooHigh",
            Self::NothingToChange => "nothingToChange",
            Self::Build => "equipmentBuild",
        }
    }
}

/// Uma mudança pedida pelo usuário.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum EquipmentChange {
    /// Entrega o item (posse + liberado); sem quantidade, usa 1 (ou a caixa, na munição).
    #[serde(rename_all = "camelCase")]
    Give {
        /// Id do jogo.
        id: String,
        /// Quantidade.
        amount: Option<u32>,
    },
    /// Muda a quantidade de um item que o jogador já tem.
    #[serde(rename_all = "camelCase")]
    SetAmount {
        /// Id do jogo.
        id: String,
        /// Nova quantidade.
        amount: u32,
    },
    /// Libera o item para compra na loja do jogo.
    #[serde(rename_all = "camelCase")]
    Unlock {
        /// Id do jogo.
        id: String,
    },
    /// Define o dinheiro.
    #[serde(rename_all = "camelCase")]
    SetCash {
        /// Dinheiro novo.
        cash: u64,
    },
}

/// Onde cada coisa mora dentro dos bytes do ADF.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Layout {
    cash: usize,
    unlocked: (usize, usize),
    owned: (usize, usize),
    amount: (usize, usize),
    weapons: WeaponLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WeaponLayout {
    at: usize,
    count: usize,
    stride: usize,
    hash: usize,
    bought: usize,
    furthest: usize,
}

fn member_of(adf: &Adf, type_hash: u32, name: &str) -> Option<(usize, usize, u32)> {
    let m = adf
        .type_def(type_hash)?
        .members
        .iter()
        .find(|m| m.name == name)?;
    Some((m.offset as usize, m.size as usize, m.type_hash))
}

fn layout_of(adf: &Adf) -> Option<Layout> {
    let instance = adf.instances().first()?;
    let base = instance.offset as usize;
    let root = instance.type_hash;
    // Arrays inline: o membro guarda o tipo do array, que sabe quantos elementos tem.
    let list = |name: &str| {
        let (offset, _, type_hash) = member_of(adf, root, name)?;
        Some((base + offset, adf.type_def(type_hash)?.element_len as usize))
    };
    let (weapons_offset, _, weapons_type) = member_of(adf, root, "WeaponData")?;
    let weapons_array = adf.type_def(weapons_type)?;
    let element = weapons_array.element_hash;
    let stride = adf.type_def(element)?.size as usize;
    let field = |name: &str| member_of(adf, element, name).map(|(offset, _, _)| offset);
    Some(Layout {
        cash: base + member_of(adf, root, "Cash")?.0,
        unlocked: list("EquipmentUnlocked")?,
        owned: list("EquipmentOwned")?,
        amount: list("EquipmentAmount")?,
        weapons: WeaponLayout {
            at: base + weapons_offset,
            count: weapons_array.element_len as usize,
            stride,
            hash: field("EquipmentHash")?,
            bought: field("PurchaseTimestamp")?,
            furthest: field("FurthestVitalOrganShotDistance")?,
        },
    })
}

fn word(data: &[u8], at: usize) -> Option<u32> {
    data.get(at..at + 4)
        .and_then(|b| b.try_into().ok())
        .map(u32::from_le_bytes)
}

fn put(data: &mut [u8], at: usize, value: u32) -> Result<(), EquipmentError> {
    data.get_mut(at..at + 4)
        .ok_or(EquipmentError::Parse)?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

/// Posição de `hash` numa lista de palavras, se estiver nela.
fn position(data: &[u8], list: (usize, usize), hash: u32) -> Option<usize> {
    (0..list.1).find(|i| word(data, list.0 + i * 4) == Some(hash))
}

/// Primeira vaga depois do último item da lista (a lista é compacta, com zeros no fim).
fn next_slot(data: &[u8], list: (usize, usize)) -> Option<usize> {
    let used = (0..list.1)
        .rev()
        .find(|i| word(data, list.0 + i * 4).is_some_and(|w| w != 0))
        .map_or(0, |i| i + 1);
    (used < list.1).then_some(used)
}

fn max_amount(kind: Kind) -> u32 {
    match kind {
        Kind::Ammo => MAX_AMMO,
        Kind::Consumable | Kind::LureCaller | Kind::Deployable => MAX_STACK,
        Kind::Weapon | Kind::Sight | Kind::Optic | Kind::Vehicle => 1,
    }
}

fn check_amount(kind: Kind, amount: u32) -> Result<u32, EquipmentError> {
    if amount == 0 || amount > max_amount(kind) {
        return Err(EquipmentError::InvalidAmount);
    }
    Ok(amount)
}

fn item_of(id: &str) -> Result<(String, equipment::CatalogItem), EquipmentError> {
    equipment::catalog()
        .get(id)
        .map(|item| (id.to_owned(), item.clone()))
        .ok_or_else(|| EquipmentError::UnknownItem(id.to_owned()))
}

fn name_of(id: &str, item: &equipment::CatalogItem) -> String {
    if item.name.is_empty() {
        id.trim_start_matches("equipment_").replace('_', " ")
    } else {
        item.name.clone()
    }
}

fn now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u32::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}

/// Aplica as mudanças aos bytes do ADF (mesmo tamanho) e devolve o que mudou, para a prévia.
fn apply(
    data: &mut [u8],
    layout: &Layout,
    changes: &[EquipmentChange],
    stamp: u32,
) -> Result<Vec<ItemChange>, EquipmentError> {
    let mut items = Vec::new();
    for change in changes {
        match change {
            EquipmentChange::SetCash { cash } => {
                if *cash > MAX_CASH {
                    return Err(EquipmentError::CashTooHigh);
                }
                let before = word(data, layout.cash).ok_or(EquipmentError::Parse)?;
                let after = u32::try_from(*cash).map_err(|_| EquipmentError::CashTooHigh)?;
                if before != after {
                    put(data, layout.cash, after)?;
                    items.push(ItemChange {
                        label: "Dinheiro".into(),
                        before: before.to_string(),
                        after: after.to_string(),
                    });
                }
            }
            EquipmentChange::Unlock { id } => {
                let (id, item) = item_of(id)?;
                if unlock(data, layout, hash_of(&id))? {
                    items.push(ItemChange {
                        label: name_of(&id, &item),
                        before: "bloqueado".into(),
                        after: "liberado para compra".into(),
                    });
                }
            }
            EquipmentChange::SetAmount { id, amount } => {
                let (id, item) = item_of(id)?;
                let amount = check_amount(item.kind, *amount)?;
                let slot = position(data, layout.owned, hash_of(&id))
                    .ok_or_else(|| EquipmentError::NotOwned(id.clone()))?;
                let at = layout.amount.0 + slot * 4;
                let before = word(data, at).ok_or(EquipmentError::Parse)?;
                if before != amount {
                    put(data, at, amount)?;
                    items.push(ItemChange {
                        label: name_of(&id, &item),
                        before: before.to_string(),
                        after: amount.to_string(),
                    });
                }
            }
            EquipmentChange::Give { id, amount } => {
                let (id, item) = item_of(id)?;
                let hash = hash_of(&id);
                let wanted = match amount {
                    Some(n) => check_amount(item.kind, *n)?,
                    None => item.box_size.unwrap_or(1),
                };
                if let Some(slot) = position(data, layout.owned, hash) {
                    // Já tem: só muda a quantidade se o usuário pediu uma.
                    if amount.is_some() {
                        let at = layout.amount.0 + slot * 4;
                        let before = word(data, at).ok_or(EquipmentError::Parse)?;
                        if before != wanted {
                            put(data, at, wanted)?;
                            items.push(ItemChange {
                                label: name_of(&id, &item),
                                before: before.to_string(),
                                after: wanted.to_string(),
                            });
                        }
                    }
                    continue;
                }
                let slot = next_slot(data, layout.owned).ok_or(EquipmentError::NoSlot)?;
                if next_slot(data, layout.amount).is_none() {
                    return Err(EquipmentError::NoSlot);
                }
                unlock(data, layout, hash)?;
                put(data, layout.owned.0 + slot * 4, hash)?;
                put(data, layout.amount.0 + slot * 4, wanted)?;
                if item.kind == Kind::Weapon {
                    add_weapon_entry(data, &layout.weapons, hash, stamp)?;
                }
                items.push(ItemChange {
                    label: name_of(&id, &item),
                    before: "não tem".into(),
                    after: if item.kind == Kind::Weapon || wanted == 1 {
                        "entregue".into()
                    } else {
                        format!("{wanted} unidades")
                    },
                });
            }
        }
    }
    Ok(items)
}

/// Põe o hash na lista de itens liberados; `false` se já estava.
fn unlock(data: &mut [u8], layout: &Layout, hash: u32) -> Result<bool, EquipmentError> {
    if position(data, layout.unlocked, hash).is_some() {
        return Ok(false);
    }
    let slot = next_slot(data, layout.unlocked).ok_or(EquipmentError::NoSlot)?;
    put(data, layout.unlocked.0 + slot * 4, hash)?;
    Ok(true)
}

/// Cria o registro de estatísticas da arma (zerado, com a data da entrega) numa vaga em branco.
fn add_weapon_entry(
    data: &mut [u8],
    weapons: &WeaponLayout,
    hash: u32,
    stamp: u32,
) -> Result<(), EquipmentError> {
    let mut blank = None;
    for i in 0..weapons.count {
        let at = weapons.at + i * weapons.stride;
        match word(data, at + weapons.hash) {
            Some(h) if h == hash => return Ok(()),
            Some(0) if blank.is_none() => blank = Some(at),
            Some(_) => {}
            None => return Err(EquipmentError::Parse),
        }
    }
    let at = blank.ok_or(EquipmentError::NoSlot)?;
    data.get_mut(at..at + weapons.stride)
        .ok_or(EquipmentError::Parse)?
        .fill(0);
    put(data, at + weapons.hash, hash)?;
    put(data, at + weapons.bought, stamp)?;
    put(data, at + weapons.furthest, (-1.0_f32).to_bits())?;
    Ok(())
}

/// Arquivo do perfil pronto para a gravação segura.
pub struct EditPlan {
    /// Arquivo para a transação.
    pub file: FileChange,
    /// Quantas mudanças de fato alteram alguma coisa.
    pub changes: usize,
}

/// Confere depois de gravar: o arquivo relido tem o mesmo tamanho, os bytes esperados e a cadeia
/// de ponteiros sem quebras.
fn verifier(expected: Vec<u8>) -> Verifier {
    Box::new(move |bytes| {
        let again = SaveEnvelope::parse(bytes).map_err(|e| e.to_string())?;
        if again.adf() != expected.as_slice() {
            return Err("o perfil gravado difere do pedido".into());
        }
        let parsed = Adf::parse(again.adf().to_vec()).map_err(|e| e.to_string())?;
        for instance in parsed.instances() {
            if !parsed
                .chain_issues(instance)
                .map_err(|e| e.to_string())?
                .is_empty()
            {
                return Err("a cadeia de ponteiros do arquivo ficou quebrada".into());
            }
        }
        Ok(())
    })
}

/// Monta o perfil novo com as mudanças, conferindo tudo antes de gravar.
///
/// # Errors
///
/// [`EquipmentError`] se o perfil não puder ser lido, um item não existir ou não estiver com o
/// jogador, uma quantidade ou o dinheiro passarem do limite, não houver vaga ou nada mudar.
pub fn build_edit(
    save_dir: &Path,
    changes: &[EquipmentChange],
) -> Result<EditPlan, EquipmentError> {
    let raw = fs::read(save_dir.join(PROFILE)).map_err(|_| EquipmentError::Read)?;
    let mut envelope = SaveEnvelope::parse(&raw).map_err(|_| EquipmentError::Parse)?;
    let mut adf = envelope.adf().to_vec();
    let parsed = Adf::parse(adf.clone()).map_err(|_| EquipmentError::Parse)?;
    let layout = layout_of(&parsed).ok_or(EquipmentError::Parse)?;
    let items = apply(&mut adf, &layout, changes, now())?;
    if items.is_empty() {
        return Err(EquipmentError::NothingToChange);
    }
    if adf.len() != envelope.adf().len() {
        return Err(EquipmentError::Build);
    }
    let verify = verifier(adf.clone());
    envelope.set_adf(adf);
    let new_content = envelope.to_bytes().map_err(|_| EquipmentError::Build)?;
    let count = items.len();
    Ok(EditPlan {
        file: FileChange {
            name: PROFILE.to_owned(),
            new_content,
            items,
            verify: Some(verify),
        },
        changes: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: usize = 8;

    /// Perfil de mentira: dinheiro, três listas de 8 palavras e 4 registros de arma de 16 bytes.
    fn layout() -> Layout {
        Layout {
            cash: 0,
            unlocked: (4, LIST),
            owned: (4 + LIST * 4, LIST),
            amount: (4 + LIST * 8, LIST),
            weapons: WeaponLayout {
                at: 4 + LIST * 12,
                count: 4,
                stride: 16,
                hash: 0,
                bought: 4,
                furthest: 12,
            },
        }
    }

    fn blank() -> Vec<u8> {
        let mut data = vec![0_u8; 4 + LIST * 12 + 4 * 16];
        put(&mut data, 0, 1000).unwrap();
        data
    }

    fn give(id: &str, amount: Option<u32>) -> EquipmentChange {
        EquipmentChange::Give {
            id: id.into(),
            amount,
        }
    }

    const RIFLE: &str = "equipment_weapon_ba_rifle_270_01";
    const AMMO: &str = "equipment_ammo_270_sp_01";

    fn owned(data: &[u8], id: &str) -> Option<u32> {
        let l = layout();
        position(data, l.owned, hash_of(id)).and_then(|s| word(data, l.amount.0 + s * 4))
    }

    #[test]
    fn giving_a_weapon_owns_unlocks_and_creates_its_stats_without_resizing() {
        let mut data = blank();
        let size = data.len();
        let items = apply(&mut data, &layout(), &[give(RIFLE, None)], 1_700_000_000).unwrap();
        assert_eq!(data.len(), size);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].after, "entregue");
        let l = layout();
        assert_eq!(owned(&data, RIFLE), Some(1));
        assert!(position(&data, l.unlocked, hash_of(RIFLE)).is_some());
        let entry = l.weapons.at;
        assert_eq!(word(&data, entry), Some(hash_of(RIFLE)));
        assert_eq!(word(&data, entry + 4), Some(1_700_000_000));
        assert_eq!(word(&data, entry + 12), Some((-1.0_f32).to_bits()));
        assert_eq!(word(&data, 0), Some(1000));
    }

    #[test]
    fn ammo_defaults_to_one_box_and_accepts_an_amount_up_to_the_limit() {
        let mut data = blank();
        apply(&mut data, &layout(), &[give(AMMO, None)], 0).unwrap();
        assert_eq!(owned(&data, AMMO), Some(10));
        let items = apply(&mut data, &layout(), &[give(AMMO, Some(200))], 0).unwrap();
        assert_eq!(owned(&data, AMMO), Some(200));
        assert_eq!(
            (items[0].before.as_str(), items[0].after.as_str()),
            ("10", "200")
        );
        assert!(matches!(
            apply(&mut data, &layout(), &[give(AMMO, Some(1000))], 0),
            Err(EquipmentError::InvalidAmount)
        ));
        assert!(matches!(
            apply(&mut data, &layout(), &[give(AMMO, Some(0))], 0),
            Err(EquipmentError::InvalidAmount)
        ));
    }

    #[test]
    fn giving_what_is_already_owned_changes_nothing_without_an_amount() {
        let mut data = blank();
        apply(&mut data, &layout(), &[give(RIFLE, None)], 5).unwrap();
        let before = data.clone();
        let items = apply(&mut data, &layout(), &[give(RIFLE, None)], 9).unwrap();
        assert_eq!(items, Vec::new());
        assert_eq!(data, before);
    }

    #[test]
    fn a_weapon_cannot_have_more_than_one_unit() {
        let mut data = blank();
        assert!(matches!(
            apply(&mut data, &layout(), &[give(RIFLE, Some(2))], 0),
            Err(EquipmentError::InvalidAmount)
        ));
    }

    #[test]
    fn set_amount_needs_an_owned_item_and_respects_the_limit() {
        let mut data = blank();
        let set = |id: &str, amount| EquipmentChange::SetAmount {
            id: id.into(),
            amount,
        };
        assert!(matches!(
            apply(&mut data, &layout(), &[set(AMMO, 50)], 0),
            Err(EquipmentError::NotOwned(_))
        ));
        apply(&mut data, &layout(), &[give(AMMO, Some(20))], 0).unwrap();
        let items = apply(&mut data, &layout(), &[set(AMMO, 500)], 0).unwrap();
        assert_eq!(owned(&data, AMMO), Some(500));
        assert_eq!(items.len(), 1);
        assert_eq!(
            apply(&mut data, &layout(), &[set(AMMO, 500)], 0).unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn unlock_adds_the_item_to_the_shop_list_once() {
        let mut data = blank();
        let unlock = EquipmentChange::Unlock { id: RIFLE.into() };
        let items = apply(&mut data, &layout(), std::slice::from_ref(&unlock), 0).unwrap();
        assert_eq!(items.len(), 1);
        assert!(position(&data, layout().unlocked, hash_of(RIFLE)).is_some());
        assert_eq!(owned(&data, RIFLE), None);
        assert_eq!(
            apply(&mut data, &layout(), &[unlock], 0).unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn cash_is_set_within_the_limit() {
        let mut data = blank();
        let cash = |cash| EquipmentChange::SetCash { cash };
        let items = apply(&mut data, &layout(), &[cash(50_000)], 0).unwrap();
        assert_eq!(word(&data, 0), Some(50_000));
        assert_eq!(
            (items[0].before.as_str(), items[0].after.as_str()),
            ("1000", "50000")
        );
        assert!(matches!(
            apply(&mut data, &layout(), &[cash(MAX_CASH + 1)], 0),
            Err(EquipmentError::CashTooHigh)
        ));
        assert_eq!(
            apply(&mut data, &layout(), &[cash(50_000)], 0).unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn unknown_items_are_refused() {
        let mut data = blank();
        assert!(matches!(
            apply(
                &mut data,
                &layout(),
                &[give("equipment_nao_existe", None)],
                0
            ),
            Err(EquipmentError::UnknownItem(_))
        ));
        let untouched = blank();
        assert_eq!(data, untouched);
    }

    #[test]
    fn a_full_list_refuses_instead_of_overwriting() {
        let mut data = blank();
        let l = layout();
        for i in 0..LIST {
            put(
                &mut data,
                l.owned.0 + i * 4,
                0xAAAA_0000 + u32::try_from(i).unwrap(),
            )
            .unwrap();
            put(&mut data, l.amount.0 + i * 4, 1).unwrap();
        }
        let before = data.clone();
        assert!(matches!(
            apply(&mut data, &l, &[give(AMMO, None)], 0),
            Err(EquipmentError::NoSlot)
        ));
        assert_eq!(data.len(), before.len());
    }

    #[test]
    fn items_go_after_the_last_used_slot_and_leave_holes_alone() {
        let mut data = blank();
        let l = layout();
        put(&mut data, l.owned.0, 0x1111).unwrap();
        // Buraco no meio (item removido pelo jogo) e outro item depois dele.
        put(&mut data, l.owned.0 + 8, 0x2222).unwrap();
        apply(&mut data, &l, &[give(AMMO, None)], 0).unwrap();
        assert_eq!(word(&data, l.owned.0 + 4), Some(0));
        assert_eq!(word(&data, l.owned.0 + 12), Some(hash_of(AMMO)));
    }

    #[test]
    fn weapon_stats_need_a_blank_record() {
        let mut data = blank();
        let l = layout();
        for i in 0..4 {
            put(
                &mut data,
                l.weapons.at + i * 16,
                0xBBBB_0000 + u32::try_from(i).unwrap(),
            )
            .unwrap();
        }
        assert!(matches!(
            apply(&mut data, &l, &[give(RIFLE, None)], 0),
            Err(EquipmentError::NoSlot)
        ));
    }

    #[test]
    fn requests_arrive_as_tagged_json() {
        let parsed: Vec<EquipmentChange> = serde_json::from_str(
            r#"[{"action":"give","id":"a","amount":3},{"action":"setCash","cash":5},{"action":"unlock","id":"b"},{"action":"setAmount","id":"c","amount":2}]"#,
        )
        .unwrap();
        assert_eq!(parsed[0], give("a", Some(3)));
        assert_eq!(parsed[1], EquipmentChange::SetCash { cash: 5 });
        assert_eq!(parsed.len(), 4);
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(EquipmentError::NoSlot.code(), "equipmentNoSlot");
        assert_eq!(EquipmentError::NothingToChange.code(), "nothingToChange");
        assert_eq!(EquipmentError::Read.code(), "read");
    }
}
