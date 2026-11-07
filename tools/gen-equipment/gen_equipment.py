"""Gera crates/cotw-core/data/equipment.json a partir da instalação do jogo (uso de desenvolvimento).

Só guarda FATOS: id do item, categoria, nome em inglês, preço, peso, caixa de munição, classe da
arma, caminho do ícone e a compatibilidade arma, munição e mira. Nenhum arquivo ou textura do jogo vai
para o repositório. Peles e suportes de troféu ficam de fora (só contam como categoria).

Fontes (todas em um dos arquivos game*.tab):
- settings/hp_settings/equipment_data.bin (RTPC): catálogo.
- planilha de estatísticas dos itens (ADF `XLSBook`, hash 0x651d89a7): precisão, recuo e recarga das armas;
  alcance, penetração, expansão e classes de caça recomendadas das munições; aumento das miras.
- settings/hp_settings/player_rewards.bin (ADF `XLSBook`): nível do caçador e das armas que libera cada item.

Uso:  python -I tools/gen-equipment/gen_equipment.py [pasta archives_win64]
Rode de novo quando o jogo atualizar e confira o diff.
"""
import json
import os
import struct
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "research", "reference"))

from aaf import aaf  # noqa: E402
from hash import hashlittle  # noqa: E402
import rtpc  # noqa: E402
import adf as adf_reader  # noqa: E402

DEFAULT = r"C:\Program Files (x86)\Steam\steamapps\common\theHunterCotW\archives_win64"
TARGET = "settings/hp_settings/equipment_data.bin"
REWARDS = "settings/hp_settings/player_rewards.bin"
# A planilha de estatísticas não tem caminho conhecido; é achada pelo hash.
STATS_HASH = 0x651D89A7

ID, NAME, ICON, PRICE = 0xD31AB684, 0x2314C9EA, 0x31E9A4E0, 0x33DF3B2F
WEIGHT, CLASS, BOX = 0x3D214106, 0x765FBED2, 0xB19E6900
COMPAT_KID, AMMO_WEAPON = 0x77C2B3BA, 0x36642277
NONE = 0xFFFFFFFF

# Ordem dos grupos no arquivo do jogo.
GROUPS = ["ammo", "consumable", "trophy_holder", "sight", "optic", "vehicle", "skin", "weapon", "deployable", "lure_caller"]
SKIPPED = {"trophy_holder", "skin"}
WEAPON_CLASSES = ["rifle", "handgun", "shotgun", "archery"]


def find_hash(folder, want):
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
    raise SystemExit(f"arquivo {want:#x} não encontrado")


def find_file(folder):
    return find_hash(folder, hashlittle(TARGET.encode()))


def sheets(data):
    """Planilhas de um ADF `XLSBook` como {nome: linhas}; célula vazia vira None."""
    with tempfile.NamedTemporaryFile(suffix=".adf", delete=False) as f:
        f.write(data)
        path = f.name
    try:
        root = adf_reader.dump(path)[0]
    finally:
        os.unlink(path)
    book = root[next(iter(root))]
    cells, strings, values = book["Cell"], book["StringData"], book["ValueData"]

    def value(i):
        c = cells[i] if i < len(cells) else None
        if c is None:
            return None
        return strings[c["DataIndex"]] if c["Type"] == 1 else values[c["DataIndex"]] if c["Type"] == 2 else None

    out = {}
    for sh in book["Sheet"]:
        cols = sh["Cols"]
        out[sh["Name"]] = [[value(sh["CellIndex"][r * cols + c]) for c in range(cols)] for r in range(sh["Rows"])]
    return out


def rows_by_item(table):
    """Linhas da planilha indexadas pelo id do item (primeira coluna), com o cabeçalho como chave."""
    header = table[0]
    return {r[0]: dict(zip(header, r)) for r in table[1:] if isinstance(r[0], str) and r[0].startswith("equipment_")}


