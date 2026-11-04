//! Liberar o mapa: junta o save e o jogo para decidir, reserva por reserva, quais zonas
//! de necessidade acrescentar ao `found_need_zones_adf` (spec `need-zone-unlock`).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use cotw_formats::archive::GameArchives;
use cotw_formats::envelope::SaveEnvelope;
use cotw_formats::need_zones::{NeedZones, NeedZonesError};
use cotw_formats::population::parse_groups;
use cotw_formats::spawn_map::{
    SpawnMap, SpawnMapError, load_cover_image, load_map_image, load_reserve,
};
use cotw_formats::texture::Image;
use serde::Serialize;

use crate::discovery::reserve_name;
use crate::language::Language;
use crate::species::display_name;
use crate::transaction::{FileChange, ItemChange};
use crate::zones::{PlanError, ReserveInput, ReservePlan, assign_species, plan_reserve};

/// Arquivo do save que guarda as zonas descobertas.
pub const ZONES_FILE: &str = "found_need_zones_adf";

/// Erros ao analisar ou montar a liberação.
#[derive(Debug, thiserror::Error)]
pub enum UnlockError {
    /// Não foi possível ler um arquivo do save.
    #[error("não foi possível ler {file}: {reason}")]
    Read {
        /// Arquivo.
        file: String,
        /// Motivo.
        reason: String,
    },
    /// O arquivo de zonas não tem a estrutura esperada.
    #[error(transparent)]
    Zones(#[from] NeedZonesError),
    /// O envelope do save é inválido.
    #[error("{ZONES_FILE} inválido")]
    Envelope,
}

/// De onde vêm os mapas de spawn (o jogo instalado; nos testes, dados de exemplo).
pub trait SpawnSource {
    /// Mapas de spawn da reserva.
    ///
    /// # Errors
    ///
    /// [`SpawnMapError`] se a reserva não tiver mapas ou eles não puderem ser lidos.
    fn maps(&self, reserve: u32) -> Result<Vec<SpawnMap>, SpawnMapError>;

    /// Imagem do mapa da reserva, se o jogo a tiver.
    fn map_image(&self, _reserve: u32) -> Option<Image> {
        None
    }

    /// Foto de capa da reserva, se o jogo a tiver.
    fn cover_image(&self, _reserve: u32) -> Option<Image> {
        None
    }
}

impl SpawnSource for GameArchives {
    fn maps(&self, reserve: u32) -> Result<Vec<SpawnMap>, SpawnMapError> {
        load_reserve(self, reserve)
    }

    fn map_image(&self, reserve: u32) -> Option<Image> {
        load_map_image(self, reserve).ok().flatten()
    }

    fn cover_image(&self, reserve: u32) -> Option<Image> {
        load_cover_image(self, reserve).ok().flatten()
    }
}

/// Zonas a liberar de uma espécie, por necessidade (comer, beber, descansar).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesNeeds {
    /// Nome que o jogo usa (vazio para zonas de espécie não identificada).
    pub id: String,
    /// Nome para exibir.
    pub name: String,
    /// Quantas zonas de comer, beber e descansar.
    pub needs: [usize; 3],
}

/// Situação de uma reserva para a interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReserveState {
    /// Há zonas a liberar.
    Ready,
    /// Todas as zonas dos grupos já estão no save.
    NothingToDo,
    /// Poucas zonas descobertas no jogo para calibrar o mapa desta reserva.
    NeedsDiscovery,
    /// Falta o arquivo da reserva no save ou os mapas dela no jogo.
    NoData,
}

/// Resumo de uma reserva.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveSummary {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
    /// Situação.
    pub state: ReserveState,
    /// Zonas que serão acrescentadas.
    pub new_zones: usize,
    /// Dessas, quantas têm horário estimado (o jogo corrige quando o jogador passa por lá).
    pub estimated: usize,
    /// Zonas que não puderam ser calculadas (sem horário ou sem referência de altura).
    pub skipped: usize,
    /// As zonas a liberar divididas por espécie e necessidade, para a interface escolher o que liberar.
    pub species: Vec<SpeciesNeeds>,
}

