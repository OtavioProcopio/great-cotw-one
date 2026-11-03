//! `found_need_zones_adf`: zonas de necessidade (comer, beber, descansar) que o jogador já descobriu.
//!
//! O arquivo guarda, por reserva, uma lista de zonas; cada zona tem uma lista própria de grupos
//! que a descobriram. Adicionar zonas muda o tamanho de tudo que vem depois, então o escritor
//! reconstrói a instância inteira (ponteiros, alinhamento, cadeia de relocação, cabeçalho) e
//! confere o resultado relendo-o.
//!
//! ```text
//! raiz (24 B):   @0 AlreadyConverted, @8 array de reservas {u32 offset, u32 next, u64 count}
//! reserva (24):  u64 ReserveId, array de zonas (16 B)
//! zona (64):     x,y,z f32 @0 · chave da área u32 @12 · tipo u32 @16 · id do ícone u64 @24 ·
//!                início f32 @32 · fim f32 @36 · nome da espécie u32 @40 · horário u32 @44 ·
//!                grupos que descobriram: array de u32 @48
//! ```

use crate::adf::{Adf, AdfError};
use crate::bytes;

const ROOT_SIZE: usize = 24;
const RESERVE_SIZE: usize = 24;
const ZONE_SIZE: usize = 64;
const ZONE_FIELDS: usize = 48;
const ARRAY_HEADER: usize = 16;
const ROOT_ARRAY_AT: usize = 8;
/// Marca que o jogo deixa logo depois dos dados da instância.
const AFTER_DATA_MARK: u32 = 12;

