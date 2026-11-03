//! "Liberar o mapa": calcula as zonas de necessidade que os grupos de uma reserva usam e o save
//! ainda não conhece (spec `need-zone-unlock`).
//!
//! Tudo aqui é cálculo puro sobre dados já lidos; ler o jogo e gravar o save é com o serviço.
//!
//! Regras, todas confirmadas contra as zonas que o próprio jogo gravou no save:
//! - a posição de uma área é o centro da caixa das células dela, a 32 m por célula, mais um
//!   deslocamento fixo por reserva, que sai das zonas já gravadas pelo jogo;
//! - a altura vem das zonas vizinhas gravadas pelo jogo (nunca 1,0);
//! - o horário é um horário real da espécie que cabe entre as paradas vizinhas da rota;
//! - a constante da espécie é a que o jogo já usou nas zonas dela.

use std::collections::{BTreeMap, HashMap, HashSet};

use cotw_formats::need_zones::Zone;
use cotw_formats::population::Group;
use cotw_formats::spawn_map::{SpawnArea, SpawnMap};

/// Zonas gravadas pelo jogo têm altura de verdade; as de ferramentas antigas ficaram com 1,0.
const PLACEHOLDER_HEIGHT: f32 = 1.0;
/// Células têm 32 m, então o centro (soma dos extremos / 2) vale 16 m por unidade da soma.
const METERS_PER_HALF_CELL: f64 = 16.0;
/// Menos zonas do jogo que isso e a calibração não é confiável.
const MIN_CALIBRATION_ZONES: usize = 3;
/// Fração mínima de zonas do jogo que precisa concordar com o deslocamento escolhido.
const MIN_CALIBRATION_AGREEMENT: f64 = 0.9;
/// Raio para interpolar a altura a partir de zonas vizinhas.
const NEAR_RADIUS: f64 = 400.0;
/// Quantas zonas vizinhas entram na média da altura.
const HEIGHT_NEIGHBOURS: usize = 3;
const DAY: f32 = 24.0;

/// Quantas vezes cada horário aparece nas zonas do jogo de uma (espécie, tipo).
type SlotCounts = HashMap<(u32, u32, u32), (Slot, usize)>;

/// Por que uma reserva não pôde ser calculada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    /// Poucas zonas gravadas pelo jogo para descobrir o deslocamento do mapa.
    #[error("poucas zonas descobertas no jogo para calibrar o mapa desta reserva")]
    NotEnoughGameZones,
    /// As zonas do jogo não concordam entre si sobre o deslocamento.
    #[error("as zonas do jogo não concordam com o mapa desta reserva")]
    InconsistentGameZones,
}

/// Resultado do cálculo de uma reserva.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservePlan {
    /// Número da reserva.
    pub reserve: u32,
    /// Zonas novas, em ordem de chave.
    pub new_zones: Vec<Zone>,
    /// Zonas cujo horário não coube entre as paradas vizinhas e usou o mais comum da espécie.
    pub estimated: usize,
    /// Áreas da rota que não existem nos mapas atuais do jogo.
    pub unknown_areas: usize,
    /// Zonas não criadas porque o jogo não tem horário conhecido da espécie.
    pub no_schedule: usize,
    /// Zonas não criadas porque o save não tem nenhuma zona do jogo para estimar a altura.
    pub no_height: usize,
}

/// Dados de uma reserva para o cálculo.
pub struct ReserveInput<'a> {
    /// Número da reserva.
    pub reserve: u32,
    /// Grupos de animais e suas rotas.
    pub groups: &'a [Group],
    /// Mapas de spawn por espécie.
    pub maps: &'a [SpawnMap],
    /// Zonas que o save já tem.
    pub existing: &'a [Zone],
}

/// Horário de uma zona: início, fim e índice do horário.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slot {
    start: f32,
    end: f32,
    schedule: u32,
}

impl Slot {
    fn of(zone: &Zone) -> Self {
        Self {
            start: zone.start,
            end: zone.end,
            schedule: zone.schedule,
        }
    }

    fn id(self) -> (u32, u32, u32) {
        (self.start.to_bits(), self.end.to_bits(), self.schedule)
    }
}

