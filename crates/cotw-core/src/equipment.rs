//! Equipamentos do jogador (spec `equipment-page`): o catálogo do jogo embutido como fatos
//! (`data/equipment.json`, gerado por `tools/gen-equipment`) cruzado com o que o perfil guarda de posse,
//! quantidades, estatísticas por arma e loadouts.
//!
//! No save os itens aparecem pelo `hashlittle` do id (`equipment_weapon_...`); aqui o hash vira o item
//! do catálogo. Itens que o catálogo não traz (peles, suportes de troféu) são ignorados.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

use base64::Engine as _;
use cotw_formats::adf::Value;
use cotw_formats::archive::GameArchives;
use cotw_formats::hash::hashlittle;
use cotw_formats::texture::{self, Image};
use serde::{Deserialize, Serialize};

use crate::career::CareerError;
use crate::language::Language;
use crate::mapview::read_adf;
use crate::species;
use crate::species_stats;

const DATA: &str = include_str!("../data/equipment.json");
const PROFILE: &str = "thp_player_profile_adf";

/// Categoria de um item do catálogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Arma.
    Weapon,
    /// Munição.
    Ammo,
    /// Mira.
    Sight,
    /// Binóculos, telêmetro e visão noturna.
    Optic,
    /// Iscas, chamados e iscas de odor.
    LureCaller,
    /// Barracas, esconderijos, tripés e comedouros.
    Deployable,
    /// Consumíveis, roupas e mochilas.
    Consumable,
    /// Veículos.
    Vehicle,
}

/// Classe de arma (a mesma ordem de `WeaponLevel` no perfil).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponClass {
    /// Rifles.
    Rifle,
    /// Pistolas e revólveres.
    Handgun,
    /// Espingardas.
    Shotgun,
    /// Arcos e bestas.
    Archery,
}

/// Onde o jogo libera um item: nível do caçador ou de uma classe de arma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unlock {
    /// De onde vem o nível: `character`, `rifle`, `handgun`, `shotgun` ou `archery`.
    pub track: UnlockTrack,
    /// Nível necessário.
    pub level: u32,
    /// Experiência ou pontos da classe de arma que o nível pede.
    pub need: u32,
}

/// De onde vem o nível que libera um item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnlockTrack {
    /// Nível do caçador.
    Character,
    /// Nível dos rifles.
    Rifle,
    /// Nível das pistolas.
    Handgun,
    /// Nível das espingardas.
    Shotgun,
    /// Nível do arco.
    Archery,
}

/// Atributos que o jogo mostra na loja para um item (todos opcionais: cada categoria tem os seus).
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    /// Precisão da arma (0 a 100).
    #[serde(default)]
    pub accuracy: Option<f64>,
    /// Recuo da arma (0 a 100, menor é melhor).
    #[serde(default)]
    pub recoil: Option<f64>,
    /// Velocidade de recarga da arma.
    #[serde(default)]
    pub reload: Option<f64>,
    /// Alcance efetivo da munição, em metros.
    #[serde(default)]
    pub range: Option<f64>,
    /// Penetração da munição.
    #[serde(default)]
    pub penetration: Option<f64>,
    /// Expansão da munição.
    #[serde(default)]
    pub expansion: Option<f64>,
    /// Classes de caça (1 a 9) para as quais a munição é recomendada.
    #[serde(default)]
    pub classes: Vec<u32>,
    /// Aumento da mira ou do binóculo.
    #[serde(default)]
    pub zoom: Option<f64>,
    /// Espécies que a isca ou o chamado atrai (nomes do jogo sem sublinhado, como `graywolf`).
    #[serde(default)]
    pub attracts: Vec<String>,
    /// Alcance do chamado, em metros.
    #[serde(default)]
    pub lure_range: Option<f64>,
    /// Duração do efeito, em segundos.
    #[serde(default)]
    pub lure_duration: Option<f64>,
    /// Chance de atrair (0 a 100).
    #[serde(default)]
    pub lure_chance: Option<f64>,
}

