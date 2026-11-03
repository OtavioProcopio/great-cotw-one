//! Reconhecimento dos arquivos de uma pasta de save e das reservas presentes.

use std::fs;
use std::io;
use std::path::Path;

/// Arquivos de save com nome fixo.
const KNOWN_NAMES: [&str; 20] = [
    "achievements_adf",
    "achievements_progress_missiondata_adf",
    "codex_seen_status_adf",
    "collectibles_adf",
    "contextual_help_adf",
    "found_need_zones_adf",
    "health_component_manager_adf",
    "hunting_log_adf",
    "leaderboard_adf",
    "missions_missiondata_adf",
    "playerinformation_adf",
    "regions_adf",
    "reserveworlddata_adf",
    "settings_adf",
    "socialconfig",
    "statistics_adf",
    "thp_dog_profile_adf",
    "thp_player_profile_adf",
    "trophy_lodges_adf",
    "worlditemsdata_adf",
];

/// Reservas conhecidas (sufixo `_N` dos arquivos por reserva).
const RESERVES: [(u32, &str); 19] = [
    (0, "Hirschfelden"),
    (1, "Layton Lake"),
    (2, "Medved-Taiga"),
    (3, "Vurhonga Savanna"),
    (4, "Parque Fernando"),
    (6, "Yukon Valley"),
    (8, "Cuatro Colinas"),
    (9, "Silver Ridge Peaks"),
    (10, "Te Awaroa"),
    (11, "Rancho del Arroyo"),
    (12, "Mississippi Acres"),
    (13, "Revontuli Coast"),
    (14, "New England Mountains"),
    (16, "Emerald Coast"),
    (17, "Sundarpatan"),
    (18, "Salzwiesen Park"),
    (19, "Askiy Ridge"),
    (20, "Tòrr nan Sithean"),
    (21, "Intisuyu"),
];

/// O que um arquivo da pasta de save é.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFileKind {
    /// `animal_population_N`: animais e grupos da reserva N.
    Population(u32),
    /// `found_icons_N`: ícones descobertos na reserva N.
    FoundIcons(u32),
    /// `adf_fog_of_war_data_faster_N`: névoa de guerra da reserva N.
    FogOfWar(u32),
    /// Arquivo de nome fixo conhecido.
    Known,
    /// Qualquer outro arquivo: é mostrado como não reconhecido e nunca editado.
    Unrecognized,
}

/// Reserva presente no save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reserve {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir ("Reserva N" quando o número for desconhecido).
    pub name: String,
}

/// Nome de exibição da reserva `id`.
#[must_use]
pub fn reserve_name(id: u32) -> String {
    RESERVES
        .iter()
        .find(|(known, _)| *known == id)
        .map_or_else(|| format!("Reserva {id}"), |(_, name)| (*name).to_owned())
}

/// Classifica um arquivo pelo nome.
#[must_use]
pub fn classify(name: &str) -> SaveFileKind {
    let numbered = |prefix: &str| -> Option<u32> {
        let digits = name.strip_prefix(prefix)?;
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        digits.parse().ok()
    };
    if let Some(id) = numbered("animal_population_") {
        SaveFileKind::Population(id)
    } else if let Some(id) = numbered("found_icons_") {
        SaveFileKind::FoundIcons(id)
    } else if let Some(id) = numbered("adf_fog_of_war_data_faster_") {
        SaveFileKind::FogOfWar(id)
    } else if KNOWN_NAMES.contains(&name) {
        SaveFileKind::Known
    } else {
        SaveFileKind::Unrecognized
    }
}

/// Reservas presentes na pasta (pelos `animal_population_N`), em ordem de número.
///
/// # Errors
///
/// Retorna o erro de E/S se a pasta não puder ser lida.
pub fn list_reserves(save_dir: &Path) -> io::Result<Vec<Reserve>> {
    let mut ids: Vec<u32> = fs::read_dir(save_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| match classify(&e.file_name().to_string_lossy()) {
            SaveFileKind::Population(id) => Some(id),
            _ => None,
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids
        .into_iter()
        .map(|id| Reserve {
            id,
            name: reserve_name(id),
        })
        .collect())
}

/// Pelo menos um arquivo da pasta é de save do jogo (reconhecido pelo nome).
pub(crate) fn looks_like_save_dir(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .filter_map(Result::ok)
            .any(|e| classify(&e.file_name().to_string_lossy()) != SaveFileKind::Unrecognized)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_numbered_and_fixed_names() {
        assert_eq!(classify("animal_population_2"), SaveFileKind::Population(2));
        assert_eq!(classify("found_icons_17"), SaveFileKind::FoundIcons(17));
        assert_eq!(
            classify("adf_fog_of_war_data_faster_9"),
            SaveFileKind::FogOfWar(9)
        );
        assert_eq!(classify("found_need_zones_adf"), SaveFileKind::Known);
        assert_eq!(classify("socialconfig"), SaveFileKind::Known);
    }

    #[test]
    fn everything_else_is_unrecognized() {
        for name in [
            "steam_autocloud.vdf",
            "achievements_adf-Otavio",
            "animal_population_",
            "animal_population_x",
            "animal_population_2.bak",
            "",
        ] {
            assert_eq!(classify(name), SaveFileKind::Unrecognized, "{name}");
        }
    }

    #[test]
    fn known_reserve_gets_its_name() {
        assert_eq!(reserve_name(2), "Medved-Taiga");
        assert_eq!(reserve_name(17), "Sundarpatan");
    }

    #[test]
    fn unknown_reserve_is_numbered() {
        assert_eq!(reserve_name(99), "Reserva 99");
        assert_eq!(reserve_name(5), "Reserva 5");
    }

    #[test]
    fn lists_reserves_in_order_ignoring_other_files() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "animal_population_12",
            "animal_population_2",
            "animal_population_99",
            "found_icons_2",
            "settings_adf",
        ] {
            fs::write(dir.path().join(name), b"x").unwrap();
        }
        fs::create_dir(dir.path().join("animal_population_7")).unwrap();
        let reserves = list_reserves(dir.path()).unwrap();
        let summary: Vec<_> = reserves.iter().map(|r| (r.id, r.name.as_str())).collect();
        assert_eq!(
            summary,
            [
                (2, "Medved-Taiga"),
                (12, "Mississippi Acres"),
                (99, "Reserva 99")
            ]
        );
    }

    #[test]
    fn missing_folder_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list_reserves(&dir.path().join("nada")).is_err());
    }

    #[test]
    fn save_dir_detection_needs_a_recognized_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("readme.txt"), b"x").unwrap();
        assert!(!looks_like_save_dir(dir.path()));
        fs::write(dir.path().join("settings_adf"), b"x").unwrap();
        assert!(looks_like_save_dir(dir.path()));
        assert!(!looks_like_save_dir(&dir.path().join("nada")));
    }
}
