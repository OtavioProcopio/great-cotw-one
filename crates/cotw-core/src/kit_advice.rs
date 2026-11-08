//! Kit para uma reserva inteira (spec `equipment-page`, "Kit para a reserva"): o menor conjunto de
//! armas do arsenal que cobre as classes de caça de todas as espécies da reserva, mais as iscas e
//! chamados que atraem essas espécies, os extras úteis (telêmetro, kit de primeiros socorros,
//! eliminador de cheiro) e o peso total.
//!
//! Tudo vem do jogo: classes de caça dos animais, classes recomendadas de cada munição, espécies que
//! cada isca atrai e peso de cada item. O limite de peso do jogo não é conhecido, então o app só
//! soma; quem decide é o jogador.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::equipment::{Inventory, ItemView, Kind};
use crate::gear_advice::{Pick, advise};
use crate::language::Language;
use crate::species::display_name;
use crate::species_stats;

/// No máximo quantas armas o kit leva.
const MAX_WEAPONS: usize = 4;

/// Uma arma com munição e mira, e as espécies da reserva que ela atende.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitWeapon {
    /// A combinação sugerida.
    pub pick: Pick,
    /// Espécies atendidas (nomes no idioma do app).
    pub covers: Vec<String>,
}

/// Uma isca ou chamado e as espécies da reserva que ele atrai.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitLure {
    /// Id do item.
    pub id: String,
    /// O jogador tem o item.
    pub owned: bool,
    /// Preço, para o que ainda não tem.
    pub price: u32,
    /// Espécies da reserva que ele atrai (nomes no idioma do app).
    pub covers: Vec<String>,
}

/// O kit completo de uma reserva.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kit {
    /// Espécies consideradas (com classe de caça informada pelo jogo), no idioma do app.
    pub species: Vec<String>,
    /// Armas do arsenal que cobrem as espécies.
    pub weapons: Vec<KitWeapon>,
    /// Armas a comprar para cobrir o que o arsenal não cobre.
    pub to_buy: Vec<KitWeapon>,
    /// Espécies que nem o arsenal nem a loja cobrem.
    pub uncovered: Vec<String>,
    /// Iscas e chamados que o jogador tem e servem na reserva.
    pub lures: Vec<KitLure>,
    /// Iscas e chamados que valem comprar para atrair o que falta.
    pub lures_to_buy: Vec<KitLure>,
    /// Extras que o jogador tem (ids): telêmetro, primeiros socorros, eliminador de cheiro.
    pub extras: Vec<String>,
    /// Peso do que o kit já tem (armas, munição, miras, iscas e extras), em kg.
    pub weight: f64,
}

/// Nome do jogo sem sublinhado, como as iscas listam as espécies.
fn normalized(id: &str) -> String {
    id.replace('_', "")
}

/// A isca atrai a espécie? (`genericbear` vale para qualquer urso.)
fn attracts(lure: &ItemView, species: &str) -> bool {
    let name = normalized(species);
    lure.attributes
        .attracts
        .iter()
        .any(|a| *a == name || (a == "genericbear" && name.ends_with("bear")))
}

/// Armas que cobrem mais espécies primeiro; empate pela nota e depois pelo menor peso.
fn cover(
    picks: &[(Pick, Vec<String>)],
    inv: &Inventory,
    mut left: BTreeSet<String>,
    limit: usize,
) -> Vec<(Pick, Vec<String>)> {
    let weight_of = |pick: &Pick| -> f64 {
        inv.items
            .iter()
            .find(|i| i.id == pick.weapon)
            .and_then(|i| i.weight)
            .unwrap_or(0.0)
    };
    let mut chosen: Vec<(Pick, Vec<String>)> = Vec::new();
    while !left.is_empty() && chosen.len() < limit {
        let best = picks
            .iter()
            .filter(|(p, _)| !chosen.iter().any(|(c, _)| c.weapon == p.weapon))
            .map(|(p, covered)| {
                let new: Vec<String> = covered
                    .iter()
                    .filter(|s| left.contains(*s))
                    .cloned()
                    .collect();
                (p, new)
            })
            .filter(|(_, new)| !new.is_empty())
            .max_by(|(a, na), (b, nb)| {
                na.len()
                    .cmp(&nb.len())
                    .then(a.score.total_cmp(&b.score))
                    .then(weight_of(b).total_cmp(&weight_of(a)))
            });
        let Some((pick, new)) = best else { break };
        for s in &new {
            left.remove(s);
        }
        chosen.push((pick.clone(), new));
    }
    chosen
}