/// Um item do catálogo do jogo.
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogItem {
    /// Atributos da loja.
    #[serde(flatten)]
    pub attributes: Attributes,
    /// Onde o jogo libera o item.
    #[serde(default)]
    pub unlock: Option<Unlock>,
    /// Categoria.
    pub kind: Kind,
    /// Nome em inglês do jogo (pode vir vazio).
    #[serde(default)]
    pub name: String,
    /// Preço em dinheiro do jogo (0: não se compra com dinheiro).
    #[serde(default)]
    pub price: u32,
    /// Peso na mochila.
    #[serde(default)]
    pub weight: Option<f64>,
    /// Classe da arma.
    #[serde(default)]
    pub class: Option<WeaponClass>,
    /// Quantas unidades vêm numa caixa de munição.
    #[serde(default, rename = "box")]
    pub box_size: Option<u32>,
    /// Caminho do ícone nos arquivos do jogo.
    #[serde(default)]
    pub icon: Option<String>,
    /// Munições que servem na arma.
    #[serde(default)]
    pub ammo: Vec<String>,
    /// Miras que servem na arma.
    #[serde(default)]
    pub sights: Vec<String>,
}

#[derive(Deserialize)]
struct File {
    items: BTreeMap<String, CatalogItem>,
}

/// O catálogo embutido, com o índice por hash do id.
pub struct Catalog {
    items: BTreeMap<String, CatalogItem>,
    by_hash: HashMap<u32, String>,
}

impl Catalog {
    /// Itens por id, em ordem alfabética.
    pub fn items(&self) -> impl Iterator<Item = (&str, &CatalogItem)> {
        self.items.iter().map(|(id, item)| (id.as_str(), item))
    }

    /// Item pelo id do jogo (`equipment_weapon_ba_rifle_270_01`).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&CatalogItem> {
        self.items.get(id)
    }

    /// Id do item que tem este hash no save.
    #[must_use]
    pub fn id_of(&self, hash: u32) -> Option<&str> {
        self.by_hash.get(&hash).map(String::as_str)
    }
}

/// O catálogo de equipamentos.
#[must_use]
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let items = serde_json::from_str::<File>(DATA)
            .map(|f| f.items)
            .unwrap_or_default();
        let by_hash = items
            .keys()
            .map(|id| (hashlittle(id.as_bytes(), 0), id.clone()))
            .collect();
        Catalog { items, by_hash }
    })
}

/// Hash que o save usa para o id do item.
#[must_use]
pub fn hash_of(id: &str) -> u32 {
    hashlittle(id.as_bytes(), 0)
}

/// Estatísticas de uma arma (`WeaponData` do perfil).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaponStats {
    /// Tiros disparados.
    pub shots_fired: u64,
    /// Tiros que erraram.
    pub shots_missed: u64,
    /// Precisão em porcentagem (`None` sem tiros).
    pub accuracy: Option<f64>,
    /// Abates.
    pub kills: u64,
    /// Abates com um tiro só.
    pub one_shot_kills: u64,
    /// Maior pontuação de um abate.
    pub record_score: f64,
    /// Espécie do recorde (nome no idioma do app), se conhecida.
    pub record_species: Option<String>,
    /// Maior distância de um tiro em órgão vital, em metros.
    pub furthest_shot: Option<f64>,
    /// Data da compra (segundos desde 1970), se o jogo guardou.
    pub purchased_at: Option<u64>,
}

/// Um item do catálogo com o que o jogador tem dele.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    /// Id do jogo.
    pub id: String,
    /// Categoria.
    pub kind: Kind,
    /// Nome em inglês (se o jogo não traz, o id sem o prefixo).
    pub name: String,
    /// Preço em dinheiro do jogo.
    pub price: u32,
    /// Peso na mochila.
    pub weight: Option<f64>,
    /// Classe da arma.
    pub class: Option<WeaponClass>,
    /// Unidades por caixa de munição.
    pub box_size: Option<u32>,
    /// Caminho do ícone nos arquivos do jogo.
    pub icon: Option<String>,
    /// O jogador tem o item.
    pub owned: bool,
    /// O item está liberado para compra.
    pub unlocked: bool,
    /// Está na mochila.
    pub in_backpack: bool,
    /// Quantidade (munição e consumíveis; 1 para os demais itens que o jogador tem).
    pub amount: u32,
    /// Munições compatíveis (só armas).
    pub ammo: Vec<String>,
    /// Miras compatíveis (só armas).
    pub sights: Vec<String>,
    /// Estatísticas, para armas com histórico.
    pub stats: Option<WeaponStats>,
    /// Atributos da loja (precisão, recuo, alcance, classes de caça...).
    pub attributes: Attributes,
    /// Onde o jogo libera o item.
    pub unlock: Option<Unlock>,
}

