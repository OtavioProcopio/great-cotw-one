//! Dados do mapa de uma reserva para a interface: zonas, grupos de animais, espécies e a imagem
//! de fundo do próprio jogo (spec `reserve-map`).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use base64::Engine as _;
use cotw_formats::envelope::SaveEnvelope;
use cotw_formats::need_zones::NeedZones;
use cotw_formats::population::parse_groups;
use cotw_formats::spawn_map::{MapRect, SpawnMap, map_rect};
use cotw_formats::texture::Image;
use serde::Serialize;

use crate::discovery::reserve_name;
use crate::species::display_name;
use crate::unlock::{SpawnSource, ZONES_FILE};
use crate::zones::{ReserveInput, assign_species, plan_reserve};

/// Qualidade do JPEG do fundo (o mapa é só pano de fundo).
const JPEG_QUALITY: u8 = 85;
/// Largura, em pixels, das fotos de capa na lista de reservas.
const COVER_WIDTH: u32 = 360;

/// Erros ao montar o mapa.
#[derive(Debug, thiserror::Error)]
pub enum MapError {
    /// Não foi possível ler um arquivo do save.
    #[error("não foi possível ler {file}: {reason}")]
    Read {
        /// Arquivo.
        file: String,
        /// Motivo.
        reason: String,
    },
    /// O arquivo do save não tem a estrutura esperada.
    #[error("{0} não pôde ser interpretado")]
    Parse(String),
    /// A reserva não está no save ou não tem dados do jogo.
    #[error("a reserva {0} não tem dados para mostrar no mapa")]
    NoData(u32),
}

/// Uma espécie que aparece no mapa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapSpecies {
    /// Nome que o jogo usa.
    pub id: String,
    /// Nome para exibir.
    pub name: String,
}

/// Uma zona de necessidade no mapa.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapZone {
    /// Posição X no mundo.
    pub x: f32,
    /// Posição Z no mundo.
    pub z: f32,
    /// 1 comer, 2 beber, 3 descansar.
    pub kind: u8,
    /// Índice em `species`, ou -1 se a espécie não for conhecida.
    pub species: i32,
    /// Ainda não está no save: o app a acrescentaria ao liberar o mapa.
    pub pending: bool,
    /// Hora em que começa.
    pub start: f32,
    /// Hora em que termina.
    pub end: f32,
}

/// Um animal de um grupo.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAnimal {
    /// Macho.
    pub male: bool,
    /// Peso em quilos.
    pub weight: f32,
    /// Pontuação do troféu.
    pub score: f32,
    /// Great One.
    pub great_one: bool,
}

/// Um grupo de animais e a rota dele.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapGroup {
    /// Número do grupo para o jogador (posição + 1).
    pub number: u32,
    /// Índice em `species`.
    pub species: usize,
    /// Zonas da rota, em ordem, como índices em `zones`.
    pub stops: Vec<usize>,
    /// Animais do grupo.
    pub animals: Vec<MapAnimal>,
}

/// Tudo que a tela do mapa precisa de uma reserva.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReserveMap {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
    /// Imagem de fundo (`data:image/jpeg;base64,...`), se o jogo a tiver.
    pub image: Option<String>,
    /// X do canto superior esquerdo da imagem no mundo.
    pub x0: f64,
    /// Z do canto superior esquerdo da imagem no mundo.
    pub z0: f64,
    /// Lado da imagem no mundo, em metros.
    pub span: f64,
    /// O alinhamento da imagem foi calculado pelas zonas (erro típico de algumas centenas de metros).
    pub approximate: bool,
    /// Espécies, em ordem alfabética do nome.
    pub species: Vec<MapSpecies>,
    /// Zonas do save e as que faltam descobrir.
    pub zones: Vec<MapZone>,
    /// Grupos com rota e animais.
    pub groups: Vec<MapGroup>,
}

/// Uma reserva que o mapa consegue mostrar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapReserve {
    /// Número da reserva.
    pub id: u32,
    /// Nome para exibir.
    pub name: String,
    /// Foto de capa pequena (`data:image/jpeg;base64,...`), se o jogo a tiver.
    pub cover: Option<String>,
}

pub(crate) fn read_adf(path: &Path, name: &str) -> Result<Vec<u8>, MapError> {
    let raw = fs::read(path).map_err(|e| MapError::Read {
        file: name.to_owned(),
        reason: e.to_string(),
    })?;
    SaveEnvelope::parse(&raw)
        .map(|envelope| envelope.adf().to_vec())
        .map_err(|_| MapError::Parse(name.to_owned()))
}