fn is_game_written(zone: &Zone) -> bool {
    zone.y.to_bits() != PLACEHOLDER_HEIGHT.to_bits()
}

/// Calcula as zonas que faltam numa reserva.
///
/// # Errors
///
/// [`PlanError`] se as zonas já gravadas pelo jogo não bastam para calibrar o mapa da reserva.
pub fn plan_reserve(input: &ReserveInput<'_>) -> Result<ReservePlan, PlanError> {
    let areas = index_areas(input.maps);
    let (offset_x, offset_z) = calibrate(input.existing, &areas)?;
    let game_zones: Vec<&Zone> = input
        .existing
        .iter()
        .filter(|z| is_game_written(z))
        .collect();

    let (group_species, key_species) = assign_species(input.groups, input.maps);
    let species_constants = species_constants(&game_zones, &key_species);
    let slots = slot_table(&game_zones, &key_species);

    let existing_keys: HashSet<u32> = input.existing.iter().map(|z| z.key).collect();
    let mut known_times: HashMap<u32, Slot> = input
        .existing
        .iter()
        .map(|z| (z.key, Slot::of(z)))
        .collect();
    let mut used_icons: HashSet<u64> = input.existing.iter().map(|z| z.icon_id).collect();

    let mut plan = ReservePlan {
        reserve: input.reserve,
        new_zones: Vec::new(),
        estimated: 0,
        unknown_areas: 0,
        no_schedule: 0,
        no_height: 0,
    };
    let mut handled: HashSet<u32> = HashSet::new();

    for (group, species) in input.groups.iter().zip(&group_species) {
        for index in 0..group.route.len() {
            let key = group.route[index];
            if existing_keys.contains(&key) || !handled.insert(key) {
                continue;
            }
            let Some(area) = areas.get(&key) else {
                plan.unknown_areas += 1;
                continue;
            };
            let Some(species) = species else {
                plan.no_schedule += 1;
                continue;
            };
            let candidates = slots
                .get(&(species.clone(), area.kind))
                .map_or(&[][..], Vec::as_slice);
            let (Some(constant), Some((slot, fitted))) = (
                species_constants.get(species),
                choose_slot(group, index, &known_times, candidates),
            ) else {
                plan.no_schedule += 1;
                continue;
            };
            let position = world_position(area, offset_x, offset_z);
            let Some(height) = estimate_height(position, &game_zones) else {
                plan.no_height += 1;
                continue;
            };
            known_times.insert(key, slot);
            if !fitted {
                plan.estimated += 1;
            }
            #[allow(
                clippy::cast_possible_truncation,
                reason = "coordenadas do mundo cabem em f32 (o jogo as guarda assim)"
            )]
            let zone = Zone::new(
                (position.0 as f32, height as f32, position.1 as f32),
                key,
                area.kind,
                fresh_icon_id(input.reserve, key, &mut used_icons),
                (slot.start, slot.end),
                *constant,
                slot.schedule,
            );
            plan.new_zones.push(zone);
        }
    }
    plan.new_zones.sort_by_key(|z| z.key);
    Ok(plan)
}

/// Chave da área → área (a primeira que aparece; a geometria é a mesma entre espécies).
pub(crate) fn index_areas(maps: &[SpawnMap]) -> HashMap<u32, SpawnArea> {
    let mut areas = HashMap::new();
    for map in maps {
        for area in &map.areas {
            areas.entry(area.key).or_insert(*area);
        }
    }
    areas
}

fn world_position(area: &SpawnArea, offset_x: i64, offset_z: i64) -> (f64, f64) {
    #[allow(
        clippy::cast_precision_loss,
        reason = "deslocamentos e células cabem folgados em f64"
    )]
    let (ox, oz) = (offset_x as f64, offset_z as f64);
    (
        METERS_PER_HALF_CELL.mul_add(f64::from(area.center_x_twice()), ox),
        METERS_PER_HALF_CELL.mul_add(f64::from(area.center_y_twice()), oz),
    )
}

