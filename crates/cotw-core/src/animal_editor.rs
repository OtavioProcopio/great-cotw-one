//! Editor de animais (Premium): ver e mudar peso, pontuação, sexo e Great One de animais que já
//! existem no save, sempre no mesmo tamanho de arquivo (spec `animal-editor`).
//!
//! O app nunca cria nem remove animais: só troca valores dentro dos registros existentes.

use std::fs;
use std::path::Path;

use cotw_formats::adf::Adf;
use cotw_formats::envelope::SaveEnvelope;
use cotw_formats::population::{
    AnimalEdit, PopulationError, edit_animals, locate_animals, parse_groups,
};
use serde::{Deserialize, Serialize};

use crate::discovery::reserve_name;
use crate::language::Language;
use crate::species::display_name;
use crate::transaction::{FileChange, ItemChange};
use crate::unlock::SpawnSource;
use crate::zones::assign_species;

/// Erros do editor.
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    /// Não foi possível ler o arquivo da reserva.
    #[error("não foi possível ler {0}")]
    Read(String),
    /// O arquivo da reserva não tem a estrutura esperada.
    #[error("{0} não pôde ser interpretado")]
    Parse(String),
    /// A reserva não tem mapas de spawn no jogo (não dá para saber a espécie dos grupos).
    #[error("a reserva {0} não tem dados para editar")]
    NoData(u32),
    /// A espécie não existe nesta reserva.
    #[error("a espécie {0} não existe nesta reserva")]
    NoSpecies(String),
    /// Uma edição aponta para um animal que não é da espécie escolhida ou que não existe.
    #[error("a edição aponta para um animal que não existe nesta espécie")]
    WrongAnimal,
    /// Um valor da edição não vale (peso ou pontuação negativos ou que não são números).
    #[error("há um valor inválido na edição")]
    InvalidValue,
    /// Peso ou pontuação fora do que existe no jogo para a espécie e o sexo.
    #[error("peso ou pontuação fora do que o jogo tem para esta espécie e este sexo")]
    OutOfRange,
    /// A marca de Great One não pode ser mudada nesta espécie.
    #[error("esta espécie não permite mudar a marca de Great One")]
    GreatOneNotAllowed,
    /// Nenhuma edição muda alguma coisa.
    #[error("nada a mudar")]
    NothingToChange,
    /// Falha ao montar o arquivo novo.
    #[error("não foi possível montar o arquivo novo")]
    Build,
}

impl EditorError {
    /// Código estável para a interface traduzir.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Read(_) => "read",
            Self::Parse(_) => "editorParse",
            Self::NoData(_) => "editorNoData",
            Self::NoSpecies(_) => "editorNoSpecies",
            Self::WrongAnimal => "editorWrongAnimal",
            Self::InvalidValue => "editorInvalidValue",
            Self::OutOfRange => "editorOutOfRange",
            Self::GreatOneNotAllowed => "editorGreatOne",
            Self::NothingToChange => "nothingToChange",
            Self::Build => "editorBuild",
        }
    }
}

/// Um animal como o editor o mostra.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableAnimal {
    /// Posição do animal no grupo (a partir de 0).
    pub index: usize,
    /// Macho.
    pub male: bool,
    /// Peso em quilos.
    pub weight: f32,
    /// Pontuação do troféu.
    pub score: f32,
    /// Marca de Great One.
    pub great_one: bool,
}

/// Um grupo com os animais da espécie.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableGroup {
    /// Número do grupo para o jogador (posição + 1).
    pub number: usize,
    /// Animais do grupo.
    pub animals: Vec<EditableAnimal>,
}

/// Margem que o app aceita além do que já existe no jogo (10 %).
const MARGIN: f32 = 0.10;

/// O que o app aceita para um sexo de uma espécie: o que já existe no jogo, com 10 % de folga.
///
/// Fica dentro do que o jogo mesmo gera, para a edição não desequilibrar a caça nem a população.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// Menor peso aceito.
    pub min_weight: f32,
    /// Maior peso aceito.
    pub max_weight: f32,
    /// Menor pontuação aceita.
    pub min_score: f32,
    /// Maior pontuação aceita.
    pub max_score: f32,
    /// Peso típico (mediana dos animais do sexo), sugerido ao trocar de sexo.
    pub typical_weight: f32,
    /// Pontuação típica (mediana), sugerida ao trocar de sexo.
    pub typical_score: f32,
}

