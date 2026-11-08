//! Gera a tabela de referência das páginas de conteúdo do site (reservas, espécies, classes,
//! medalhas e chances de pelagem), lendo o jogo instalado neste PC.
//!
//! Uso: `cargo run -p cotw-core --example site_data > ../great-cotw-one-site/data/game.json`
//!
//! Só sai texto e números (nomes, faixas, chances); nenhum arquivo ou imagem do jogo. Rode de novo a
//! cada DLC ou patch que mude reservas ou espécies.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::redundant_closure_for_method_calls,
    reason = "ferramenta de linha de comando: qualquer falha deve parar e aparecer na tela"
)]

use cotw_core::diagnostics::SearchRoots;
use cotw_core::language::Language;
use cotw_core::process::SystemProcessList;
use cotw_core::service::AppService;
use cotw_core::{species, species_stats};
use serde_json::json;

fn main() {
    let appdata = std::env::temp_dir().join("gco-dump");
    std::fs::create_dir_all(&appdata).unwrap();
    let service = AppService::new(
        &appdata,
        SearchRoots::from_system(),
        Box::new(SystemProcessList),
    );
    let reserves = service.animals().expect("animals");
    let mut out = Vec::new();
    for r in reserves {
        let mut list = Vec::new();
        for s in &r.species {
            let stats = species_stats::lookup(&s.id);
            let furs: Vec<_> = stats
                .map(|st| {
                    st.fur_chances()
                        .into_iter()
                        .map(|(f, c)| json!({"key": f.key, "chance": c, "rarity": f.rarity}))
                        .collect()
                })
                .unwrap_or_default();
            let go_furs: Vec<_> = stats
                .map(|st| {
                    st.furs
                        .iter()
                        .filter(|f| f.great_one)
                        .map(|f| f.key.clone())
                        .collect()
                })
                .unwrap_or_default();
            let bands = stats.and_then(|st| st.medal_bands());
            list.push(json!({
                "id": s.id,
                "pt": species::display_name(&s.id, Language::PtBr),
                "en": species::display_name(&s.id, Language::En),
                "es": species::display_name(&s.id, Language::Es),
                "class": stats.map(|st| st.hunt_class),
                "group": stats.map(|st| st.group.clone()),
                "trophy": stats.map(|st| st.trophy.clone()),
                "scoring": stats.map(|st| st.scoring.iter().map(|(k, v)| (k.clone(), json!({"weight": v.weight, "score": v.score}))).collect::<serde_json::Map<_, _>>()),
                "bands": bands.map(|b| json!({"bronze": b.bronze, "silver": b.silver, "gold": b.gold, "diamond": b.diamond, "max": b.max})),
                "furs": furs,
                "great_one_furs": go_furs,
                "has_zones": s.areas > 0,
            }));
        }
        out.push(json!({"id": r.id, "name": r.name, "species": list}));
    }
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
