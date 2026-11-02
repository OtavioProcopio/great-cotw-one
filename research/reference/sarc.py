import struct
def sarc(b):
    assert b[4:8]==b'SARC'
    ds = struct.unpack_from('<I', b, 12)[0]; p = 16; end = 16 + ds; out = []
    while p < end:
        nl = struct.unpack_from('<I', b, p)[0]
        if nl == 0: break
        name = b[p+4:p+4+nl].rstrip(b'\0').decode(); p += 4 + nl
        o, s = struct.unpack_from('<II', b, p); p += 8
        out.append((name, o, s))
    return out
