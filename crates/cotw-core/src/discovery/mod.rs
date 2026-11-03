//! Descoberta do save e da instalação do jogo (spec `game-install-discovery`).

pub mod files;
pub mod saves;
pub mod steam;

pub use files::{Reserve, SaveFileKind, list_reserves, reserve_name};
pub use saves::{SaveAccount, SaveLocation};
pub use steam::GameInstall;
