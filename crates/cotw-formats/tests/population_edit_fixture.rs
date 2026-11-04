//! Edição de animais em `animal_population_N` reais (pula se `fixtures/saves` não existir).

#![allow(clippy::unwrap_used, reason = "arquivo só de testes")]

mod common;

use common::saves;
use cotw_formats::adf::Adf;
use cotw_formats::population::{
    AnimalEdit, PopulationError, edit_animals, locate_animals, parse_groups,
};

fn played() -> Vec<(String, Vec<u8>)> {
    saves()
        .into_iter()
        .filter(|s| s.name.starts_with("animal_population_"))
        .map(|s| (s.name.clone(), s.envelope.adf().to_vec()))
        .filter(|(_, adf)| parse_groups(adf).is_ok_and(|g| g.iter().any(|g| !g.animals.is_empty())))
        .collect()
}

#[test]
fn every_animal_of_every_played_reserve_is_located_in_reading_order() {
    for (name, adf) in played() {
        let groups = parse_groups(&adf).unwrap();
        let slots = locate_animals(&Adf::parse(adf.clone()).unwrap()).unwrap();
        let total: usize = groups.iter().map(|g| g.animals.len()).sum();
        assert_eq!(slots.len(), total, "{name}");
        let mut expected = groups
            .iter()
            .enumerate()
            .flat_map(|(g, group)| (0..group.animals.len()).map(move |a| (g, a)));
        assert!(
            slots
                .iter()
                .all(|s| expected.next() == Some((s.group, s.index))),
            "{name}"
        );
    }
}

#[test]
fn an_empty_edit_changes_nothing() {
    for (name, adf) in played() {
        assert_eq!(edit_animals(&adf, &[]).unwrap(), adf, "{name}");
    }
}

#[test]
fn editing_one_animal_changes_only_its_values_and_keeps_the_file_sound() {
    for (name, adf) in played() {
        let before = parse_groups(&adf).unwrap();
        let (g, a) = before
            .iter()
            .enumerate()
            .find_map(|(g, group)| (!group.animals.is_empty()).then_some((g, 0)))
            .unwrap();
        let old = before[g].animals[a];
        let supports = locate_animals(&Adf::parse(adf.clone()).unwrap())
            .unwrap()
            .iter()
            .find(|s| (s.group, s.index) == (g, a))
            .unwrap()
            .supports_great_one();
        let edit = AnimalEdit {
            male: Some(!old.male),
            weight: Some(old.weight + 10.0),
            score: Some(old.score + 5.0),
            great_one: supports.then_some(!old.great_one),
        };
        let edited = edit_animals(&adf, &[(g, a, edit)]).unwrap();
        assert_eq!(edited.len(), adf.len(), "{name}: mesmo tamanho");

        let after = parse_groups(&edited).unwrap();
        for (gi, (b, f)) in before.iter().zip(&after).enumerate() {
            assert_eq!(b.route, f.route, "{name}: rotas intactas");
            for (ai, (x, y)) in b.animals.iter().zip(&f.animals).enumerate() {
                if (gi, ai) == (g, a) {
                    assert_eq!(y.male, !old.male);
                    assert!((y.weight - (old.weight + 10.0)).abs() < 1e-3);
                    assert!((y.score - (old.score + 5.0)).abs() < 1e-3);
                    assert_eq!(
                        y.great_one,
                        if supports {
                            !old.great_one
                        } else {
                            old.great_one
                        }
                    );
                } else {
                    assert_eq!(x, y, "{name}: animal {gi}/{ai} não podia mudar");
                }
            }
        }
        // Só os bytes desse animal diferem, e a cadeia de ponteiros continua íntegra.
        let changed = adf.iter().zip(&edited).filter(|(x, y)| x != y).count();
        assert!(
            changed > 0 && changed <= 10,
            "{name}: {changed} bytes mudaram"
        );
        let parsed = Adf::parse(edited.clone()).unwrap();
        for instance in parsed.instances() {
            assert!(parsed.chain_issues(instance).unwrap().is_empty(), "{name}");
        }
        // Voltar os valores antigos devolve o arquivo original, byte a byte.
        let back = AnimalEdit {
            male: Some(old.male),
            weight: Some(old.weight),
            score: Some(old.score),
            great_one: supports.then_some(old.great_one),
        };
        assert_eq!(
            edit_animals(&edited, &[(g, a, back)]).unwrap(),
            adf,
            "{name}"
        );
    }
}

#[test]
fn edits_to_animals_that_do_not_exist_or_with_bad_numbers_are_refused() {
    let Some((_, adf)) = played().into_iter().next() else {
        return; // sem saves reais (ex.: na CI): nada a conferir
    };
    let missing = edit_animals(&adf, &[(usize::MAX, 0, AnimalEdit::default())]);
    assert!(matches!(missing, Err(PopulationError::NoSuchAnimal { .. })));
    for bad in [f32::NAN, f32::INFINITY, -1.0] {
        let edit = AnimalEdit {
            weight: Some(bad),
            ..AnimalEdit::default()
        };
        assert!(matches!(
            edit_animals(&adf, &[(0, 0, edit)]),
            Err(PopulationError::InvalidValue)
        ));
    }
}
