//! Montador de equipamento por caça (spec `equipment-page`, recomendação): dada uma espécie, sugere
//! arma, munição e mira do arsenal do jogador e o que vale comprar.
//!
//! Os números vêm do próprio jogo: a classe de caça do animal (1 a 9) e as classes que cada munição
//! recomenda, mais precisão, recuo, alcance, penetração e expansão da loja. A regra é simples e
//! aberta: só entra munição cuja faixa de classes inclui a do animal; entre elas, a nota mistura os
//! atributos (com mais peso na penetração para caça grande e na precisão para caça pequena).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::Serialize;

use crate::equipment::{CatalogItem, Inventory, ItemView, Kind, Unlock, UnlockTrack, catalog};
use crate::language::Language;
use crate::species::display_name;
use crate::species_stats;

/// Quantas sugestões de cada grupo a tela recebe.
const PER_GROUP: usize = 6;
/// No máximo quantas sugestões de uma mesma classe de arma em cada grupo.
const PER_CLASS: usize = 2;

/// Uma combinação sugerida.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pick {
    /// Arma (id do jogo).
    pub weapon: String,
    /// Munição (id do jogo).
    pub ammo: String,
    /// Melhor mira que o jogador tem e serve na arma, se houver.
    pub sight: Option<String>,
    /// O jogador tem a arma.
    pub has_weapon: bool,
    /// O jogador tem a munição.
    pub has_ammo: bool,
    /// Dinheiro que falta gastar para ter a combinação.
    pub cost: u32,
    /// O jogador já alcançou o nível que libera o que falta.
    pub available: bool,
    /// Nível que libera o que falta (o mais alto), se algum.
    pub unlock: Option<Unlock>,
    /// Faixa de classes da munição (menor e maior).
    pub class_range: (u32, u32),
    /// Nota de 0 a 100 (só compara combinações da mesma espécie).
    pub score: f64,
}

/// Sugestões para uma espécie.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    /// Espécie (id do jogo).
    pub species: String,
    /// Nome no idioma do app.
    pub name: String,
    /// Classe de caça do animal (1 a 9).
    pub hunt_class: u32,
    /// Combinações que o jogador já pode usar, da melhor para a pior.
    pub ready: Vec<Pick>,
    /// Combinações que pedem uma compra, da melhor para a pior.
    pub to_buy: Vec<Pick>,
}

/// Menor e maior classe de uma munição.
fn class_range(item: &CatalogItem) -> Option<(u32, u32)> {
    let min = item.attributes.classes.iter().min()?;
    let max = item.attributes.classes.iter().max()?;
    Some((*min, *max))
}

/// Normaliza `value` entre o menor e o maior da lista (0 a 1); 0,5 se todos forem iguais.
fn scale(value: f64, all: &[f64]) -> f64 {
    let min = all.iter().copied().fold(f64::INFINITY, f64::min);
    let max = all.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max > min {
        (value - min) / (max - min)
    } else {
        0.5
    }
}

fn reached(inv: &Inventory, unlock: Unlock) -> bool {
    let have = match unlock.track {
        UnlockTrack::Character => inv.level,
        UnlockTrack::Rifle => inv.weapon_levels[0],
        UnlockTrack::Handgun => inv.weapon_levels[1],
        UnlockTrack::Shotgun => inv.weapon_levels[2],
        UnlockTrack::Archery => inv.weapon_levels[3],
    };
    have >= u64::from(unlock.level)
}

struct Candidate<'a> {
    weapon: &'a ItemView,
    ammo: &'a ItemView,
    range: (u32, u32),
}

