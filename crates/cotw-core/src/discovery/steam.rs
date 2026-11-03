//! Localização da instalação do jogo pelas bibliotecas da Steam.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// App id do theHunter: Call of the Wild na Steam.
pub const APP_ID: u32 = 518_790;
const DEFAULT_INSTALL_DIR: &str = "theHunterCotW";
const ARCHIVES_DIR: &str = "archives_win64";

/// Instalação encontrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInstall {
    /// Pasta do jogo (`.../common/theHunterCotW`).
    pub root: PathBuf,
    /// Pasta com os `gameN.tab` / `gameN.arc`.
    pub archives: PathBuf,
}

/// Extrai os valores das chaves `"path"` de um `libraryfolders.vdf`.
#[must_use]
pub fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let tokens = quoted_tokens(line);
            (tokens.first().map(String::as_str) == Some("path")).then(|| tokens.get(1).cloned())?
        })
        .map(PathBuf::from)
        .collect()
}

/// Número da versão (build) instalada, lido do `appmanifest` da biblioteca que contém `game_root`
/// (`<biblioteca>/steamapps/common/<jogo>`).
#[must_use]
pub fn build_id_for(game_root: &Path) -> Option<String> {
    let steamapps = game_root.parent()?.parent()?;
    let acf = fs::read_to_string(steamapps.join(format!("appmanifest_{APP_ID}.acf"))).ok()?;
    acf.lines().find_map(|line| {
        let tokens = quoted_tokens(line);
        (tokens.first().map(String::as_str) == Some("buildid")).then(|| tokens.get(1).cloned())?
    })
}

/// Valor de `"installdir"` num `appmanifest_*.acf`.
#[must_use]
pub fn parse_install_dir(acf: &str) -> Option<String> {
    acf.lines().find_map(|line| {
        let tokens = quoted_tokens(line);
        (tokens.first().map(String::as_str) == Some("installdir"))
            .then(|| tokens.get(1).cloned())?
    })
}

/// Textos entre aspas de uma linha VDF, já com `\\` e `\"` desfeitos.
fn quoted_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut token = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => match chars.next() {
                    Some('n') => token.push('\n'),
                    Some('t') => token.push('\t'),
                    Some(other) => token.push(other),
                    None => break,
                },
                other => token.push(other),
            }
        }
        tokens.push(token);
    }
    tokens
}

/// Raízes prováveis da Steam: registro, pastas padrão e variáveis de ambiente.
#[must_use]
pub fn steam_roots_from_system() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = registry_steam_path().into_iter().collect();
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(base) = std::env::var_os(var) {
            roots.push(PathBuf::from(base).join("Steam"));
        }
    }
    roots.dedup();
    roots
}

/// `SteamPath` do registro do usuário, consultado com `reg.exe`.
fn registry_steam_path() -> Option<PathBuf> {
    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .ok()?;
    parse_reg_value(&String::from_utf8_lossy(&output.stdout))
}

/// Valor da linha `SteamPath    REG_SZ    <caminho>` da saída do `reg query`.
fn parse_reg_value(output: &str) -> Option<PathBuf> {
    output.lines().find_map(|line| {
        let (_, value) = line
            .trim()
            .strip_prefix("SteamPath")?
            .trim_start()
            .split_once("REG_SZ")?;
        let value = value.trim();
        // O registro guarda `c:/program files (x86)/steam`; no Windows usamos barras invertidas.
        (!value.is_empty())
            .then(|| PathBuf::from(value.replace('/', std::path::MAIN_SEPARATOR_STR)))
    })
}

/// Bibliotecas Steam configuradas: a própria raiz e as listadas em `libraryfolders.vdf`.
#[must_use]
pub fn libraries(steam_root: &Path) -> Vec<PathBuf> {
    let mut libs = vec![steam_root.to_path_buf()];
    let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(text) = fs::read_to_string(vdf) {
        for path in parse_library_paths(&text) {
            if !libs.contains(&path) {
                libs.push(path);
            }
        }
    }
    libs
}

/// Procura o jogo nas raízes da Steam dadas.
#[must_use]
pub fn find_install(steam_roots: &[PathBuf]) -> Option<GameInstall> {
    let mut seen: Vec<PathBuf> = Vec::new();
    for library in steam_roots.iter().flat_map(|root| libraries(root)) {
        if seen.contains(&library) {
            continue;
        }
        let steamapps = library.join("steamapps");
        let manifest = steamapps.join(format!("appmanifest_{APP_ID}.acf"));
        let install_dir = fs::read_to_string(manifest)
            .ok()
            .and_then(|acf| parse_install_dir(&acf))
            .unwrap_or_else(|| DEFAULT_INSTALL_DIR.to_owned());
        if let Some(install) = validate_game_dir(&steamapps.join("common").join(install_dir)) {
            return Some(install);
        }
        seen.push(library);
    }
    None
}

/// Confere uma pasta escolhida manualmente: pode ser a pasta do jogo ou a `archives_win64`.
#[must_use]
pub fn validate_game_dir(chosen: &Path) -> Option<GameInstall> {
    let candidates = [
        (chosen.to_path_buf(), chosen.join(ARCHIVES_DIR)),
        (
            chosen.parent().map(Path::to_path_buf)?,
            chosen.to_path_buf(),
        ),
    ];
    candidates
        .into_iter()
        .find(|(_, archives)| has_tab_files(archives))
        .map(|(root, archives)| GameInstall { root, archives })
}