/// Deslocamento (x, z) do mapa da reserva: o valor em que a maioria das zonas do jogo concorda.
fn calibrate(existing: &[Zone], areas: &HashMap<u32, SpawnArea>) -> Result<(i64, i64), PlanError> {
    let mut votes_x: HashMap<i64, usize> = HashMap::new();
    let mut votes_z: HashMap<i64, usize> = HashMap::new();
    let mut total = 0usize;
    for zone in existing.iter().filter(|z| is_game_written(z)) {
        let Some(area) = areas.get(&zone.key) else {
            continue;
        };
        let (base_x, base_z) = world_position(area, 0, 0);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "o deslocamento arredondado cabe em i64"
        )]
        {
            *votes_x
                .entry((f64::from(zone.x) - base_x).round() as i64)
                .or_default() += 1;
            *votes_z
                .entry((f64::from(zone.z) - base_z).round() as i64)
                .or_default() += 1;
        }
        total += 1;
    }
    if total < MIN_CALIBRATION_ZONES {
        return Err(PlanError::NotEnoughGameZones);
    }
    let best = |votes: &HashMap<i64, usize>| {
        votes
            .iter()
            .max_by_key(|(offset, count)| (**count, std::cmp::Reverse(**offset)))
            .map(|(offset, count)| (*offset, *count))
    };
    let (Some((x, x_count)), Some((z, z_count))) = (best(&votes_x), best(&votes_z)) else {
        return Err(PlanError::NotEnoughGameZones);
    };
    #[allow(
        clippy::cast_precision_loss,
        reason = "contagens pequenas cabem em f64"
    )]
    let agreement = |count: usize| count as f64 / total as f64;
    if agreement(x_count) < MIN_CALIBRATION_AGREEMENT
        || agreement(z_count) < MIN_CALIBRATION_AGREEMENT
    {
        return Err(PlanError::InconsistentGameZones);
    }
    Ok((x, z))
}

/// Espécie de cada grupo (a que mais aparece entre as áreas da rota) e de cada chave de área
/// (a espécie dos grupos que mais a usam).
pub(crate) fn assign_species(
    groups: &[Group],
    maps: &[SpawnMap],
) -> (Vec<Option<String>>, HashMap<u32, String>) {
    let mut species_of_key: HashMap<u32, Vec<&str>> = HashMap::new();
    for map in maps {
        for area in &map.areas {
            species_of_key
                .entry(area.key)
                .or_default()
                .push(map.species.as_str());
        }
    }
    let mut group_species = Vec::with_capacity(groups.len());
    let mut key_votes: HashMap<u32, BTreeMap<String, usize>> = HashMap::new();
    for group in groups {
        let mut votes: BTreeMap<&str, usize> = BTreeMap::new();
        for key in &group.route {
            for species in species_of_key.get(key).into_iter().flatten() {
                *votes.entry(species).or_default() += 1;
            }
        }
        let winner = top(&votes).map(str::to_owned);
        if let Some(winner) = &winner {
            for key in &group.route {
                *key_votes
                    .entry(*key)
                    .or_default()
                    .entry(winner.clone())
                    .or_default() += 1;
            }
        }
        group_species.push(winner);
    }
    let mut key_species: HashMap<u32, String> = key_votes
        .into_iter()
        .filter_map(|(key, votes)| {
            top(&votes.iter().map(|(s, n)| (s.as_str(), *n)).collect()).map(|s| (key, s.to_owned()))
        })
        .collect();
    // Áreas que nenhum grupo usa, mas que só existem no mapa de uma espécie, são dessa espécie.
    for (key, species) in &species_of_key {
        if let (false, [only]) = (key_species.contains_key(key), species.as_slice()) {
            key_species.insert(*key, (*only).to_owned());
        }
    }
    (group_species, key_species)
}

/// Item com mais votos; empate vai para o menor (ordem alfabética), para o resultado ser estável.
fn top<'a>(votes: &BTreeMap<&'a str, usize>) -> Option<&'a str> {
    votes
        .iter()
        .max_by_key(|(name, count)| (**count, std::cmp::Reverse(**name)))
        .map(|(name, _)| *name)
}

