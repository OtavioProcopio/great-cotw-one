//! Compara índice `.tab`, AAF/SARC, RTPC e XLS com `research/reference` nos arquivos reais do jogo.
//!
//! Precisa da instalação do jogo (pasta `archives_win64`, de `COTW_GAME_DIR` ou do caminho
//! padrão da Steam) e do oráculo `fixtures/game.oracle.txt`, gerado por
//! `scripts/make_game_oracle.py`. Sem eles o teste é ignorado. Nada do jogo é versionado.

#![allow(
    clippy::unwrap_used,
    reason = "arquivo só de testes; escrever em String não falha"
)]

use std::fmt::Write as _;
use std::path::PathBuf;

use cotw_formats::aaf;
use cotw_formats::adf::Adf;
use cotw_formats::archive::GameArchives;
use cotw_formats::hash::hashlittle;
use cotw_formats::rtpc::{self, Node, PropValue, Property};
use cotw_formats::sarc::Sarc;
use cotw_formats::xls::{CellValue, XlsBook};
use sha2::{Digest, Sha256};

const PATHS: [&str; 5] = [
    "missions/achievements.hpmissionsc",
    "settings/hp_settings/player_skills.bin",
    "settings/hp_settings/player_skill_trees.bin",
    "settings/player.bin",
    "nao/existe.bin",
];
const AAF_HASH: u32 = 0x7820_9FCC;
const DEFAULT_GAME_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64";

fn game_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("COTW_GAME_DIR")
        .map_or_else(|| PathBuf::from(DEFAULT_GAME_DIR), PathBuf::from);
    dir.is_dir().then_some(dir)
}

fn oracle() -> Option<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/game.oracle.txt");
    std::fs::read_to_string(path).ok()
}

fn wire_type(value: &PropValue) -> u8 {
    match value {
        PropValue::U32(_) => 1,
        PropValue::F32(_) => 2,
        PropValue::String(_) => 3,
        PropValue::Floats(v) => match v.len() {
            2 => 4,
            3 => 5,
            4 => 6,
            9 => 7,
            _ => 8,
        },
        PropValue::U32Array(_) => 9,
        PropValue::F32Array(_) => 10,
        PropValue::Bytes(_) => 11,
        PropValue::U64(_) => 13,
        PropValue::ObjectIds(_) => 14,
        PropValue::Raw { type_id, .. } => *type_id,
    }
}