/// Sugere equipamento para a espécie; `None` se a espécie não existir ou o jogo não der classe.
#[must_use]
pub fn advise(inv: &Inventory, species: &str, language: Language) -> Option<Advice> {
    let stats = species_stats::lookup(species)?;
    let class = stats.hunt_class;
    if class == 0 {
        return None;
    }
    let by_id: BTreeMap<&str, &ItemView> = inv.items.iter().map(|i| (i.id.as_str(), i)).collect();
    let mut candidates = Vec::new();
    for weapon in inv.items.iter().filter(|i| i.kind == Kind::Weapon) {
        for ammo_id in &weapon.ammo {
            let Some(ammo) = by_id.get(ammo_id.as_str()) else {
                continue;
            };
            let Some(range) = catalog().get(ammo_id).and_then(class_range) else {
                continue;
            };
            if (range.0..=range.1).contains(&class) {
                candidates.push(Candidate {
                    weapon,
                    ammo,
                    range,
                });
            }
        }
    }
    let column = |f: &dyn Fn(&Candidate) -> f64| candidates.iter().map(f).collect::<Vec<_>>();
    let pen = column(&|c| c.ammo.attributes.penetration.unwrap_or(0.0));
    let exp = column(&|c| c.ammo.attributes.expansion.unwrap_or(0.0));
    let acc = column(&|c| c.weapon.attributes.accuracy.unwrap_or(0.0));
    let range = column(&|c| c.ammo.attributes.range.unwrap_or(0.0));
    // Caça pequena pede precisão; caça grande pede penetração.
    let (w_pen, w_exp, w_acc, w_range) = if class <= 3 {
        (0.15, 0.15, 0.45, 0.25)
    } else if class >= 7 {
        (0.45, 0.2, 0.15, 0.2)
    } else {
        (0.3, 0.2, 0.25, 0.25)
    };
    let mut picks: Vec<Pick> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let score = 100.0
                * (w_pen * scale(pen[i], &pen)
                    + w_exp * scale(exp[i], &exp)
                    + w_acc * scale(acc[i], &acc)
                    + w_range * scale(range[i], &range));
            let missing: Vec<&ItemView> = [c.weapon, c.ammo]
                .into_iter()
                .filter(|x| !x.owned)
                .collect();
            let unlock = missing
                .iter()
                .filter_map(|x| x.unlock)
                .max_by_key(|u| u.level);
            Pick {
                weapon: c.weapon.id.clone(),
                ammo: c.ammo.id.clone(),
                sight: best_sight(c.weapon, &by_id),
                has_weapon: c.weapon.owned,
                has_ammo: c.ammo.owned,
                cost: missing.iter().map(|x| x.price).sum(),
                available: missing
                    .iter()
                    .all(|x| x.unlock.is_none_or(|u| reached(inv, u)) && x.unlocked_or_free()),
                unlock,
                class_range: c.range,
                score,
            }
        })
        .collect();
    picks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    // Uma sugestão por arma e, para mostrar opções de tipos diferentes, no máximo duas por classe de
    // arma (rifle, pistola, espingarda, arco) em cada grupo.
    let class_of = |id: &str| by_id.get(id).and_then(|i| i.class);
    let mut ready = Vec::new();
    let mut to_buy = Vec::new();
    for pick in picks {
        let list = if pick.has_weapon && pick.has_ammo {
            &mut ready
        } else {
            &mut to_buy
        };
        let same_weapon = list.iter().any(|p: &Pick| p.weapon == pick.weapon);
        let same_class = list
            .iter()
            .filter(|p: &&Pick| class_of(&p.weapon) == class_of(&pick.weapon))
            .count();
        if list.len() < PER_GROUP && !same_weapon && same_class < PER_CLASS {
            list.push(pick);
        }
    }
    Some(Advice {
        species: species.to_owned(),
        name: display_name(species, language),
        hunt_class: class,
        ready,
        to_buy,
    })
}

/// Mira que o jogador tem, serve na arma e dá o maior aumento.
fn best_sight(weapon: &ItemView, by_id: &BTreeMap<&str, &ItemView>) -> Option<String> {
    weapon
        .sights
        .iter()
        .filter_map(|id| by_id.get(id.as_str()))
        .filter(|s| s.owned)
        .max_by(|a, b| {
            a.attributes
                .zoom
                .partial_cmp(&b.attributes.zoom)
                .unwrap_or(Ordering::Equal)
        })
        .map(|s| s.id.clone())
}

