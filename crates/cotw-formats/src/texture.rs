//! Texturas `.ddsc` do jogo (contêiner AVTX com blocos DXT/BC): só o necessário para ler o mapa.
//!
//! ```text
//! 0   "AVTX"
//! 4   u16 versão
//! 8   u32 formato (código DXGI: 71 BC1, 77 BC3, 80 BC4, 83 BC5, 87 BGRA8)
//! 12  u16 largura · 14  u16 altura (do nível maior, mesmo que ele esteja em outro arquivo)
//! 32  u32 offset dos dados · 36  u32 tamanho dos dados
//! ```
//!
//! Os dados começam no maior nível guardado neste arquivo, seguido dos menores. Texturas grandes
//! guardam só os níveis pequenos aqui e o resto em arquivos separados; por isso a largura do
//! cabeçalho pode ser maior que a da imagem que este módulo consegue decodificar.

use crate::bytes;

const MAGIC: &[u8; 4] = b"AVTX";
const FORMAT_AT: usize = 8;
const WIDTH_AT: usize = 12;
const HEIGHT_AT: usize = 14;
const DATA_OFFSET_AT: usize = 32;
const DATA_SIZE_AT: usize = 36;
/// Maior lado aceito (proteção contra cabeçalhos corrompidos).
const MAX_SIDE: u32 = 8192;

/// Erros ao decodificar uma textura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TextureError {
    /// Não é um AVTX.
    #[error("textura sem a assinatura AVTX")]
    BadMagic,
    /// Cabeçalho ou dados truncados.
    #[error("textura truncada")]
    Truncated,
    /// O formato de pixel não é um dos que sabemos ler.
    #[error("formato de textura {0} não suportado")]
    UnsupportedFormat(u32),
    /// Os dados não correspondem a nenhuma cadeia de níveis do tamanho do cabeçalho.
    #[error("o tamanho dos dados não bate com o cabeçalho")]
    InconsistentSize,
}

/// Imagem decodificada, em RGBA de 8 bits por canal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Largura em pixels.
    pub width: u32,
    /// Altura em pixels.
    pub height: u32,
    /// Pixels RGBA, linha por linha de cima para baixo.
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Bc1,
    Bc3,
    Bc4,
    Bgra8,
}

impl Format {
    fn from_code(code: u32) -> Result<Self, TextureError> {
        match code {
            71 => Ok(Self::Bc1),
            77 => Ok(Self::Bc3),
            80 => Ok(Self::Bc4),
            87 => Ok(Self::Bgra8),
            other => Err(TextureError::UnsupportedFormat(other)),
        }
    }

    /// Bytes de um nível `width` x `height`.
    fn level_size(self, width: u32, height: u32) -> usize {
        let blocks = |side: u32| side.div_ceil(4).max(1) as usize;
        match self {
            Self::Bc1 | Self::Bc4 => blocks(width) * blocks(height) * 8,
            Self::Bc3 => blocks(width) * blocks(height) * 16,
            Self::Bgra8 => width.max(1) as usize * height.max(1) as usize * 4,
        }
    }

    /// `true` se `size` é a soma dos `k` primeiros níveis a partir de `width` x `height`, para
    /// algum `k`: a cadeia completa até 1 x 1 ou uma cadeia cortada (inclusive um nível só).
    fn stores_exactly(self, mut width: u32, mut height: u32, size: usize) -> bool {
        let mut total = 0;
        loop {
            total += self.level_size(width, height);
            if total >= size || (width <= 1 && height <= 1) {
                return total == size;
            }
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }
    }
}

/// Decodifica o maior nível guardado no arquivo.
///
/// # Errors
///
/// [`TextureError`] se a textura for inválida, truncada ou de um formato que não lemos.
pub fn decode_top_level(file: &[u8]) -> Result<Image, TextureError> {
    if file.get(..4) != Some(&MAGIC[..]) {
        return Err(TextureError::BadMagic);
    }
    let field = |at: usize| bytes::u32_at(file, at).ok_or(TextureError::Truncated);
    let format = Format::from_code(field(FORMAT_AT)?)?;
    let width = u32::from(bytes::u16_at(file, WIDTH_AT).ok_or(TextureError::Truncated)?);
    let height = u32::from(bytes::u16_at(file, HEIGHT_AT).ok_or(TextureError::Truncated)?);
    let offset = field(DATA_OFFSET_AT)? as usize;
    let size = field(DATA_SIZE_AT)? as usize;
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err(TextureError::InconsistentSize);
    }
    let data = bytes::slice(file, offset, size).ok_or(TextureError::Truncated)?;

    // O maior nível guardado é o primeiro da cadeia cujos níveis (todos ou só alguns) somam o tamanho dos dados.
    let (mut w, mut h) = (width, height);
    loop {
        if format.stores_exactly(w, h, size) {
            return Ok(Image {
                width: w,
                height: h,
                rgba: decode_level(format, data, w, h),
            });
        }
        if w <= 1 && h <= 1 {
            return Err(TextureError::InconsistentSize);
        }
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
}

