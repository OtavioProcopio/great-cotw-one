//! Perfil do jogador e progresso das missões (spec `career-progress`): tudo lido do save, dos
//! arquivos do jogo e do cache de conquistas da Steam, sem tabelas fixas por reserva.

use std::path::Path;

use cotw_formats::adf::{Adf, Value};
use cotw_formats::archive::GameArchives;
use cotw_formats::missions::{
    self, MissionDef, MissionProgress, State, StringLookup, missions_file_number,
};
use cotw_formats::steam_kv::{self, Kv};
use serde::Serialize;

use crate::discovery::reserve_name;
use crate::language::Language;
use crate::mapview::read_adf;

/// Diferença entre o Steam ID de 64 bits e o número da conta usado nos arquivos da Steam.
const STEAM_ID_BASE: u64 = 76_561_197_960_265_728;
/// Número da Steam do jogo.
const APP_ID: u32 = 518_790;

/// Erros ao montar o perfil ou as missões.
#[derive(Debug, thiserror::Error)]
pub enum CareerError {
    /// Falha ao ler um arquivo do save.
    #[error(transparent)]
    Save(#[from] crate::mapview::MapError),
    /// O arquivo do save não tem a estrutura esperada.
    #[error("{0} não pôde ser interpretado")]
    Parse(String),
}

/// Uma conquista da Steam com o estado na conta do jogador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementView {
    /// Nome interno.
    pub id: String,
    /// Nome para exibir.
    pub name: String,
    /// Como obter.
    pub description: String,
    /// Escondida até ser obtida.
    pub hidden: bool,
    /// Já obtida.
    pub unlocked: bool,
}

/// Perfil do jogador: nível, dinheiro, tiros e conquistas.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    /// Número da conta Steam.
    pub steam_id: String,
    /// Nível do caçador.
    pub level: u64,
    /// Pontos de experiência.
    pub xp: u64,
    /// Dinheiro.
    pub cash: u64,
    /// Tiros disparados com todas as armas.
    pub shots_fired: u64,
    /// Tiros que erraram.
    pub shots_missed: u64,
    /// Animais abatidos.
    pub total_kills: u64,
    /// Abates com um tiro só.
    pub one_shot_kills: u64,
    /// Maior pontuação de um abate.
    pub record_score: f64,
    /// Maior distância de um tiro em órgão vital, em metros.
    pub furthest_shot: f64,
    /// Troféus no salão.
    pub trophies: usize,
    /// Reservas com progresso no save.
    pub reserves: usize,
    /// Conquistas (nenhuma quando a Steam não tem os arquivos neste computador).
    pub achievements: Vec<AchievementView>,
    /// A lista de conquistas veio da Steam deste computador.
    pub has_achievements: bool,
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn items(value: Option<&Value>) -> &[Value] {
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

pub(crate) fn first_instance(adf: &[u8], name: &str) -> Result<Value, CareerError> {
    let parsed = Adf::parse(adf.to_vec()).map_err(|_| CareerError::Parse(name.to_owned()))?;
    let instance = parsed
        .instances()
        .first()
        .ok_or_else(|| CareerError::Parse(name.to_owned()))?;
    parsed
        .read_instance(instance)
        .map_err(|_| CareerError::Parse(name.to_owned()))
}

/// Número da conta usado nos arquivos da Steam, a partir do Steam ID de 64 bits.
#[must_use]
pub fn account_id(steam_id: &str) -> Option<u32> {
    u32::try_from(steam_id.parse::<u64>().ok()?.checked_sub(STEAM_ID_BASE)?).ok()
}

/// Conquistas da Steam com o estado da conta, ou `None` se os arquivos não estiverem no computador.
fn achievements(
    stats_dir: &Path,
    steam_id: &str,
    language: Language,
) -> Option<Vec<AchievementView>> {
    let schema = Kv::parse(
        &std::fs::read(stats_dir.join(format!("UserGameStatsSchema_{APP_ID}.bin"))).ok()?,
    )
    .ok()?;
    let infos = steam_kv::achievements(&schema, language.steam_key());
    if infos.is_empty() {
        return None;
    }
    let masks = account_id(steam_id)
        .and_then(|id| {
            std::fs::read(stats_dir.join(format!("UserGameStats_{id}_{APP_ID}.bin"))).ok()
        })
        .and_then(|bytes| Kv::parse(&bytes).ok())
        .map(|user| steam_kv::unlocked_masks(&user))
        .unwrap_or_default();
    Some(
        infos
            .into_iter()
            .map(|info| {
                let unlocked = masks
                    .iter()
                    .find(|(stat, _)| *stat == info.stat)
                    .is_some_and(|(_, mask)| info.bit < 32 && mask >> info.bit & 1 == 1);
                AchievementView {
                    id: info.api_name,
                    name: info.name,
                    description: info.description,
                    hidden: info.hidden && !unlocked,
                    unlocked,
                }
            })
            .collect(),
    )
}

/// Monta o perfil do jogador.
///
/// `stats_dir` é a pasta `appcache/stats` da Steam; sem ela, o perfil vem sem conquistas.
///
/// # Errors
///
/// [`CareerError`] se o perfil do jogador não puder ser lido.
pub fn profile(
    save_dir: &Path,
    steam_id: &str,
    stats_dir: Option<&Path>,
    language: Language,
) -> Result<ProfileView, CareerError> {
    const PROFILE: &str = "thp_player_profile_adf";
    let root = first_instance(&read_adf(&save_dir.join(PROFILE), PROFILE)?, PROFILE)?;
    let (mut fired, mut missed, mut kills, mut one_shot) = (0, 0, 0, 0);
    let (mut record, mut furthest) = (0.0_f64, 0.0_f64);
    for weapon in items(member(&root, "WeaponData")) {
        fired += unsigned(member(weapon, "ShotsFired"));
        missed += unsigned(member(weapon, "ShotsMissed"));
        kills += unsigned(member(weapon, "TotalKills"));
        one_shot += unsigned(member(weapon, "TotalOneShotKills"));
        record = record.max(float(member(weapon, "RecordKillScore")));
        furthest = furthest.max(float(member(weapon, "FurthestVitalOrganShotDistance")));
    }
    let trophies = read_adf(&save_dir.join("trophy_lodges_adf"), "trophy_lodges_adf")
        .ok()
        .and_then(|adf| first_instance(&adf, "trophy_lodges_adf").ok())
        .map_or(0, |lodge| {
            items(member(
                member(&lodge, "TrophyAnimals").unwrap_or(&lodge),
                "Trophies",
            ))
            .len()
        });
    let reserves = std::fs::read_dir(save_dir).map_or(0, |dir| {
        dir.filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("animal_population_")
            })
            .count()
    });
    let list = stats_dir.and_then(|dir| achievements(dir, steam_id, language));
    Ok(ProfileView {
        steam_id: steam_id.to_owned(),
        level: unsigned(member(&root, "Level")),
        xp: unsigned(member(&root, "Xp")),
        cash: unsigned(member(&root, "Cash")),
        shots_fired: fired,
        shots_missed: missed,
        total_kills: kills,
        one_shot_kills: one_shot,
        record_score: record,
        furthest_shot: furthest,
        trophies,
        reserves,
        has_achievements: list.is_some(),
        achievements: list.unwrap_or_default(),
    })
}