fn prop_text(value: &PropValue) -> String {
    let floats = |v: &[f32]| {
        v.iter()
            .map(|x| format!("{x:.4}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let hex = |b: &[u8]| {
        b.iter().fold(String::new(), |mut s, x| {
            write!(s, "{x:02x}").unwrap();
            s
        })
    };
    match value {
        PropValue::U32(n) => n.to_string(),
        PropValue::U64(n) => n.to_string(),
        PropValue::F32(x) => format!("{x:.4}"),
        PropValue::String(s) => s.clone(),
        PropValue::Floats(v) | PropValue::F32Array(v) => floats(v),
        PropValue::U32Array(v) => v.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        PropValue::ObjectIds(v) => v
            .iter()
            .map(|(a, b)| format!("{a}:{b}"))
            .collect::<Vec<_>>()
            .join(";"),
        PropValue::Bytes(b) => hex(b),
        PropValue::Raw { bytes, .. } => hex(bytes),
    }
}

/// Propriedades com a semântica do `dict` do `rtpc.py`: hash repetido mantém a posição da
/// primeira ocorrência e o valor da última. O leitor em Rust guarda todas as repetidas.
fn python_like_props(node: &Node) -> Vec<&Property> {
    let mut props: Vec<&Property> = Vec::new();
    for prop in &node.props {
        match props.iter().position(|p| p.hash == prop.hash) {
            Some(i) => props[i] = prop,
            None => props.push(prop),
        }
    }
    props
}

fn render_node(out: &mut String, node: &Node, depth: usize) {
    let props = python_like_props(node);
    writeln!(
        out,
        "node\t{depth}\t{:08x}\t{}\t{}",
        node.hash,
        props.len(),
        node.children.len()
    )
    .unwrap();
    for prop in props {
        writeln!(
            out,
            "prop\t{depth}\t{:08x}\t{}\t{}",
            prop.hash,
            wire_type(&prop.value),
            prop_text(&prop.value)
        )
        .unwrap();
    }
    for child in &node.children {
        render_node(out, child, depth + 1);
    }
}

fn render_xls(out: &mut String, archives: &GameArchives, path: &str) {
    let location = archives.find_path(path)[0];
    let adf = Adf::parse(archives.read(&location).unwrap()).unwrap();
    let book = XlsBook::from_adf(&adf).unwrap();
    writeln!(out, "xls\t{path}").unwrap();
    for sheet in book.sheets() {
        writeln!(out, "sheet\t{}\t{}\t{}", sheet.name, sheet.rows, sheet.cols).unwrap();
        for row in 0..sheet.rows {
            for col in 0..sheet.cols {
                let text = match book.cell(sheet, row, col).unwrap() {
                    CellValue::Text(s) => format!("T\t{s}"),
                    CellValue::Number(n) => format!("N\t{n:.4}"),
                    CellValue::Unsupported(kind) => format!("U\t{kind}"),
                };
                writeln!(out, "cell\t{row}\t{col}\t{text}").unwrap();
            }
        }
    }
}

fn render(archives: &GameArchives) -> String {
    let mut out = String::new();
    writeln!(out, "tab_entries\t{}", archives.entry_count()).unwrap();
    for path in PATHS {
        let hash = hashlittle(path.as_bytes(), 0);
        let locations = archives
            .find_path(path)
            .iter()
            .map(|l| format!("{}:{}:{}", l.archive, l.offset, l.size))
            .collect::<Vec<_>>()
            .join(";");
        writeln!(out, "path\t{path}\t{hash:08x}\t{locations}").unwrap();
    }
    render_xls(&mut out, archives, PATHS[1]);
    render_xls(&mut out, archives, PATHS[2]);

    let player = archives.find_path(PATHS[3])[0];
    writeln!(out, "rtpc\t{}", PATHS[3]).unwrap();
    let root = rtpc::parse(&archives.read(&player).unwrap()).unwrap();
    render_node(&mut out, &root, 0);

    let bundle = archives.read(&archives.find(AAF_HASH)[0]).unwrap();
    let decoded = aaf::decode(&bundle).unwrap();
    writeln!(
        out,
        "aaf\t{AAF_HASH:08x}\t{}\t{}",
        decoded.len(),
        Sha256::digest(&decoded)
            .iter()
            .fold(String::new(), |mut s, b| {
                write!(s, "{b:02x}").unwrap();
                s
            })
    )
    .unwrap();
    let sarc = Sarc::parse(decoded).unwrap();
    for entry in sarc.entries() {
        writeln!(
            out,
            "sarc\t{}\t{}\t{}",
            entry.name, entry.offset, entry.size
        )
        .unwrap();
    }
    out
}

#[test]
fn game_files_match_python_reference() {
    let (Some(dir), Some(expected)) = (game_dir(), oracle()) else {
        eprintln!("instalação do jogo ou fixtures/game.oracle.txt ausentes: teste ignorado");
        return;
    };
    let archives = GameArchives::open(&dir).unwrap();
    // Compara linha a linha para que a falha aponte onde divergiu.
    let actual = render(&archives);
    for (i, (a, e)) in actual.lines().zip(expected.lines()).enumerate() {
        assert_eq!(a, e, "linha {} diverge", i + 1);
    }
    assert_eq!(actual.lines().count(), expected.lines().count());
}

#[test]
fn sarc_internal_entries_point_inside_the_decoded_data() {
    let Some(dir) = game_dir() else {
        eprintln!("instalação do jogo ausente: teste ignorado");
        return;
    };
    let archives = GameArchives::open(&dir).unwrap();
    let bundle = archives.read(&archives.find(AAF_HASH)[0]).unwrap();
    let sarc = Sarc::parse(aaf::decode(&bundle).unwrap()).unwrap();
    assert_ne!(sarc.entries(), []);
    for entry in sarc.entries().iter().filter(|e| e.offset != 0) {
        assert!(sarc.data_of(entry).is_some(), "{} fora do SARC", entry.name);
    }
}
