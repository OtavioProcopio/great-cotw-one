//! Fluxo completo com um save real: editar (escritor ADF) → prévia → confirmar → gravar com
//! backup e conferência → restaurar. Pula se `fixtures/saves` não existir.

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

use std::fs;
use std::io;
use std::path::PathBuf;

use cotw_core::backup::dirs_identical;
use cotw_core::history::{History, HistoryKind};
use cotw_core::process::ProcessList;
use cotw_core::transaction::{Change, FileChange, ItemChange, SaveTransaction};
use cotw_formats::adf::{Adf, ArrayReplacement, PointerKind};
use cotw_formats::envelope::SaveEnvelope;

struct GameClosed;

impl ProcessList for GameClosed {
    fn running_image_names(&self) -> io::Result<Vec<String>> {
        Ok(vec!["explorer.exe".into()])
    }
}

/// Número de animais (elementos de 32 bytes) do primeiro array de animais do arquivo.
fn first_animal_array(adf: &Adf) -> (usize, usize, u64) {
    let instance = &adf.instances()[0];
    let fields = adf.pointer_fields(instance).unwrap();
    let base = instance.offset as usize;
    fields
        .iter()
        .find_map(|f| match f.kind {
            PointerKind::Array {
                count,
                element_size: 32,
            } if count > 0 => Some((f.position, f.target(base), count)),
            _ => None,
        })
        .unwrap()
}

#[test]
fn editing_a_real_save_through_the_transaction_and_restoring_it() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    if !fixtures.is_dir() {
        eprintln!("fixtures/saves ausente: teste ignorado");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let save = work.path().join("save");
    fs::create_dir_all(&save).unwrap();
    for entry in fs::read_dir(&fixtures).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), save.join(entry.file_name())).unwrap();
        }
    }
    let original_files = work.path().join("original");
    fs::create_dir_all(&original_files).unwrap();
    for entry in fs::read_dir(&save).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), original_files.join(entry.file_name())).unwrap();
    }

    // 1. Edita: acrescenta um animal (cópia do último) ao primeiro grupo.
    let name = "animal_population_0";
    let raw = fs::read(save.join(name)).unwrap();
    let mut envelope = SaveEnvelope::parse(&raw).unwrap();
    let adf = Adf::parse(envelope.adf().to_vec()).unwrap();
    let (header, start, count) = first_animal_array(&adf);
    let old_len = usize::try_from(count).unwrap() * 32;
    let mut elements = adf.as_bytes()[start..start + old_len].to_vec();
    let last = elements[old_len - 32..].to_vec();
    elements.extend_from_slice(&last); // +32 bytes: múltiplo de 8
    let edited = adf
        .rewrite_arrays(
            0,
            &[ArrayReplacement {
                position: header,
                elements,
            }],
        )
        .unwrap();
    envelope.set_adf(edited);
    let new_bytes = envelope.to_bytes().unwrap();

    // 2. Prévia, confirmação e gravação com conferência por releitura do arquivo.
    let backups = work.path().join("backups");
    let history = History::open_in_memory().unwrap();
    let transaction = SaveTransaction::new(&save, &backups);
    let file = FileChange {
        name: name.into(),
        new_content: new_bytes.clone(),
        items: vec![ItemChange {
            label: "Animais do primeiro grupo".into(),
            before: count.to_string(),
            after: (count + 1).to_string(),
        }],
        verify: Some(Box::new(|bytes| {
            let envelope = SaveEnvelope::parse(bytes).map_err(|e| e.to_string())?;
            let adf = Adf::parse(envelope.adf().to_vec()).map_err(|e| e.to_string())?;
            for instance in adf.instances() {
                let issues = adf.chain_issues(instance).map_err(|e| e.to_string())?;
                if !issues.is_empty() {
                    return Err(format!("{} ponteiros fora da cadeia", issues.len()));
                }
            }
            Ok(())
        })),
    };
    let preview = transaction
        .preview(Change {
            function: "Teste ponta a ponta".into(),
            summary: "+1 animal".into(),
            files: vec![file],
        })
        .unwrap();
    assert_eq!(preview.files[0].new_size, new_bytes.len() as u64);
    assert_eq!(fs::read(save.join(name)).unwrap(), raw, "prévia não grava");

    let done = transaction
        .commit(preview.confirm(), &GameClosed, &history)
        .unwrap();

    // 3. O arquivo tem um animal a mais; o backup guarda o estado anterior completo.
    let after = fs::read(save.join(name)).unwrap();
    assert_eq!(after, new_bytes);
    let reread = Adf::parse(SaveEnvelope::parse(&after).unwrap().adf().to_vec()).unwrap();
    assert_eq!(first_animal_array(&reread).2, count + 1);
    assert!(dirs_identical(&original_files, &done.backup.path).unwrap());

    // 4. Restaura: a pasta volta exatamente ao original e o histórico registra.
    transaction
        .restore(&done.backup.path, &GameClosed, &history)
        .unwrap();
    assert!(dirs_identical(&original_files, &save).unwrap());
    let kinds: Vec<_> = history
        .list()
        .unwrap()
        .iter()
        .map(|e| e.data.kind)
        .collect();
    assert_eq!(kinds, [HistoryKind::Restore, HistoryKind::Change]);
}
