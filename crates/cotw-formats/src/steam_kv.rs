//! Arquivos de estatísticas que a Steam guarda no computador (`appcache/stats/*.bin`): o formato
//! binário de `KeyValues`, usado pelo esquema do jogo (nomes e descrições das conquistas) e pelo
//! progresso de cada conta (quais conquistas foram obtidas).
//!
//! Cada nó é `tipo (u8), nome (texto terminado em NUL), valor`; o tipo 0 abre um grupo, que termina
//! no byte 8; 1 é texto, 2/4/6 inteiros de 32 bits, 3 decimal de 32 bits, 5 texto largo e 7/10
//! inteiros de 64 bits.

use crate::bytes;

/// Valor de um nó.
#[derive(Debug, Clone, PartialEq)]
pub enum Kv {
    /// Grupo de nós, na ordem do arquivo.
    Group(Vec<(String, Kv)>),
    /// Texto.
    Text(String),
    /// Inteiro de 32 bits (com sinal).
    Int(i32),
    /// Decimal de 32 bits.
    Float(f32),
    /// Inteiro de 64 bits.
    Long(u64),
}

/// Erros ao ler o arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum KvError {
    /// O arquivo termina no meio de um nó.
    #[error("arquivo de estatísticas da Steam truncado")]
    Truncated,
    /// Tipo de nó que não conhecemos.
    #[error("tipo de nó {0} desconhecido no arquivo da Steam")]
    UnknownType(u8),
    /// Grupos aninhados demais (arquivo corrompido).
    #[error("arquivo da Steam com grupos fundos demais")]
    TooDeep,
}

const MAX_DEPTH: usize = 32;

impl Kv {
    /// Filho de um grupo pelo nome.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Kv> {
        match self {
            Self::Group(items) => items.iter().find(|(k, _)| k == name).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Filhos de um grupo, na ordem do arquivo (vazio se não for um grupo).
    #[must_use]
    pub fn items(&self) -> &[(String, Kv)] {
        match self {
            Self::Group(items) => items,
            _ => &[],
        }
    }

    /// Texto, se for um texto.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Número inteiro, se for um inteiro (32 ou 64 bits) ou um texto com um número.
    #[must_use]
    pub fn number(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(i64::from(*n)),
            Self::Long(n) => i64::try_from(*n).ok(),
            Self::Text(text) => text.trim().parse().ok(),
            _ => None,
        }
    }

    /// Lê o arquivo inteiro como um grupo raiz.
    ///
    /// # Errors
    ///
    /// [`KvError`] se o arquivo estiver truncado, tiver um tipo desconhecido ou grupos fundos demais.
    pub fn parse(data: &[u8]) -> Result<Self, KvError> {
        let mut at = 0;
        let items = read_group(data, &mut at, 0)?;
        Ok(Self::Group(items))
    }
}

fn read_text(data: &[u8], at: &mut usize) -> Result<String, KvError> {
    let rest = data.get(*at..).ok_or(KvError::Truncated)?;
    let end = rest
        .iter()
        .position(|b| *b == 0)
        .ok_or(KvError::Truncated)?;
    let text = String::from_utf8_lossy(&rest[..end]).into_owned();
    *at += end + 1;
    Ok(text)
}

fn read_group(data: &[u8], at: &mut usize, depth: usize) -> Result<Vec<(String, Kv)>, KvError> {
    if depth > MAX_DEPTH {
        return Err(KvError::TooDeep);
    }
    let mut items = Vec::new();
    loop {
        let Some(kind) = data.get(*at).copied() else {
            // O arquivo raiz pode terminar sem o byte de fechamento.
            return if depth == 0 {
                Ok(items)
            } else {
                Err(KvError::Truncated)
            };
        };
        *at += 1;
        if kind == 8 {
            return Ok(items);
        }
        let name = read_text(data, at)?;
        let value = match kind {
            0 => Kv::Group(read_group(data, at, depth + 1)?),
            1 => Kv::Text(read_text(data, at)?),
            2 | 4 | 6 => {
                let n = bytes::u32_at(data, *at).ok_or(KvError::Truncated)?;
                *at += 4;
                Kv::Int(n.cast_signed())
            }
            3 => {
                let n = bytes::u32_at(data, *at).ok_or(KvError::Truncated)?;
                *at += 4;
                Kv::Float(f32::from_bits(n))
            }
            7 | 10 => {
                let low = u64::from(bytes::u32_at(data, *at).ok_or(KvError::Truncated)?);
                let high = u64::from(bytes::u32_at(data, *at + 4).ok_or(KvError::Truncated)?);
                *at += 8;
                Kv::Long(low | (high << 32))
            }
            other => return Err(KvError::UnknownType(other)),
        };
        items.push((name, value));
    }
}

