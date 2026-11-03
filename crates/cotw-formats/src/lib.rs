//! Núcleo de formatos do Great COTW One.
//!
//! Este crate não conhece interface nem janelas: só transforma bytes em estruturas e de
//! volta. Isso o mantém fácil de testar.
//!
//! - Save: [`envelope`] (cabeçalho + zlib) → [`adf`] (dados tipados).
//! - Jogo: [`archive`] (índices `.tab`/`.arc` por hash de caminho) → [`aaf`] → [`sarc`];
//!   [`rtpc`] e [`xls`] para configurações e planilhas.

pub mod aaf;
pub mod adf;
pub mod archive;
mod bytes;
pub mod envelope;
pub mod hash;
pub mod missions;
pub mod need_zones;
pub mod population;
pub mod rtpc;
pub mod sarc;
pub mod spawn_map;
pub mod steam_kv;
pub mod texture;
pub mod xls;