/// Um conjunto de equipamento salvo.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadoutView {
    /// Posição na lista (começa em 0).
    pub index: usize,
    /// Nome dado pelo jogador.
    pub name: String,
    /// Liberado no jogo.
    pub unlocked: bool,
    /// É o conjunto em uso.
    pub active: bool,
    /// Itens do conjunto (ids do jogo).
    pub items: Vec<String>,
    /// Itens que o conjunto pede e o jogador não tem (ids do jogo).
    pub missing: Vec<String>,
}

/// Tudo o que a página de equipamentos mostra.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    /// Dinheiro.
    pub cash: u64,
    /// Nível do caçador.
    pub level: u64,
    /// Nível de cada classe de arma (rifle, pistola, espingarda, arco).
    pub weapon_levels: [u64; 4],
    /// Itens do catálogo, em ordem de categoria e nome.
    pub items: Vec<ItemView>,
    /// Conjuntos de equipamento.
    pub loadouts: Vec<LoadoutView>,
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn list(value: Option<&Value>) -> &[Value] {
    match value {
        Some(Value::Array(items)) => items,
        _ => &[],
    }
}

fn unsigned(value: Option<&Value>) -> u64 {
    match value {
        Some(Value::Unsigned(n)) => *n,
        Some(Value::Signed(n)) => u64::try_from(*n).unwrap_or(0),
        _ => 0,
    }
}

fn float(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Float(n)) => *n,
        _ => 0.0,
    }
}

fn text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    }
}

/// Hashes não nulos de um array do perfil.
fn hashes(value: Option<&Value>) -> Vec<u32> {
    list(value)
        .iter()
        .filter_map(|v| u32::try_from(unsigned(Some(v))).ok())
        .filter(|h| *h != 0)
        .collect()
}

/// Nome de espécie (`id` do jogo) a partir do hash que o perfil guarda.
fn species_of(hash: u64) -> Option<String> {
    static BY_HASH: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    let table = BY_HASH.get_or_init(|| {
        species_stats::all()
            .map(|(id, _)| (hashlittle(id.as_bytes(), 0), id))
            .collect()
    });
    u32::try_from(hash)
        .ok()
        .and_then(|h| table.get(&h))
        .map(|id| (*id).to_owned())
}

fn weapon_stats(entry: &Value, language: Language) -> WeaponStats {
    let fired = unsigned(member(entry, "ShotsFired"));
    let missed = unsigned(member(entry, "ShotsMissed"));
    let furthest = float(member(entry, "FurthestVitalOrganShotDistance"));
    let bought = unsigned(member(entry, "PurchaseTimestamp"));
    #[allow(
        clippy::cast_precision_loss,
        reason = "contagem de tiros, muito abaixo do limite de precisão"
    )]
    let accuracy = (fired > 0).then(|| {
        let hits = fired.saturating_sub(missed);
        hits as f64 * 100.0 / fired as f64
    });
    WeaponStats {
        shots_fired: fired,
        shots_missed: missed,
        accuracy,
        kills: unsigned(member(entry, "TotalKills")),
        one_shot_kills: unsigned(member(entry, "TotalOneShotKills")),
        record_score: float(member(entry, "RecordKillScore")),
        record_species: species_of(unsigned(member(entry, "RecordKillSpecies")))
            .map(|id| species::display_name(&id, language)),
        furthest_shot: (furthest >= 0.0).then_some(furthest),
        purchased_at: (bought > 0).then_some(bought),
    }
}

fn readable_name(id: &str, name: &str) -> String {
    if name.is_empty() {
        id.trim_start_matches("equipment_").replace('_', " ")
    } else {
        name.to_owned()
    }
}

/// Monta o inventário a partir do `thp_player_profile_adf` do save.
///
/// # Errors
///
/// [`CareerError`] se o perfil não puder ser lido.
pub fn inventory(save_dir: &Path, language: Language) -> Result<Inventory, CareerError> {
    let adf = read_adf(&save_dir.join(PROFILE), PROFILE)?;
    let root = crate::career::first_instance(&adf, PROFILE)?;
    Ok(build(&root, language))
}