/// Uma conquista descrita no esquema do jogo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementInfo {
    /// Nome interno (`ach_012`).
    pub api_name: String,
    /// Nome para exibir.
    pub name: String,
    /// Como obter.
    pub description: String,
    /// Escondida até ser obtida.
    pub hidden: bool,
    /// Número da estatística e do bit que guardam o progresso na conta do jogador.
    pub stat: u32,
    /// Posição do bit dentro dessa estatística.
    pub bit: u32,
}

/// Conquistas do esquema da Steam, com o idioma pedido (`english` se faltar).
///
/// O esquema tem, para cada estatística (`stats.<id>`), um tipo; as de tipo 4 guardam um campo de bits
/// (`bits.<n>`) onde cada bit é uma conquista, com nome e descrição em `display`.
#[must_use]
pub fn achievements(schema: &Kv, language: &str) -> Vec<AchievementInfo> {
    let Some(stats) = schema
        .items()
        .iter()
        .find_map(|(_, game)| game.get("stats"))
    else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for (id, stat) in stats.items() {
        let Ok(stat_id) = id.parse::<u32>() else {
            continue;
        };
        let Some(bits) = stat.get("bits") else {
            continue;
        };
        for (bit, entry) in bits.items() {
            let Some(api_name) = entry.get("name").and_then(Kv::text) else {
                continue;
            };
            let display = |field: &str| {
                let node = entry.get("display")?.get(field)?;
                node.get(language)
                    .or_else(|| node.get("english"))
                    .and_then(Kv::text)
                    .map(str::to_owned)
            };
            found.push(AchievementInfo {
                api_name: api_name.to_owned(),
                name: display("name").unwrap_or_else(|| api_name.to_owned()),
                description: display("desc").unwrap_or_default(),
                hidden: entry
                    .get("display")
                    .and_then(|d| d.get("hidden"))
                    .and_then(Kv::number)
                    == Some(1),
                stat: stat_id,
                bit: bit.parse().unwrap_or(0),
            });
        }
    }
    found
}