/// Erros ao ler ou reescrever as zonas.
#[derive(Debug, thiserror::Error)]
pub enum NeedZonesError {
    /// O ADF em si é inválido.
    #[error("ADF inválido: {0}")]
    Adf(#[from] AdfError),
    /// A estrutura não é a esperada para este arquivo.
    #[error("estrutura inesperada em found_need_zones_adf: {0}")]
    Layout(&'static str),
    /// O arquivo gerado não passou na conferência.
    #[error("o arquivo gerado não passou na conferência: {0}")]
    Verification(&'static str),
}

/// Uma zona de necessidade.
#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    /// Posição X no mundo.
    pub x: f32,
    /// Altura.
    pub y: f32,
    /// Posição Z no mundo.
    pub z: f32,
    /// Chave da área (a mesma que as rotas dos grupos e os mapas do jogo usam).
    pub key: u32,
    /// 1 comer, 2 beber, 3 descansar.
    pub kind: u32,
    /// Identificador único do ícone no mapa.
    pub icon_id: u64,
    /// Hora em que a zona começa (0 a 24).
    pub start: f32,
    /// Hora em que a zona termina (0 a 24; pode ser menor que o início).
    pub end: f32,
    /// Constante da espécie (nome localizado).
    pub species_name: u32,
    /// Índice do horário.
    pub schedule: u32,
    /// Grupos que já descobriram a zona.
    pub discovered_groups: Vec<u32>,
    /// Bytes de preenchimento entre o tipo e o id do ícone, preservados como vieram.
    padding: [u8; 4],
}

impl Zone {
    /// Zona nova, sem grupos que a descobriram.
    #[must_use]
    #[allow(clippy::too_many_arguments, reason = "espelha os campos do formato")]
    pub fn new(
        position: (f32, f32, f32),
        key: u32,
        kind: u32,
        icon_id: u64,
        hours: (f32, f32),
        species_name: u32,
        schedule: u32,
    ) -> Self {
        Self {
            x: position.0,
            y: position.1,
            z: position.2,
            key,
            kind,
            icon_id,
            start: hours.0,
            end: hours.1,
            species_name,
            schedule,
            discovered_groups: Vec::new(),
            padding: [0; 4],
        }
    }
}

/// As zonas de uma reserva.
#[derive(Debug, Clone, PartialEq)]
pub struct ReserveZones {
    /// Número da reserva.
    pub id: u64,
    /// Zonas descobertas.
    pub zones: Vec<Zone>,
}

/// Conteúdo de `found_need_zones_adf`.
#[derive(Debug, Clone, PartialEq)]
pub struct NeedZones {
    /// Valor de `AlreadyConverted` e seu preenchimento, preservados.
    root_head: [u8; 8],
    /// Reservas, na ordem do arquivo.
    pub reserves: Vec<ReserveZones>,
}

fn layout(message: &'static str) -> NeedZonesError {
    NeedZonesError::Layout(message)
}

fn f32_at(data: &[u8], offset: usize) -> Option<f32> {
    bytes::u32_at(data, offset).map(f32::from_bits)
}

impl NeedZones {
    /// Lê as zonas de um ADF de `found_need_zones_adf`.
    ///
    /// # Errors
    ///
    /// Retorna [`NeedZonesError`] se o ADF for inválido ou a estrutura não for a esperada.
    pub fn parse(adf: &[u8]) -> Result<Self, NeedZonesError> {
        let parsed = Adf::parse(adf.to_vec())?;
        let instance = parsed
            .instances()
            .first()
            .ok_or_else(|| layout("sem instância"))?;
        let base = instance.offset as usize;
        let size = instance.size as usize;
        let data =
            bytes::slice(adf, base, size).ok_or_else(|| layout("instância fora do arquivo"))?;
        if size < ROOT_SIZE {
            return Err(layout("instância menor que a raiz"));
        }
        let mut root_head = [0u8; 8];
        root_head.copy_from_slice(&data[..8]);

        let array = read_array(data, ROOT_ARRAY_AT)?;
        let reserves_at = array.offset;
        let count = array.count;
        check_span(data, reserves_at, count, RESERVE_SIZE)?;
        let mut reserves = Vec::with_capacity(count);
        for i in 0..count {
            let at = reserves_at + i * RESERVE_SIZE;
            let id = bytes::u64_at(data, at).ok_or_else(|| layout("reserva truncada"))?;
            let zones_array = read_array(data, at + 8)?;
            check_span(data, zones_array.offset, zones_array.count, ZONE_SIZE)?;
            let mut zones = Vec::with_capacity(zones_array.count);
            for k in 0..zones_array.count {
                zones.push(read_zone(data, zones_array.offset + k * ZONE_SIZE)?);
            }
            reserves.push(ReserveZones { id, zones });
        }
        Ok(Self {
            root_head,
            reserves,
        })
    }

    /// Zonas de uma reserva, se existir.
    #[must_use]
    pub fn reserve(&self, id: u64) -> Option<&ReserveZones> {
        self.reserves.iter().find(|r| r.id == id)
    }

    /// Zonas de uma reserva (mutável), se existir.
    pub fn reserve_mut(&mut self, id: u64) -> Option<&mut ReserveZones> {
        self.reserves.iter_mut().find(|r| r.id == id)
    }

    /// Reconstrói o ADF com estas zonas, usando `original` como modelo do cabeçalho e das tabelas.
    ///
    /// O resultado é relido e conferido: mesmas zonas, ADF válido e cadeia de ponteiros íntegra.
    ///
    /// # Errors
    ///
    /// Retorna [`NeedZonesError`] se o modelo não for um `found_need_zones_adf` válido ou se o
    /// arquivo gerado não passar na conferência.
    pub fn to_adf(&self, original: &[u8]) -> Result<Vec<u8>, NeedZonesError> {
        let parsed = Adf::parse(original.to_vec())?;
        let instance = parsed
            .instances()
            .first()
            .ok_or_else(|| layout("sem instância"))?;
        let header = Header::read(original)?;
        let data_at = instance.offset as usize;
        let old_size = instance.size as usize;

        let mut payload = self.build_payload();
        if old_size.is_multiple_of(8) {
            payload.resize(payload.len().next_multiple_of(8), 0);
        }
        let end = data_at + payload.len();
        let new_table = (end + 4).next_multiple_of(8);
        if new_table < header.instance_table {
            return Err(layout("tabela de instâncias antes dos dados"));
        }

        let tail = original
            .get(header.instance_table..)
            .ok_or_else(|| layout("tabela de instâncias fora do arquivo"))?;
        let mut out = Vec::with_capacity(new_table + tail.len());
        out.extend_from_slice(
            original
                .get(..data_at)
                .ok_or_else(|| layout("cabeçalho truncado"))?,
        );
        out.extend_from_slice(&payload);
        out.resize(new_table, 0);
        out.extend_from_slice(tail);

        // O u32 logo depois dos dados e o cabeçalho da instância e do ADF.
        put_u32(&mut out, end, AFTER_DATA_MARK);
        let payload_len =
            u32::try_from(payload.len()).map_err(|_| layout("instância grande demais"))?;
        put_u32(&mut out, new_table + 12, payload_len);
        header.write_shifted(&mut out, new_table)?;

        self.verify(&out)?;
        Ok(out)
    }

    fn build_payload(&self) -> Vec<u8> {
        let mut buf = vec![0u8; ROOT_SIZE];
        buf[..8].copy_from_slice(&self.root_head);
        let mut pointers: Vec<usize> = Vec::new();
        let reserves = self.reserves.len();
        let array_at = ROOT_SIZE;
        if reserves > 0 {
            put_array(&mut buf, ROOT_ARRAY_AT, array_at, reserves);
            pointers.push(ROOT_ARRAY_AT);
        }
        let mut cursor = array_at + reserves * RESERVE_SIZE;
        for (i, reserve) in self.reserves.iter().enumerate() {
            let entry = array_at + i * RESERVE_SIZE;
            cursor = cursor.next_multiple_of(8);
            let zones_at = if reserve.zones.is_empty() { 0 } else { cursor };
            ensure_len(&mut buf, entry + RESERVE_SIZE);
            buf[entry..entry + 8].copy_from_slice(&reserve.id.to_le_bytes());
            put_array(&mut buf, entry + 8, zones_at, reserve.zones.len());
            if !reserve.zones.is_empty() {
                pointers.push(entry + 8);
                cursor += reserve.zones.len() * ZONE_SIZE;
            }
            ensure_len(&mut buf, cursor);
            for (k, zone) in reserve.zones.iter().enumerate() {
                let at = zones_at + k * ZONE_SIZE;
                write_zone_fields(&mut buf, at, zone);
            }
            for (k, zone) in reserve.zones.iter().enumerate() {
                let at = zones_at + k * ZONE_SIZE;
                if zone.discovered_groups.is_empty() {
                    put_array(&mut buf, at + ZONE_FIELDS, 0, 0);
                    continue;
                }
                cursor = cursor.next_multiple_of(8);
                put_array(
                    &mut buf,
                    at + ZONE_FIELDS,
                    cursor,
                    zone.discovered_groups.len(),
                );
                pointers.push(at + ZONE_FIELDS);
                ensure_len(&mut buf, cursor + zone.discovered_groups.len() * 4);
                for (j, group) in zone.discovered_groups.iter().enumerate() {
                    put_u32(&mut buf, cursor + j * 4, *group);
                }
                cursor += zone.discovered_groups.len() * 4;
            }
        }
        pointers.sort_unstable();
        for pair in pointers.windows(2) {
            // `pair[1] - pair[0]` cabe em u32: o buffer tem poucos MB.
            put_u32(
                &mut buf,
                pair[0] + 4,
                u32::try_from(pair[1] - pair[0]).unwrap_or(0),
            );
        }
        buf
    }

    fn verify(&self, adf: &[u8]) -> Result<(), NeedZonesError> {
        let parsed = Adf::parse(adf.to_vec())?;
        let instance = parsed
            .instances()
            .first()
            .ok_or(NeedZonesError::Verification("sem instância"))?;
        if !parsed.chain_issues(instance)?.is_empty() {
            return Err(NeedZonesError::Verification("cadeia de ponteiros quebrada"));
        }
        if &Self::parse(adf)? != self {
            return Err(NeedZonesError::Verification(
                "zonas diferentes depois de reler",
            ));
        }
        Ok(())
    }
}

/// Cabeçalho do ADF que muda quando a instância muda de tamanho.
struct Header {
    instance_table: usize,
}

impl Header {
    fn read(adf: &[u8]) -> Result<Self, NeedZonesError> {
        let table = bytes::u32_at(adf, 12).ok_or_else(|| layout("cabeçalho truncado"))?;
        Ok(Self {
            instance_table: table as usize,
        })
    }

    /// Ajusta os offsets do cabeçalho do ADF para a tabela de instâncias em `new_table`.
    fn write_shifted(&self, out: &mut [u8], new_table: usize) -> Result<(), NeedZonesError> {
        let delta = new_table - self.instance_table;
        let total = u32::try_from(out.len()).map_err(|_| layout("arquivo grande demais"))?;
        put_u32(
            out,
            12,
            u32::try_from(new_table).map_err(|_| layout("arquivo grande demais"))?,
        );
        for field in [20usize, 36] {
            shift_field(out, field, delta)?;
        }
        // O offset dos string hashes é 0 quando a tabela não existe.
        if bytes::u32_at(out, 28).ok_or_else(|| layout("cabeçalho truncado"))? != 0 {
            shift_field(out, 28, delta)?;
        }
        put_u32(out, 40, total);
        Ok(())
    }
}

fn shift_field(out: &mut [u8], at: usize, delta: usize) -> Result<(), NeedZonesError> {
    let value = bytes::u32_at(out, at).ok_or_else(|| layout("cabeçalho truncado"))?;
    let delta = u32::try_from(delta).map_err(|_| layout("arquivo grande demais"))?;
    put_u32(
        out,
        at,
        value
            .checked_add(delta)
            .ok_or_else(|| layout("offset estourou"))?,
    );
    Ok(())
}

struct ArrayHeader {
    offset: usize,
    count: usize,
}

fn read_array(data: &[u8], at: usize) -> Result<ArrayHeader, NeedZonesError> {
    let offset = bytes::u32_at(data, at).ok_or_else(|| layout("array truncado"))?;
    let count = bytes::u64_at(data, at + 8).ok_or_else(|| layout("array truncado"))?;
    let count = usize::try_from(count).map_err(|_| layout("array com tamanho absurdo"))?;
    Ok(ArrayHeader {
        offset: offset as usize,
        count,
    })
}

/// Confere que `count` itens de `item` bytes a partir de `offset` cabem na instância.
fn check_span(data: &[u8], offset: usize, count: usize, item: usize) -> Result<(), NeedZonesError> {
    if count == 0 {
        return Ok(());
    }
    let bytes_needed = count
        .checked_mul(item)
        .ok_or_else(|| layout("array com tamanho absurdo"))?;
    bytes::slice(data, offset, bytes_needed)
        .map(|_| ())
        .ok_or_else(|| layout("array fora da instância"))
}

fn read_zone(data: &[u8], at: usize) -> Result<Zone, NeedZonesError> {
    let truncated = || layout("zona truncada");
    let groups = read_array(data, at + ZONE_FIELDS)?;
    check_span(data, groups.offset, groups.count, 4)?;
    let mut discovered_groups = Vec::with_capacity(groups.count);
    for j in 0..groups.count {
        discovered_groups.push(bytes::u32_at(data, groups.offset + j * 4).ok_or_else(truncated)?);
    }
    let mut padding = [0u8; 4];
    padding.copy_from_slice(bytes::slice(data, at + 20, 4).ok_or_else(truncated)?);
    Ok(Zone {
        x: f32_at(data, at).ok_or_else(truncated)?,
        y: f32_at(data, at + 4).ok_or_else(truncated)?,
        z: f32_at(data, at + 8).ok_or_else(truncated)?,
        key: bytes::u32_at(data, at + 12).ok_or_else(truncated)?,
        kind: bytes::u32_at(data, at + 16).ok_or_else(truncated)?,
        icon_id: bytes::u64_at(data, at + 24).ok_or_else(truncated)?,
        start: f32_at(data, at + 32).ok_or_else(truncated)?,
        end: f32_at(data, at + 36).ok_or_else(truncated)?,
        species_name: bytes::u32_at(data, at + 40).ok_or_else(truncated)?,
        schedule: bytes::u32_at(data, at + 44).ok_or_else(truncated)?,
        discovered_groups,
        padding,
    })
}

fn write_zone_fields(buf: &mut [u8], at: usize, zone: &Zone) {
    put_u32(buf, at, zone.x.to_bits());
    put_u32(buf, at + 4, zone.y.to_bits());
    put_u32(buf, at + 8, zone.z.to_bits());
    put_u32(buf, at + 12, zone.key);
    put_u32(buf, at + 16, zone.kind);
    buf[at + 20..at + 24].copy_from_slice(&zone.padding);
    buf[at + 24..at + 32].copy_from_slice(&zone.icon_id.to_le_bytes());
    put_u32(buf, at + 32, zone.start.to_bits());
    put_u32(buf, at + 36, zone.end.to_bits());
    put_u32(buf, at + 40, zone.species_name);
    put_u32(buf, at + 44, zone.schedule);
}

fn ensure_len(buf: &mut Vec<u8>, len: usize) {
    if buf.len() < len {
        buf.resize(len, 0);
    }
}

fn put_u32(buf: &mut [u8], at: usize, value: u32) {
    buf[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// Escreve um cabeçalho de array (`offset`, `next` = 0, `count`) em `at`.
fn put_array(buf: &mut Vec<u8>, at: usize, offset: usize, count: usize) {
    ensure_len(buf, at + ARRAY_HEADER);
    put_u32(buf, at, u32::try_from(offset).unwrap_or(0));
    put_u32(buf, at + 4, 0);
    buf[at + 8..at + 16].copy_from_slice(&(count as u64).to_le_bytes());
}
