"""Gera o oráculo textual do ADF usando research/reference/adf.py (somente leitura)."""
import sys, zlib, tempfile, os
sys.path.insert(0, sys.argv[1])  # research/reference
import adf

raw = open(sys.argv[2], 'rb').read()
body = zlib.decompress(raw[32:])[5:]
with tempfile.NamedTemporaryFile(delete=False) as f:
    f.write(body)
    tmp = f.name
A = adf.load(tmp)
os.unlink(tmp)

lines = [f"version\t{A['ver']}", f"comment\t{A['comment']}", f"types\t{len(A['types'])}",
         f"string_hashes\t{len(A['sh'])}"]
for h in sorted(A['types']):
    t = A['types'][h]
    lines.append(f"type\t{h:08x}\t{t['meta']}\t{t['name']}\t{t['size']}\t{t['align']}\t{t['flags']}\t{t['eth']:08x}\t{t['elen']}\t{len(t['members'])}")
    for m in t['members']:
        lines.append(f"member\t{m['name']}\t{m['th']:08x}\t{m['size']}\t{m['off']}\t{m['bit']}\t{m['flags']}")
    for v in sorted(t.get('enum', {})):
        lines.append(f"enum\t{v}\t{t['enum'][v]}")
for i in A['insts']:
    lines.append(f"instance\t{i['name']}\t{i['th']:08x}\t{i['off']}\t{i['size']}")
open(sys.argv[3], 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
print(len(lines), 'linhas')
