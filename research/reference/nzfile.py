import sys,zlib,struct; import nzwrite
def make_builder(D):
    ver,ic,io,tc,to,sc,so,nc,no,tot=struct.unpack_from('<10I',D,4); nh,th,ioff,isz,nm=struct.unpack_from('<IIIIQ',D,io); tail=D[io:]
    pad8 = isz%8==0
    def build(res,pad=None):
        p8=pad8 if pad is None else pad
        payload=nzwrite.build_payload(res)
        if p8: payload+=b'\0'*((-len(payload))%8)
        end=ioff+len(payload); new_io=(end+4+7)//8*8; delta=new_io-io
        body=bytearray(D[:ioff])+payload+b'\0'*(new_io-end)+bytearray(tail)
        struct.pack_into('<I',body,end,12); struct.pack_into('<IIIIQ',body,new_io,nh,th,ioff,len(payload),nm)
        struct.pack_into('<10I',body,4,ver,ic,new_io,tc,to+delta,sc,(so+delta) if so else 0,nc,no+delta,len(body)); return bytes(body)
    return build
