//! Mapas de spawn do jogo: onde cada espécie come, bebe e descansa.
//!
//! Cada reserva tem um pacote (atlas) com um mapa por espécie. O mapa é uma grade de 256 x 256
//! células e cada área (identificada por uma chave) lista as células que ocupa. O jogo grava a
//! zona de uma área no centro da caixa que envolve essas células, com 32 metros por célula.

use crate::aaf::{self, AafError};
use crate::adf::{Adf, AdfError, Value};
use crate::archive::{ArchiveError, GameArchives};
use crate::sarc::{Sarc, SarcError};
use crate::texture::{self, Image, TextureError};

const MAP_DIR: &str = "spawn_maps/";
const MAP_SUFFIX: &str = ".bmp_datac";
/// Colunas da grade de cada mapa.
const GRID_WIDTH: u64 = 256;

/// Hash do caminho do atlas de cada reserva (`reserva`, `hash`). Reservas sem atlas não aparecem.
const ATLAS_HASHES: [(u32, u32); 19] = [
    (0, 0x3480_9A57),
    (1, 0x0852_FBDF),
    (2, 0x8F6A_684D),
    (3, 0xDDF4_762E),
    (4, 0x1187_8564),
    (6, 0x5847_5146),
    (8, 0xFF5D_1214),
    (9, 0xF2E1_39D1),
    (10, 0x0798_0EA6),
    (11, 0x49E1_590F),
    (12, 0x12B9_84B5),
    (13, 0x77FF_826D),
    (14, 0x5E1F_AF3D),
    (16, 0x34E0_5690),
    (17, 0x308C_1BA5),
    (18, 0x1110_1275),
    (19, 0xA9AB_7233),
    (20, 0xA2CD_9708),
    (21, 0xE73D_A043),
];

/// Hash do caminho da textura do mapa de cada reserva (`reserva`, `hash`).
const MAP_TEXTURES: [(u32, u32); 19] = [
    (0, 0xB736_886E),
    (1, 0xBF79_24A5),
    (2, 0x8CFF_942B),
    (3, 0x948C_369B),
    (4, 0x9431_B920),
    (6, 0xD503_FCED),
    (8, 0x0767_4339),
    (9, 0xAEEC_86BE),
    (11, 0x3405_B0D8),
    (12, 0xF0FC_A050),
    (13, 0xD5E9_BBA6),
    (14, 0xB753_0973),
    (16, 0x99E8_32FA),
    (17, 0xC0DC_3587),
    // Reservas mais novas: o mapa não tem retângulo conhecido, o app o calcula pelas zonas.
    (10, 0x8914_EB46),
    (18, 0x5A90_7679),
    (19, 0xE2FA_BF1C),
    (20, 0x017D_F772),
    (21, 0x2795_1518),
];

/// Quadrado do mundo que a imagem do mapa cobre: canto (x0, z0) e lado, em metros.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapRect {
    /// X do canto superior esquerdo.
    pub x0: f64,
    /// Z do canto superior esquerdo.
    pub z0: f64,
    /// Lado do quadrado.
    pub span: f64,
}

/// Onde a imagem do mapa da reserva fica no mundo; `None` se a reserva não tem imagem conhecida.
#[must_use]
pub fn map_rect(reserve: u32) -> Option<MapRect> {
    let (x0, z0, span) = match reserve {
        0 | 4 | 8 | 11 | 13 => (-16384.0, 0.0, 16384.0),
        1 | 12 | 14 => (0.0, 0.0, 16384.0),
        2 | 6 | 16 => (-16384.0, -16384.0, 16384.0),
        3 => (0.0, -16384.0, 16384.0),
        9 => (-12960.0, -17136.0, 16325.0),
        17 => (-16384.0, -4096.0, 16384.0),
        _ => return None,
    };
    Some(MapRect { x0, z0, span })
}

