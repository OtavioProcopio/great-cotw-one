//! Nomes das espécies em português, inglês e espanhol, indexados pelo nome que o jogo usa nos arquivos.

use crate::language::Language;

/// (`id` do jogo, nome em português, em inglês e em espanhol), em ordem alfabética do `id`.
const NAMES: &[(&str, &str, &str, &str)] = &[
    // Nome que o mapa de aparição de Te Awaroa usa para a cabra-selvagem.
    (
        "alpine_goat",
        "Cabra-selvagem",
        "Feral Goat",
        "Cabra asilvestrada",
    ),
    (
        "american_alligator",
        "Jacaré-americano",
        "American Alligator",
        "Aligátor americano",
    ),
    (
        "american_mink",
        "Vison-americano",
        "American Mink",
        "Visón americano",
    ),
    (
        "antelope_jackrabbit",
        "Lebre-antílope",
        "Antelope Jackrabbit",
        "Liebre antílope",
    ),
    ("axis_deer", "Cervo-áxis", "Axis Deer", "Ciervo axis"),
    ("banteng", "Banteng", "Banteng", "Banteng"),
    ("barasingha", "Barasingha", "Barasingha", "Barasinga"),
    (
        "bean_goose",
        "Ganso-campestre",
        "Tundra Bean Goose",
        "Ganso campestre",
    ),
    ("bear", "Urso-negro", "Black Bear", "Oso negro"),
    (
        "beceite_ibex",
        "Íbex de Beceite",
        "Beceite Ibex",
        "Cabra montés de Beceite",
    ),
    (
        "bengal_tiger",
        "Tigre-de-bengala",
        "Bengal Tiger",
        "Tigre de Bengala",
    ),
    (
        "bighorn_sheep",
        "Carneiro-selvagem",
        "Bighorn Sheep",
        "Borrego cimarrón",
    ),
    (
        "bison_upper",
        "Bisão-europeu",
        "European Bison",
        "Bisonte europeo",
    ),
    ("black_bear", "Urso-negro", "Black Bear", "Oso negro"),
    ("black_caiman", "Jacaré-açu", "Black Caiman", "Caimán negro"),
    ("black_grouse", "Galo-lira", "Black Grouse", "Gallo lira"),
    (
        "blackbuck",
        "Antílope-indiano",
        "Blackbuck",
        "Antílope negro",
    ),
    (
        "blacktail",
        "Cervo-de-cauda-preta",
        "Blacktail Deer",
        "Ciervo de cola negra",
    ),
    (
        "blacktail_deer",
        "Veado-de-cauda-preta",
        "Blacktail Deer",
        "Ciervo de cola negra",
    ),
    ("blue_sheep", "Bharal", "Blue Sheep", "Bharal"),
    ("blue_wildebeest", "Gnu-azul", "Blue Wildebeest", "Ñu azul"),
    ("boar", "Javali", "Wild Boar", "Jabalí"),
    ("bobcat", "Lince-pardo", "Bobcat", "Lince rojo"),
    (
        "brown_bear",
        "Urso-pardo",
        "Eurasian Brown Bear",
        "Oso pardo euroasiático",
    ),
    (
        "canada_goose",
        "Ganso-do-canadá",
        "Canada Goose",
        "Ganso del Canadá",
    ),
    (
        "cape_buffalo",
        "Búfalo-africano",
        "Cape Buffalo",
        "Búfalo africano",
    ),
    (
        "capercaillie",
        "Tetraz-grande",
        "Western Capercaillie",
        "Urogallo común",
    ),
    ("capybara", "Capivara", "Capybara", "Capibara"),
    ("caribou", "Caribu", "Caribou", "Caribú"),
    ("chamois", "Camurça", "Chamois", "Rebeco"),
    (
        "cinnamon_teal",
        "Marreca-canela",
        "Cinnamon Teal",
        "Cerceta canela",
    ),
    (
        "collared_peccary",
        "Cateto",
        "Collared Peccary",
        "Pecarí de collar",
    ),
    ("coyote", "Coiote", "Coyote", "Coyote"),
    (
        "dusky_grouse",
        "Tetraz-sombrio",
        "Dusky Grouse",
        "Gallo oscuro",
    ),
    (
        "eastern_cottontail_rabbit",
        "Coelho-de-cauda-de-algodão",
        "Eastern Cottontail Rabbit",
        "Conejo de cola de algodón",
    ),
    (
        "eastern_grey_kangaroo",
        "Canguru-cinzento",
        "Eastern Gray Kangaroo",
        "Canguro gris oriental",
    ),
    (
        "eastern_wild_turkey",
        "Peru-selvagem oriental",
        "Eastern Wild Turkey",
        "Pavo salvaje oriental",
    ),
    ("elk", "Alce-americano", "Elk", "Wapití"),
    (
        "eu_rabbit",
        "Coelho-europeu",
        "European Rabbit",
        "Conejo europeo",
    ),
    (
        "eurasian_pine_marten",
        "Marta-europeia",
        "Eurasian Pine Marten",
        "Marta europea",
    ),
    (
        "eurasian_teal",
        "Marrequinha",
        "Eurasian Teal",
        "Cerceta común",
    ),
    (
        "eurasian_wigeon",
        "Piadeira",
        "Eurasian Wigeon",
        "Silbón europeo",
    ),
    (
        "eurasian_woodcock",
        "Galinhola",
        "Eurasian Woodcock",
        "Chocha perdiz",
    ),
    (
        "european_badger",
        "Texugo-europeu",
        "European Badger",
        "Tejón europeo",
    ),
    (
        "european_bison",
        "Bisão-europeu",
        "European Bison",
        "Bisonte europeo",
    ),
    (
        "european_hare",
        "Lebre-europeia",
        "European Hare",
        "Liebre europea",
    ),
    ("fallow", "Gamo", "Fallow Deer", "Gamo"),
    ("fallow_deer", "Gamo", "Fallow Deer", "Gamo"),
    (
        "feral_goat",
        "Cabra-selvagem",
        "Feral Goat",
        "Cabra asilvestrada",
    ),
    (
        "feral_pig",
        "Porco-selvagem",
        "Feral Pig",
        "Cerdo asilvestrado",
    ),
    (
        "ferruginous_duck",
        "Zarro-castanho",
        "Ferruginous Duck",
        "Porrón pardo",
    ),
    ("fox", "Raposa-vermelha", "Red Fox", "Zorro rojo"),
    ("gadwall", "Frisada", "Gadwall", "Ánade friso"),
    ("gemsbok", "Órix", "Gemsbok", "Órice"),
    ("goldeneye", "Olho-dourado", "Goldeneye", "Porrón osculado"),
    ("gray_fox", "Raposa-cinzenta", "Gray Fox", "Zorro gris"),
    ("gray_wolf", "Lobo-cinzento", "Gray Wolf", "Lobo gris"),
    ("greater_grison", "Furão-grande", "Greater Grison", "Grisón"),
    (
        "gredos_ibex",
        "Íbex de Gredos",
        "Gredos Ibex",
        "Cabra montés de Gredos",
    ),
    (
        "green_wing_teal",
        "Marreca-de-asa-verde",
        "Green-winged Teal",
        "Cerceta aliverde",
    ),
    ("grey_fox", "Raposa-cinzenta", "Gray Fox", "Zorro gris"),
    (
        "greylag_goose",
        "Ganso-bravo",
        "Greylag Goose",
        "Ganso común",
    ),
    (
        "grizzly_bear",
        "Urso-pardo (grizzly)",
        "Grizzly Bear",
        "Oso grizzly",
    ),
    (
        "harlequin_duck",
        "Pato-arlequim",
        "Harlequin Duck",
        "Pato arlequín",
    ),
    ("hazel_grouse", "Galinha-do-mato", "Hazel Grouse", "Grévol"),
    ("hog_deer", "Cervo-porco", "Hog Deer", "Ciervo porcino"),
    (
        "iberian_mouflon",
        "Muflão-ibérico",
        "Iberian Mouflon",
        "Muflón ibérico",
    ),
    (
        "iberian_wolf",
        "Lobo-ibérico",
        "Iberian Wolf",
        "Lobo ibérico",
    ),
    ("jackal", "Chacal", "Side-striped Jackal", "Chacal rayado"),
    (
        "jackrabbit",
        "Lebre-americana",
        "White-tailed Jackrabbit",
        "Liebre de cola blanca",
    ),
    ("jaguar", "Onça-pintada", "Jaguar", "Jaguar"),
    (
        "javan_rusa",
        "Rusa-de-java",
        "Javan Rusa",
        "Ciervo rusa de Java",
    ),
    ("lesser_kudu", "Cudo-menor", "Lesser Kudu", "Kudú menor"),
    ("lion", "Leão", "Lion", "León"),
    (
        "lynx",
        "Lince-euroasiático",
        "Eurasian Lynx",
        "Lince euroasiático",
    ),
    ("magpie_goose", "Ganso-pega", "Magpie Goose", "Ganso urraca"),
    ("mallard", "Pato-real", "Mallard", "Ánade real"),
    (
        "manitoban_elk",
        "Uapiti-de-manitoba",
        "Manitoban Elk",
        "Uapití de Manitoba",
    ),
    ("moose", "Alce", "Moose", "Alce"),
    (
        "mountain_goat",
        "Cabra-das-rochosas",
        "Mountain Goat",
        "Cabra de las Rocosas",
    ),
    (
        "mountain_hare",
        "Lebre-variável",
        "Mountain Hare",
        "Liebre variable",
    ),
    ("mule_deer", "Veado-mula", "Mule Deer", "Ciervo mulo"),
    (
        "musk_deer",
        "Almiscareiro-siberiano",
        "Siberian Musk Deer",
        "Ciervo almizclero siberiano",
    ),
    ("nilgai", "Nilgai", "Nilgai", "Nilgó"),
    (
        "north_american_beaver",
        "Castor-americano",
        "North American Beaver",
        "Castor americano",
    ),
    (
        "northern_bobwhite_quail",
        "Codorna-colinus",
        "Northern Bobwhite Quail",
        "Codorniz bobwhite",
    ),
    (
        "northern_pintail",
        "Arrábio",
        "Northern Pintail",
        "Ánade rabudo",
    ),
    (
        "northern_red_muntjac",
        "Muntíaco",
        "Northern Red Muntjac",
        "Muntíaco rojo",
    ),
    ("ocelot", "Jaguatirica", "Ocelot", "Ocelote"),
    ("peccary", "Caititu", "Collared Peccary", "Pecarí de collar"),
    (
        "pheasant",
        "Faisão",
        "Ring-necked Pheasant",
        "Faisán de collar",
    ),
    (
        "plains_bison",
        "Bisão-das-planícies",
        "Plains Bison",
        "Bisonte de las llanuras",
    ),
    ("pronghorn", "Antilocapra", "Pronghorn", "Berrendo"),
    ("puma", "Puma", "Puma", "Puma"),
    ("raccoon", "Guaxinim", "Common Raccoon", "Mapache"),
    (
        "raccoon_dog",
        "Cão-guaxinim",
        "Raccoon Dog",
        "Perro mapache",
    ),
    ("red_deer", "Cervo-vermelho", "Red Deer", "Ciervo rojo"),
    ("red_fox", "Raposa-vermelha", "Red Fox", "Zorro rojo"),
    (
        "red_grouse",
        "Lagópode-escocês",
        "Red Grouse",
        "Lagópodo escocés",
    ),
    ("reindeer", "Rena", "Mountain Reindeer", "Reno"),
    (
        "rio_grande_turkey",
        "Peru Rio Grande",
        "Rio Grande Turkey",
        "Pavo del Río Grande",
    ),
    (
        "rock_ptarmigan",
        "Lagópode-branco",
        "Rock Ptarmigan",
        "Perdiz nival",
    ),
    (
        "rocky_mountain_elk",
        "Uapiti-das-rochosas",
        "Rocky Mountain Elk",
        "Wapití de las Rocosas",
    ),
    (
        "rockymountain_elk",
        "Alce-das-montanhas-rochosas",
        "Rocky Mountain Elk",
        "Wapití de las Rocosas",
    ),
    ("roe", "Corço", "Roe Deer", "Corzo"),
    ("roe_deer", "Corço", "Roe Deer", "Corzo"),
    (
        "ronda_ibex",
        "Íbex de Ronda",
        "Ronda Ibex",
        "Cabra montés de Ronda",
    ),
    (
        "roosevelt_elk",
        "Uapiti-de-roosevelt",
        "Roosevelt Elk",
        "Wapití de Roosevelt",
    ),
    (
        "saltwater_crocodile",
        "Crocodilo-de-água-salgada",
        "Saltwater Crocodile",
        "Cocodrilo de agua salada",
    ),
    ("sambar", "Sambar", "Sambar", "Sambar"),
    (
        "scrub_hare",
        "Lebre-do-mato",
        "Scrub Hare",
        "Liebre de matorral",
    ),
    (
        "siberian_musk_deer",
        "Cervo-almiscarado",
        "Siberian Musk Deer",
        "Ciervo almizclero siberiano",
    ),
    (
        "side_striped_jackal",
        "Chacal-listrado",
        "Side-striped Jackal",
        "Chacal rayado",
    ),
    ("sika_deer", "Cervo-sika", "Sika Deer", "Ciervo sika"),
    ("snow_goose", "Ganso-das-neves", "Snow Goose", "Ánsar nival"),
    (
        "snow_leopard",
        "Leopardo-das-neves",
        "Snow Leopard",
        "Leopardo de las nieves",
    ),
    (
        "south_american_tapir",
        "Anta",
        "South American Tapir",
        "Tapir amazónico",
    ),
    (
        "southeastern_ibex",
        "Íbex-do-sudeste",
        "Southeastern Spanish Ibex",
        "Cabra montés del sudeste",
    ),
    (
        "southeastern_spanish_ibex",
        "Íbex do sudeste",
        "Southeastern Spanish Ibex",
        "Cabra montés del sudeste",
    ),
    (
        "spectacled_bear",
        "Urso-de-óculos",
        "Spectacled Bear",
        "Oso de anteojos",
    ),
    ("springbok", "Springbok", "Springbok", "Springbok"),
    (
        "stubble_quail",
        "Codorniz-australiana",
        "Stubble Quail",
        "Codorniz australiana",
    ),
    (
        "tahr",
        "Tahr-do-himalaia",
        "Himalayan Tahr",
        "Tahr del Himalaya",
    ),
    ("taruca", "Taruca", "Taruca", "Taruca"),
    (
        "tibetan_fox",
        "Raposa-tibetana",
        "Tibetan Fox",
        "Zorro tibetano",
    ),
    (
        "tufted_duck",
        "Zarro-negrinha",
        "Tufted Duck",
        "Porrón moñudo",
    ),
    ("vicuna", "Vicunha", "Vicuña", "Vicuña"),
    ("warthog", "Javali-africano", "Warthog", "Facóquero"),
    (
        "water_buffalo",
        "Búfalo-asiático",
        "Water Buffalo",
        "Búfalo de agua",
    ),
    (
        "western_capercaillie",
        "Tetraz-grande",
        "Western Capercaillie",
        "Urogallo común",
    ),
    (
        "western_mountain_coati",
        "Quati-da-montanha",
        "Western Mountain Coati",
        "Coatí de montaña",
    ),
    (
        "whitetail",
        "Cervo-de-cauda-branca",
        "Whitetail Deer",
        "Ciervo de cola blanca",
    ),
    (
        "whitetail_deer",
        "Veado-de-cauda-branca",
        "Whitetail Deer",
        "Ciervo de cola blanca",
    ),
    ("wild_boar", "Javali", "Wild Boar", "Jabalí"),
    (
        "wild_haggis",
        "Haggis-selvagem",
        "Wild Haggis",
        "Haggis salvaje",
    ),
    (
        "wild_turkey",
        "Peru-selvagem",
        "Wild Turkey",
        "Pavo salvaje",
    ),
    ("wild_yak", "Iaque-selvagem", "Wild Yak", "Yak salvaje"),
    ("wildhog", "Porco-asselvajado", "Wild Hog", "Cerdo salvaje"),
    (
        "willow_ptarmigan",
        "Lagópode-dos-salgueiros",
        "Willow Ptarmigan",
        "Lagópodo común",
    ),
    (
        "wood_bison",
        "Bisão-da-floresta",
        "Wood Bison",
        "Bisonte de bosque",
    ),
    ("wood_duck", "Pato-carolino", "Wood Duck", "Pato joyuyo"),
    (
        "woolly_hare",
        "Lebre-lanosa",
        "Woolly Hare",
        "Liebre lanuda",
    ),
];

