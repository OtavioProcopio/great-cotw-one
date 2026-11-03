"""Gera app-icon.png (1024x1024): alvo de caça sobre fundo verde. Sem dependências.

Uso: python -I scripts/make_icon.py app-icon.png
Depois: ui/node_modules/.bin/tauri icon app-icon.png   (gera src-tauri/icons)
"""
import math, struct, sys, zlib

SIZE = 1024
BG = (47, 107, 79)
FG = (245, 247, 244)
RADIUS = 220  # cantos arredondados


def inside_rounded_square(x, y):
    cx = min(max(x, RADIUS), SIZE - 1 - RADIUS)
    cy = min(max(y, RADIUS), SIZE - 1 - RADIUS)
    return (x - cx) ** 2 + (y - cy) ** 2 <= RADIUS ** 2


def pixel(x, y):
    if not inside_rounded_square(x, y):
        return (0, 0, 0, 0)
    c = SIZE / 2
    d = math.hypot(x - c, y - c)
    ring = 250 <= d <= 300 or 120 <= d <= 150
    dot = d <= 48
    cross = (abs(x - c) <= 18 or abs(y - c) <= 18) and 90 <= d <= 380 and not (150 < d < 250)
    return (*FG, 255) if (ring or dot or cross) else (*BG, 255)


rows = bytearray()
for y in range(SIZE):
    rows.append(0)
    for x in range(SIZE):
        rows.extend(pixel(x, y))


def chunk(kind, data):
    body = kind + data
    return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body) & 0xFFFFFFFF)


png = b'\x89PNG\r\n\x1a\n'
png += chunk(b'IHDR', struct.pack('>IIBBBBB', SIZE, SIZE, 8, 6, 0, 0, 0))
png += chunk(b'IDAT', zlib.compress(bytes(rows), 9))
png += chunk(b'IEND', b'')
open(sys.argv[1], 'wb').write(png)
print('ok', len(png), 'bytes')