impl Limits {
    fn from_samples(weights: &mut [f32], scores: &mut [f32]) -> Option<Self> {
        weights.sort_by(f32::total_cmp);
        scores.sort_by(f32::total_cmp);
        let (min_w, max_w) = (*weights.first()?, *weights.last()?);
        let (min_s, max_s) = (*scores.first()?, *scores.last()?);
        Some(Self {
            min_weight: min_w * (1.0 - MARGIN),
            max_weight: max_w * (1.0 + MARGIN),
            min_score: min_s * (1.0 - MARGIN),
            max_score: max_s * (1.0 + MARGIN),
            typical_weight: weights[weights.len() / 2],
            typical_score: scores[scores.len() / 2],
        })
    }
}

/// Limites de cada sexo (se faltar um sexo na espécie, usa os do outro).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SexLimits {
    /// Machos.
    pub male: Limits,
    /// Fêmeas.
    pub female: Limits,
}

impl SexLimits {
    fn of(&self, male: bool) -> &Limits {
        if male { &self.male } else { &self.female }
    }
}

fn sex_limits(groups: &[EditableGroup]) -> Option<SexLimits> {
    let collect = |male: bool| {
        let mut weights = Vec::new();
        let mut scores = Vec::new();
        for a in groups
            .iter()
            .flat_map(|g| &g.animals)
            .filter(|a| a.male == male)
        {
            weights.push(a.weight);
            scores.push(a.score);
        }
        Limits::from_samples(&mut weights, &mut scores)
    };
    let (males, females) = (collect(true), collect(false));
    Some(SexLimits {
        male: males.or(females)?,
        female: females.or(males)?,
    })
}

/// Tudo o que a tela do editor precisa de uma espécie numa reserva.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeciesEditor {
    /// Número da reserva.
    pub reserve: u32,
    /// Nome da reserva.
    pub reserve_name: String,
    /// Nome que o jogo usa para a espécie.
    pub species: String,
    /// Nome para exibir.
    pub name: String,
    /// A marca de Great One pode ser mudada (o save a guarda e a espécie já tem Great One aqui).
    pub great_one_allowed: bool,
    /// O que o app aceita de peso e pontuação, por sexo.
    pub limits: SexLimits,
    /// Grupos da espécie.
    pub groups: Vec<EditableGroup>,
}

/// Uma mudança pedida pelo jogador.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalChange {
    /// Número do grupo (posição + 1).
    pub group: usize,
    /// Posição do animal no grupo (a partir de 0).
    pub index: usize,
    /// Novo sexo.
    #[serde(default)]
    pub male: Option<bool>,
    /// Novo peso.
    #[serde(default)]
    pub weight: Option<f32>,
    /// Nova pontuação.
    #[serde(default)]
    pub score: Option<f32>,
    /// Nova marca de Great One.
    #[serde(default)]
    pub great_one: Option<bool>,
}

fn population_file(reserve: u32) -> String {
    format!("animal_population_{reserve}")
}

struct Loaded {
    envelope: SaveEnvelope,
    adf: Vec<u8>,
    groups: Vec<cotw_formats::population::Group>,
    species_of_group: Vec<Option<String>>,
    great_one_field: bool,
}

fn load(save_dir: &Path, source: &dyn SpawnSource, reserve: u32) -> Result<Loaded, EditorError> {
    let file = population_file(reserve);
    let raw = fs::read(save_dir.join(&file)).map_err(|_| EditorError::Read(file.clone()))?;
    let envelope = SaveEnvelope::parse(&raw).map_err(|_| EditorError::Parse(file.clone()))?;
    let adf = envelope.adf().to_vec();
    let groups = parse_groups(&adf).map_err(|_| EditorError::Parse(file.clone()))?;
    let maps = source
        .maps(reserve)
        .map_err(|_| EditorError::NoData(reserve))?;
    let (species_of_group, _) = assign_species(&groups, &maps);
    let great_one_field = Adf::parse(adf.clone())
        .ok()
        .and_then(|parsed| locate_animals(&parsed).ok())
        .is_some_and(|slots| {
            slots
                .first()
                .is_none_or(cotw_formats::population::AnimalSlot::supports_great_one)
        });
    Ok(Loaded {
        envelope,
        adf,
        groups,
        species_of_group,
        great_one_field,
    })
}