/// Lê a imagem do mapa da reserva nos arquivos do jogo.
///
/// # Errors
///
/// [`SpawnMapError`] se a imagem não estiver nos arquivos ou não puder ser decodificada.
pub fn load_map_image(
    archives: &GameArchives,
    reserve: u32,
) -> Result<Option<Image>, SpawnMapError> {
    let Some((_, hash)) = MAP_TEXTURES.iter().find(|(id, _)| *id == reserve) else {
        return Ok(None);
    };
    let location = archives
        .find(*hash)
        .first()
        .ok_or(SpawnMapError::AtlasMissing(reserve))?;
    let image = texture::decode_top_level(&archives.read(location)?)?;
    Ok(Some(image))
}

/// Caminho da imagem de capa da reserva dentro do arquivo de configuração dela.
#[must_use]
pub fn cover_path(settings: &[u8]) -> Option<String> {
    const MARK: &[u8] = b"select_reserve_image/";
    const SUFFIX: &[u8] = b".ddsc";
    settings
        .split(|byte| !(0x20..0x7f).contains(byte))
        .filter(|text| text.len() > MARK.len() + SUFFIX.len() && text.ends_with(SUFFIX))
        .find_map(|text| {
            let at = text.windows(MARK.len()).position(|w| w == MARK)?;
            String::from_utf8(
                text.get(at.saturating_sub("textures/ui/".len())..)?
                    .to_vec(),
            )
            .ok()
        })
        .filter(|path| path.starts_with("textures/ui/select_reserve_image/"))
}

/// Lê a imagem de capa da reserva (a foto que o jogo mostra na escolha da reserva).
///
/// Devolve `None` quando a reserva não está nos arquivos do jogo ou não tem capa: isso acontece
/// com reservas que o jogador não tem instaladas.
///
/// # Errors
///
/// [`SpawnMapError`] se a capa existir mas não puder ser lida ou decodificada.
pub fn load_cover_image(
    archives: &GameArchives,
    reserve: u32,
) -> Result<Option<Image>, SpawnMapError> {
    let settings = format!("settings/hp_settings/reserve_{reserve}.bin");
    let Some(location) = archives.find_path(&settings).first() else {
        return Ok(None);
    };
    let Some(path) = cover_path(&archives.read(location)?) else {
        return Ok(None);
    };
    let Some(cover) = archives.find_path(&path).first() else {
        return Ok(None);
    };
    Ok(Some(texture::decode_top_level(&archives.read(cover)?)?))
}

const LAYER_EAT: u64 = 0x0407_CDFC;
const LAYER_DRINK: u64 = 0x48D1_8EBE;
const LAYER_REST: u64 = 0xAE82_5F0D;

