import struct
def parse(b):
    assert b[:4]==b'RTPC'
    def node(off):
        h,do,pc,cc=struct.unpack_from('<IIHH',b,off)
        props={}
        p=do
        for i in range(pc):
            ph,raw,t=struct.unpack_from('<I4sB',b,p); p+=9
            v=raw
            if t==1: v=struct.unpack('<I',raw)[0]
            elif t==2: v=struct.unpack('<f',raw)[0]
            elif t==3:
                o=struct.unpack('<I',raw)[0]; e=b.index(0,o); v=b[o:e].decode('utf-8','replace')
            elif t in (4,5,6,7,8):
                o=struct.unpack('<I',raw)[0]; n={4:2,5:3,6:4,7:9,8:16}[t]; v=list(struct.unpack_from('<%df'%n,b,o))
            elif t in (9,10):
                o=struct.unpack('<I',raw)[0]; c=struct.unpack_from('<I',b,o)[0]; v=list(struct.unpack_from('<%d%s'%(c,'I' if t==9 else 'f'),b,o+4))
            elif t==11:
                o=struct.unpack('<I',raw)[0]; c=struct.unpack_from('<I',b,o)[0]; v=b[o+4:o+4+c]
            elif t==13:
                o=struct.unpack('<I',raw)[0]; v=struct.unpack_from('<Q',b,o)[0]
            elif t==14:
                o=struct.unpack('<I',raw)[0]; c=struct.unpack_from('<I',b,o)[0]; v=[struct.unpack_from('<II',b,o+4+8*i) for i in range(c)]
            props[ph]=(t,v)
        p=(p+3)&~3
        kids=[node(p+12*i) for i in range(cc)]
        return dict(h=h,props=props,kids=kids)
    return node(8)