fn build(root: &Value, language: Language) -> Inventory {
    let catalog = catalog();
    let unlocked = hashes(member(root, "EquipmentUnlocked"));
    let backpack = hashes(member(root, "EquipmentBackPack"));
    // Posse e quantidade andam juntas: a posição `i` de uma lista corresponde à da outra.
    let owned: HashMap<u32, u32> = list(member(root, "EquipmentOwned"))
        .iter()
        .zip(list(member(root, "EquipmentAmount")))
        .filter_map(|(hash, amount)| {
            let hash = u32::try_from(unsigned(Some(hash)))
                .ok()
                .filter(|h| *h != 0)?;
            Some((
                hash,
                u32::try_from(unsigned(Some(amount))).unwrap_or(u32::MAX),
            ))
        })
        .collect();
    let stats: HashMap<u32, WeaponStats> = list(member(root, "WeaponData"))
        .iter()
        .filter_map(|entry| {
            let hash = u32::try_from(unsigned(member(entry, "EquipmentHash")))
                .ok()
                .filter(|h| *h != 0)?;
            catalog.id_of(hash)?;
            Some((hash, weapon_stats(entry, language)))
        })
        .collect();
    let mut items: Vec<ItemView> = catalog
        .items()
        .map(|(id, item)| {
            let hash = hash_of(id);
            let amount = owned.get(&hash).copied();
            ItemView {
                id: id.to_owned(),
                kind: item.kind,
                name: readable_name(id, &item.name),
                price: item.price,
                weight: item.weight,
                class: item.class,
                box_size: item.box_size,
                icon: item.icon.clone(),
                owned: amount.is_some(),
                unlocked: unlocked.contains(&hash),
                in_backpack: backpack.contains(&hash),
                amount: amount.unwrap_or(0),
                ammo: item.ammo.clone(),
                sights: item.sights.clone(),
                stats: stats.get(&hash).cloned(),
                attributes: item.attributes.clone(),
                unlock: item.unlock,
            }
        })
        .collect();
    items.sort_by(|a, b| (a.kind, &a.name, &a.id).cmp(&(b.kind, &b.name, &b.id)));
    let loadouts_info = member(root, "LoadoutInfo");
    let active = usize::try_from(unsigned(
        loadouts_info.and_then(|l| member(l, "ActiveLoadout")),
    ))
    .unwrap_or(0);
    let loadouts = list(loadouts_info.and_then(|l| member(l, "LoadoutData")))
        .iter()
        .enumerate()
        .map(|(index, data)| LoadoutView {
            index,
            name: text(member(data, "LoadoutName")),
            unlocked: unsigned(member(data, "IsUnlocked")) != 0,
            active: index == active,
            items: ids(member(data, "LoadoutAllEquipment")),
            missing: ids(member(data, "LoadoutMissingItems")),
        })
        .collect();
    let levels: Vec<u64> = list(member(root, "WeaponLevel"))
        .iter()
        .map(|v| unsigned(Some(v)))
        .collect();
    Inventory {
        cash: unsigned(member(root, "Cash")),
        level: unsigned(member(root, "Level")),
        weapon_levels: [
            levels.first().copied().unwrap_or(0),
            levels.get(1).copied().unwrap_or(0),
            levels.get(2).copied().unwrap_or(0),
            levels.get(3).copied().unwrap_or(0),
        ],
        items,
        loadouts,
    }
}

/// Ids de itens do catálogo a partir de uma lista de hashes (ignora o que o catálogo não conhece).
fn ids(value: Option<&Value>) -> Vec<String> {
    hashes(value)
        .into_iter()
        .filter_map(|h| catalog().id_of(h).map(str::to_owned))
        .collect()
}

/// Imagem codificada em PNG (com transparência), ou `None` se não couber no formato.
fn png_bytes(image: &Image) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(&image.rgba).ok()?;
    writer.finish().ok()?;
    Some(out)
}