/// Constante da espécie (campo `species_name`) mais comum nas zonas gravadas pelo jogo.
fn species_constants(
    game_zones: &[&Zone],
    key_species: &HashMap<u32, String>,
) -> HashMap<String, u32> {
    let mut votes: HashMap<&str, BTreeMap<u32, usize>> = HashMap::new();
    for zone in game_zones {
        if let Some(species) = key_species.get(&zone.key) {
            *votes
                .entry(species)
                .or_default()
                .entry(zone.species_name)
                .or_default() += 1;
        }
    }
    votes
        .into_iter()
        .filter_map(|(species, counts)| {
            counts
                .into_iter()
                .max_by_key(|(value, count)| (*count, std::cmp::Reverse(*value)))
                .map(|(value, _)| (species.to_owned(), value))
        })
        .collect()
}

/// Horários reais por (espécie, tipo): os vistos ao menos 2 vezes e em 20% ou mais do mais comum,
/// do mais usado para o menos usado.
fn slot_table(
    game_zones: &[&Zone],
    key_species: &HashMap<u32, String>,
) -> HashMap<(String, u32), Vec<(Slot, usize)>> {
    let mut counts: HashMap<(String, u32), SlotCounts> = HashMap::new();
    for zone in game_zones {
        let Some(species) = key_species.get(&zone.key) else {
            continue;
        };
        let slot = Slot::of(zone);
        counts
            .entry((species.clone(), zone.kind))
            .or_default()
            .entry(slot.id())
            .or_insert((slot, 0))
            .1 += 1;
    }
    counts
        .into_iter()
        .map(|(key, by_slot)| {
            let mut list: Vec<(Slot, usize)> = by_slot.into_values().collect();
            let max = list.iter().map(|(_, n)| *n).max().unwrap_or(0);
            let keep: Vec<(Slot, usize)> = list
                .iter()
                .filter(|(_, n)| *n >= 2 && *n * 5 >= max)
                .copied()
                .collect();
            if !keep.is_empty() {
                list = keep;
            }
            list.sort_by(|a, b| {
                b.1.cmp(&a.1)
                    .then(a.0.start.total_cmp(&b.0.start))
                    .then(a.0.end.total_cmp(&b.0.end))
            });
            (key, list)
        })
        .collect()
}

/// Horas de `from` até `to` andando para a frente no relógio de 24 h.
fn forward(from: f32, to: f32) -> f32 {
    (to - from).rem_euclid(DAY)
}

/// Escolhe o horário da parada `index` da rota entre os horários reais da espécie.
///
/// Devolve o horário e se ele coube entre as paradas conhecidas vizinhas (senão, usa o mais comum
/// da espécie). `None` quando a espécie não tem nenhum horário conhecido para esse tipo.
fn choose_slot(
    group: &Group,
    index: usize,
    known: &HashMap<u32, Slot>,
    candidates: &[(Slot, usize)],
) -> Option<(Slot, bool)> {
    let len = group.route.len();
    let neighbour = |back: bool| {
        (1..len).find_map(|distance| {
            let at = if back {
                (index + len - distance) % len
            } else {
                (index + distance) % len
            };
            known.get(&group.route[at]).copied()
        })
    };
    let (before, after) = (neighbour(true), neighbour(false));
    let fits = |slot: &Slot| match (before, after) {
        (Some(before), Some(after)) => {
            // Uma única parada conhecida dá a volta inteira: qualquer horário cabe depois dela.
            let span = match forward(before.start, after.start) {
                s if s == 0.0 && len > 1 => DAY,
                s => s,
            };
            forward(before.start, slot.start) <= span
        }
        _ => true,
    };
    // O índice do horário gravado pelo jogo segue a posição da parada na rota: dá a primeira escolha.
    let by_position = candidates
        .iter()
        .map(|(slot, _)| slot)
        .find(|slot| usize::try_from(slot.schedule) == Ok(index) && fits(slot));
    let fitting = by_position.or_else(|| {
        candidates
            .iter()
            .map(|(slot, _)| slot)
            .find(|slot| fits(slot))
    });
    match (fitting, candidates.first()) {
        (Some(slot), _) => Some((*slot, true)),
        (None, Some((slot, _))) => Some((*slot, false)),
        (None, None) => None,
    }
}

