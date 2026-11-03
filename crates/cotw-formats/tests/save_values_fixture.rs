//! Compara, valor a valor, a leitura dos saves com `research/reference/adf.py`.
//!
//! Usa os saves em `fixtures/saves/` e o índice `fixtures/saves/_index.oracle.txt`, gerado por
//! `scripts/make_save_oracle.py`. Sem eles o teste é ignorado. Os saves nunca são versionados.
//! Em caso de divergência, gere o texto completo com `--dump` e compare com `diff`.

#![allow(
    clippy::unwrap_used,
    reason = "arquivo só de testes; escrever em String não falha"
)]

use std::fmt::Write as _;
use std::path::PathBuf;

use cotw_formats::adf::{Adf, AdfError, TypeKind, Value};
use cotw_formats::envelope::SaveEnvelope;
use sha2::{Digest, Sha256};

fn saves_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn number(value: f64) -> String {
    if value.is_nan() {
        "nan".to_owned()
    } else {
        format!("{value:.4}")
    }
}

fn meta_number(kind: TypeKind) -> u32 {
    match kind {
        TypeKind::Primitive => 0,
        TypeKind::Struct => 1,
        TypeKind::Pointer => 2,
        TypeKind::Array => 3,
        TypeKind::InlineArray => 4,
        TypeKind::String => 5,
        TypeKind::BitField => 7,
        TypeKind::Enum => 8,
        TypeKind::StringHash => 9,
        TypeKind::Unknown(n) => n,
    }
}

/// Acumula o hash e a contagem das linhas achatadas, no mesmo formato do script Python.
struct Sink {
    hasher: Sha256,
    lines: usize,
}

impl Sink {
    fn push(&mut self, line: &str) {
        self.hasher.update(line.as_bytes());
        self.hasher.update(b"\n");
        self.lines += 1;
    }
}

fn flatten(adf: &Adf, sink: &mut Sink, value: &Value, path: &str) {
    match value {
        Value::Struct(fields) => {
            for (name, field) in fields {
                flatten(adf, sink, field, &format!("{path}.{name}"));
            }
        }
        Value::Array(items) => {
            sink.push(&format!("{path}\tarray\t{}", items.len()));
            for (i, item) in items.iter().enumerate() {
                flatten(adf, sink, item, &format!("{path}[{i}]"));
            }
        }
        Value::Bytes(bytes) => sink.push(&format!("{path}\tbytes\t{}", hex(bytes))),
        Value::Signed(n) => sink.push(&format!("{path}\tint\t{n}")),
        Value::Unsigned(n) => sink.push(&format!("{path}\tint\t{n}")),
        Value::Float(x) => sink.push(&format!("{path}\tfloat\t{}", number(*x))),
        Value::String(s) => sink.push(&format!("{path}\tstr\t{}", hex(s.as_bytes()))),
        Value::Enum {
            name: Some(name), ..
        } => sink.push(&format!("{path}\tstr\t{}", hex(name.as_bytes()))),
        Value::Enum { value, name: None } => sink.push(&format!("{path}\tint\t{value}")),
        Value::StringHash {
            text: Some(text), ..
        } => sink.push(&format!("{path}\tstr\t{}", hex(text.as_bytes()))),
        Value::StringHash { hash, text: None } => {
            sink.push(&format!(
                "{path}\tstr\t{}",
                hex(format!("{hash:#x}").as_bytes())
            ));
        }
        Value::Raw { type_hash, bytes } => {
            let line = match adf.type_def(*type_hash) {
                None => format!("{path}\traw\t{type_hash:#x}:{}", hex(bytes)),
                Some(def) => match def.kind {
                    TypeKind::Primitive => format!("{path}\tprim\t{}:{}", def.name, hex(bytes)),
                    TypeKind::Pointer | TypeKind::String => {
                        let word: [u8; 8] = bytes.get(..8).unwrap().try_into().unwrap();
                        format!("{path}\tptr\t{}", u64::from_le_bytes(word))
                    }
                    other => format!("{path}\tmeta\t{}:{}", meta_number(other), hex(bytes)),
                },
            };
            sink.push(&line);
        }
    }
}

/// Achata todas as instâncias; devolve a contagem de linhas e o hash do texto.
fn flat_digest(adf: &Adf) -> (usize, String) {
    let mut sink = Sink {
        hasher: Sha256::new(),
        lines: 0,
    };
    for (n, instance) in adf.instances().iter().enumerate() {
        let head = format!("[{n}:{}]", instance.name);
        match adf.read_instance(instance) {
            Ok(value) => flatten(adf, &mut sink, &value, &head),
            Err(_) => sink.push(&format!("{head}\tERROR")),
        }
    }
    (sink.lines, hex(&sink.hasher.finalize()))
}

/// Confere um arquivo; devolve a descrição do problema, se houver.
fn check(name: &str, fields: &[&str]) -> Result<(), String> {
    let raw = std::fs::read(saves_dir().join(name)).map_err(|e| e.to_string())?;
    let class = fields[0];
    if class == "skip" {
        return match SaveEnvelope::parse(&raw) {
            Err(_) => Ok(()),
            Ok(_) => Err("deveria ser rejeitado como não reconhecido".into()),
        };
    }

    let envelope = SaveEnvelope::parse(&raw).map_err(|e| format!("envelope: {e}"))?;
    let mut payload = envelope.prefix().to_vec();
    payload.extend_from_slice(envelope.adf());
    if payload.len().to_string() != fields[1] || sha256_hex(&payload) != fields[2] {
        return Err("conteúdo descomprimido difere do Python".into());
    }
    let again = SaveEnvelope::parse(&envelope.to_bytes().map_err(|e| e.to_string())?)
        .map_err(|e| format!("releitura: {e}"))?;
    if again != envelope {
        return Err("ida e volta do envelope mudou o conteúdo".into());
    }

    if class == "noadf" {
        return match Adf::parse(envelope.adf().to_vec()) {
            Err(AdfError::BadMagic) => Ok(()),
            other => Err(format!("esperava BadMagic, veio {:?}", other.map(|_| ()))),
        };
    }

    let adf = Adf::parse(envelope.adf().to_vec()).map_err(|e| format!("ADF: {e}"))?;
    let (lines, digest) = flat_digest(&adf);
    if lines.to_string() != fields[3] {
        return Err(format!("{lines} linhas, o Python teve {}", fields[3]));
    }
    if digest != fields[4] {
        return Err("valores diferem do Python (gere --dump e compare com diff)".into());
    }
    Ok(())
}

#[test]
fn every_save_matches_python_reference() {
    let Ok(index) = std::fs::read_to_string(saves_dir().join("_index.oracle.txt")) else {
        eprintln!("fixtures/saves/_index.oracle.txt ausente: teste ignorado");
        return;
    };
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in index.lines() {
        let mut parts = line.split('\t');
        let name = parts.next().unwrap();
        let fields: Vec<&str> = parts.collect();
        checked += 1;
        if let Err(problem) = check(name, &fields) {
            failures.push(format!("{name}: {problem}"));
        }
    }
    assert!(checked > 0, "índice vazio");
    assert!(
        failures.is_empty(),
        "{} de {checked} arquivos divergem:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
