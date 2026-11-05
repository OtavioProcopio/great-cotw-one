//! Regras do Great COTW One que tocam o computador do usuário: onde estão o save e o jogo,
//! se o jogo está aberto, backups, histórico e gravação segura.
//!
//! Os formatos de arquivo ficam em `cotw-formats`; este crate não conhece interface.

pub mod animal_editor;
pub mod animals;
pub mod backup;
pub mod career;
pub mod diagnostics;
pub mod discovery;
pub mod history;
pub mod language;
pub mod license;
pub mod mapview;
pub mod paths;
pub mod process;
pub mod purchase;
pub mod service;
pub mod settings;
pub mod species;
pub mod species_stats;
pub mod transaction;
pub mod unlock;
pub mod zones;

#[cfg(test)]
mod test_util;