/// Resultado da análise: o resumo por reserva e o necessário para gravar.
pub struct Analysis {
    /// Uma entrada por reserva do save que tem zonas.
    pub reserves: Vec<ReserveSummary>,
    plans: Vec<(u32, ReservePlan, HashMap<u32, String>)>,
    zones: NeedZones,
    original: Vec<u8>,
    envelope: SaveEnvelope,
}

/// Lê o save e o jogo e calcula o que há para liberar em cada reserva.
///
/// # Errors
///
/// [`UnlockError`] se o `found_need_zones_adf` não puder ser lido ou não for válido.
pub fn analyze(
    save_dir: &Path,
    source: &dyn SpawnSource,
    language: Language,
) -> Result<Analysis, UnlockError> {
    let raw = fs::read(save_dir.join(ZONES_FILE)).map_err(|e| UnlockError::Read {
        file: ZONES_FILE.into(),
        reason: e.to_string(),
    })?;
    let envelope = SaveEnvelope::parse(&raw).map_err(|_| UnlockError::Envelope)?;
    let original = envelope.adf().to_vec();
    let zones = NeedZones::parse(&original)?;

    let mut reserves = Vec::new();
    let mut plans = Vec::new();
    for reserve in &zones.reserves {
        let Ok(id) = u32::try_from(reserve.id) else {
            continue;
        };
        let summary = |state, new_zones, estimated, skipped, species| ReserveSummary {
            id,
            name: reserve_name(id),
            state,
            new_zones,
            estimated,
            skipped,
            species,
        };
        let population = fs::read(save_dir.join(format!("animal_population_{id}")))
            .ok()
            .and_then(|raw| SaveEnvelope::parse(&raw).ok())
            .and_then(|envelope| parse_groups(envelope.adf()).ok());
        let (Some(groups), Ok(maps)) = (population, source.maps(id)) else {
            reserves.push(summary(ReserveState::NoData, 0, 0, 0, Vec::new()));
            continue;
        };
        match plan_reserve(&ReserveInput {
            reserve: id,
            groups: &groups,
            maps: &maps,
            existing: &reserve.zones,
        }) {
            Ok(plan) => {
                let skipped = plan.no_schedule + plan.no_height;
                let state = if plan.new_zones.is_empty() {
                    ReserveState::NothingToDo
                } else {
                    ReserveState::Ready
                };
                let (_, key_species) = assign_species(&groups, &maps);
                reserves.push(summary(
                    state,
                    plan.new_zones.len(),
                    plan.estimated,
                    skipped,
                    species_needs(&plan, &key_species, language),
                ));
                plans.push((id, plan, key_species));
            }
            Err(PlanError::NotEnoughGameZones | PlanError::InconsistentGameZones) => {
                reserves.push(summary(ReserveState::NeedsDiscovery, 0, 0, 0, Vec::new()));
            }
        }
    }
    Ok(Analysis {
        reserves,
        plans,
        zones,
        original,
        envelope,
    })
}

/// Zonas novas de um plano, por espécie e necessidade, em ordem alfabética do nome.
fn species_needs(
    plan: &ReservePlan,
    key_species: &HashMap<u32, String>,
    language: Language,
) -> Vec<SpeciesNeeds> {
    let mut counts: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for zone in &plan.new_zones {
        let species = key_species.get(&zone.key).cloned().unwrap_or_default();
        if let Some(slot) = zone
            .kind
            .checked_sub(1)
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| counts.entry(species).or_default().get_mut(i))
        {
            *slot += 1;
        }
    }
    let mut list: Vec<SpeciesNeeds> = counts
        .into_iter()
        .map(|(id, needs)| SpeciesNeeds {
            name: if id.is_empty() {
                String::new()
            } else {
                display_name(&id, language)
            },
            id,
            needs,
        })
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

