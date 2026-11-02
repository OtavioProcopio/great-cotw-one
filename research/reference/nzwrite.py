import struct, zlib
def align(n,a): return (n+a-1)//a*a
def build_payload(reserves):
    """reserves: list of (ReserveId, [zone dicts with raw 64-byte template + DiscoveredGroups list])"""
    buf=bytearray(); ptrs=[]  # positions of pointer fields (relative to instance)
    def put(pos,data):
        if len(buf)<pos+len(data): buf.extend(b'\0'*(pos+len(data)-len(buf)))
        buf[pos:pos+len(data)]=data
    # root
    put(0,b'\x01'+b'\0'*23)
    n=len(reserves)
    arr_off=24
    put(8,struct.pack('<IIQ',arr_off if n else 0,0,n)); 
    if n: ptrs.append(8)
    cur=arr_off+n*24
    for i,(rid,zones) in enumerate(reserves):
        e=arr_off+i*24
        cur=align(cur,8)
        zoff=cur if zones else 0
        put(e,struct.pack('<QIIQ',rid,zoff,0,len(zones)))
        if zones: ptrs.append(e+8)
        cur=zoff+len(zones)*64 if zones else cur
        # zones
        for k,z in enumerate(zones):
            zp=zoff+k*64
            put(zp,z['raw'][:48]+b'\0'*16)
        # DG data
        for k,z in enumerate(zones):
            zp=zoff+k*64; dg=z['dg']
            if dg:
                cur=align(cur,8)
                put(zp+48,struct.pack('<IIQ',cur,0,len(dg))); ptrs.append(zp+48)
                put(cur,struct.pack('<%dI'%len(dg),*dg)); cur+=4*len(dg)
            else:
                put(zp+48,struct.pack('<IIQ',0,0,0))
    # chain: x = distance to next pointer, keep in sorted order
    ptrs.sort()
    for a,b in zip(ptrs,ptrs[1:]): struct.pack_into('<I',buf,a+4,b-a)
    return bytes(buf)
