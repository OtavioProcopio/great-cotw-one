//! Liberar o mapa: junta o save e o jogo para decidir, reserva por reserva, quais zonas
//! de necessidade acrescentar ao `found_need_zones_adf` (spec `need-zone-unlock`).

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
use crate::transaction::{FileChange, ItemChange};
use crate::zones::{PlanError, ReserveInput, ReservePlan, plan_reserve};

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
}

/// Resultado da análise: o resumo por reserva e o necessário para gravar.
pub struct Analysis {
    /// Uma entrada por reserva do save que tem zonas.
    pub reserves: Vec<ReserveSummary>,
    plans: Vec<(u32, ReservePlan)>,
    zones: NeedZones,
    original: Vec<u8>,
    envelope: SaveEnvelope,
}

/// Lê o save e o jogo e calcula o que há para liberar em cada reserva.
///
/// # Errors
///
/// [`UnlockError`] se o `found_need_zones_adf` não puder ser lido ou não for válido.
pub fn analyze(save_dir: &Path, source: &dyn SpawnSource) -> Result<Analysis, UnlockError> {
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
        let summary = |state, new_zones, estimated, skipped| ReserveSummary {
            id,
            name: reserve_name(id),
            state,
            new_zones,
            estimated,
            skipped,
        };
        let population = fs::read(save_dir.join(format!("animal_population_{id}")))
            .ok()
            .and_then(|raw| SaveEnvelope::parse(&raw).ok())
            .and_then(|envelope| parse_groups(envelope.adf()).ok());
        let (Some(groups), Ok(maps)) = (population, source.maps(id)) else {
            reserves.push(summary(ReserveState::NoData, 0, 0, 0));
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
                reserves.push(summary(
                    state,
                    plan.new_zones.len(),
                    plan.estimated,
                    skipped,
                ));
                plans.push((id, plan));
            }
            Err(PlanError::NotEnoughGameZones | PlanError::InconsistentGameZones) => {
                reserves.push(summary(ReserveState::NeedsDiscovery, 0, 0, 0));
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
            .map(|(_, plan)| plan.new_zones.len())
            .sum()
    }

    /// Monta o novo `found_need_zones_adf` com as zonas das reservas em `selected`.
    ///
    /// Devolve `None` se nenhuma das reservas escolhidas tiver zonas a liberar.
    ///
    /// # Errors
    ///
    /// [`UnlockError`] se o arquivo novo não puder ser montado ou não passar na conferência.
    pub fn build(&self, selected: &[u32]) -> Result<Option<Unlock>, UnlockError> {
        let mut zones = self.zones.clone();
        let mut items = Vec::new();
        let mut zones_added = 0;
        for (id, plan) in &self.plans {
            if !selected.contains(id) || plan.new_zones.is_empty() {
                continue;
            }
            let Some(reserve) = zones.reserve_mut(u64::from(*id)) else {
                continue;
            };
            let before = reserve.zones.len();
            reserve.zones.extend(plan.new_zones.iter().cloned());
            zones_added += plan.new_zones.len();
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