/// Um objetivo de uma missão.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveView {
    /// Quantas vezes já foi cumprido.
    pub done: i64,
    /// Quantas vezes é preciso cumpri-lo.
    pub required: i64,
}

/// Uma missão de uma reserva.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionView {
    /// Número da missão.
    pub id: i64,
    /// O que fazer, no texto do jogo (vazio se o jogo não trouxer).
    pub text: String,
    /// `done`, `active` ou `locked`.
    pub state: &'static str,
    /// Faz parte da história principal.
    pub main: bool,
    /// Objetivos com o andamento de cada um.
    pub objectives: Vec<ObjectiveView>,
}

/// Progresso das missões de uma reserva.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveMissions {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
    /// Missões da reserva.
    pub total: usize,
    /// Concluídas.
    pub done: usize,
    /// Em andamento.
    pub active: usize,
    /// Da história principal.
    pub main_total: usize,
    /// Da história principal já concluídas.
    pub main_done: usize,
    /// Lista completa, em ordem de número.
    pub missions: Vec<MissionView>,
}

fn state_name(state: State) -> &'static str {
    match state {
        State::Done => "done",
        State::Active => "active",
        State::Locked => "locked",
    }
}

fn mission_view(
    def: &MissionDef,
    progress: Option<&MissionProgress>,
    lookup: &StringLookup,
) -> MissionView {
    let text = [def.summary, def.description]
        .into_iter()
        .find_map(|hash| lookup.get(hash & 0xFFFF_FFFF))
        .unwrap_or_default()
        .to_owned();
    let objectives = def
        .objectives
        .iter()
        .map(|objective| ObjectiveView {
            done: progress
                .and_then(|p| p.objectives.iter().find(|o| o.id == objective.id))
                .map_or(0, |o| o.done.min(objective.count_required)),
            required: objective.count_required,
        })
        .collect();
    MissionView {
        id: def.id,
        text,
        state: state_name(progress.map_or(State::Locked, |p| p.state)),
        main: def.required,
        objectives,
    }
}