/// Os animais de uma espécie numa reserva, agrupados, para a tela do editor.
///
/// # Errors
///
/// [`EditorError`] se o arquivo da reserva não puder ser lido ou a espécie não existir nela.
pub fn species_editor(
    save_dir: &Path,
    source: &dyn SpawnSource,
    reserve: u32,
    species: &str,
    language: Language,
) -> Result<SpeciesEditor, EditorError> {
    let loaded = load(save_dir, source, reserve)?;
    let mut groups = Vec::new();
    let mut any_great_one = false;
    for (number, (group, of)) in loaded
        .groups
        .iter()
        .zip(&loaded.species_of_group)
        .enumerate()
    {
        if of.as_deref() != Some(species) {
            continue;
        }
        let animals: Vec<EditableAnimal> = group
            .animals
            .iter()
            .enumerate()
            .map(|(index, a)| EditableAnimal {
                index,
                male: a.male,
                weight: a.weight,
                score: a.score,
                great_one: a.great_one,
            })
            .collect();
        any_great_one |= animals.iter().any(|a| a.great_one);
        groups.push(EditableGroup {
            number: number + 1,
            animals,
        });
    }
    let limits = sex_limits(&groups).ok_or_else(|| EditorError::NoSpecies(species.to_owned()))?;
    Ok(SpeciesEditor {
        reserve,
        reserve_name: reserve_name(reserve),
        species: species.to_owned(),
        name: display_name(species, language),
        great_one_allowed: loaded.great_one_field && any_great_one,
        limits,
        groups,
    })
}

/// O que gravar para aplicar as mudanças.
pub struct EditPlan {
    /// Arquivo pronto para a gravação segura.
    pub file: FileChange,
    /// Quantos animais mudam.
    pub animals: usize,
}

fn describe(old: &EditableAnimal, new: &EditableAnimal) -> (String, String) {
    let part = |a: &EditableAnimal| {
        format!(
            "{} · {} kg · pontuação {}{}",
            if a.male { "macho" } else { "fêmea" },
            a.weight.round(),
            a.score.round(),
            if a.great_one { " · Great One" } else { "" }
        )
    };
    (part(old), part(new))
}

/// Confere uma mudança contra o editor e devolve o animal como era e como ficaria; `None` se a
/// mudança não altera nada.
fn evaluate(
    editor: &SpeciesEditor,
    change: &AnimalChange,
) -> Result<Option<(EditableAnimal, EditableAnimal)>, EditorError> {
    let group = editor
        .groups
        .iter()
        .find(|g| g.number == change.group)
        .ok_or(EditorError::WrongAnimal)?;
    let old = *group
        .animals
        .iter()
        .find(|a| a.index == change.index)
        .ok_or(EditorError::WrongAnimal)?;
    if change.great_one.is_some_and(|g| g != old.great_one) && !editor.great_one_allowed {
        return Err(EditorError::GreatOneNotAllowed);
    }
    let new = EditableAnimal {
        index: old.index,
        male: change.male.unwrap_or(old.male),
        weight: change.weight.unwrap_or(old.weight),
        score: change.score.unwrap_or(old.score),
        great_one: change.great_one.unwrap_or(old.great_one),
    };
    if !(new.weight.is_finite() && new.score.is_finite() && new.weight >= 0.0 && new.score >= 0.0) {
        return Err(EditorError::InvalidValue);
    }
    if new == old {
        return Ok(None);
    }
    // Equilíbrio: o que mudou precisa caber no que o jogo tem para o sexo final.
    let sex_changed = new.male != old.male;
    let limits = editor.limits.of(new.male);
    let weight_changed = new.weight.to_bits() != old.weight.to_bits() || sex_changed;
    let score_changed = new.score.to_bits() != old.score.to_bits() || sex_changed;
    if (weight_changed && !(limits.min_weight..=limits.max_weight).contains(&new.weight))
        || (score_changed && !(limits.min_score..=limits.max_score).contains(&new.score))
    {
        return Err(EditorError::OutOfRange);
    }
    Ok(Some((old, new)))
}

/// Conferência depois de gravar: mesmos grupos e rotas, os animais com os valores pedidos e a
/// cadeia de ponteiros sem quebras.
fn verifier(expected: Vec<cotw_formats::population::Group>) -> crate::transaction::Verifier {
    Box::new(move |bytes| {
        let again = SaveEnvelope::parse(bytes).map_err(|e| e.to_string())?;
        let groups = parse_groups(again.adf()).map_err(|e| e.to_string())?;
        if groups != expected {
            return Err("os animais gravados diferem dos pedidos".into());
        }
        let parsed = Adf::parse(again.adf().to_vec()).map_err(|e| e.to_string())?;
        for instance in parsed.instances() {
            if !parsed
                .chain_issues(instance)
                .map_err(|e| e.to_string())?
                .is_empty()
            {
                return Err("a cadeia de ponteiros do arquivo ficou quebrada".into());
            }
        }
        Ok(())
    })
}

