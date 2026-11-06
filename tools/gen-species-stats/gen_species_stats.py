"""Gera crates/cotw-core/data/species_stats.json a partir da instalação do jogo (uso de desenvolvimento).

Só guarda FATOS numéricos e chaves de nome (faixas de peso e de pontos, tipo de troféu, pelagens com
raridade e peso de sorteio). Nenhum arquivo, textura ou texto do jogo vai para o repositório.

Uso:  python -I tools/gen-species-stats/gen_species_stats.py [pasta archives_win64]
Rode de novo quando o jogo atualizar e confira o diff.
"""
import json
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "research", "reference"))

from aaf import aaf  # noqa: E402
from sarc import sarc  # noqa: E402
import rtpc  # noqa: E402

DEFAULT = r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64"
TARGET = "global/global_animal_types.blo"

NAME, KIND, TROPHY, GROUP = 0xD31AB684, 0x1473B179, 0x3CB71423, 0x00836D1B
SCORE_MIN, SCORE_MAX, WEIGHT_MIN, WEIGHT_MAX = 0x140687D8, 0xE3062A0E, 0x87EA42C8, 0x29F241F4
V_INDEX, V_RARITY, V_WEIGHT, V_NAME, V_GO = 0x73EAD679, 0xC82AF7B4, 0xD8A03DB0, 0xF336B29F, 0xEB13FCD3


def find_blob(folder):
    for n in range(144):
        tab_path = os.path.join(folder, f"game{n}.tab")
        if not os.path.exists(tab_path):
            continue
        tab = open(tab_path, "rb").read()
        arc = open(os.path.join(folder, f"game{n}.arc"), "rb")
        for i in range((len(tab) - 12) // 12):
            _, off, size = struct.unpack_from("<III", tab, 12 + 12 * i)
            arc.seek(off)
            raw = arc.read(size)
            if raw[:3] != b"AAF":
                continue
            data = aaf(raw)
            if data[4:8] != b"SARC":
                continue
            for name, o, s in sarc(data):
                if name == TARGET:
                    return data[o : o + s]
    raise SystemExit("global_animal_types.blo não encontrado")


def prop(node, key, default=None):
    p = node["props"].get(key)
    return p[1] if p else default


def r(x):
    return round(float(x), 3)


def main():
    folder = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
    root = rtpc.parse(find_blob(folder))
    species = {}
    for lst in root["kids"]:
        for sp in lst["kids"]:
            sid = prop(sp, NAME)
            entry = {"trophy": (prop(sp, TROPHY) or "").replace("harvest_trophy_type_", ""), "group": prop(sp, GROUP) or "", "scoring": {}, "furs": []}
            for kid in sp["kids"]:
                for c in kid["kids"]:
                    k = prop(c, KIND)
                    if k == "SAnimalTypeScoringDistributionSettings":
                        entry["scoring"][prop(c, NAME)] = {
                            "weight": [r(prop(c, WEIGHT_MIN, 0)), r(prop(c, WEIGHT_MAX, 0))],
                            "score": [r(prop(c, SCORE_MIN, 0)), r(prop(c, SCORE_MAX, 0))],
                        }
                    elif k == "SAnimalTypeVisualVariation":
                        entry["furs"].append(
                            {
                                "index": prop(c, V_INDEX),
                                "key": (prop(c, V_NAME) or "").replace("animal_visual_variation_", ""),
                                "rarity": prop(c, V_RARITY),
                                "odds": prop(c, V_WEIGHT),
                                "great_one": bool(prop(c, V_GO, 0)),
                            }
                        )
            entry["furs"].sort(key=lambda f: f["index"])
            species[sid] = entry
    out = os.path.join(REPO, "crates", "cotw-core", "data", "species_stats.json")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf8", newline="\n") as f:
        json.dump({"species": dict(sorted(species.items()))}, f, indent=1, sort_keys=True)
        f.write("\n")
    print("species:", len(species), "->", out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