def number(x):
    return round(float(x), 3) if isinstance(x, (int, float)) else None


def stats(folder):
    """Atributos de cada item, da planilha de estatísticas do jogo."""
    book = sheets(find_hash(folder, STATS_HASH))
    out = {}
    for iid, r in rows_by_item(book["weapons"]).items():
        e = {"accuracy": number(r.get("Stat Accuracy")), "recoil": number(r.get("Stat Recoil")), "reload": number(r.get("Stat Reload Speed"))}
        draw = number(r.get("Stat Peak Draw-weight"))
        if draw is not None and draw >= 0:
            e["draw"] = draw
        out[iid] = {k: v for k, v in e.items() if v is not None}
    for iid, r in rows_by_item(book["ammo"]).items():
        classes = [int(c) for c in str(r.get("Stat Recommended Game Class") or "").split(";") if c.strip().isdigit()]
        e = {"range": number(r.get("Stat Effective Range")), "penetration": number(r.get("Stat Bullet Penetration")), "expansion": number(r.get("Stat Bullet Expansion"))}
        out[iid] = {**{k: v for k, v in e.items() if v is not None}, "classes": classes}
    for iid, r in rows_by_item(book["sights"]).items():
        out[iid] = {"zoom": number(r.get("Stat Magnification"))}
    for iid, r in rows_by_item(book["optics"]).items():
        out[iid] = {"zoom": number(r.get("Stat Magnification"))}
    return out


TRACKS = {"character_levels": "character", "rifle_score_levels": "rifle", "handgun_score_levels": "handgun", "shotgun_score_levels": "shotgun", "bow_score_levels": "archery"}


def unlocks(folder):
    """Onde cada item é liberado: nível do caçador ou nível de uma classe de arma, e a condição."""
    book = sheets(find_hash(folder, hashlittle(REWARDS.encode())))
    out = {}
    for sheet, track in TRACKS.items():
        for row in book[sheet][1:]:
            level, need, ids = row[0], row[1], row[4] if track == "character" else row[2]
            if not isinstance(ids, str):
                continue
            for iid in (i.strip() for i in ids.split(",")):
                if iid.startswith("equipment_") and iid not in out:
                    out[iid] = {"track": track, "level": int(level), "need": int(need or 0)}
    return out


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
    extra, unlock = stats(folder), unlocks(folder)
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
            entry.update(extra.get(iid, {}))
            if iid in unlock:
                entry["unlock"] = unlock[iid]
            icon = prop(it, ICON)
            if icon:
                entry["icon"] = icon.replace("\\", "/")
            items[iid] = entry
    # Armas que usam cada munição e mira: o inverso fica fácil de consultar no app.
    for iid, entry in items.items():
        if entry["kind"] == "weapon":
            entry["ammo"] = sorted(i for i, e in items.items() if e["kind"] == "ammo" and iid in e.get("fits", []))
            entry["sights"] = sorted(i for i, e in items.items() if e["kind"] == "sight" and iid in e.get("fits", []))
    for entry in items.values():
        entry.pop("fits", None)
    # Variantes de uma arma (mesma base, outro número no fim) sem estatísticas própria usam as da irmã.
    for iid, entry in items.items():
        if entry["kind"] == "weapon" and "accuracy" not in entry:
            base = iid.rsplit("_", 1)[0]
            sibling = next((e for i, e in sorted(items.items()) if i.startswith(base + "_") and "accuracy" in e), None)
            if sibling:
                for key in ("accuracy", "recoil", "reload", "draw"):
                    if key in sibling:
                        entry[key] = sibling[key]
    out = os.path.join(REPO, "crates", "cotw-core", "data", "equipment.json")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf8", newline="\n") as f:
        json.dump({"counts": counts, "items": dict(sorted(items.items()))}, f, indent=1, sort_keys=True, ensure_ascii=False)
        f.write("\n")
    print("itens:", len(items), counts, "->", out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
