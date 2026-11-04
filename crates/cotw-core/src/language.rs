//! Idiomas do app: o que cada um muda no que o núcleo entrega (nomes de espécies, textos de
//! missões do jogo e conquistas da Steam).

use std::str::FromStr;

/// Idioma escolhido pelo jogador.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    /// Português do Brasil.
    #[default]
    PtBr,
    /// Inglês.
    En,
    /// Espanhol.
    Es,
}

impl Language {
    /// Posição do idioma nos textos de missões do jogo (0 inglês, 3 espanhol, 7 português do Brasil).
    #[must_use]
    pub fn mission_index(self) -> usize {
        match self {
            Self::PtBr => 7,
            Self::En => 0,
            Self::Es => 3,
        }
    }

    /// Nome do idioma no esquema de conquistas da Steam.
    #[must_use]
    pub fn steam_key(self) -> &'static str {
        match self {
            Self::PtBr => "brazilian",
            Self::En => "english",
            Self::Es => "spanish",
        }
    }
}

/// O texto não é um idioma que o app conhece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("idioma desconhecido")]
pub struct UnknownLanguage;

impl FromStr for Language {
    type Err = UnknownLanguage;

    /// Aceita a etiqueta do navegador (`pt-BR`, `pt`, `en-US`, `es-MX`…), sem diferenciar maiúsculas.
    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        let tag = tag.trim().to_ascii_lowercase();
        match tag.split(['-', '_']).next() {
            Some("pt") => Ok(Self::PtBr),
            Some("en") => Ok(Self::En),
            Some("es") => Ok(Self::Es),
            _ => Err(UnknownLanguage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_map_to_languages() {
        assert_eq!("pt-BR".parse(), Ok(Language::PtBr));
        assert_eq!("pt".parse(), Ok(Language::PtBr));
        assert_eq!("en-US".parse(), Ok(Language::En));
        assert_eq!("EN".parse(), Ok(Language::En));
        assert_eq!("es_MX".parse(), Ok(Language::Es));
        assert_eq!(" es ".parse(), Ok(Language::Es));
        assert_eq!("fr".parse::<Language>(), Err(UnknownLanguage));
        assert_eq!("".parse::<Language>(), Err(UnknownLanguage));
    }

    #[test]
    fn each_language_points_at_the_game_texts_and_the_steam_names() {
        assert_eq!(Language::default(), Language::PtBr);
        assert_eq!(Language::PtBr.mission_index(), 7);
        assert_eq!(Language::En.mission_index(), 0);
        assert_eq!(Language::Es.mission_index(), 3);
        assert_eq!(Language::PtBr.steam_key(), "brazilian");
        assert_eq!(Language::En.steam_key(), "english");
        assert_eq!(Language::Es.steam_key(), "spanish");
    }
}