fn decode_level(format: Format, data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    let mut rgba = vec![0u8; w * h * 4];
    match format {
        Format::Bgra8 => {
            for (out, px) in rgba
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(data.as_chunks::<4>().0)
            {
                *out = [px[2], px[1], px[0], px[3]];
            }
        }
        Format::Bc1 | Format::Bc3 | Format::Bc4 => {
            let block_bytes = if format == Format::Bc3 { 16 } else { 8 };
            let blocks_x = w.div_ceil(4);
            for (index, block) in data.chunks_exact(block_bytes).enumerate() {
                let (bx, by) = (index % blocks_x, index / blocks_x);
                let pixels = match format {
                    Format::Bc1 => bc1_block(block, true),
                    Format::Bc3 => {
                        let mut pixels = bc1_block(&block[8..], false);
                        let alpha = alpha_block(&block[..8]);
                        for (pixel, a) in pixels.iter_mut().zip(alpha) {
                            pixel[3] = a;
                        }
                        pixels
                    }
                    _ => alpha_block(block).map(|v| [v, v, v, 255]),
                };
                for (i, pixel) in pixels.iter().enumerate() {
                    let (x, y) = (bx * 4 + i % 4, by * 4 + i / 4);
                    if x < w && y < h {
                        let at = (y * w + x) * 4;
                        rgba[at..at + 4].copy_from_slice(pixel);
                    }
                }
            }
        }
    }
    rgba
}

fn rgb565(value: u16) -> [u8; 3] {
    let expand = |v: u16, bits: u32| {
        let max = (1u16 << bits) - 1;
        u8::try_from((v * 255 + max / 2) / max).unwrap_or(255)
    };
    [
        expand(value >> 11, 5),
        expand((value >> 5) & 0x3F, 6),
        expand(value & 0x1F, 5),
    ]
}

/// Bloco BC1 (8 bytes). `punch_through`: no modo BC1 puro, c0 <= c1 dá um quarto pixel transparente.
fn bc1_block(block: &[u8], punch_through: bool) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let (a, b) = (rgb565(c0), rgb565(c1));
    let mix = |wa: u32, wb: u32, div: u32| {
        let channel = |i: usize| {
            u8::try_from((u32::from(a[i]) * wa + u32::from(b[i]) * wb) / div).unwrap_or(255)
        };
        [channel(0), channel(1), channel(2), 255]
    };
    let palette = if c0 > c1 {
        [
            [a[0], a[1], a[2], 255],
            [b[0], b[1], b[2], 255],
            mix(2, 1, 3),
            mix(1, 2, 3),
        ]
    } else {
        let last = if punch_through {
            [0, 0, 0, 0]
        } else {
            [0, 0, 0, 255]
        };
        [
            [a[0], a[1], a[2], 255],
            [b[0], b[1], b[2], 255],
            mix(1, 1, 2),
            last,
        ]
    };
    let indices = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
    let mut out = [[0u8; 4]; 16];
    for (i, pixel) in out.iter_mut().enumerate() {
        *pixel = palette[((indices >> (2 * i)) & 3) as usize];
    }
    out
}

