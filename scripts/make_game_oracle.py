"""Gera o oráculo textual dos formatos do jogo usando research/reference (somente leitura).

Uso: python -I scripts/make_game_oracle.py <pasta archives_win64> research/reference fixtures/game.oracle.txt

O arquivo gerado contém só contagens, offsets e valores de configuração para comparação;
fica em fixtures/ (fora do git) e nunca deve ser versionado.
"""
import glob, hashlib, os, struct, sys, tempfile

D, REF, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
sys.path.insert(0, REF)
import adf, aaf, sarc, rtpc
from hash import hashlittle

PATHS = [
    'missions/achievements.hpmissionsc',
    'settings/hp_settings/player_skills.bin',
    'settings/hp_settings/player_skill_trees.bin',
    'settings/player.bin',
    'nao/existe.bin',
]
AAF_HASH = 0x78209FCC

index, total = {}, 0
for f in sorted(glob.glob(D + '/game*.tab'), key=lambda p: int(os.path.basename(p)[4:-4])):
    n = int(os.path.basename(f)[4:-4])
    t = open(f, 'rb').read()
    for i in range((len(t) - 12) // 12):
        h, o, s = struct.unpack_from('<III', t, 12 + 12 * i)
        index.setdefault(h, []).append((n, o, s))
        total += 1


def read(loc):
    n, o, s = loc
    with open(f'{D}/game{n}.arc', 'rb') as a:
        a.seek(o)
        return a.read(s)


def fnum(v):
    return f'{v:.4f}'


lines = [f'tab_entries\t{total}']
for p in PATHS:
    h = hashlittle(p)
    locs = index.get(h, [])
    lines.append(f'path\t{p}\t{h:08x}\t' + ';'.join(f'{n}:{o}:{s}' for n, o, s in locs))


def load_adf(data):
    tmp = tempfile.NamedTemporaryFile(delete=False)
    tmp.write(data)
    tmp.close()
    A = adf.load(tmp.name)
    os.unlink(tmp.name)
    return A


for p in PATHS[1:3]:
    data = read(index[hashlittle(p)][0])
    A = load_adf(data)
    inst = A['insts'][0]
    A['base'] = inst['off']
    book = adf.read_val(A, inst['th'], inst['off'])
    lines.append(f'xls\t{p}')
    for sh in book['Sheet']:
        lines.append(f"sheet\t{sh['Name']}\t{sh['Rows']}\t{sh['Cols']}")
        for r in range(sh['Rows']):
            for c in range(sh['Cols']):
                pos = r * sh['Cols'] + c
                if pos >= len(sh['CellIndex']):
                    continue
                cell = book['Cell'][sh['CellIndex'][pos]]
                if cell['Type'] == 1:
                    val = 'T\t' + book['StringData'][cell['DataIndex']]
                elif cell['Type'] == 2:
                    val = 'N\t' + fnum(book['ValueData'][cell['DataIndex']])
                else:
                    val = f"U\t{cell['Type']}"
                lines.append(f'cell\t{r}\t{c}\t{val}')


def rprop(t, v):
    if t == 1 or t == 13:
        return str(v)
    if t == 2:
        return fnum(v)
    if t == 3:
        return v
    if t in (4, 5, 6, 7, 8, 10):
        return ','.join(fnum(x) for x in v)
    if t == 9:
        return ','.join(str(x) for x in v)
    if t == 14:
        return ';'.join(f'{a}:{b}' for a, b in v)
    return v.hex()  # 11 (bytes) e tipos desconhecidos (4 bytes brutos)


def rnode(n, depth):
    lines.append(f"node\t{depth}\t{n['h']:08x}\t{len(n['props'])}\t{len(n['kids'])}")
    for ph, (t, v) in n['props'].items():
        lines.append(f'prop\t{depth}\t{ph:08x}\t{t}\t{rprop(t, v)}')
    for k in n['kids']:
        rnode(k, depth + 1)


p = 'settings/player.bin'
lines.append(f'rtpc\t{p}')
rnode(rtpc.parse(read(index[hashlittle(p)][0])), 0)

raw = read(index[AAF_HASH][0])
out = aaf.aaf(raw)
lines.append(f'aaf\t{AAF_HASH:08x}\t{len(out)}\t{hashlib.sha256(out).hexdigest()}')
for name, o, s in sarc.sarc(out):
    lines.append(f'sarc\t{name}\t{o}\t{s}')

open(OUT, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
print(len(lines), 'linhas')
