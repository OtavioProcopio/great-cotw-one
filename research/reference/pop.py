"""Leitor/editor do animal_population_N (theHunter: Call of the Wild).
Formato (mesma lógica documentada pelo APC): cabeçalho de 32 bytes + zlib;
o conteúdo descomprimido tem 5 bytes de cabeçalho + um ADF.
Cada animal ocupa 32 bytes: Gender u8@0, Weight f32@4, Score f32@8, FeatureModifiers u8@12 (bit0 = Great One),
IsScripted u8@13, VisualVariationSeed u32@16, Id u32@20, MapPosition vec2@24 (sempre 0).
ATENÇÃO: add_animals NÃO reconstrói a cadeia de relocação do ADF (campo +4 de cada ponteiro).
Use a versão de core.js (build) ou corrija antes de usar: foi isso que travou o jogo.
"""
import struct, zlib, re

U32 = lambda b, o: struct.unpack_from('<I', b, o)[0]
U64 = lambda b, o: struct.unpack_from('<Q', b, o)[0]
PRIM = {1477249634: 1, 211976733: 1, 3510620051: 2, 2261865149: 2, 422569523: 4, 123620943: 4,
        2940286287: 8, 2704924703: 8, 1964352007: 4, 3322541667: 8}
ANIMAL = struct.Struct('<IffIIIff')


class Pop:
    def __init__(self, raw: bytes):
        self.file_header = bytearray(raw[:32])
        d = zlib.decompress(raw[32:])
        self.pre = bytearray(d[:5])
        self.adf = bytearray(d[5:])
        self._profile()

    # ---------- ADF ----------
    def _profile(self):
        a = self.adf
        self.inst_count, self.inst_off = U32(a, 8), U32(a, 12)
        self.td_count, self.td_off = U32(a, 16), U32(a, 20)
        self.nt_count, self.nt_off = U32(a, 32), U32(a, 36)
        self.total = U32(a, 40)
        lens = [a[self.nt_off + i] for i in range(self.nt_count)]
        p = self.nt_off + self.nt_count
        self.names = []
        for L in lens:
            self.names.append(a[p:p + L].decode()); p += L + 1
        self.types = {}
        p = self.td_off
        for _ in range(self.td_count):
            meta, size, th = U32(a, p), U32(a, p + 4), U32(a, p + 12)
            name = self.names[U64(a, p + 16)]; elem = U32(a, p + 28)
            if meta == 1:
                n = U32(a, p + 36); mem = []
                for i in range(n):
                    q = p + 40 + 32 * i
                    mem.append((self.names[U64(a, q)], U32(a, q + 8), U32(a, q + 16)))
                self.types[th] = dict(name=name, meta=1, size=size, members=mem); p += 40 + 32 * n
            elif meta == 0:
                self.types[th] = dict(name=name, meta=0); p += 36
            else:
                self.types[th] = dict(name=name, meta=meta, elem=elem); p += 40
        # instância 0
        h = self.inst_off
        self.inst_type = U32(a, h + 4); self.inst_data = U32(a, h + 8); self.inst_size = U32(a, h + 12)
        self.arrays = []   # (caminho, header_off_abs, data_start_abs, length)
        self._walk(self.inst_type, self.inst_data, '')
        self.animal_arrays = []
        for path, hdr, start, length in self.arrays:
            if path.endswith('Animals'):
                pop, grp = map(int, re.findall(r'\[(\d+)\]', path)[:2])
                self.animal_arrays.append(dict(pop=pop, grp=grp, hdr=hdr, start=start, length=length))

    def _walk(self, th, pos, path):
        if th in PRIM:
            return
        t = self.types[th]
        if t['meta'] == 1:
            for name, mth, moff in t['members']:
                self._walk(mth, pos + moff, f'{path}.{name}')
        elif t['meta'] == 3:
            a = self.adf
            rel, length = U32(a, pos), U32(a, pos + 8)
            start = self.inst_data + rel
            self.arrays.append((path, pos, start, length))
            elem = t['elem']
            if length and elem not in PRIM:
                es = self.types[elem]['size']
                for i in range(length):
                    self._walk(elem, start + i * es, f'{path}[{i}]')

    # ---------- animais ----------
    def animals(self):
        for arr in self.animal_arrays:
            for i in range(arr['length']):
                off = arr['start'] + 32 * i
                g, w, s, go, seed, aid, x, y = ANIMAL.unpack_from(self.adf, off)
                yield dict(pop=arr['pop'], grp=arr['grp'], idx=i, off=off, gender=g, weight=w, score=s,
                           go=go, seed=seed, id=aid, x=x, y=y)

    def set_animal(self, off, **kw):
        g, w, s, go, seed, aid, x, y = ANIMAL.unpack_from(self.adf, off)
        v = dict(gender=g, weight=w, score=s, go=go, seed=seed, id=aid, x=x, y=y); v.update(kw)
        ANIMAL.pack_into(self.adf, off, v['gender'], v['weight'], v['score'], v['go'], v['seed'], v['id'], v['x'], v['y'])

    def add_animals(self, additions):
        """additions: lista de (índice do array em animal_arrays, bytes_do_animal).
        Insere no fim de cada array e corrige todos os offsets que vêm depois."""
        a = self.adf
        per = {}
        for ai, blob in additions:
            per.setdefault(ai, []).append(blob)
        # insere de trás para frente para que offsets anteriores não mudem
        order = sorted(per, key=lambda ai: self.animal_arrays[ai]['start'] + 32 * self.animal_arrays[ai]['length'], reverse=True)
        inserted_at = []
        for ai in order:
            arr = self.animal_arrays[ai]
            blob = b''.join(per[ai])
            end = arr['start'] + 32 * arr['length']
            a[end:end] = blob
            inserted_at.append((end, len(blob), ai))
        total_added = sum(n for _, n, _ in inserted_at)

        def shift(abs_pos):
            # quantos bytes foram inseridos antes de abs_pos (estritamente antes)
            return sum(n for e, n, _ in inserted_at if e <= abs_pos)

        def shift_le(abs_pos):
            return sum(n for e, n, _ in inserted_at if e <= abs_pos)
        # cabeçalhos de array estão dentro da instância; recalcula cada um
        for path, hdr, start, length in self.arrays:
            new_hdr = hdr + shift(hdr)
            new_start = start + shift(start)
            if U32(a, new_hdr) != 0 or length:
                a[new_hdr:new_hdr + 4] = struct.pack('<I', new_start - self.inst_data)
        for ai, (end, n, _) in zip([x[2] for x in inserted_at], inserted_at):
            arr = self.animal_arrays[ai]
            new_hdr = arr['hdr'] + shift(arr['hdr'])
            a[new_hdr + 8:new_hdr + 12] = struct.pack('<I', arr['length'] + n // 32)
        # cabeçalho do ADF e da instância
        for field in (12, 20, 28, 36, 40):
            v = U32(a, field)
            if v > self.inst_data:
                a[field:field + 4] = struct.pack('<I', v + total_added)
        h = U32(a, 12)
        a[h + 12:h + 16] = struct.pack('<I', self.inst_size + total_added)
        self._profile()

    def save(self) -> bytes:
        d = bytes(self.pre) + bytes(self.adf)
        fh = bytearray(self.file_header)
        fh[8:12] = struct.pack('<I', len(d)); fh[24:28] = struct.pack('<I', len(d))
        return bytes(fh) + zlib.compress(d)