/// Bloco de um canal de 8 bytes (alfa do BC3 ou BC4).
fn alpha_block(block: &[u8]) -> [u8; 16] {
    let (a0, a1) = (u32::from(block[0]), u32::from(block[1]));
    let mut palette = [0u8; 8];
    palette[0] = block[0];
    palette[1] = block[1];
    let lerp = |wa: u32, wb: u32, div: u32| u8::try_from((a0 * wa + a1 * wb) / div).unwrap_or(255);
    if a0 > a1 {
        for k in 1..7u32 {
            palette[k as usize + 1] = lerp(7 - k, k, 7);
        }
    } else {
        for k in 1..5u32 {
            palette[k as usize + 1] = lerp(5 - k, k, 5);
        }
        palette[6] = 0;
        palette[7] = 255;
    }
    let mut bits = 0u64;
    for (i, byte) in block[2..8].iter().enumerate() {
        bits |= u64::from(*byte) << (8 * i);
    }
    let mut out = [0u8; 16];
    for (i, value) in out.iter_mut().enumerate() {
        *value = palette[((bits >> (3 * i)) & 7) as usize];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Arquivo AVTX mínimo com um único nível.
    fn file(format: u32, width: u16, height: u16, data: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; 0x80];
        out[..4].copy_from_slice(MAGIC);
        out[4] = 1;
        out[FORMAT_AT..FORMAT_AT + 4].copy_from_slice(&format.to_le_bytes());
        out[WIDTH_AT..WIDTH_AT + 2].copy_from_slice(&width.to_le_bytes());
        out[HEIGHT_AT..HEIGHT_AT + 2].copy_from_slice(&height.to_le_bytes());
        out[DATA_OFFSET_AT..DATA_OFFSET_AT + 4].copy_from_slice(&0x80u32.to_le_bytes());
        let len = u32::try_from(data.len()).unwrap();
        out[DATA_SIZE_AT..DATA_SIZE_AT + 4].copy_from_slice(&len.to_le_bytes());
        out.extend_from_slice(data);
        out
    }

    #[test]
    fn a_solid_bc1_block_fills_four_by_four_pixels() {
        // c0 = vermelho puro (0xF800), c1 = 0, todos os índices 0 → só c0.
        let block = [0x00, 0xF8, 0x00, 0x00, 0, 0, 0, 0];
        // 4x4 tem cadeia 4x4 + 2x2 + 1x1 = 3 blocos
        let mut data = block.to_vec();
        data.extend_from_slice(&block);
        data.extend_from_slice(&block);
        let image = decode_top_level(&file(71, 4, 4, &data)).unwrap();
        assert_eq!((image.width, image.height), (4, 4));
        assert!(
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [255, 0, 0, 255])
        );
    }

    #[test]
    fn the_top_level_is_found_when_only_small_levels_are_stored() {
        // Cabeçalho diz 8x8, mas os dados são só 4x4 + 2x2 + 1x1 (3 blocos BC1).
        let block = [0x00, 0xF8, 0x00, 0x00, 0, 0, 0, 0];
        let data: Vec<u8> = block.iter().chain(&block).chain(&block).copied().collect();
        let image = decode_top_level(&file(71, 8, 8, &data)).unwrap();
        assert_eq!((image.width, image.height), (4, 4));
    }

    #[test]
    fn a_chain_cut_short_is_read_from_its_first_level() {
        // 8x8 com só dois níveis guardados (8x8 = 4 blocos, 4x4 = 1 bloco): 5 blocos BC1.
        let block = [0x00, 0xF8, 0x00, 0x00, 0, 0, 0, 0];
        let data: Vec<u8> = std::iter::repeat_n(block, 5).flatten().collect();
        let image = decode_top_level(&file(71, 8, 8, &data)).unwrap();
        assert_eq!((image.width, image.height), (8, 8));
        // Um nível só, de tamanho não quadrado.
        let single: Vec<u8> = std::iter::repeat_n(block, 2).flatten().collect();
        let image = decode_top_level(&file(71, 8, 4, &single)).unwrap();
        assert_eq!((image.width, image.height), (8, 4));
    }

    #[test]
    fn bc3_uses_the_alpha_block() {
        // alfa: a0 = 255, a1 = 255 → todos opacos; cor vermelha.
        let mut block = vec![255, 255, 0, 0, 0, 0, 0, 0];
        block.extend_from_slice(&[0x00, 0xF8, 0x00, 0x00, 0, 0, 0, 0]);
        let mut data = block.clone();
        data.extend_from_slice(&block);
        data.extend_from_slice(&block);
        let image = decode_top_level(&file(77, 4, 4, &data)).unwrap();
        assert!(
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [255, 0, 0, 255])
        );
    }

    #[test]
    fn bgra_is_reordered_to_rgba() {
        let data = [1, 2, 3, 4];
        let image = decode_top_level(&file(87, 1, 1, &data)).unwrap();
        assert_eq!(image.rgba, [3, 2, 1, 4]);
    }

    #[test]
    fn invalid_files_are_errors() {
        assert_eq!(decode_top_level(b"DDS "), Err(TextureError::BadMagic));
        assert_eq!(decode_top_level(b"AVTX"), Err(TextureError::Truncated));
        assert_eq!(
            decode_top_level(&file(98, 4, 4, &[0; 16])),
            Err(TextureError::UnsupportedFormat(98))
        );
        assert_eq!(
            decode_top_level(&file(71, 4, 4, &[0; 10])),
            Err(TextureError::InconsistentSize)
        );
    }
}
