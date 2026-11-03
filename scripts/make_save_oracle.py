"""Gera o oráculo valor a valor dos saves usando research/reference/adf.py (somente leitura).

Uso: python -I scripts/make_save_oracle.py research/reference fixtures/saves [--dump <pasta>]

Escreve fixtures/saves/_index.oracle.txt com uma linha por arquivo:
  nome  skip                                          sem cabeçalho SAVE (não é um save com envelope)
  nome  noadf  tamanho  sha256                        envelope válido, conteúdo que não é ADF
  nome  adf    tamanho  sha256  linhas  sha256-linhas envelope + ADF, valores achatados
(`sha256` é do conteúdo descomprimido; `sha256-linhas` é do texto achatado, uma linha por valor).

Com --dump grava também o texto achatado completo de cada arquivo em <pasta>, para comparar
com `diff` quando o teste Rust apontar uma divergência. Os textos são grandes (centenas de MB):
use uma pasta temporária fora do OneDrive.

O índice fica em fixtures/ (fora do git): deriva do save do desenvolvedor.
"""
import hashlib, os, struct, sys, tempfile, zlib

REF, SAVES = sys.argv[1], sys.argv[2]
DUMP = sys.argv[sys.argv.index('--dump') + 1] if '--dump' in sys.argv else None
if DUMP:
    os.makedirs(DUMP, exist_ok=True)
sys.path.insert(0, REF)
import adf

U8, S8 = 0x0CA2821D, 0x580D0A62
_original = adf.read_val


def read_val(A, th, off, size=None):
    """Igual ao de referência, mas arrays de bytes saem crus (sem as heurísticas de texto)."""
    t = A['types'].get(th) if th not in adf.PRIM else None
    if t and t['meta'] == 3 and t['eth'] in (U8, S8):
        p, _chain, c = struct.unpack_from('<IIQ', A['d'], off)
        p += A.get('base', 0)
        return {'__bytes__': A['d'][p:p + c].hex()}
    if t and t['meta'] == 4 and t['eth'] in (U8, S8):
        return {'__bytes__': A['d'][off:off + t['elen']].hex()}
    return _original(A, th, off, size)


adf.read_val = read_val  # a recursão do leitor de referência passa a usar esta versão


def num(v):
    if v != v:
        return 'nan'
    return f'{v:.4f}'


def flat(v, path, out):
    if isinstance(v, dict):
        if '__bytes__' in v:
            out.append(f'{path}\tbytes\t{v["__bytes__"]}')
        elif '?hash' in v:
            out.append(f'{path}\traw\t{v["?hash"]}:{v["raw"]}')
        elif 'prim?' in v:
            out.append(f'{path}\tprim\t{v["prim?"]}:{v["raw"]}')
        elif list(v) == ['ptr']:
            out.append(f'{path}\tptr\t{v["ptr"]}')
        elif 'meta' in v and 'raw' in v and len(v) == 2:
            out.append(f'{path}\tmeta\t{v["meta"]}:{v["raw"]}')
        else:
            for k, x in v.items():
                flat(x, f'{path}.{k}', out)
    elif isinstance(v, list):
        out.append(f'{path}\tarray\t{len(v)}')
        for i, x in enumerate(v):
            flat(x, f'{path}[{i}]', out)
    elif isinstance(v, str):
        out.append(f'{path}\tstr\t{v.encode().hex()}')
    elif isinstance(v, float):
        out.append(f'{path}\tfloat\t{num(v)}')
    else:
        out.append(f'{path}\tint\t{v}')


index = []
for name in sorted(os.listdir(SAVES)):
    full = os.path.join(SAVES, name)
    if not os.path.isfile(full) or name.endswith('.oracle.txt'):
        continue
    raw = open(full, 'rb').read()
    if raw[:4] != b'SAVE' or len(raw) < 40:
        index.append(f'{name}\tskip')
        continue
    payload = zlib.decompress(raw[32:])
    digest = hashlib.sha256(payload).hexdigest()
    if payload[5:9] != b' FDA':
        index.append(f'{name}\tnoadf\t{len(payload)}\t{digest}')
        continue
    tmp = tempfile.NamedTemporaryFile(delete=False)
    tmp.write(payload[5:])
    tmp.close()
    A = adf.load(tmp.name)
    os.unlink(tmp.name)
    lines = []
    for n, inst in enumerate(A['insts']):
        A['base'] = inst['off']
        head = f'[{n}:{inst["name"]}]'
        try:
            flat(adf.read_val(A, inst['th'], inst['off']), head, lines)
        except Exception:  # o leitor de referência falha em dados fora dos limites
            lines.append(f'{head}\tERROR')
    text = '\n'.join(lines) + '\n'
    if DUMP:
        open(os.path.join(DUMP, name + '.txt'), 'w', encoding='utf-8', newline='\n').write(text)
    flat_digest = hashlib.sha256(text.encode()).hexdigest()
    index.append(f'{name}\tadf\t{len(payload)}\t{digest}\t{len(lines)}\t{flat_digest}')

open(os.path.join(SAVES, '_index.oracle.txt'), 'w', encoding='utf-8', newline='\n').write('\n'.join(index) + '\n')
print(len(index), 'arquivos')
