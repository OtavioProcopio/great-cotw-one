import struct,zlib,sys
def align(n,a): return (n+a-1)//a*a
def read(D,I):
    wp,_,wc=struct.unpack_from('<IIQ',D,I+8); rp,_,rc=struct.unpack_from('<IIQ',D,I+24)
    items=[]
    for k in range(wc):
        e=I+wp+k*112; rawi=D[e:e+112]; arrs=[]
        for fo,es in ((48,4),(64,16),(80,16)):
            p,_,c=struct.unpack_from('<IIQ',D,e+fo); arrs.append(D[I+p:I+p+c*es] if c else b'')
        items.append({'raw':rawi,'arrs':arrs})
    ret=D[I+rp:I+rp+rc*4] if rc else b''
    return D[I:I+8],items,ret
def payload(head,items,ret,arr_align=(8,16,16)):
    buf=bytearray(head)+b'\0'*32; ptrs=[]
    n=len(items); wp=40
    struct.pack_into('<IIQ',buf,8,wp if n else 0,0,n)
    if n: ptrs.append(8)
    buf+=b'\0'*(n*112)
    for k,it in enumerate(items): buf[wp+k*112:wp+k*112+112]=it['raw']
    cur=len(buf)
    for k,it in enumerate(items):
        e=wp+k*112
        for j,(fo,es) in enumerate(((48,4),(64,16),(80,16))):
            data=it['arrs'][j]
            if data:
                cur=align(cur,arr_align[j]); buf+=b'\0'*(cur-len(buf)); buf+=data
                struct.pack_into('<IIQ',buf,e+fo,cur,0,len(data)//es); ptrs.append(e+fo); cur=len(buf)
            else: struct.pack_into('<IIQ',buf,e+fo,0,0,0)
    if ret:
        cur=align(cur,8); buf+=b'\0'*(cur-len(buf)); buf+=ret; struct.pack_into('<IIQ',buf,24,cur,0,len(ret)//4); ptrs.append(24)
    else: struct.pack_into('<IIQ',buf,24,0,0,0)
    ptrs.sort()
    for a,b in zip(ptrs,ptrs[1:]): struct.pack_into('<I',buf,a+4,b-a)
    return bytes(buf)
def builder(D):
    ver,ic,io,tc,to,sc,so,nc,no,tot=struct.unpack_from('<10I',D,4); nh,th,ioff,isz,nm=struct.unpack_from('<IIIIQ',D,io); tail=D[io:]
    pad8=isz%8==0
    def build(head,items,ret):
        p=payload(head,items,ret)
        if pad8: p+=b'\0'*((-len(p))%8)
        end=ioff+len(p); new_io=(end+4+7)//8*8; delta=new_io-io
        body=bytearray(D[:ioff])+p+b'\0'*(new_io-end)+bytearray(tail)
        struct.pack_into('<I',body,end,12); struct.pack_into('<IIIIQ',body,new_io,nh,th,ioff,len(p),nm)
        struct.pack_into('<10I',body,4,ver,ic,new_io,tc,to+delta,sc,(so+delta) if so else 0,nc,no+delta,len(body)); return bytes(body)
    return build,ioff
