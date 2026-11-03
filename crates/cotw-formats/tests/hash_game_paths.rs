//! Testes de integração: usam só a API pública do crate, como um consumidor externo.

use cotw_formats::hash::hashlittle;

#[test]
fn achievements_mission_path_matches_known_game_hash() {
    // Valor conferido contra `research/reference/hash.py` e o jogo real.
    assert_eq!(
        hashlittle(b"missions/achievements.hpmissionsc", 0),
        0xb0e8_8e20
    );
}