/// Progresso das missões de todas as reservas que têm campanha no save e missões no jogo.
///
/// O arquivo de missões de cada reserva é o que a própria configuração dela indica, então reservas
/// novas (de qualquer expansão) aparecem sozinhas.
///
/// # Errors
///
/// [`CareerError`] se o arquivo de progresso das missões não puder ser lido.
pub fn missions(
    save_dir: &Path,
    archives: &GameArchives,
    language: Language,
) -> Result<Vec<ReserveMissions>, CareerError> {
    const FILE: &str = "missions_missiondata_adf";
    let campaigns = missions::parse_progress(&read_adf(&save_dir.join(FILE), FILE)?)
        .map_err(|_| CareerError::Parse(FILE.to_owned()))?;
    let mut found = Vec::new();
    for campaign in campaigns {
        let Ok(id) = u32::try_from(campaign.reserve) else {
            continue;
        };
        let Some(number) = archives
            .find_path(&format!("settings/hp_settings/reserve_{id}.bin"))
            .first()
            .and_then(|at| archives.read(at).ok())
            .and_then(|settings| missions_file_number(&settings))
        else {
            continue;
        };
        let Some(defs) = archives
            .find_path(&format!("missions/missions_{number}.hpmissionsc"))
            .first()
            .and_then(|at| archives.read(at).ok())
            .and_then(|adf| missions::parse_definitions(&adf).ok())
        else {
            continue;
        };
        let lookup = archives
            .find_path(&format!("missions/missions_{number}.stringlookup"))
            .first()
            .and_then(|at| archives.read(at).ok())
            .and_then(|adf| missions::parse_lookup(&adf, language.mission_index()).ok())
            .unwrap_or_default();
        let mut views: Vec<MissionView> = defs
            .iter()
            .map(|def| {
                let progress = campaign.missions.iter().find(|m| m.id == def.id);
                mission_view(def, progress, &lookup)
            })
            .collect();
        views.sort_by_key(|m| m.id);
        found.push(ReserveMissions {
            id,
            name: reserve_name(id),
            total: views.len(),
            done: views.iter().filter(|m| m.state == "done").count(),
            active: views.iter().filter(|m| m.state == "active").count(),
            main_total: views.iter().filter(|m| m.main).count(),
            main_done: views.iter().filter(|m| m.main && m.state == "done").count(),
            missions: views,
        });
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_account_number_is_the_steam_id_minus_the_base() {
        assert_eq!(account_id("76561198125950350"), Some(165_684_622));
        assert_eq!(account_id("12"), None);
        assert_eq!(account_id("abc"), None);
    }

    #[test]
    fn value_helpers_default_to_zero() {
        assert_eq!(unsigned(Some(&Value::Signed(-1))), 0);
        assert_eq!(unsigned(Some(&Value::Signed(4))), 4);
        assert_eq!(unsigned(Some(&Value::Float(1.0))), 0);
        assert_eq!(unsigned(None), 0);
        assert!((float(Some(&Value::Float(2.5))) - 2.5).abs() < f64::EPSILON);
        assert!(float(None).abs() < f64::EPSILON);
        assert_eq!(items(None).len(), 0);
        assert!(member(&Value::Signed(1), "x").is_none());
    }

    #[test]
    fn a_mission_with_no_progress_is_locked_with_empty_objectives() {
        let def = MissionDef {
            id: 7,
            summary: 1,
            description: 2,
            required: true,
            visible: true,
            objectives: vec![missions::ObjectiveDef {
                id: 70,
                count_required: 3,
            }],
        };
        let view = mission_view(&def, None, &StringLookup::default());
        assert_eq!(view.state, "locked");
        assert!(view.main);
        assert_eq!(view.text, "");
        assert_eq!(
            view.objectives,
            vec![ObjectiveView {
                done: 0,
                required: 3
            }]
        );
    }

    #[test]
    fn progress_is_capped_at_what_the_objective_requires() {
        let def = MissionDef {
            id: 7,
            summary: 1,
            description: 2,
            required: false,
            visible: true,
            objectives: vec![missions::ObjectiveDef {
                id: 70,
                count_required: 3,
            }],
        };
        let progress = MissionProgress {
            id: 7,
            state: State::Active,
            objectives: vec![missions::ObjectiveProgress {
                id: 70,
                done: 9,
                state: State::Active,
            }],
        };
        let view = mission_view(&def, Some(&progress), &StringLookup::default());
        assert_eq!(view.state, "active");
        assert_eq!(view.objectives[0].done, 3);
    }

    #[test]
    fn a_missing_steam_folder_means_no_achievements() {
        assert_eq!(
            achievements(Path::new("não existe"), "76561198125950350", Language::PtBr),
            None
        );
    }
}