/// Iscas e chamados que atraem as espécies, escolhendo os que atraem mais espécies ainda sem isca.
/// `covers` volta com ids de espécie; quem chama traduz para nomes.
fn lures_for(inv: &Inventory, all: &BTreeSet<String>, owned_only: bool) -> Vec<KitLure> {
    let mut left = all.clone();
    let mut out: Vec<KitLure> = Vec::new();
    let candidates: Vec<&ItemView> = inv
        .items
        .iter()
        .filter(|i| i.kind == Kind::LureCaller && i.owned == owned_only)
        .collect();
    while !left.is_empty() {
        let best = candidates
            .iter()
            .filter(|l| !out.iter().any(|o| o.id == l.id))
            .map(|l| {
                let new: Vec<String> = left.iter().filter(|s| attracts(l, s)).cloned().collect();
                (*l, new)
            })
            .filter(|(_, new)| !new.is_empty())
            .max_by(|(a, na), (b, nb)| {
                na.len()
                    .cmp(&nb.len())
                    .then(
                        a.attributes
                            .lure_chance
                            .partial_cmp(&b.attributes.lure_chance)
                            .unwrap_or(std::cmp::Ordering::Equal),
                    )
                    .then(b.price.cmp(&a.price))
            });
        let Some((lure, new)) = best else { break };
        for s in &new {
            left.remove(s);
        }
        out.push(KitLure {
            id: lure.id.clone(),
            owned: lure.owned,
            price: lure.price,
            covers: new,
        });
    }
    out
}

