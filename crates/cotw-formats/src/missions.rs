//! Missões: as definições que vêm com o jogo (`missions/missions_N.hpmissionsc`), os textos
//! (`missions/missions.stringlookup`) e o progresso gravado no save (`missions_missiondata_adf`).

use std::collections::HashMap;

use crate::adf::{Adf, AdfError, Value};

/// Erros ao ler missões.
#[derive(Debug, thiserror::Error)]
pub enum MissionError {
    /// O arquivo não é um ADF válido.
    #[error(transparent)]
    Adf(#[from] AdfError),
    /// O arquivo não tem a estrutura esperada.
    #[error("arquivo de missões com estrutura inesperada")]
    Layout,
}

/// Um objetivo de uma missão, como o jogo o define.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveDef {
    /// Número do objetivo.
    pub id: i64,
    /// Quantas vezes é preciso cumpri-lo.
    pub count_required: i64,
}

/// Uma missão, como o jogo a define.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionDef {
    /// Número da missão (o mesmo que o save usa).
    pub id: i64,
    /// Hash do título, para procurar o texto.
    pub summary: u64,
    /// Hash da descrição, para procurar o texto.
    pub description: u64,
    /// Faz parte da história principal da reserva.
    pub required: bool,
    /// Aparece na lista de missões do jogador desde o começo.
    pub visible: bool,
    /// Objetivos, na ordem do jogo.
    pub objectives: Vec<ObjectiveDef>,
}

fn member<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match value {
        Value::Struct(fields) => fields.iter().find(|(k, _)| k == name).map(|(_, v)| v),
        _ => None,
    }
}

fn items(value: &Value) -> &[Value] {
    match value {
        Value::Array(items) => items,
        _ => &[],
    }
}

fn number(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Signed(n) => Some(*n),
        Value::Unsigned(n) => i64::try_from(*n).ok(),
        _ => None,
    }
}

fn hash64(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::StringHash { hash, .. } => Some(*hash),
        Value::Unsigned(n) => Some(*n),
        _ => None,
    }
}

fn root(adf: &Adf) -> Result<Value, MissionError> {
    let instance = adf.instances().first().ok_or(MissionError::Layout)?;
    Ok(adf.read_instance(instance)?)
}

/// Lê as missões de um arquivo `missions_N.hpmissionsc`.
///
/// # Errors
///
/// [`MissionError`] se o arquivo for inválido.
pub fn parse_definitions(adf: &[u8]) -> Result<Vec<MissionDef>, MissionError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let root = root(&parsed)?;
    let missions = member(&root, "Missions").ok_or(MissionError::Layout)?;
    items(missions)
        .iter()
        .map(|mission| {
            let objectives = member(mission, "Objectives")
                .map(items)
                .unwrap_or_default()
                .iter()
                .filter_map(|o| {
                    Some(ObjectiveDef {
                        id: number(member(o, "Id"))?,
                        count_required: number(member(o, "CountRequired")).unwrap_or(1),
                    })
                })
                .collect();
            Ok(MissionDef {
                id: number(member(mission, "Id")).ok_or(MissionError::Layout)?,
                summary: hash64(member(mission, "Summary")).ok_or(MissionError::Layout)?,
                description: hash64(member(mission, "Description")).ok_or(MissionError::Layout)?,
                required: number(member(mission, "Required")) == Some(1),
                visible: number(member(mission, "Visible")) == Some(1),
                objectives,
            })
        })
        .collect()
}

/// Número do arquivo de missões (`missions/missions_N.hpmissions`) citado na configuração da reserva.
///
/// Cada reserva diz qual é o seu; ele nem sempre coincide com o número da reserva.
#[must_use]
pub fn missions_file_number(reserve_settings: &[u8]) -> Option<u32> {
    const MARK: &[u8] = b"missions/missions_";
    let at = reserve_settings
        .windows(MARK.len())
        .position(|window| window == MARK)?;
    let digits: Vec<u8> = reserve_settings[at + MARK.len()..]
        .iter()
        .copied()
        .take_while(u8::is_ascii_digit)
        .collect();
    std::str::from_utf8(&digits).ok()?.parse().ok()
}

/// Textos de todas as missões num idioma: hash do nome → texto.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringLookup {
    texts: HashMap<u64, String>,
}

impl StringLookup {
    /// Texto do hash, se existir.
    #[must_use]
    pub fn get(&self, hash: u64) -> Option<&str> {
        self.texts.get(&hash).map(String::as_str)
    }

    /// Todos os hashes que têm texto.
    #[must_use]
    pub fn keys(&self) -> Vec<u64> {
        self.texts.keys().copied().collect()
    }

    /// Quantos textos há.
    #[must_use]
    pub fn len(&self) -> usize {
        self.texts.len()
    }

    /// Sem nenhum texto.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
}

/// Lê `missions.stringlookup` no idioma de índice `language` (7 é o português do Brasil; se o
/// arquivo tiver menos idiomas, usa o primeiro).
///
/// # Errors
///
/// [`MissionError`] se o arquivo for inválido.
pub fn parse_lookup(adf: &[u8], language: usize) -> Result<StringLookup, MissionError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let root = root(&parsed)?;
    let languages = items(member(&root, "Languages").ok_or(MissionError::Layout)?);
    let chosen = languages
        .get(language)
        .or_else(|| languages.first())
        .ok_or(MissionError::Layout)?;
    let Some(Value::Bytes(text)) = member(&root, "Text") else {
        return Err(MissionError::Layout);
    };
    let mut texts = HashMap::new();
    for pair in items(member(chosen, "SortedPairs").ok_or(MissionError::Layout)?) {
        let (Some(hash), Some(offset)) = (
            hash64(member(pair, "Hash")),
            number(member(pair, "TextOffset")).and_then(|n| usize::try_from(n).ok()),
        ) else {
            continue;
        };
        let Some(rest) = text.get(offset..) else {
            continue;
        };
        let end = rest.iter().position(|b| *b == 0).unwrap_or(rest.len());
        texts.insert(hash, String::from_utf8_lossy(&rest[..end]).into_owned());
    }
    Ok(StringLookup { texts })
}