/// Altura estimada pela média das zonas do jogo mais próximas (nunca 1,0).
fn estimate_height(position: (f64, f64), game_zones: &[&Zone]) -> Option<f64> {
    let mut near: Vec<(f64, f64)> = game_zones
        .iter()
        .map(|z| {
            let dx = f64::from(z.x) - position.0;
            let dz = f64::from(z.z) - position.1;
            (dx.hypot(dz), f64::from(z.y))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Sem vizinhas por perto, vale a altura da mais próxima: melhor que 1,0, que o jogo não aceita bem.
    let closest = near.first()?;
    let within: Vec<&(f64, f64)> = near
        .iter()
        .take(HEIGHT_NEIGHBOURS)
        .filter(|(distance, _)| *distance <= NEAR_RADIUS)
        .collect();
    if within.is_empty() {
        return Some(closest.1);
    }
    // Média ponderada pelo inverso da distância: o vizinho colado pesa mais.
    let (weighted, weights) = within.iter().fold((0.0, 0.0), |(sum, total), (d, y)| {
        let weight = 1.0 / (d + 1.0);
        (weight.mul_add(*y, sum), total + weight)
    });
    Some(weighted / weights)
}

/// Id de ícone novo e único: sai de um embaralhamento fixo da reserva e da chave, para o resultado
/// ser o mesmo a cada cálculo.
fn fresh_icon_id(reserve: u32, key: u32, used: &mut HashSet<u64>) -> u64 {
    let mut state = (u64::from(reserve) << 32 | u64::from(key)) ^ 0x9E37_79B9_7F4A_7C15;
    loop {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        if z != 0 && used.insert(z) {
            return z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFFSET_X: i64 = -12_272;
    const OFFSET_Z: i64 = -12_784;
    const SPECIES_CONSTANT: u32 = 966_469_536;

    fn area(key: u32, kind: u32, cell: u32) -> SpawnArea {
        SpawnArea {
            key,
            kind,
            min_x: cell,
            max_x: cell + 2,
            min_y: cell,
            max_y: cell + 2,
        }
    }

    /// Zona como o jogo grava: no centro da área, com altura de verdade.
    fn game_zone(area: &SpawnArea, hours: (f32, f32), schedule: u32) -> Zone {
        let (x, z) = world_position(area, OFFSET_X, OFFSET_Z);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            reason = "valores pequenos de teste"
        )]
        let mut zone = Zone::new(
            (x as f32, 100.0 + area.key as f32, z as f32),
            area.key,
            area.kind,
            u64::from(area.key) + 1,
            hours,
            SPECIES_CONSTANT,
            schedule,
        );
        zone.discovered_groups.push(1);
        zone
    }

    /// Reserva de teste: 3 zonas do jogo (comer 4-8, comer 4-8, beber 8-12) e uma rota com 2 áreas
    /// novas (comer, beber) entre elas.
    struct Fixture {
        maps: Vec<SpawnMap>,
        groups: Vec<Group>,
        existing: Vec<Zone>,
    }

    fn fixture() -> Fixture {
        let eat_a = area(10, 1, 20);
        let eat_b = area(11, 1, 24);
        let drink = area(12, 2, 28);
        let new_eat = area(13, 1, 32);
        let new_drink = area(14, 2, 36);
        let maps = vec![SpawnMap {
            species: "moose".into(),
            areas: vec![eat_a, eat_b, drink, new_eat, new_drink],
        }];
        let existing = vec![
            game_zone(&eat_a, (4.0, 8.0), 1),
            game_zone(&eat_b, (4.0, 8.0), 1),
            game_zone(&drink, (8.0, 12.0), 2),
        ];
        let groups = vec![Group::new(vec![10, 13, 12, 14, 11])];
        Fixture {
            maps,
            groups,
            existing,
        }
    }

    fn plan(f: &Fixture) -> Result<ReservePlan, PlanError> {
        plan_reserve(&ReserveInput {
            reserve: 2,
            groups: &f.groups,
            maps: &f.maps,
            existing: &f.existing,
        })
    }

    #[test]
    fn creates_the_missing_zones_at_the_center_of_their_areas() {
        let f = fixture();
        let result = plan(&f).unwrap();
        let keys: Vec<u32> = result.new_zones.iter().map(|z| z.key).collect();
        assert_eq!(keys, [13, 14]);
        let zone = &result.new_zones[0];
        // área 13: células 32..34 nos dois eixos → centro 33 → 33 * 32 + deslocamento
        assert!((f64::from(zone.x) - (33.0 * 32.0 - 12_272.0)).abs() < 1e-3);
        assert!((f64::from(zone.z) - (33.0 * 32.0 - 12_784.0)).abs() < 1e-3);
        assert_eq!(zone.kind, 1);
        assert_eq!(zone.species_name, SPECIES_CONSTANT);
        assert_eq!(zone.discovered_groups, Vec::<u32>::new());
    }

    #[test]
    fn height_comes_from_neighbours_and_is_never_the_placeholder() {
        let f = fixture();
        for zone in plan(&f).unwrap().new_zones {
            assert!(zone.y > 50.0, "altura {}", zone.y);
            assert_ne!(zone.y.to_bits(), PLACEHOLDER_HEIGHT.to_bits());
        }
    }

    #[test]
    fn time_slots_are_real_ones_of_the_species() {
        let f = fixture();
        let result = plan(&f).unwrap();
        let eat = result.new_zones.iter().find(|z| z.kind == 1).unwrap();
        assert_eq!((eat.start, eat.end, eat.schedule), (4.0, 8.0, 1));
        let drink = result.new_zones.iter().find(|z| z.kind == 2).unwrap();
        assert_eq!((drink.start, drink.end, drink.schedule), (8.0, 12.0, 2));
    }

    #[test]
    fn zones_already_in_the_save_are_not_added_again() {
        let f = fixture();
        let result = plan(&f).unwrap();
        assert!(
            result
                .new_zones
                .iter()
                .all(|z| ![10, 11, 12].contains(&z.key))
        );
        // Calcular de novo com as zonas já acrescentadas não acha nada a liberar.
        let mut again = fixture();
        again.existing.extend(result.new_zones);
        assert_eq!(plan(&again).unwrap().new_zones, []);
    }

    #[test]
    fn the_result_is_the_same_every_time() {
        let f = fixture();
        assert_eq!(plan(&f).unwrap(), plan(&f).unwrap());
    }

    #[test]
    fn icon_ids_are_unique_and_do_not_clash_with_existing_ones() {
        let f = fixture();
        let result = plan(&f).unwrap();
        let mut ids: HashSet<u64> = f.existing.iter().map(|z| z.icon_id).collect();
        for zone in &result.new_zones {
            assert!(ids.insert(zone.icon_id));
        }
    }

    #[test]
    fn a_time_slot_must_fit_between_the_neighbouring_known_stops() {
        // Horários de comer da espécie: 4-8 (3 zonas), 8-12 (2) e 16-20 (2). O mais comum é o 4-8,
        // mas a parada nova fica entre uma parada às 8 e outra às 16: só 8-12 e 16-20 cabem.
        let cells = [
            (10, 20),
            (11, 22),
            (15, 26),
            (20, 28),
            (22, 30),
            (21, 32),
            (23, 34),
            (13, 24),
        ];
        let areas: Vec<SpawnArea> = cells
            .iter()
            .map(|(key, cell)| area(*key, 1, *cell))
            .collect();
        let by_key = |key: u32| *areas.iter().find(|a| a.key == key).unwrap();
        let existing = vec![
            game_zone(&by_key(10), (4.0, 8.0), 1),
            game_zone(&by_key(11), (4.0, 8.0), 1),
            game_zone(&by_key(15), (4.0, 8.0), 1),
            game_zone(&by_key(20), (8.0, 12.0), 2),
            game_zone(&by_key(22), (8.0, 12.0), 2),
            game_zone(&by_key(21), (16.0, 20.0), 4),
            game_zone(&by_key(23), (16.0, 20.0), 4),
        ];
        let maps = vec![SpawnMap {
            species: "moose".into(),
            areas,
        }];
        let groups = vec![Group::new(vec![20, 13, 21])];
        let result = plan_reserve(&ReserveInput {
            reserve: 2,
            groups: &groups,
            maps: &maps,
            existing: &existing,
        })
        .unwrap();
        let zone = &result.new_zones[0];
        assert_eq!((zone.start, zone.end), (8.0, 12.0));
        assert_eq!(result.estimated, 0);
    }

    #[test]
    fn a_slot_that_fits_nowhere_falls_back_to_the_most_common_and_is_marked_estimated() {
        // Só existem os horários 4-8 e 12-16; a parada nova fica entre uma parada às 10 e outra às 11.
        let cells = [(10, 20), (11, 22), (12, 24), (13, 26), (14, 28)];
        let areas: Vec<SpawnArea> = cells
            .iter()
            .map(|(key, cell)| area(*key, 1, *cell))
            .collect();
        let by_key = |key: u32| *areas.iter().find(|a| a.key == key).unwrap();
        let existing = vec![
            game_zone(&by_key(10), (4.0, 8.0), 1),
            game_zone(&by_key(11), (4.0, 8.0), 1),
            game_zone(&by_key(12), (10.0, 11.0), 3),
            game_zone(&by_key(14), (11.0, 14.0), 3),
        ];
        let maps = vec![SpawnMap {
            species: "moose".into(),
            areas,
        }];
        let groups = vec![Group::new(vec![12, 13, 14])];
        let result = plan_reserve(&ReserveInput {
            reserve: 2,
            groups: &groups,
            maps: &maps,
            existing: &existing,
        })
        .unwrap();
        assert_eq!(result.new_zones.len(), 1);
        assert_eq!(result.estimated, 1);
    }

    #[test]
    fn species_without_known_slots_are_listed_instead_of_guessed() {
        let mut f = fixture();
        f.maps.push(SpawnMap {
            species: "lynx".into(),
            areas: vec![area(20, 1, 150)],
        });
        f.groups.push(Group::new(vec![20]));
        let result = plan(&f).unwrap();
        assert_eq!(result.no_schedule, 1);
        assert!(result.new_zones.iter().all(|z| z.key != 20));
    }

    #[test]
    fn route_keys_missing_from_the_game_maps_are_counted() {
        let mut f = fixture();
        f.groups[0].route.push(9999);
        assert_eq!(plan(&f).unwrap().unknown_areas, 1);
    }

    #[test]
    fn too_few_game_zones_cannot_calibrate_the_map() {
        let mut f = fixture();
        f.existing.truncate(2);
        assert_eq!(plan(&f), Err(PlanError::NotEnoughGameZones));
    }

    #[test]
    fn game_zones_that_disagree_cannot_calibrate_the_map() {
        let mut f = fixture();
        f.existing[0].x += 640.0;
        f.existing[1].x -= 640.0;
        assert_eq!(plan(&f), Err(PlanError::InconsistentGameZones));
    }

    #[test]
    fn placeholder_height_zones_are_not_used_for_calibration() {
        let mut f = fixture();
        for zone in &mut f.existing {
            zone.y = PLACEHOLDER_HEIGHT;
        }
        assert_eq!(plan(&f), Err(PlanError::NotEnoughGameZones));
    }

    #[test]
    fn a_zone_far_from_every_reference_takes_the_height_of_the_nearest_game_zone() {
        let mut f = fixture();
        f.maps[0].areas.push(SpawnArea {
            key: 50,
            kind: 1,
            min_x: 250,
            max_x: 252,
            min_y: 250,
            max_y: 252,
        });
        f.groups[0].route.push(50);
        let result = plan(&f).unwrap();
        let far = result.new_zones.iter().find(|z| z.key == 50).unwrap();
        // A zona do jogo mais próxima é a de chave 12 (altura 100 + chave).
        assert!((far.y - 112.0).abs() < 1e-3, "altura {}", far.y);
    }

    #[test]
    fn forward_wraps_around_midnight() {
        assert!((forward(22.0, 2.0) - 4.0).abs() < f32::EPSILON);
        assert!(forward(5.0, 5.0).abs() < f32::EPSILON);
    }
}