/// Extras que ajudam em qualquer caça: o melhor telêmetro, primeiros socorros e eliminador de cheiro.
fn extras_of(inv: &Inventory) -> Vec<String> {
    let mut extras: Vec<String> = inv
        .items
        .iter()
        .filter(|i| i.owned && i.kind == Kind::Optic && i.id.contains("rangefinder"))
        .max_by(|a, b| {
            a.attributes
                .zoom
                .partial_cmp(&b.attributes.zoom)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|optic| optic.id.clone())
        .into_iter()
        .collect();
    extras.extend(
        inv.items
            .iter()
            .filter(|i| {
                i.owned
                    && i.kind == Kind::Consumable
                    && (i.id.contains("first_aid") || i.id.contains("scent_eliminator"))
            })
            .map(|i| i.id.clone()),
    );
    extras
}

/// Armas do arsenal e a comprar, e as espécies que nenhuma das duas cobre.
struct Weapons {
    owned: Vec<(Pick, Vec<String>)>,
    bought: Vec<(Pick, Vec<String>)>,
    uncovered: Vec<String>,
}

fn choose_weapons(inv: &Inventory, classes: &BTreeMap<&str, u32>, language: Language) -> Weapons {
    let by_id: BTreeMap<&str, &ItemView> = inv.items.iter().map(|i| (i.id.as_str(), i)).collect();
    // Quais espécies cada combinação atende: as da reserva cuja classe cabe na faixa da munição.
    let serves = |pick: &Pick| -> Vec<String> {
        let ammo_classes = by_id
            .get(pick.ammo.as_str())
            .map(|a| a.attributes.classes.clone())
            .unwrap_or_default();
        classes
            .iter()
            .filter(|(_, c)| ammo_classes.contains(c))
            .map(|(s, _)| (*s).to_owned())
            .collect()
    };
    let mut ready: Vec<(Pick, Vec<String>)> = Vec::new();
    let mut buy: Vec<(Pick, Vec<String>)> = Vec::new();
    for id in classes.keys() {
        let Some(advice) = advise(inv, id, language) else {
            continue;
        };
        for (list, source) in [(&mut ready, &advice.ready), (&mut buy, &advice.to_buy)] {
            for pick in source {
                if !list
                    .iter()
                    .any(|(p, _): &(Pick, _)| p.weapon == pick.weapon && p.ammo == pick.ammo)
                {
                    list.push((pick.clone(), serves(pick)));
                }
            }
        }
    }
    let all: BTreeSet<String> = classes.keys().map(|s| (*s).to_owned()).collect();
    let owned = cover(&ready, inv, all.clone(), MAX_WEAPONS);
    let covered: BTreeSet<String> = owned.iter().flat_map(|(_, c)| c.iter().cloned()).collect();
    let missing: BTreeSet<String> = all.difference(&covered).cloned().collect();
    let room = MAX_WEAPONS.saturating_sub(owned.len()).max(1);
    let bought = cover(&buy, inv, missing.clone(), room);
    let uncovered = missing
        .into_iter()
        .filter(|s| !bought.iter().any(|(_, c)| c.contains(s)))
        .collect();
    Weapons {
        owned,
        bought,
        uncovered,
    }
}

/// Monta o kit para as espécies dadas (ids do jogo).
#[must_use]
pub fn advise_reserve(inv: &Inventory, species: &[String], language: Language) -> Kit {
    let by_id: BTreeMap<&str, &ItemView> = inv.items.iter().map(|i| (i.id.as_str(), i)).collect();
    let classes: BTreeMap<&str, u32> = species
        .iter()
        .filter_map(|s| Some((s.as_str(), species_stats::lookup(s)?.hunt_class)))
        .filter(|(_, c)| *c > 0)
        .collect();
    let all: BTreeSet<String> = classes.keys().map(|s| (*s).to_owned()).collect();
    let names = |ids: &[String]| {
        ids.iter()
            .map(|s| display_name(s, language))
            .collect::<Vec<_>>()
    };
    let weapons = choose_weapons(inv, &classes, language);

    let translate = |l: KitLure| KitLure {
        covers: names(&l.covers),
        ..l
    };
    let lures_ids = lures_for(inv, &all, true);
    let with_lure: BTreeSet<String> = lures_ids
        .iter()
        .flat_map(|l| l.covers.iter().cloned())
        .collect();
    let lures_to_buy: Vec<KitLure> = lures_for(inv, &all, false)
        .into_iter()
        .map(|mut l| {
            l.covers.retain(|c| !with_lure.contains(c));
            l
        })
        .filter(|l| !l.covers.is_empty())
        .map(translate)
        .collect();
    let lures: Vec<KitLure> = lures_ids.into_iter().map(translate).collect();
    let extras = extras_of(inv);

    let weight_of = |id: &str| by_id.get(id).and_then(|i| i.weight).unwrap_or(0.0);
    let gear: f64 = weapons
        .owned
        .iter()
        .map(|(p, _)| {
            weight_of(&p.weapon) + weight_of(&p.ammo) + p.sight.as_deref().map_or(0.0, weight_of)
        })
        .sum();
    let weight = gear
        + lures.iter().map(|l| weight_of(&l.id)).sum::<f64>()
        + extras.iter().map(|id| weight_of(id)).sum::<f64>();

    let to_weapon = |(pick, covers): (Pick, Vec<String>)| KitWeapon {
        covers: names(&covers),
        pick,
    };
    Kit {
        species: all.iter().map(|s| display_name(s, language)).collect(),
        uncovered: names(&weapons.uncovered),
        weapons: weapons.owned.into_iter().map(to_weapon).collect(),
        to_buy: weapons.bought.into_iter().map(to_weapon).collect(),
        lures,
        lures_to_buy,
        extras,
        weight: (weight * 100.0).round() / 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::{Attributes, catalog};

    fn inventory(owned: &[&str]) -> Inventory {
        let mut inv = Inventory {
            cash: 0,
            level: 60,
            weapon_levels: [30, 30, 30, 30],
            items: vec![],
            loadouts: vec![],
        };
        for (id, item) in catalog().items() {
            if matches!(
                item.kind,
                Kind::Weapon
                    | Kind::Ammo
                    | Kind::Sight
                    | Kind::LureCaller
                    | Kind::Optic
                    | Kind::Consumable
            ) {
                inv.items.push(ItemView {
                    id: id.into(),
                    kind: item.kind,
                    name: item.name.clone(),
                    price: item.price,
                    weight: item.weight,
                    class: item.class,
                    box_size: None,
                    icon: None,
                    owned: owned.contains(&id),
                    unlocked: false,
                    in_backpack: false,
                    amount: 1,
                    ammo: item.ammo.clone(),
                    sights: item.sights.clone(),
                    stats: None,
                    attributes: Attributes {
                        ..item.attributes.clone()
                    },
                    unlock: item.unlock,
                });
            }
        }
        inv
    }

    fn species(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn one_weapon_that_covers_every_class_is_enough() {
        let inv = inventory(&[
            "equipment_weapon_ba_rifle_270_01",
            "equipment_ammo_270_sp_01",
        ]);
        // Cervo-de-cauda-branca (4), alce (8): a .270 recomenda as classes 4 a 8.
        let kit = advise_reserve(&inv, &species(&["whitetail_deer", "moose"]), Language::PtBr);
        assert_eq!(kit.weapons.len(), 1);
        assert_eq!(kit.weapons[0].covers.len(), 2);
        assert_eq!(kit.uncovered, Vec::<String>::new());
    }

    #[test]
    fn a_small_and_a_big_animal_need_two_weapons_and_the_missing_one_is_suggested_to_buy() {
        let inv = inventory(&[
            "equipment_weapon_ba_rifle_270_01",
            "equipment_ammo_270_sp_01",
        ]);
        let kit = advise_reserve(&inv, &species(&["moose", "eu_rabbit"]), Language::PtBr);
        assert_eq!(kit.weapons.len(), 1);
        assert_eq!(kit.to_buy.len(), 1);
        assert!(
            kit.to_buy[0]
                .covers
                .iter()
                .any(|c| c.to_lowercase().contains("coelho"))
        );
    }

    #[test]
    fn lures_that_attract_the_reserve_animals_are_chosen_and_the_rest_is_offered_to_buy() {
        let wolf = catalog()
            .items()
            .find(|(_, i)| {
                i.kind == Kind::LureCaller && i.attributes.attracts.iter().any(|a| a == "graywolf")
            })
            .map(|(id, _)| id.to_owned())
            .unwrap();
        let inv = inventory(&[wolf.as_str()]);
        let kit = advise_reserve(&inv, &species(&["gray_wolf", "moose"]), Language::PtBr);
        assert!(kit.lures.iter().any(|l| l.id == wolf && l.owned));
        assert!(kit.lures.iter().all(|l| l.owned));
        assert!(kit.lures_to_buy.iter().all(|l| !l.owned));
    }

    #[test]
    fn the_weight_adds_weapons_ammo_sights_lures_and_extras() {
        let inv = inventory(&[
            "equipment_weapon_ba_rifle_270_01",
            "equipment_ammo_270_sp_01",
            "equipment_consumable_first_aid_kit_01",
        ]);
        let kit = advise_reserve(&inv, &species(&["moose"]), Language::PtBr);
        let rifle = inv
            .items
            .iter()
            .find(|i| i.id == "equipment_weapon_ba_rifle_270_01")
            .unwrap();
        assert!(kit.weight >= rifle.weight.unwrap_or(0.0));
        assert!(
            kit.extras
                .contains(&"equipment_consumable_first_aid_kit_01".to_owned())
        );
    }

    #[test]
    fn species_without_a_hunting_class_are_ignored() {
        let inv = inventory(&[]);
        let kit = advise_reserve(&inv, &species(&["nao_existe"]), Language::PtBr);
        assert_eq!(kit.species, Vec::<String>::new());
        assert_eq!(kit.weapons, Vec::new());
    }
}