/// Erros ao ler os mapas de spawn.
#[derive(Debug, thiserror::Error)]
pub enum SpawnMapError {
    /// A reserva não tem atlas conhecido.
    #[error("a reserva {0} não tem mapa de spawn conhecido")]
    UnknownReserve(u32),
    /// O atlas não está nos arquivos do jogo.
    #[error("atlas da reserva {0} não encontrado nos arquivos do jogo")]
    AtlasMissing(u32),
    /// Falha ao ler o `.arc`.
    #[error(transparent)]
    Archive(#[from] ArchiveError),
    /// Falha ao descompactar o atlas.
    #[error(transparent)]
    Aaf(#[from] AafError),
    /// O atlas não é um SARC válido.
    #[error(transparent)]
    Sarc(#[from] SarcError),
    /// Um mapa não é um ADF válido.
    #[error(transparent)]
    Adf(#[from] AdfError),
    /// A imagem do mapa não pôde ser decodificada.
    #[error(transparent)]
    Texture(#[from] TextureError),
    /// O mapa não tem a estrutura esperada.
    #[error("mapa de spawn com estrutura inesperada")]
    Layout,
}

/// Uma área onde a espécie come, bebe ou descansa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnArea {
    /// Chave da área (a mesma das rotas dos grupos e das zonas).
    pub key: u32,
    /// 1 comer, 2 beber, 3 descansar.
    pub kind: u32,
    /// Menor coluna de célula ocupada.
    pub min_x: u32,
    /// Maior coluna de célula ocupada.
    pub max_x: u32,
    /// Menor linha de célula ocupada.
    pub min_y: u32,
    /// Maior linha de célula ocupada.
    pub max_y: u32,
}

impl SpawnArea {
    /// Soma da menor e da maior coluna: o centro em colunas é a metade disso.
    #[must_use]
    pub fn center_x_twice(&self) -> u32 {
        self.min_x + self.max_x
    }

    /// Soma da menor e da maior linha: o centro em linhas é a metade disso.
    #[must_use]
    pub fn center_y_twice(&self) -> u32 {
        self.min_y + self.max_y
    }
}

/// Mapa de uma espécie numa reserva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnMap {
    /// Nome da espécie, sem prefixos (`moose`, `red_deer`).
    pub species: String,
    /// Áreas das camadas de comer, beber e descansar.
    pub areas: Vec<SpawnArea>,
}

/// Nome da espécie a partir do nome da entrada (`.../spawn_maps/reserve_2_moose.bmp_datac`).
#[must_use]
pub fn species_from_entry(name: &str) -> Option<String> {
    let file = name.rsplit('/').next()?;
    let stem = file.strip_suffix(MAP_SUFFIX)?;
    let species = if let Some(rest) = stem.strip_prefix("spawn_") {
        rest
    } else {
        let rest = stem.strip_prefix("reserve_")?;
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        rest.get(digits..)?.strip_prefix('_')?
    };
    (!species.is_empty()).then(|| species.to_owned())
}

/// Lê os mapas de spawn de uma reserva nos arquivos do jogo.
///
/// # Errors
///
/// Retorna [`SpawnMapError`] se a reserva não tiver atlas, o atlas faltar ou algum mapa for inválido.
pub fn load_reserve(archives: &GameArchives, reserve: u32) -> Result<Vec<SpawnMap>, SpawnMapError> {
    let hash = ATLAS_HASHES
        .iter()
        .find(|(id, _)| *id == reserve)
        .map(|(_, hash)| *hash)
        .ok_or(SpawnMapError::UnknownReserve(reserve))?;
    let location = archives
        .find(hash)
        .first()
        .ok_or(SpawnMapError::AtlasMissing(reserve))?;
    let sarc = Sarc::parse(aaf::decode(&archives.read(location)?)?)?;
    let mut maps = Vec::new();
    for entry in sarc.entries() {
        if entry.offset == 0 || !entry.name.contains(MAP_DIR) {
            continue;
        }
        let (Some(species), Some(data)) = (species_from_entry(&entry.name), sarc.data_of(entry))
        else {
            continue;
        };
        maps.push(SpawnMap {
            species,
            areas: parse_areas(data)?,
        });
    }
    Ok(maps)
}

/// Reservas que têm atlas conhecido.
pub fn reserves_with_atlas() -> impl Iterator<Item = u32> {
    ATLAS_HASHES.iter().map(|(id, _)| *id)
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn as_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Unsigned(n) => Some(*n),
        Value::Signed(n) => u64::try_from(*n).ok(),
        _ => None,
    }
}

fn items(value: &Value) -> &[Value] {
    match value {
        Value::Array(items) => items,
        _ => &[],
    }
}

/// A chave vem como inteiro com sinal; as rotas e as zonas a guardam como `u32` com os mesmos bits.
fn area_key(value: &Value) -> Result<u32, SpawnMapError> {
    match value {
        Value::Signed(n) => i32::try_from(*n)
            .map(i32::cast_unsigned)
            .map_err(|_| SpawnMapError::Layout),
        Value::Unsigned(n) => u32::try_from(*n).map_err(|_| SpawnMapError::Layout),
        _ => Err(SpawnMapError::Layout),
    }
}

fn parse_areas(adf: &[u8]) -> Result<Vec<SpawnArea>, SpawnMapError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let instance = parsed.instances().first().ok_or(SpawnMapError::Layout)?;
    let root = parsed.read_instance(instance)?;
    let mut areas = Vec::new();
    for layer in items(member(&root, "Layers").ok_or(SpawnMapError::Layout)?) {
        let kind = match member(layer, "TypeHash").and_then(as_u64) {
            Some(LAYER_EAT) => 1,
            Some(LAYER_DRINK) => 2,
            Some(LAYER_REST) => 3,
            _ => continue,
        };
        for area in items(member(layer, "Areas").ok_or(SpawnMapError::Layout)?) {
            let key = area_key(member(area, "key").ok_or(SpawnMapError::Layout)?)?;
            let (mut min_x, mut max_x, mut min_y, mut max_y) = (u32::MAX, 0, u32::MAX, 0);
            let mut any = false;
            for tile in items(member(area, "Tiles").ok_or(SpawnMapError::Layout)?) {
                let tile = as_u64(tile).ok_or(SpawnMapError::Layout)?;
                let x = u32::try_from(tile % GRID_WIDTH).map_err(|_| SpawnMapError::Layout)?;
                let y = u32::try_from(tile / GRID_WIDTH).map_err(|_| SpawnMapError::Layout)?;
                (min_x, max_x, min_y, max_y) =
                    (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y));
                any = true;
            }
            if any {
                areas.push(SpawnArea {
                    key,
                    kind,
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                });
            }
        }
    }
    Ok(areas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_names_lose_their_prefixes() {
        let dir = "settings/hp_settings/hp_ai_textures/spawn_maps/";
        assert_eq!(
            species_from_entry(&format!("{dir}spawn_red_deer.bmp_datac")).as_deref(),
            Some("red_deer")
        );
        assert_eq!(
            species_from_entry(&format!("{dir}reserve_13_moose.bmp_datac")).as_deref(),
            Some("moose")
        );
        assert_eq!(
            species_from_entry(&format!("{dir}reserve_2_brown_bear.bmp_datac")).as_deref(),
            Some("brown_bear")
        );
    }

    #[test]
    fn other_entries_are_not_species_maps() {
        assert_eq!(species_from_entry("climate/tree_diffuse.ddsc"), None);
        assert_eq!(species_from_entry("spawn_maps/reserve_2.bmp_datac"), None);
        assert_eq!(species_from_entry("spawn_maps/spawn_.bmp_datac"), None);
    }

    #[test]
    fn centers_are_kept_doubled_to_stay_in_integers() {
        let area = SpawnArea {
            key: 1,
            kind: 1,
            min_x: 10,
            max_x: 13,
            min_y: 4,
            max_y: 4,
        };
        assert_eq!(area.center_x_twice(), 23);
        assert_eq!(area.center_y_twice(), 8);
    }

    #[test]
    fn map_rects_exist_for_the_reserves_that_have_an_image() {
        assert_eq!(
            map_rect(2),
            Some(MapRect {
                x0: -16384.0,
                z0: -16384.0,
                span: 16384.0
            })
        );
        assert!((map_rect(9).unwrap().span - 16325.0).abs() < f64::EPSILON);
        assert_eq!(map_rect(5), None);
        // Só as reservas mais novas ficam sem retângulo conhecido.
        let without: Vec<u32> = MAP_TEXTURES
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| map_rect(*id).is_none())
            .collect();
        assert_eq!(without, vec![10, 18, 19, 20, 21]);
    }

    #[test]
    fn the_cover_path_is_found_among_the_settings_strings() {
        let mut bytes =
            b" loading_01.ddsc textures/ui/select_reserve_image/select_reserve_image_nepal.ddsc x"
                .to_vec();
        assert_eq!(
            cover_path(&bytes).as_deref(),
            Some("textures/ui/select_reserve_image/select_reserve_image_nepal.ddsc")
        );
        bytes = b"textures/ui/loading_screens/sh/loading_01.ddsc".to_vec();
        assert_eq!(cover_path(&bytes), None);
        assert_eq!(cover_path(b""), None);
    }

    #[test]
    fn only_reserves_with_an_atlas_are_listed() {
        let ids: Vec<u32> = reserves_with_atlas().collect();
        assert!(ids.contains(&2));
        assert!(!ids.contains(&5));
    }

    #[test]
    fn signed_keys_keep_their_bits() {
        assert_eq!(area_key(&Value::Signed(-1)).unwrap(), u32::MAX);
        assert_eq!(area_key(&Value::Unsigned(7)).unwrap(), 7);
        assert!(area_key(&Value::Float(1.0)).is_err());
    }
}
