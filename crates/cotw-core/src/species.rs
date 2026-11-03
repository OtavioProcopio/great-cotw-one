//! Nomes das espécies em português, indexados pelo nome que o jogo usa nos arquivos.

/// (`id` do jogo, nome para exibir), em ordem alfabética do `id`.
const NAMES: &[(&str, &str)] = &[
    ("american_alligator", "Jacaré-americano"),
    ("antelope_jackrabbit", "Lebre-antílope"),
    ("axis_deer", "Cervo-áxis"),
    ("banteng", "Banteng"),
    ("barasingha", "Barasingha"),
    ("bean_goose", "Ganso-campestre"),
    ("bear", "Urso-negro"),
    ("beceite_ibex", "Íbex de Beceite"),
    ("bengal_tiger", "Tigre-de-bengala"),
    ("bighorn_sheep", "Carneiro-selvagem"),
    ("bison_upper", "Bisão-europeu"),
    ("black_bear", "Urso-negro"),
    ("black_grouse", "Galo-lira"),
    ("blackbuck", "Antílope-indiano"),
    ("blacktail", "Cervo-de-cauda-preta"),
    ("blacktail_deer", "Veado-de-cauda-preta"),
    ("blue_sheep", "Bharal"),
    ("blue_wildebeest", "Gnu-azul"),
    ("boar", "Javali"),
    ("bobcat", "Lince-pardo"),
    ("brown_bear", "Urso-pardo"),
    ("canada_goose", "Ganso-do-canadá"),
    ("cape_buffalo", "Búfalo-africano"),
    ("capercaillie", "Tetraz-grande"),
    ("caribou", "Caribu"),
    ("cinnamon_teal", "Marreca-canela"),
    ("collared_peccary", "Cateto"),
    ("coyote", "Coiote"),
    ("eastern_cottontail_rabbit", "Coelho-de-cauda-de-algodão"),
    ("eastern_grey_kangaroo", "Canguru-cinzento"),
    ("eastern_wild_turkey", "Peru-selvagem oriental"),
    ("elk", "Alce-americano"),
    ("eu_rabbit", "Coelho-europeu"),
    ("eurasian_teal", "Marrequinha"),
    ("eurasian_wigeon", "Piadeira"),
    ("european_bison", "Bisão-europeu"),
    ("european_hare", "Lebre-europeia"),
    ("fallow", "Gamo"),
    ("fallow_deer", "Gamo"),
    ("feral_goat", "Cabra-selvagem"),
    ("feral_pig", "Porco-selvagem"),
    ("fox", "Raposa-vermelha"),
    ("gemsbok", "Órix"),
    ("goldeneye", "Olho-dourado"),
    ("gray_fox", "Raposa-cinzenta"),
    ("gray_wolf", "Lobo-cinzento"),
    ("gredos_ibex", "Íbex de Gredos"),
    ("green_wing_teal", "Marreca-de-asa-verde"),
    ("grey_fox", "Raposa-cinzenta"),
    ("greylag_goose", "Ganso-bravo"),
    ("grizzly_bear", "Urso-pardo (grizzly)"),
    ("harlequin_duck", "Pato-arlequim"),
    ("hazel_grouse", "Galinha-do-mato"),
    ("hog_deer", "Cervo-porco"),
    ("iberian_mouflon", "Muflão-ibérico"),
    ("iberian_wolf", "Lobo-ibérico"),
    ("jackal", "Chacal"),
    ("jackrabbit", "Lebre-americana"),
    ("javan_rusa", "Rusa-de-java"),
    ("lesser_kudu", "Cudo-menor"),
    ("lion", "Leão"),
    ("lynx", "Lince-euroasiático"),
    ("magpie_goose", "Ganso-pega"),
    ("mallard", "Pato-real"),
    ("moose", "Alce"),
    ("mountain_goat", "Cabra-das-rochosas"),
    ("mountain_hare", "Lebre-variável"),
    ("mule_deer", "Veado-mula"),
    ("musk_deer", "Almiscareiro-siberiano"),
    ("nilgai", "Nilgai"),
    ("northern_bobwhite_quail", "Codorna-colinus"),
    ("northern_red_muntjac", "Muntíaco"),
    ("peccary", "Caititu"),
    ("pheasant", "Faisão"),
    ("plains_bison", "Bisão-das-planícies"),
    ("pronghorn", "Antilocapra"),
    ("puma", "Puma"),
    ("raccoon", "Guaxinim"),
    ("raccoon_dog", "Cão-guaxinim"),
    ("red_deer", "Cervo-vermelho"),
    ("red_fox", "Raposa-vermelha"),
    ("reindeer", "Rena"),
    ("rio_grande_turkey", "Peru Rio Grande"),
    ("rock_ptarmigan", "Lagópode-branco"),
    ("rocky_mountain_elk", "Uapiti-das-rochosas"),
    ("rockymountain_elk", "Alce-das-montanhas-rochosas"),
    ("roe", "Corço"),
    ("roe_deer", "Corço"),
    ("ronda_ibex", "Íbex de Ronda"),
    ("roosevelt_elk", "Uapiti-de-roosevelt"),
    ("saltwater_crocodile", "Crocodilo-de-água-salgada"),
    ("sambar", "Sambar"),
    ("scrub_hare", "Lebre-do-mato"),
    ("siberian_musk_deer", "Cervo-almiscarado"),
    ("side_striped_jackal", "Chacal-listrado"),
    ("snow_leopard", "Leopardo-das-neves"),
    ("southeastern_ibex", "Íbex-do-sudeste"),
    ("southeastern_spanish_ibex", "Íbex do sudeste"),
    ("springbok", "Springbok"),
    ("stubble_quail", "Codorniz-australiana"),
    ("tahr", "Tahr-do-himalaia"),
    ("tibetan_fox", "Raposa-tibetana"),
    ("tufted_duck", "Zarro-negrinha"),
    ("warthog", "Javali-africano"),
    ("water_buffalo", "Búfalo-asiático"),
    ("western_capercaillie", "Tetraz-grande"),
    ("whitetail", "Cervo-de-cauda-branca"),
    ("whitetail_deer", "Veado-de-cauda-branca"),
    ("wild_boar", "Javali"),
    ("wild_turkey", "Peru-selvagem"),
    ("wild_yak", "Iaque-selvagem"),
    ("wildhog", "Porco-selvagem"),
    ("willow_ptarmigan", "Lagópode-escocês"),
    ("woolly_hare", "Lebre-lanosa"),
];

/// Nome para exibir. Espécie sem tradução vira o `id` com espaços e a primeira letra maiúscula.
#[must_use]
pub fn display_name(id: &str) -> String {
    if let Ok(at) = NAMES.binary_search_by_key(&id, |(key, _)| key) {
        return NAMES[at].1.to_owned();
    }
    let spaced = id.replace('_', " ");
    let mut chars = spaced.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_so_binary_search_works() {
        assert!(NAMES.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn known_species_use_the_portuguese_name() {
        assert_eq!(display_name("moose"), "Alce");
        assert_eq!(display_name("whitetail"), "Cervo-de-cauda-branca");
    }

    #[test]
    fn unknown_species_get_a_readable_fallback() {
        assert_eq!(display_name("wild_haggis"), "Wild haggis");
        assert_eq!(display_name(""), "");
    }
}
