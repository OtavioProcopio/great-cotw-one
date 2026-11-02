import struct, sys, json
PRIM = {0x580D0A62:'s8',0x0CA2821D:'u8',0xD13FCF93:'s16',0x86D152BD:'u16',0x192FE633:'s32',
        0x075E4E4F:'u32',0xAF41354F:'s64',0xA139E01F:'u64',0x7515A207:'f32',0xC609F663:'f64',
        0x8955583E:'String'}
class R:
    def __init__(s,d,p=0): s.d=d; s.p=p
    def u(s,fmt):
        v=struct.unpack_from('<'+fmt,s.d,s.p); s.p+=struct.calcsize('<'+fmt); return v if len(v)>1 else v[0]
def load(path):
    raw=open(path,'rb').read()
    i=raw.find(b' FDA'); d=raw[i:]
    r=R(d,4)
    ver,ic,io,tc,to,sc,so,nc,no,tot=r.u('10I')
    # names
    lens=d[no:no+nc]; p=no+nc; names=[]
    for L in lens: names.append(d[p:p+L].decode('latin1')); p+=L+1
    comment=d[0x40:d.index(b'\0',0x40)].decode('latin1')
    # string hashes
    sh={}
    r=R(d,so)
    if sc:
        strs=[]
        for _ in range(sc):
            e=d.index(b'\0',r.p); strs.append(d[r.p:e].decode('latin1')); r.p=e+1
        for s_ in strs: h=r.u('Q'); sh[h]=s_
    types={}
    r=R(d,to)
    for _ in range(tc):
        mt,size,al,th,nm,fl,eth,el=r.u('IIIIQIII')
        t=dict(meta=mt,size=size,align=al,hash=th,name=names[nm],flags=fl,eth=eth,elen=el,members=[])
        cnt=r.u('I')
        if mt==1:
            for _ in range(cnt):
                mn,mth,msz,off,mfl,dv=r.u('QIIIIQ')
                t['members'].append(dict(name=names[mn],th=mth,size=msz,off=off&0xffffff,bit=off>>24,flags=mfl))
        elif mt==8:
            t['enum']={}
            for _ in range(cnt):
                en,ev=r.u('QI'); t['enum'][ev]=names[en]
        else:
            r.p+=cnt*12 if mt==7 else 0
        types[th]=t
    insts=[]
    r=R(d,io)
    for _ in range(ic):
        nh,th,off,sz,nm=r.u('IIIIQ'); insts.append(dict(name=names[nm],th=th,off=off,size=sz))
    return dict(d=d,ver=ver,names=names,types=types,insts=insts,comment=comment,sh=sh)
def read_val(A,th,off,size=None):
    d=A['d']; types=A['types']; B=A.get('base',0)
    if th in PRIM:
        k=PRIM[th]
        if k=='String':
            p=struct.unpack_from('<I',d,off)[0]
            return d[p+B:d.index(b'\0',p+B)].decode('utf-8','replace') if p else ''
        f={'s8':'b','u8':'B','s16':'h','u16':'H','s32':'i','u32':'I','s64':'q','u64':'Q','f32':'f','f64':'d'}[k]
        v=struct.unpack_from('<'+f,d,off)[0]
        return round(v,4) if k.startswith('f') else v
    t=types.get(th)
    if t is None:
        # unknown primitive: guess by size
        sz=size or 4
        return {'?hash':hex(th),'raw':d[off:off+sz].hex()}
    m=t['meta']
    if m==0:
        return {'prim?':t['name'],'raw':d[off:off+t['size']].hex()}
    if m==1:
        o={}
        for mb in t['members']:
            if mb['bit'] and False: pass
            o[mb['name']]=read_val(A,mb['th'],off+mb['off'],mb['size'])
        return o
    if m==3:  # array: u64 offset, u32 count
        p,_x,c=struct.unpack_from('<IIQ',d,off); p+=B
        et=A['types'].get(t['eth']); es=(et['size'] if et else {'s8':1,'u8':1,'s16':2,'u16':2,'s64':8,'u64':8,'f64':8,'String':8}.get(PRIM.get(t['eth']),4))
        if t['eth'] in (0x0CA2821D,0x580D0A62):
            b=d[p:p+c]
            if c and all(32<=x<127 for x in b.rstrip(b'\0')): return b.rstrip(b'\0').decode()
            return {'bytes':c,'hex':b[:48].hex()+('...' if c>48 else '')}
        return [read_val(A,t['eth'],p+i*es) for i in range(c)]
    if m==4 and t['eth']==0x0CA2821D:
        b=d[off:off+t['elen']].rstrip(b'\0')
        try: return b.decode('ascii') if all(32<=x<127 for x in b) else b.hex()
        except: return b.hex()
    if m==4:  # inline array
        et=A['types'].get(t['eth']); es=(et['size'] if et else {'s8':1,'u8':1,'s16':2,'u16':2,'s64':8,'u64':8,'f64':8}.get(PRIM.get(t['eth']),4))
        return [read_val(A,t['eth'],off+i*es) for i in range(t['elen'])]
    if m==8:
        v=struct.unpack_from('<I',d,off)[0]; return t['enum'].get(v,v)
    if m==9:
        h=struct.unpack_from('<Q',d,off)[0]; return A['sh'].get(h,hex(h))
    if m==5 or m==2:
        p=struct.unpack_from('<Q',d,off)[0]; return {'ptr':p}
    return {'meta':m,'raw':d[off:off+t['size']].hex()}
def dump(path):
    A=load(path)
    out={}
    for i in A['insts']:
        A['base']=i['off']; out[i['name']]=read_val(A,i['th'],i['off'])
    return out, A
if __name__=='__main__':
    A=load(sys.argv[1])
    print('comment:',A['comment'],'| types:',len(A['types']),'| insts:',[i['name'] for i in A['insts']])
    if '-t' in sys.argv:
        for t in A['types'].values():
            print(f"  [{t['meta']}] {t['name']} size={t['size']} hash={t['hash']:08x} eth={t['eth']:08x}")
            for m in t['members']: print(f"      +{m['off']:<5} {m['name']} th={m['th']:08x} sz={m['size']}")
    else:
        v,_=dump(sys.argv[1]); print(json.dumps(v,indent=1,ensure_ascii=False,default=str)[:int(sys.argv[2]) if len(sys.argv)>2 else 6000])