/// Nome para exibir no idioma pedido. Espécie sem tradução vira o `id` com espaços e a primeira
/// letra maiúscula.
#[must_use]
pub fn display_name(id: &str, language: Language) -> String {
    if let Ok(at) = NAMES.binary_search_by_key(&id, |(key, ..)| key) {
        let (_, pt, en, es) = NAMES[at];
        return match language {
            Language::PtBr => pt,
            Language::En => en,
            Language::Es => es,
        }
        .to_owned();
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
    fn every_species_has_a_name_in_every_language() {
        for (id, pt, en, es) in NAMES {
            assert!(!pt.is_empty() && !en.is_empty() && !es.is_empty(), "{id}");
        }
    }

    #[test]
    fn names_follow_the_language() {
        assert_eq!(display_name("moose", Language::PtBr), "Alce");
        assert_eq!(display_name("moose", Language::En), "Moose");
        assert_eq!(display_name("moose", Language::Es), "Alce");
        assert_eq!(display_name("red_deer", Language::PtBr), "Cervo-vermelho");
        assert_eq!(display_name("red_deer", Language::En), "Red Deer");
        assert_eq!(display_name("red_deer", Language::Es), "Ciervo rojo");
        assert_eq!(display_name("gray_wolf", Language::Es), "Lobo gris");
    }

    #[test]
    fn an_unknown_species_becomes_a_readable_id_in_any_language() {
        for language in [Language::PtBr, Language::En, Language::Es] {
            assert_eq!(display_name("new_species_x", language), "New species x");
        }
        assert_eq!(display_name("", Language::En), "");
    }
}