/// Campos de bits de conquistas da conta (`stat` → bits), lidos do arquivo `UserGameStats_*.bin`.
#[must_use]
pub fn unlocked_masks(user: &Kv) -> Vec<(u32, u32)> {
    let Some(stats) = user
        .get("cache")
        .or_else(|| user.items().iter().find_map(|(_, game)| game.get("cache")))
    else {
        return Vec::new();
    };
    stats
        .items()
        .iter()
        .filter_map(|(id, value)| {
            let stat = id.parse::<u32>().ok()?;
            let mask = value.get("data").and_then(Kv::number)?;
            Some((stat, u32::try_from(mask & 0xFFFF_FFFF).ok()?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(kind: u8, name: &str, value: &[u8]) -> Vec<u8> {
        let mut out = vec![kind];
        out.extend_from_slice(name.as_bytes());
        out.push(0);
        out.extend_from_slice(value);
        out
    }

    fn text(value: &str) -> Vec<u8> {
        let mut out = value.as_bytes().to_vec();
        out.push(0);
        out
    }

    #[test]
    fn groups_texts_and_numbers_are_read() {
        let mut inner = node(1, "name", &text("ach_001"));
        inner.extend(node(2, "n", &7_i32.to_le_bytes()));
        inner.extend(node(7, "big", &(u64::from(u32::MAX) + 5).to_le_bytes()));
        inner.extend(node(3, "f", &1.5_f32.to_le_bytes()));
        inner.push(8);
        let mut file = node(0, "game", &inner);
        file.push(8);
        let kv = Kv::parse(&file).unwrap();
        let game = kv.get("game").unwrap();
        assert_eq!(game.get("name").and_then(Kv::text), Some("ach_001"));
        assert_eq!(game.get("n").and_then(Kv::number), Some(7));
        assert_eq!(
            game.get("big").and_then(Kv::number),
            Some(i64::from(u32::MAX) + 5)
        );
        assert_eq!(game.get("f"), Some(&Kv::Float(1.5)));
        assert_eq!(game.items().len(), 4);
        assert_eq!(kv.items().len(), 1);
    }

    #[test]
    fn a_text_with_a_number_counts_as_a_number() {
        assert_eq!(Kv::Text(" 12 ".into()).number(), Some(12));
        assert_eq!(Kv::Text("x".into()).number(), None);
        assert_eq!(Kv::Float(1.0).number(), None);
        assert_eq!(Kv::Int(1).text(), None);
        assert_eq!(Kv::Int(1).items().len(), 0);
        assert_eq!(Kv::Int(1).get("x"), None);
    }

    #[test]
    fn broken_files_are_errors() {
        assert_eq!(
            Kv::parse(&node(1, "a", b"sem fim")),
            Err(KvError::Truncated)
        );
        assert_eq!(Kv::parse(&[0, b'a', 0]), Err(KvError::Truncated));
        assert_eq!(
            Kv::parse(&node(99, "a", &[0])),
            Err(KvError::UnknownType(99))
        );
        assert_eq!(Kv::parse(&node(2, "a", &[1, 2])), Err(KvError::Truncated));
        let deep: Vec<u8> = (0..40).flat_map(|_| [0, b'g', 0]).collect();
        assert_eq!(Kv::parse(&deep), Err(KvError::TooDeep));
    }

    #[test]
    fn achievements_come_from_bit_fields_with_a_language_fallback() {
        let display = |name: &str, desc: &str, hidden: &str| {
            let mut name_group = node(1, "english", &text(name));
            name_group.extend(node(1, "brazilian", &text(&format!("{name} PT"))));
            name_group.push(8);
            let mut desc_group = node(1, "english", &text(desc));
            desc_group.push(8);
            let mut d = node(0, "name", &name_group);
            d.extend(node(0, "desc", &desc_group));
            d.extend(node(1, "hidden", &text(hidden)));
            d.push(8);
            d
        };
        let mut bit0 = node(1, "name", &text("ach_000"));
        bit0.extend(node(0, "display", &display("Primeira", "Cace.", "0")));
        bit0.push(8);
        let mut bit1 = node(1, "name", &text("ach_001"));
        bit1.extend(node(0, "display", &display("Segunda", "Segredo.", "1")));
        bit1.push(8);
        let mut fields = node(0, "0", &bit0);
        fields.extend(node(0, "1", &bit1));
        fields.push(8);
        let mut stat = node(0, "bits", &fields);
        stat.push(8);
        let mut stats = node(0, "5", &stat);
        stats.push(8);
        let mut game = node(0, "stats", &stats);
        game.push(8);
        let mut file = node(0, "518790", &game);
        file.push(8);
        let schema = Kv::parse(&file).unwrap();

        let list = achievements(&schema, "brazilian");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Primeira PT");
        assert_eq!(list[1].name, "Segunda PT");
        assert_eq!(list[0].description, "Cace.");
        assert!(!list[0].hidden && list[1].hidden);
        assert_eq!((list[1].stat, list[1].bit), (5, 1));
        // Sem o idioma pedido volta para o inglês.
        assert_eq!(achievements(&schema, "french")[0].name, "Primeira");
        assert_eq!(achievements(&Kv::Group(Vec::new()), "english").len(), 0);
    }

    #[test]
    fn unlocked_bits_come_from_the_account_cache() {
        let mut stat = node(2, "data", &0b101_i32.to_le_bytes());
        stat.push(8);
        let mut cache = node(0, "5", &stat);
        cache.push(8);
        // Como a Steam grava: o grupo `cache` na raiz; o esquema aninha tudo sob o número do jogo.
        let mut file = node(0, "cache", &cache);
        file.push(8);
        let kv = Kv::parse(&file).unwrap();
        assert_eq!(unlocked_masks(&kv), vec![(5, 0b101)]);
        let mut nested = node(0, "518790", &file);
        nested.push(8);
        assert_eq!(
            unlocked_masks(&Kv::parse(&nested).unwrap()),
            vec![(5, 0b101)]
        );
        assert_eq!(unlocked_masks(&Kv::Group(Vec::new())).len(), 0);
    }
}