fn has_tab_files(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_prefix("game")
                .and_then(|rest| rest.strip_suffix(".tab"))
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
    })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    const VDF: &str = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t\t\"label\"\t\t\"\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}\n";

    #[test]
    fn parses_library_paths_and_unescapes_backslashes() {
        assert_eq!(
            parse_library_paths(VDF),
            [
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"D:\SteamLibrary")
            ]
        );
        assert_eq!(parse_library_paths("sem nada"), Vec::<PathBuf>::new());
    }

    #[test]
    fn parses_install_dir_from_manifest() {
        let acf = "\"AppState\"\n{\n\t\"appid\"\t\t\"518790\"\n\t\"installdir\"\t\t\"theHunterCotW\"\n}\n";
        assert_eq!(parse_install_dir(acf).as_deref(), Some("theHunterCotW"));
        assert_eq!(parse_install_dir("\"AppState\" {}"), None);
    }

    #[test]
    fn unterminated_quote_does_not_panic() {
        assert_eq!(quoted_tokens("\"path\" \"abc"), ["path", "abc"]);
        assert_eq!(quoted_tokens("\"a\\"), ["a"]);
    }

    #[test]
    fn parses_reg_query_output() {
        let out = "\nHKEY_CURRENT_USER\\Software\\Valve\\Steam\n    SteamPath    REG_SZ    c:/program files (x86)/steam\n\n";
        let expected = "c:/program files (x86)/steam".replace('/', std::path::MAIN_SEPARATOR_STR);
        assert_eq!(parse_reg_value(out), Some(PathBuf::from(expected)));
        assert_eq!(parse_reg_value("ERRO: nao encontrado"), None);
        assert_eq!(parse_reg_value("    SteamPath    REG_SZ    "), None);
    }

    /// Biblioteca com o jogo instalado em `common/<install_dir>/archives_win64`.
    fn library_with_game(install_dir: &str) -> tempfile::TempDir {
        let lib = tempfile::tempdir().unwrap();
        let archives = lib
            .path()
            .join("steamapps/common")
            .join(install_dir)
            .join(ARCHIVES_DIR);
        fs::create_dir_all(&archives).unwrap();
        fs::write(archives.join("game0.tab"), b"TAB").unwrap();
        lib
    }

    /// Raiz Steam cujo `libraryfolders.vdf` aponta para `others`.
    fn steam_root(others: &[&Path]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let steamapps = root.path().join("steamapps");
        fs::create_dir_all(&steamapps).unwrap();
        let mut body = String::new();
        for (i, p) in others.iter().enumerate() {
            let escaped = p.to_string_lossy().replace('\\', "\\\\");
            writeln!(body, "\t\"{i}\"\n\t{{\n\t\t\"path\"\t\t\"{escaped}\"\n\t}}").unwrap();
        }
        fs::write(
            steamapps.join("libraryfolders.vdf"),
            format!("\"libraryfolders\"\n{{\n{body}}}\n"),
        )
        .unwrap();
        root
    }

    #[test]
    fn finds_game_in_another_steam_library() {
        let other = library_with_game(DEFAULT_INSTALL_DIR);
        let root = steam_root(&[other.path()]);
        let install = find_install(&[root.path().to_path_buf()]).unwrap();
        assert_eq!(
            install.root,
            other.path().join("steamapps/common/theHunterCotW")
        );
        assert_eq!(install.archives, install.root.join(ARCHIVES_DIR));
    }

    #[test]
    fn manifest_install_dir_overrides_the_default_name() {
        let other = library_with_game("OutroNome");
        fs::write(
            other
                .path()
                .join(format!("steamapps/appmanifest_{APP_ID}.acf")),
            "\"AppState\"\n{\n\t\"installdir\"\t\t\"OutroNome\"\n}\n",
        )
        .unwrap();
        let root = steam_root(&[other.path()]);
        assert!(find_install(&[root.path().to_path_buf()]).is_some());
    }

    #[test]
    fn game_in_the_steam_root_itself_is_found() {
        let root = library_with_game(DEFAULT_INSTALL_DIR);
        assert!(find_install(&[root.path().to_path_buf()]).is_some());
    }

    #[test]
    fn missing_game_is_none() {
        let root = steam_root(&[]);
        assert_eq!(find_install(&[root.path().to_path_buf()]), None);
        assert_eq!(find_install(&[]), None);
    }

    #[test]
    fn manual_choice_accepts_game_folder_or_archives_folder() {
        let lib = library_with_game(DEFAULT_INSTALL_DIR);
        let game = lib.path().join("steamapps/common/theHunterCotW");
        let from_root = validate_game_dir(&game).unwrap();
        let from_archives = validate_game_dir(&game.join(ARCHIVES_DIR)).unwrap();
        assert_eq!(from_root, from_archives);
    }

    #[test]
    fn manual_choice_rejects_folders_without_tab_files() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(validate_game_dir(dir.path()), None);
        fs::write(dir.path().join("gamex.tab"), b"x").unwrap();
        fs::write(dir.path().join("game.tab"), b"x").unwrap();
        assert_eq!(validate_game_dir(dir.path()), None);
    }
}