/// Reservas do save que têm população e mapas do jogo.
///
/// # Errors
///
/// [`MapError`] se o arquivo de zonas não puder ser lido.
pub fn reserves(save_dir: &Path, source: &dyn SpawnSource) -> Result<Vec<MapReserve>, MapError> {
    let zones = NeedZones::parse(&read_adf(&save_dir.join(ZONES_FILE), ZONES_FILE)?)
        .map_err(|_| MapError::Parse(ZONES_FILE.into()))?;
    let mut found: Vec<MapReserve> = zones
        .reserves
        .iter()
        .filter_map(|r| u32::try_from(r.id).ok())
        .filter(|id| {
            save_dir.join(format!("animal_population_{id}")).is_file() && source.maps(*id).is_ok()
        })
        .map(|id| MapReserve {
            id,
            name: reserve_name(id),
            cover: cover_data_url(source, id),
        })
        .collect();
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

/// Metros que cada pixel da imagem do mapa cobre (todas as imagens têm 1024 px para 16 384 m).
const METERS_PER_PIXEL: f64 = 16.0;
/// Pixel mais claro que isto conta como parte do mapa (o resto é a margem escura da imagem).
const LIT_LEVEL: u32 = 60;

/// Onde a imagem do mapa fica no mundo quando o app ainda não conhece o retângulo da reserva.
///
/// A parte iluminada da imagem é o território da reserva, e as zonas ocupam esse mesmo território:
/// o centro de uma coincide com o centro da outra. Devolve `None` sem zonas ou sem parte iluminada.
#[must_use]
pub fn infer_rect(image: &Image, zones: &[(f32, f32)]) -> Option<MapRect> {
    if zones.is_empty() || image.width == 0 {
        return None;
    }
    let width = image.width as usize;
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (usize::MAX, 0, usize::MAX, 0);
    for (i, pixel) in image.rgba.as_chunks::<4>().0.iter().enumerate() {
        let level = (u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2])) / 3;
        if level > LIT_LEVEL {
            let (x, y) = (i % width, i / width);
            (min_x, max_x, min_y, max_y) = (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y));
        }
    }
    if min_x == usize::MAX {
        return None;
    }
    let lit_center_x = f64::from(u32::try_from(min_x + max_x).ok()?) / 2.0;
    let lit_center_y = f64::from(u32::try_from(min_y + max_y).ok()?) / 2.0;
    let (lo_x, hi_x) = zones.iter().fold((f32::MAX, f32::MIN), |(lo, hi), z| {
        (lo.min(z.0), hi.max(z.0))
    });
    let (lo_z, hi_z) = zones.iter().fold((f32::MAX, f32::MIN), |(lo, hi), z| {
        (lo.min(z.1), hi.max(z.1))
    });
    let zone_center_x = f64::from(lo_x + hi_x) / 2.0;
    let zone_center_z = f64::from(lo_z + hi_z) / 2.0;
    let span = METERS_PER_PIXEL * f64::from(image.width);
    Some(MapRect {
        x0: zone_center_x - lit_center_x * METERS_PER_PIXEL,
        z0: zone_center_z - lit_center_y * METERS_PER_PIXEL,
        span,
    })
}

/// Foto de capa da reserva, reduzida, como `data:image/jpeg;base64,...`.
pub(crate) fn cover_data_url(source: &dyn SpawnSource, reserve: u32) -> Option<String> {
    source
        .cover_image(reserve)
        .map(|cover| shrink(&cover, COVER_WIDTH))
        .as_ref()
        .and_then(jpeg_data_url)
}

/// Reduz a imagem para a largura pedida (média das células de origem); não amplia.
fn shrink(image: &Image, width: u32) -> Image {
    if image.width <= width || image.width == 0 || image.height == 0 {
        return image.clone();
    }
    let height = u32::try_from(u64::from(image.height) * u64::from(width) / u64::from(image.width))
        .unwrap_or(u32::MAX)
        .max(1);
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        let y0 = u64::from(y) * u64::from(image.height) / u64::from(height);
        let y1 = (u64::from(y + 1) * u64::from(image.height) / u64::from(height)).max(y0 + 1);
        for x in 0..width {
            let x0 = u64::from(x) * u64::from(image.width) / u64::from(width);
            let x1 = (u64::from(x + 1) * u64::from(image.width) / u64::from(width)).max(x0 + 1);
            let mut sum = [0_u64; 4];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let at = usize::try_from((sy * u64::from(image.width) + sx) * 4).unwrap_or(0);
                    for (channel, total) in sum.iter_mut().enumerate() {
                        *total += u64::from(image.rgba.get(at + channel).copied().unwrap_or(0));
                    }
                }
            }
            let count = (y1 - y0) * (x1 - x0);
            rgba.extend(sum.map(|total| u8::try_from(total / count).unwrap_or(u8::MAX)));
        }
    }
    Image {
        width,
        height,
        rgba,
    }
}

