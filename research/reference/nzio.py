import sys,struct,zlib; import adf, nzwrite
def read(path):
    A=adf.load(path); d=A['d']; i=A['insts'][0]; o=i['off']; inst=d[o:o+i['size']]
    p,x,c=struct.unpack_from('<IIQ',inst,8); res=[]
    for k in range(c):
        rid,zp,_,zn=struct.unpack_from('<QIIQ',inst,p+k*24); zones=[]
        for j in range(zn):
            raw=inst[zp+j*64:zp+j*64+64]; dp,_,dc=struct.unpack_from('<IIQ',raw,48)
            zones.append({'raw':raw,'dg':list(struct.unpack_from('<%dI'%dc,inst,dp)) if dc else []})
        res.append((rid,zones))
    return A,inst,res