/// Ícone de um item como PNG, lido do jogo do usuário; guarda uma cópia em `cache_dir` para as
/// próximas aberturas. Nada disso entra no repositório nem sai do computador do usuário.
fn icon_png(archives: Option<&GameArchives>, cache_dir: &Path, id: &str) -> Option<Vec<u8>> {
    let file = cache_dir.join(format!("{id}.png"));
    if let Ok(bytes) = std::fs::read(&file) {
        return Some(bytes);
    }
    let path = catalog().get(id)?.icon.as_deref()?;
    let archives = archives?;
    let location = archives.find_path(path).first()?;
    let image = texture::decode_top_level(&archives.read(location).ok()?).ok()?;
    let bytes = png_bytes(&image)?;
    // Falhar em guardar a cópia só deixa a próxima abertura um pouco mais lenta.
    if std::fs::create_dir_all(cache_dir).is_ok() {
        let _ = std::fs::write(&file, &bytes);
    }
    Some(bytes)
}

/// Ícones dos itens pedidos como `data:image/png;base64,...` (itens sem ícone ficam de fora).
#[must_use]
pub fn icons(
    archives: Option<&GameArchives>,
    cache_dir: &Path,
    ids: &[String],
) -> BTreeMap<String, String> {
    ids.iter()
        .filter_map(|id| {
            let bytes = icon_png(archives, cache_dir, id)?;
            let url = format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            );
            Some((id.clone(), url))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unsigned_array(values: &[u64]) -> Value {
        Value::Array(values.iter().map(|v| Value::Unsigned(*v)).collect())
    }

    fn field(name: &str, value: Value) -> (String, Value) {
        (name.to_owned(), value)
    }

    fn weapon_entry(hash: u32, fired: u64, missed: u64) -> Value {
        Value::Struct(vec![
            field("EquipmentHash", Value::Unsigned(u64::from(hash))),
            field("PurchaseTimestamp", Value::Unsigned(1_700_000_000)),
            field("ShotsFired", Value::Unsigned(fired)),
            field("ShotsMissed", Value::Unsigned(missed)),
            field("TotalKills", Value::Unsigned(10)),
            field("TotalOneShotKills", Value::Unsigned(4)),
            field("RecordKillScore", Value::Float(812.5)),
            field(
                "RecordKillSpecies",
                Value::Unsigned(u64::from(hash_of("moose"))),
            ),
            field("FurthestVitalOrganShotDistance", Value::Float(250.5)),
        ])
    }

    fn profile() -> Value {
        let rifle = hash_of("equipment_weapon_ba_rifle_270_01");
        let ammo = hash_of("equipment_ammo_270_sp_01");
        let bow = hash_of("equipment_weapon_longbow_01");
        Value::Struct(vec![
            field("Cash", Value::Unsigned(1234)),
            field("WeaponLevel", unsigned_array(&[30, 23, 25, 21])),
            field("EquipmentUnlocked", unsigned_array(&[u64::from(bow), 0, 0])),
            field(
                "EquipmentOwned",
                unsigned_array(&[u64::from(rifle), u64::from(ammo), 0, 0]),
            ),
            field("EquipmentAmount", unsigned_array(&[1, 203, 0, 0])),
            field("EquipmentBackPack", unsigned_array(&[u64::from(ammo), 0])),
            field(
                "WeaponData",
                Value::Array(vec![weapon_entry(rifle, 100, 25), weapon_entry(0, 5, 1)]),
            ),
            field(
                "LoadoutInfo",
                Value::Struct(vec![
                    field("ActiveLoadout", Value::Unsigned(1)),
                    field(
                        "LoadoutData",
                        Value::Array(vec![
                            Value::Struct(vec![
                                field("LoadoutName", Value::String("Rifle".into())),
                                field("IsUnlocked", Value::Unsigned(1)),
                                field(
                                    "LoadoutAllEquipment",
                                    unsigned_array(&[u64::from(rifle), u64::from(ammo)]),
                                ),
                                field("LoadoutMissingItems", unsigned_array(&[u64::from(bow)])),
                            ]),
                            Value::Struct(vec![field(
                                "LoadoutName",
                                Value::String("Vazio".into()),
                            )]),
                        ]),
                    ),
                ]),
            ),
        ])
    }

    fn item<'a>(inv: &'a Inventory, id: &str) -> &'a ItemView {
        inv.items.iter().find(|i| i.id == id).unwrap()
    }

    #[test]
    fn the_catalog_has_the_game_items_and_their_links() {
        let c = catalog();
        assert!(c.items().count() >= 430);
        let rifle = c.get("equipment_weapon_ba_rifle_270_01").unwrap();
        assert_eq!(rifle.kind, Kind::Weapon);
        assert_eq!(rifle.class, Some(WeaponClass::Rifle));
        assert!(rifle.ammo.iter().any(|a| a == "equipment_ammo_270_sp_01"));
        let ammo = c.get("equipment_ammo_270_sp_01").unwrap();
        assert_eq!(ammo.box_size, Some(10));
        assert!(ammo.price > 0);
    }

    #[test]
    fn every_weapon_has_a_class_and_every_id_hashes_to_itself() {
        for (id, item) in catalog().items() {
            if item.kind == Kind::Weapon {
                assert!(item.class.is_some(), "{id}");
            }
            assert_eq!(catalog().id_of(hash_of(id)), Some(id), "{id}");
        }
    }

    #[test]
    fn ammo_and_weapons_agree_on_what_fits() {
        for (id, item) in catalog().items().filter(|(_, i)| i.kind == Kind::Weapon) {
            for ammo in &item.ammo {
                let other = catalog().get(ammo).unwrap();
                assert!(other.kind == Kind::Ammo, "{id} -> {ammo}");
            }
        }
    }

    #[test]
    fn ownership_amounts_and_stats_come_from_the_profile() {
        let inv = build(&profile(), Language::PtBr);
        assert_eq!(inv.cash, 1234);
        assert_eq!(inv.weapon_levels, [30, 23, 25, 21]);
        let rifle = item(&inv, "equipment_weapon_ba_rifle_270_01");
        assert!(rifle.owned && !rifle.unlocked);
        let stats = rifle.stats.as_ref().unwrap();
        assert_eq!(stats.shots_fired, 100);
        assert!((stats.accuracy.unwrap() - 75.0).abs() < 1e-9);
        assert_eq!(stats.record_species.as_deref(), Some("Alce"));
        assert_eq!(stats.furthest_shot, Some(250.5));
        let ammo = item(&inv, "equipment_ammo_270_sp_01");
        assert_eq!(
            (ammo.owned, ammo.amount, ammo.in_backpack),
            (true, 203, true)
        );
        let bow = item(&inv, "equipment_weapon_longbow_01");
        assert!(!bow.owned && bow.unlocked && bow.stats.is_none());
    }

    #[test]
    fn loadouts_list_items_and_what_is_missing() {
        let inv = build(&profile(), Language::PtBr);
        assert_eq!(inv.loadouts.len(), 2);
        let first = &inv.loadouts[0];
        assert_eq!(first.name, "Rifle");
        assert!(!first.active && first.unlocked);
        assert_eq!(first.items.len(), 2);
        assert_eq!(
            first.missing,
            vec!["equipment_weapon_longbow_01".to_owned()]
        );
        assert!(inv.loadouts[1].active);
    }

    #[test]
    fn icons_come_from_the_cache_and_unknown_items_have_none() {
        let dir = tempfile::tempdir().unwrap();
        let image = Image {
            width: 2,
            height: 2,
            rgba: vec![255; 16],
        };
        let bytes = png_bytes(&image).unwrap();
        assert_eq!(&bytes[1..4], b"PNG");
        std::fs::write(dir.path().join("equipment_ammo_270_sp_01.png"), &bytes).unwrap();
        let ids = vec![
            "equipment_ammo_270_sp_01".to_owned(),
            "equipment_weapon_ba_rifle_270_01".to_owned(),
            "nao_existe".to_owned(),
        ];
        let found = icons(None, dir.path(), &ids);
        assert_eq!(found.len(), 1);
        assert!(found["equipment_ammo_270_sp_01"].starts_with("data:image/png;base64,"));
    }

    #[test]
    fn an_empty_profile_gives_an_inventory_with_nothing_owned() {
        let inv = build(&Value::Struct(vec![]), Language::PtBr);
        assert_eq!(inv.cash, 0);
        assert!(inv.items.iter().all(|i| !i.owned));
        assert_eq!(inv.loadouts, Vec::new());
    }

    #[test]
    fn unfired_weapons_have_no_accuracy_and_unknown_distance() {
        let entry = Value::Struct(vec![field(
            "FurthestVitalOrganShotDistance",
            Value::Float(-1.0),
        )]);
        let stats = weapon_stats(&entry, Language::PtBr);
        assert_eq!(stats.accuracy, None);
        assert_eq!(stats.furthest_shot, None);
        assert_eq!(stats.purchased_at, None);
    }
}