/// Situação de uma missão ou objetivo no save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Ainda não liberada.
    Locked,
    /// Liberada e em andamento.
    Active,
    /// Concluída.
    Done,
}

impl State {
    fn from_code(code: i64) -> Self {
        match code {
            4 => Self::Done,
            2 | 3 => Self::Active,
            _ => Self::Locked,
        }
    }
}

/// Progresso de um objetivo no save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveProgress {
    /// Número do objetivo.
    pub id: i64,
    /// Quantas vezes já foi cumprido.
    pub done: i64,
    /// Situação.
    pub state: State,
}

/// Progresso de uma missão no save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionProgress {
    /// Número da missão.
    pub id: i64,
    /// Situação.
    pub state: State,
    /// Objetivos.
    pub objectives: Vec<ObjectiveProgress>,
}

/// Progresso de todas as missões de uma reserva.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignProgress {
    /// Número da reserva.
    pub reserve: i64,
    /// Situação da campanha inteira.
    pub state: State,
    /// Missões.
    pub missions: Vec<MissionProgress>,
}

fn progress_of(node: &Value) -> Option<(i64, State, i64)> {
    Some((
        number(member(node, "Id"))?,
        State::from_code(number(member(node, "CompletionState"))?),
        number(member(node, "CompletionsDone")).unwrap_or(0),
    ))
}

/// Lê o progresso das missões de `missions_missiondata_adf`.
///
/// # Errors
///
/// [`MissionError`] se o arquivo for inválido.
pub fn parse_progress(adf: &[u8]) -> Result<Vec<CampaignProgress>, MissionError> {
    let parsed = Adf::parse(adf.to_vec())?;
    let root = root(&parsed)?;
    let memory = member(&root, "Campaigns_Memory").ok_or(MissionError::Layout)?;
    let chain = items(member(memory, "ChainMemory").ok_or(MissionError::Layout)?);
    let mut campaigns = Vec::new();
    for entry in chain {
        let (Some(reserve), Some(value)) = (number(member(entry, "Key")), member(entry, "Value"))
        else {
            continue;
        };
        let state = State::from_code(number(member(value, "CompletionState")).unwrap_or(0));
        let missions = member(value, "Children")
            .map(items)
            .unwrap_or_default()
            .iter()
            .filter_map(|mission| {
                let (id, state, _) = progress_of(mission)?;
                let objectives = member(mission, "Children")
                    .map(items)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|o| {
                        let (id, state, done) = progress_of(o)?;
                        Some(ObjectiveProgress { id, done, state })
                    })
                    .collect();
                Some(MissionProgress {
                    id,
                    state,
                    objectives,
                })
            })
            .collect();
        campaigns.push(CampaignProgress {
            reserve,
            state,
            missions,
        });
    }
    Ok(campaigns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_codes_follow_the_game() {
        assert_eq!(State::from_code(4), State::Done);
        assert_eq!(State::from_code(2), State::Active);
        assert_eq!(State::from_code(3), State::Active);
        assert_eq!(State::from_code(0), State::Locked);
        assert_eq!(State::from_code(9), State::Locked);
    }

    #[test]
    fn invalid_files_are_errors_not_panics() {
        assert!(parse_definitions(b"nao e um adf").is_err());
        assert!(parse_lookup(b"nao e um adf", 7).is_err());
        assert!(parse_progress(b"nao e um adf").is_err());
    }

    #[test]
    fn helpers_read_numbers_and_hashes_from_values() {
        assert_eq!(number(Some(&Value::Signed(-3))), Some(-3));
        assert_eq!(number(Some(&Value::Unsigned(5))), Some(5));
        assert_eq!(number(Some(&Value::Float(1.0))), None);
        assert_eq!(number(None), None);
        let hash = Value::StringHash {
            hash: 7,
            text: None,
        };
        assert_eq!(hash64(Some(&hash)), Some(7));
        let wide = Value::StringHash {
            hash: u64::MAX,
            text: None,
        };
        assert_eq!(hash64(Some(&wide)), Some(u64::MAX));
        assert_eq!(hash64(Some(&Value::Unsigned(9))), Some(9));
        assert_eq!(hash64(Some(&Value::Float(1.0))), None);
        assert_eq!(items(&Value::Signed(1)).len(), 0);
        assert!(member(&Value::Signed(1), "x").is_none());
    }

    #[test]
    fn the_missions_file_is_named_inside_the_reserve_settings() {
        let settings = b" climate/x missions/missions_13.hpmissions more";
        assert_eq!(missions_file_number(settings), Some(13));
        assert_eq!(missions_file_number(b"missions/missions_.hpmissions"), None);
        assert_eq!(missions_file_number(b"nada"), None);
    }

    #[test]
    fn an_empty_lookup_has_no_texts() {
        let lookup = StringLookup::default();
        assert!(lookup.is_empty());
        assert_eq!(lookup.len(), 0);
        assert_eq!(lookup.get(1), None);
    }
}
