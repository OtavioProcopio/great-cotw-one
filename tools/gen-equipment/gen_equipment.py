"""Gera crates/cotw-core/data/equipment.json a partir da instalação do jogo (uso de desenvolvimento).

Só guarda FATOS: id do item, categoria, nome em inglês, preço, peso, caixa de munição, classe da
arma, caminho do ícone e a compatibilidade arma, munição e mira. Nenhum arquivo ou textura do jogo vai
para o repositório. Peles e suportes de troféu ficam de fora (só contam como categoria).

Fonte: settings/hp_settings/equipment_data.bin (RTPC) em um dos arquivos game*.tab.

Uso:  python -I tools/gen-equipment/gen_equipment.py [pasta archives_win64]
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
from hash import hashlittle  # noqa: E402
import rtpc  # noqa: E402

DEFAULT = r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64"
TARGET = "settings/hp_settings/equipment_data.bin"

ID, NAME, ICON, PRICE = 0xD31AB684, 0x2314C9EA, 0x31E9A4E0, 0x33DF3B2F
WEIGHT, CLASS, BOX = 0x3D214106, 0x765FBED2, 0xB19E6900
COMPAT_KID, AMMO_WEAPON = 0x77C2B3BA, 0x36642277
NONE = 0xFFFFFFFF

# Ordem dos grupos no arquivo do jogo.
GROUPS = ["ammo", "consumable", "trophy_holder", "sight", "optic", "vehicle", "skin", "weapon", "deployable", "lure_caller"]
SKIPPED = {"trophy_holder", "skin"}
WEAPON_CLASSES = ["rifle", "handgun", "shotgun", "archery"]


def find_file(folder):
    want = hashlittle(TARGET.encode())
    for n in range(144):
        tab_path = os.path.join(folder, f"game{n}.tab")
        if not os.path.exists(tab_path):
            continue
        tab = open(tab_path, "rb").read()
        for i in range((len(tab) - 12) // 12):
            h, off, size = struct.unpack_from("<III", tab, 12 + 12 * i)
            if h == want:
                with open(os.path.join(folder, f"game{n}.arc"), "rb") as arc:
                    arc.seek(off)
                    raw = arc.read(size)
                return aaf(raw) if raw[:3] == b"AAF" else raw
    raise SystemExit("equipment_data.bin não encontrado")


def prop(node, key, default=None):
    p = node["props"].get(key)
    return p[1] if p else default


def compatible(node):
    """Ids dos itens ligados a este (armas de uma munição ou de uma mira)."""
    out = []
    for kid in node["kids"]:
        if kid["h"] != COMPAT_KID:
            continue
        for c in kid["kids"]:
            wid = prop(c, AMMO_WEAPON)
            if wid:
                out.append(wid)
        for key, (t, v) in kid["props"].items():
            if t == 3 and isinstance(v, str) and v.startswith("equipment_"):
                out.append(v)
    return sorted(set(out))


def main():
    folder = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
    root = rtpc.parse(find_file(folder))
    items, counts = {}, {}
    for kind, group in zip(GROUPS, root["kids"]):
        members = [k for k in group["kids"] if ID in k["props"]]
        counts[kind] = len(members)
        if kind in SKIPPED:
            continue
        for it in members:
            iid = prop(it, ID)
            entry = {"kind": kind, "name": (prop(it, NAME) or "").strip(), "price": prop(it, PRICE, 0)}
            weight = prop(it, WEIGHT)
            if weight:
                entry["weight"] = round(float(weight), 3)
            if kind == "weapon":
                entry["class"] = WEAPON_CLASSES[prop(it, CLASS, 0)]
            if kind == "ammo" and prop(it, BOX, NONE) != NONE:
                entry["box"] = prop(it, BOX)
            if kind in ("ammo", "sight"):
                entry["fits"] = compatible(it)
            icon = prop(it, ICON)
            if icon:
                entry["icon"] = icon.replace("\\", "/")
            items[iid] = entry
    # Armas que usam cada munição e mira: o inverso fica fácil de consultar no app.
    for iid, entry in items.items():
        if entry["kind"] == "weapon":
            entry["ammo"] = sorted(i for i, e in items.items() if e["kind"] == "ammo" and iid in e.get("fits", []))
            entry["sights"] = sorted(i for i, e in items.items() if e["kind"] == "sight" and iid in e.get("fits", []))
    out = os.path.join(REPO, "crates", "cotw-core", "data", "equipment.json")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf8", newline="\n") as f:
        json.dump({"counts": counts, "items": dict(sorted(items.items()))}, f, indent=1, sort_keys=True, ensure_ascii=False)
        f.write("\n")
    print("itens:", len(items), counts, "->", out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
