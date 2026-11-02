import struct, zlib
def aaf(b):
    n = struct.unpack_from('<I', b, 44)[0]; p = 48; out = []
    for _ in range(n):
        cs, us, chs, mg = struct.unpack_from('<IIII', b, p)
        out.append(zlib.decompress(b[p+16:p+16+cs], -15) if cs != us else b[p+16:p+16+cs])
        p += chs
    return b''.join(out)