impl ItemView {
    /// Item que o jogo já deixou comprar (liberado) ou que não tem condição de nível.
    fn unlocked_or_free(&self) -> bool {
        self.unlocked || self.unlock.is_none() || self.price == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Attributes;

    fn view(id: &str, kind: Kind, owned: bool) -> ItemView {
        ItemView {
            id: id.into(),
            kind,
            name: id.into(),
            price: 100,
            weight: None,
            class: None,
            box_size: None,
            icon: None,
            owned,
            unlocked: false,
            in_backpack: false,
            amount: u32::from(owned),
            ammo: vec![],
            sights: vec![],
            stats: None,
            attributes: Attributes::default(),
            unlock: None,
        }
    }

    fn inventory(owned: &[&str]) -> Inventory {
        let mut inv = Inventory {
            cash: 0,
            level: 60,
            weapon_levels: [30, 30, 30, 30],
            items: vec![],
            loadouts: vec![],
        };
        for (id, item) in catalog().items() {
            if matches!(item.kind, Kind::Weapon | Kind::Ammo | Kind::Sight) {
                let mut v = view(id, item.kind, owned.contains(&id));
                v.name = item.name.clone();
                v.class = item.class;
                v.price = item.price;
                v.ammo = item.ammo.clone();
                v.sights = item.sights.clone();
                v.attributes = item.attributes.clone();
                v.unlock = item.unlock;
                inv.items.push(v);
            }
        }
        inv
    }

    #[test]
    fn a_big_animal_never_gets_ammo_made_for_small_game() {
        let inv = inventory(&[]);
        let advice = advise(&inv, "moose", Language::PtBr).unwrap();
        assert_eq!(advice.hunt_class, 8);
        assert_eq!(advice.name, "Alce");
        for pick in advice.ready.iter().chain(&advice.to_buy) {
            assert!(
                pick.class_range.0 <= 8 && 8 <= pick.class_range.1,
                "{pick:?}"
            );
        }
        assert_ne!(advice.to_buy.len(), 0);
        assert!(advice.to_buy.iter().all(|p| !p.ammo.contains("22lr")));
    }

    #[test]
    fn a_small_animal_gets_small_calibers() {
        let inv = inventory(&[]);
        let advice = advise(&inv, "eu_rabbit", Language::PtBr).unwrap();
        assert_eq!(advice.hunt_class, 1);
        assert!(
            advice
                .to_buy
                .iter()
                .any(|p| p.ammo.contains("22lr") || p.ammo.contains("bird"))
        );
    }

    #[test]
    fn what_the_player_owns_comes_first_as_ready() {
        let inv = inventory(&[
            "equipment_weapon_ba_rifle_270_01",
            "equipment_ammo_270_sp_01",
        ]);
        let advice = advise(&inv, "moose", Language::PtBr).unwrap();
        let first = &advice.ready[0];
        assert_eq!(first.weapon, "equipment_weapon_ba_rifle_270_01");
        assert_eq!(first.ammo, "equipment_ammo_270_sp_01");
        assert_eq!((first.cost, first.available), (0, true));
        assert!(advice.to_buy.iter().all(|p| !(p.has_weapon && p.has_ammo)));
        assert!(advice.to_buy.iter().all(|p| p.cost > 0));
    }

    #[test]
    fn a_missing_item_above_the_players_level_is_not_available() {
        let mut inv = inventory(&[]);
        inv.weapon_levels = [1, 1, 1, 1];
        inv.level = 1;
        let advice = advise(&inv, "moose", Language::PtBr).unwrap();
        assert!(advice.to_buy.iter().any(|p| !p.available));
        let blocked = advice.to_buy.iter().find(|p| !p.available).unwrap();
        assert!(blocked.unlock.is_some());
    }

    #[test]
    fn at_most_one_pick_per_weapon_and_a_limited_list() {
        let inv = inventory(&[]);
        let advice = advise(&inv, "red_deer", Language::PtBr).unwrap();
        assert!(advice.to_buy.len() <= PER_GROUP);
        let mut weapons: Vec<_> = advice.to_buy.iter().map(|p| &p.weapon).collect();
        weapons.sort();
        weapons.dedup();
        assert_eq!(weapons.len(), advice.to_buy.len());
        for pair in advice.to_buy.windows(2) {
            assert!(pair[0].score >= pair[1].score);
        }
    }

    #[test]
    fn suggestions_mix_weapon_classes_instead_of_repeating_one() {
        let inv = inventory(&[]);
        let advice = advise(&inv, "red_deer", Language::PtBr).unwrap();
        let class_of = |id: &str| inv.items.iter().find(|i| i.id == id).and_then(|i| i.class);
        for class in [
            Some(crate::equipment::WeaponClass::Rifle),
            Some(crate::equipment::WeaponClass::Shotgun),
        ] {
            let count = advice
                .to_buy
                .iter()
                .filter(|p| class_of(&p.weapon) == class)
                .count();
            assert!(count <= PER_CLASS, "{class:?}: {count}");
        }
        assert!(advice.to_buy.len() > 1);
    }

    #[test]
    fn the_best_owned_sight_is_suggested() {
        let rifle = "equipment_weapon_ba_rifle_270_01";
        let catalog_rifle = catalog().get(rifle).unwrap();
        let sight = catalog_rifle.sights[0].clone();
        let inv = inventory(&[rifle, "equipment_ammo_270_sp_01", sight.as_str()]);
        let advice = advise(&inv, "moose", Language::PtBr).unwrap();
        assert_eq!(advice.ready[0].sight.as_deref(), Some(sight.as_str()));
    }

    #[test]
    fn unknown_species_and_species_without_class_give_nothing() {
        let inv = inventory(&[]);
        assert!(advise(&inv, "dragon", Language::PtBr).is_none());
    }

    #[test]
    fn the_scale_handles_equal_values() {
        assert!((scale(5.0, &[5.0, 5.0]) - 0.5).abs() < 1e-9);
        assert!((scale(10.0, &[0.0, 10.0]) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn hunt_classes_match_the_ones_the_community_uses() {
        for (id, class) in [
            ("eu_rabbit", 1),
            ("red_fox", 2),
            ("whitetail_deer", 4),
            ("wild_boar", 5),
            ("red_deer", 6),
            ("moose", 8),
            ("cape_buffalo", 9),
        ] {
            assert_eq!(species_stats::get(id).unwrap().hunt_class, class, "{id}");
        }
    }

    #[test]
    fn the_270_range_is_four_to_eight_as_the_game_says() {
        let item = catalog().get("equipment_ammo_270_sp_01").unwrap();
        assert_eq!(class_range(item), Some((4, 8)));
        let small = catalog().get("equipment_ammo_243_sp_01").unwrap();
        assert_eq!(class_range(small), Some((2, 6)));
    }
}