/// Monta o arquivo novo com as mudanças, conferindo tudo antes de gravar.
///
/// # Errors
///
/// [`EditorError`] se uma mudança apontar para um animal fora da espécie, tiver valor inválido
/// ou fora do que o jogo tem para o sexo, tentar mudar o Great One onde não é permitido ou se
/// nada mudar de fato.
pub fn build_edit(
    save_dir: &Path,
    source: &dyn SpawnSource,
    reserve: u32,
    species: &str,
    changes: &[AnimalChange],
    language: Language,
) -> Result<EditPlan, EditorError> {
    let loaded = load(save_dir, source, reserve)?;
    let editor = species_editor(save_dir, source, reserve, species, language)?;
    let mut edits = Vec::new();
    let mut items = Vec::new();
    let mut expected = loaded.groups.clone();
    for change in changes {
        let Some((old, new)) = evaluate(&editor, change)? else {
            continue;
        };
        let position = change.group - 1;
        edits.push((
            position,
            change.index,
            AnimalEdit {
                male: (new.male != old.male).then_some(new.male),
                weight: (new.weight.to_bits() != old.weight.to_bits()).then_some(new.weight),
                score: (new.score.to_bits() != old.score.to_bits()).then_some(new.score),
                great_one: (new.great_one != old.great_one).then_some(new.great_one),
            },
        ));
        let (before, after) = describe(&old, &new);
        items.push(ItemChange {
            label: format!(
                "{} · grupo {} · animal {}",
                editor.name,
                change.group,
                change.index + 1
            ),
            before,
            after,
        });
        if let Some(animal) = expected
            .get_mut(position)
            .and_then(|g| g.animals.get_mut(change.index))
        {
            animal.male = new.male;
            animal.weight = new.weight;
            animal.score = new.score;
            animal.great_one = new.great_one;
        }
    }
    if edits.is_empty() {
        return Err(EditorError::NothingToChange);
    }
    let adf = edit_animals(&loaded.adf, &edits).map_err(|e| match e {
        PopulationError::InvalidValue => EditorError::InvalidValue,
        PopulationError::NoSuchAnimal { .. } => EditorError::WrongAnimal,
        _ => EditorError::Build,
    })?;
    if adf.len() != loaded.adf.len() {
        return Err(EditorError::Build);
    }
    let mut envelope = loaded.envelope.clone();
    envelope.set_adf(adf);
    let new_content = envelope.to_bytes().map_err(|_| EditorError::Build)?;
    let animals = items.len();
    Ok(EditPlan {
        file: FileChange {
            name: population_file(reserve),
            new_content,
            items,
            verify: Some(verifier(expected)),
        },
        animals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(animals: &[(bool, f32, f32)]) -> EditableGroup {
        EditableGroup {
            number: 1,
            animals: animals
                .iter()
                .enumerate()
                .map(|(index, (male, weight, score))| EditableAnimal {
                    index,
                    male: *male,
                    weight: *weight,
                    score: *score,
                    great_one: false,
                })
                .collect(),
        }
    }

    #[test]
    fn limits_follow_the_animals_of_each_sex_with_ten_percent_of_slack() {
        let limits = sex_limits(&[group(&[
            (true, 100.0, 200.0),
            (true, 300.0, 400.0),
            (true, 200.0, 300.0),
            (false, 50.0, 10.0),
            (false, 150.0, 90.0),
        ])])
        .unwrap();
        assert!((limits.male.min_weight - 90.0).abs() < 1e-3);
        assert!((limits.male.max_weight - 330.0).abs() < 1e-3);
        assert!((limits.male.max_score - 440.0).abs() < 1e-3);
        assert!((limits.male.typical_weight - 200.0).abs() < 1e-3);
        assert!((limits.female.max_weight - 165.0).abs() < 1e-3);
        assert!((limits.female.typical_score - 90.0).abs() < 1e-3);
    }

    #[test]
    fn a_species_with_only_one_sex_lends_its_limits_to_the_other() {
        let limits = sex_limits(&[group(&[(true, 100.0, 200.0), (true, 120.0, 220.0)])]).unwrap();
        assert_eq!(limits.male, limits.female);
        assert!(sex_limits(&[]).is_none());
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(EditorError::OutOfRange.code(), "editorOutOfRange");
        assert_eq!(EditorError::WrongAnimal.code(), "editorWrongAnimal");
        assert_eq!(EditorError::InvalidValue.code(), "editorInvalidValue");
        assert_eq!(EditorError::GreatOneNotAllowed.code(), "editorGreatOne");
        assert_eq!(EditorError::NothingToChange.code(), "nothingToChange");
        assert_eq!(EditorError::Build.code(), "editorBuild");
        assert_eq!(EditorError::NoData(1).code(), "editorNoData");
        assert_eq!(EditorError::NoSpecies("x".into()).code(), "editorNoSpecies");
        assert_eq!(EditorError::Read("x".into()).code(), "read");
        assert_eq!(EditorError::Parse("x".into()).code(), "editorParse");
    }
}