fn jpeg_data_url(image: &Image) -> Option<String> {
    let rgb: Vec<u8> = image
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();
    let mut out = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut out, JPEG_QUALITY);
    encoder
        .encode(
            &rgb,
            u16::try_from(image.width).ok()?,
            u16::try_from(image.height).ok()?,
            jpeg_encoder::ColorType::Rgb,
        )
        .ok()?;
    Some(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

/// Espécie de uma zona: a dos grupos que a usam; senão a única espécie cujo mapa a cobre.
fn zone_species(
    key: u32,
    key_species: &HashMap<u32, String>,
    maps: &[SpawnMap],
    index: &BTreeMap<&str, usize>,
) -> i32 {
    let id = key_species.get(&key).map(String::as_str).or_else(|| {
        let mut covering = maps
            .iter()
            .filter(|m| m.areas.iter().any(|a| a.key == key))
            .map(|m| m.species.as_str());
        match (covering.next(), covering.next()) {
            (Some(only), None) => Some(only),
            _ => None,
        }
    });
    id.and_then(|id| index.get(id))
        .and_then(|i| i32::try_from(*i).ok())
        .unwrap_or(-1)
}

/// Monta os dados do mapa de uma reserva.
///
/// # Errors
///
/// [`MapError`] se os arquivos do save não puderem ser lidos ou a reserva não tiver dados.
pub fn build(
    save_dir: &Path,
    source: &dyn SpawnSource,
    reserve: u32,
) -> Result<ReserveMap, MapError> {
    let saved = NeedZones::parse(&read_adf(&save_dir.join(ZONES_FILE), ZONES_FILE)?)
        .map_err(|_| MapError::Parse(ZONES_FILE.into()))?;
    let existing = &saved
        .reserve(u64::from(reserve))
        .ok_or(MapError::NoData(reserve))?
        .zones;
    let population = format!("animal_population_{reserve}");
    let groups = parse_groups(&read_adf(&save_dir.join(&population), &population)?)
        .map_err(|_| MapError::Parse(population.clone()))?;
    let maps = source
        .maps(reserve)
        .map_err(|_| MapError::NoData(reserve))?;

    // Zonas que ainda não estão no save (se o cálculo não for possível, só mostra as do save).
    let pending = plan_reserve(&ReserveInput {
        reserve,
        groups: &groups,
        maps: &maps,
        existing,
    })
    .map(|plan| plan.new_zones)
    .unwrap_or_default();

    let (group_species, key_species) = assign_species(&groups, &maps);
    let ids: BTreeSet<&str> = maps.iter().map(|m| m.species.as_str()).collect();
    let mut species: Vec<MapSpecies> = ids
        .iter()
        .map(|id| MapSpecies {
            id: (*id).to_owned(),
            name: display_name(id),
        })
        .collect();
    species.sort_by(|a, b| a.name.cmp(&b.name));
    let index: BTreeMap<&str, usize> = species
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();

    let mut zones: Vec<MapZone> = Vec::with_capacity(existing.len() + pending.len());
    let mut by_key: HashMap<u32, usize> = HashMap::new();
    for (zone, is_pending) in existing
        .iter()
        .map(|z| (z, false))
        .chain(pending.iter().map(|z| (z, true)))
    {
        by_key.insert(zone.key, zones.len());
        zones.push(MapZone {
            x: zone.x,
            z: zone.z,
            kind: u8::try_from(zone.kind).unwrap_or(0),
            species: zone_species(zone.key, &key_species, &maps, &index),
            pending: is_pending,
            start: zone.start,
            end: zone.end,
        });
    }

    let map_groups = groups
        .iter()
        .zip(&group_species)
        .enumerate()
        .filter_map(|(i, (group, species_id))| {
            let species_index = species_id.as_deref().and_then(|id| index.get(id))?;
            Some(MapGroup {
                number: u32::try_from(i + 1).unwrap_or(u32::MAX),
                species: *species_index,
                stops: group
                    .route
                    .iter()
                    .filter_map(|key| by_key.get(key).copied())
                    .collect(),
                animals: group
                    .animals
                    .iter()
                    .map(|a| MapAnimal {
                        male: a.male,
                        weight: a.weight,
                        score: a.score,
                        great_one: a.great_one,
                    })
                    .collect(),
            })
        })
        .collect();

    let map_image = source.map_image(reserve);
    let known_rect = map_rect(reserve);
    let rect = known_rect.or_else(|| {
        let points: Vec<(f32, f32)> = zones.iter().map(|z| (z.x, z.z)).collect();
        infer_rect(map_image.as_ref()?, &points)
    });
    Ok(ReserveMap {
        id: reserve,
        name: reserve_name(reserve),
        image: map_image.as_ref().and_then(jpeg_data_url),
        x0: rect.map_or(0.0, |r| r.x0),
        z0: rect.map_or(0.0, |r| r.z0),
        span: rect.map_or(16384.0, |r| r.span),
        approximate: known_rect.is_none() && rect.is_some(),
        species,
        zones,
        groups: map_groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32) -> Image {
        let rgba = (0..width * height)
            .flat_map(|i| {
                let x = u8::try_from(i % width).unwrap();
                [x, 0, 0, 255]
            })
            .collect();
        Image {
            width,
            height,
            rgba,
        }
    }

    #[test]
    fn shrinking_keeps_the_proportion_and_averages_neighbours() {
        let small = shrink(&gradient(8, 4), 4);
        assert_eq!((small.width, small.height), (4, 2));
        // Cada célula nova é a média de duas colunas vizinhas: (0+1)/2 = 0, (2+3)/2 = 2, ...
        let reds: Vec<u8> = small.rgba.chunks(4).take(4).map(|p| p[0]).collect();
        assert_eq!(reds, vec![0, 2, 4, 6]);
    }

    #[test]
    fn a_small_image_is_not_enlarged() {
        let image = gradient(4, 4);
        assert_eq!(shrink(&image, 100), image);
    }
}