/// O que liberar: reservas e, se quiser, só algumas espécies e/ou só algumas necessidades.
#[derive(Debug, Clone, Copy)]
pub struct Selection<'a> {
    /// Reservas escolhidas.
    pub reserves: &'a [u32],
    /// Só estas espécies (`None` libera todas, inclusive as de espécie não identificada).
    pub species: Option<&'a [String]>,
    /// Só estas necessidades (1 comer, 2 beber, 3 descansar); `None` libera todas.
    pub needs: Option<&'a [u32]>,
}

impl Selection<'_> {
    fn allows(
        &self,
        zone: &cotw_formats::need_zones::Zone,
        key_species: &HashMap<u32, String>,
    ) -> bool {
        let species_ok = self.species.is_none_or(|wanted| {
            key_species
                .get(&zone.key)
                .is_some_and(|species| wanted.contains(species))
        });
        let need_ok = self.needs.is_none_or(|wanted| wanted.contains(&zone.kind));
        species_ok && need_ok
    }
}

/// O que gravar para liberar as reservas escolhidas.
pub struct Unlock {
    /// Arquivo pronto para a gravação segura.
    pub file: FileChange,
    /// Zonas acrescentadas.
    pub zones_added: usize,
    /// Reservas que ganharam zonas.
    pub reserves: usize,
}

impl Analysis {
    /// Total de zonas novas em todas as reservas.
    #[must_use]
    pub fn total_new_zones(&self) -> usize {
        self.plans
            .iter()
            .map(|(_, plan, _)| plan.new_zones.len())
            .sum()
    }

    /// Monta o novo `found_need_zones_adf` com as zonas da seleção (reservas, espécies e necessidades).
    ///
    /// Devolve `None` se a seleção não tiver nenhuma zona a liberar.
    ///
    /// # Errors
    ///
    /// [`UnlockError`] se o arquivo novo não puder ser montado ou não passar na conferência.
    pub fn build(&self, selection: &Selection<'_>) -> Result<Option<Unlock>, UnlockError> {
        let mut zones = self.zones.clone();
        let mut items = Vec::new();
        let mut zones_added = 0;
        for (id, plan, key_species) in &self.plans {
            if !selection.reserves.contains(id) {
                continue;
            }
            let chosen: Vec<_> = plan
                .new_zones
                .iter()
                .filter(|zone| selection.allows(zone, key_species))
                .cloned()
                .collect();
            if chosen.is_empty() {
                continue;
            }
            let Some(reserve) = zones.reserve_mut(u64::from(*id)) else {
                continue;
            };
            let before = reserve.zones.len();
            zones_added += chosen.len();
            reserve.zones.extend(chosen);
            items.push(ItemChange {
                label: reserve_name(*id),
                before: format!("{before} zonas"),
                after: format!("{} zonas", reserve.zones.len()),
            });
        }
        if zones_added == 0 {
            return Ok(None);
        }
        let adf = zones.to_adf(&self.original)?;
        let mut envelope = self.envelope.clone();
        envelope.set_adf(adf);
        let new_content = envelope.to_bytes().map_err(|_| UnlockError::Envelope)?;
        let expected = zones;
        let reserves = items.len();
        Ok(Some(Unlock {
            file: FileChange {
                name: ZONES_FILE.into(),
                new_content,
                items,
                // Relê o arquivo gravado no disco: tem de ter exatamente as zonas previstas.
                verify: Some(Box::new(move |bytes| {
                    let again = SaveEnvelope::parse(bytes).map_err(|e| e.to_string())?;
                    let reread = NeedZones::parse(again.adf()).map_err(|e| e.to_string())?;
                    if reread == expected {
                        Ok(())
                    } else {
                        Err("as zonas gravadas diferem das previstas".into())
                    }
                })),
            },
            zones_added,
            reserves,
        }))
    }
}
