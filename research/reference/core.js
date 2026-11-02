// ===== núcleo do editor: lê e grava animal_population_N (porta de game/pop.py) =====
const EDC=(()=>{
const PRIM=new Set([1477249634,211976733,3510620051,2261865149,422569523,123620943,2940286287,2704924703,1964352007,3322541667]);
async function pipe(bytes,stream){const s=new Blob([bytes]).stream().pipeThrough(stream);return new Uint8Array(await new Response(s).arrayBuffer())}
const inflate=b=>pipe(b,new DecompressionStream('deflate'));
const deflate=b=>pipe(b,new CompressionStream('deflate'));
function profile(a){
  const dv=new DataView(a.buffer,a.byteOffset,a.byteLength),U=o=>dv.getUint32(o,true);
  const P={inst_off:U(12),td_count:U(16),td_off:U(20),nt_count:U(32),nt_off:U(36)};
  const lens=[];for(let i=0;i<P.nt_count;i++)lens.push(a[P.nt_off+i]);
  let p=P.nt_off+P.nt_count;const names=[],td=new TextDecoder();
  for(const L of lens){names.push(td.decode(a.subarray(p,p+L)));p+=L+1}
  const types={};p=P.td_off;
  for(let k=0;k<P.td_count;k++){const meta=U(p),size=U(p+4),th=U(p+12),name=names[U(p+16)],elem=U(p+28);
    if(meta===1){const n=U(p+36),mem=[];for(let i=0;i<n;i++){const q=p+40+32*i;mem.push([names[U(q)],U(q+8),U(q+16)])}
      types[th]={name,meta,size,mem};p+=40+32*n}
    else if(meta===0){types[th]={name,meta};p+=36}
    else{types[th]={name,meta,elem};p+=40}}
  const h=P.inst_off;P.inst_type=U(h+4);P.inst_data=U(h+8);P.inst_size=U(h+12);
  const arrays=[];
  (function walk(th,pos,path){if(PRIM.has(th))return;const t=types[th];
    if(t.meta===1){for(const [n,mth,mo] of t.mem)walk(mth,pos+mo,path+'.'+n)}
    else if(t.meta===3){const rel=U(pos),len=U(pos+8),start=P.inst_data+rel;arrays.push({path,hdr:pos,start,len});
      if(len&&!PRIM.has(t.elem)){const es=types[t.elem].size;for(let i=0;i<len;i++)walk(t.elem,start+i*es,path+'['+i+']')}}
  })(P.inst_type,P.inst_data,'');
  P.arrays=arrays;P.types=types;
  P.pops=[];P.groups=[];
  for(const x of arrays){let m=x.path.match(/^\.Populations\[(\d+)\]\.Groups$/);if(m)P.pops[+m[1]]={hash:U(x.hdr-8),groups:x.len};
    m=x.path.match(/^\.Populations\[(\d+)\]\.Groups\[(\d+)\]\.Animals$/);if(m)P.groups.push({pop:+m[1],gi:+m[2],hdr:x.hdr,start:x.start,len:x.len})}
  return P}
// arquivo -> modelo
async function load(raw){
  raw=new Uint8Array(raw);if(raw.length<40)throw new Error('Arquivo pequeno demais');
  const d=await inflate(raw.subarray(32));const adf=d.slice(5);
  if(String.fromCharCode(adf[0],adf[1],adf[2],adf[3])!==' FDA')throw new Error('Não é um arquivo ADF do jogo');
  const P=profile(adf),dv=new DataView(adf.buffer);
  const animals=[];let maxId=0;
  for(const g of P.groups)for(let i=0;i<g.len;i++){const o=g.start+32*i;
    const a={pop:g.pop,gi:g.gi,o,g:adf[o],w:dv.getFloat32(o+4,true),s:dv.getFloat32(o+8,true),f:adf[o+12],seed:dv.getUint32(o+16,true),id:dv.getUint32(o+20,true)};
    a.orig={g:a.g,w:a.w,s:a.s,f:a.f,seed:a.seed};animals.push(a);if(a.id>maxId)maxId=a.id}
  return {header:raw.slice(0,32),pre:d.slice(0,5),adf,P,animals,maxId}}
const changed=a=>a.add||a.g!==a.orig.g||a.w!==a.orig.w||a.s!==a.orig.s||a.f!==a.orig.f||a.seed!==a.orig.seed;
function put(buf,o,a){const dv=new DataView(buf.buffer,buf.byteOffset,buf.byteLength);buf[o]=a.g;dv.setFloat32(o+4,a.w,true);dv.setFloat32(o+8,a.s,true);buf[o+12]=a.f;dv.setUint32(o+16,a.seed,true)}
// modelo -> bytes do arquivo (mesma lógica de Pop.add_animals + Pop.save)
async function build(M){
  const P=M.P,src=M.adf;const a=src.slice();
  for(const x of M.animals)if(!x.add&&changed(x))put(a,x.o,x);
  // novos animais: clonados de um animal do mesmo grupo, Id novo
  const per=new Map();let nid=M.maxId;
  for(const x of M.animals)if(x.add){const g=P.groups.find(g=>g.pop===x.pop&&g.gi===x.gi);
    const b=a.slice(x.tpl,x.tpl+32);put(b,0,x);if(M.maxId>0)new DataView(b.buffer).setUint32(20,x.newId||(x.newId=++nid),true);
    if(!per.has(g))per.set(g,[]);per.get(g).push(b)}
  if(!per.size)return finish(M,a);
  const ins=[...per].map(([g,bs])=>({g,end:g.start+32*g.len,n:32*bs.length,bs})).sort((p,q)=>p.end-q.end);
  const total=ins.reduce((s,x)=>s+x.n,0);
  const out=new Uint8Array(a.length+total);let r=0,w=0;
  for(const x of ins){out.set(a.subarray(r,x.end),w);w+=x.end-r;r=x.end;for(const b of x.bs){out.set(b,w);w+=32}}
  out.set(a.subarray(r),w);
  const dv=new DataView(out.buffer),shift=pos=>ins.reduce((s,x)=>s+(x.end<=pos?x.n:0),0);
  for(const x of P.arrays){const nh=x.hdr+shift(x.hdr),ns=x.start+shift(x.start);
    if(dv.getUint32(nh,true)!==0||x.len)dv.setUint32(nh,ns-P.inst_data,true)}
  for(const x of ins){const nh=x.g.hdr+shift(x.g.hdr);dv.setUint32(nh+8,x.g.len+x.n/32,true)}
  // cadeia de relocação do ADF: o campo +4 de cada ponteiro guarda a distância até o próximo ponteiro
  const ptrs=P.arrays.filter(x=>x.len||new DataView(src.buffer,src.byteOffset).getUint32(x.hdr,true)!==0).map(x=>x.hdr+shift(x.hdr)).sort((p,q)=>p-q);
  for(let i=0;i+1<ptrs.length;i++)dv.setUint32(ptrs[i]+4,ptrs[i+1]-ptrs[i],true);
  for(const f of [12,20,28,36,40]){const v=dv.getUint32(f,true);if(v>P.inst_data)dv.setUint32(f,v+total,true)}
  const h=dv.getUint32(12,true);dv.setUint32(h+12,P.inst_size+total,true);
  return finish(M,out)}
async function finish(M,adf){
  const d=new Uint8Array(5+adf.length);d.set(M.pre);d.set(adf,5);
  const z=await deflate(d);const fh=M.header.slice();const dv=new DataView(fh.buffer);
  dv.setUint32(8,d.length,true);dv.setUint32(24,d.length,true);
  const out=new Uint8Array(32+z.length);out.set(fh);out.set(z,32);return out}
// zip simples (sem compressão) com um arquivo
const CRC=(()=>{const t=new Uint32Array(256);for(let n=0;n<256;n++){let c=n;for(let k=0;k<8;k++)c=c&1?0xedb88320^(c>>>1):c>>>1;t[n]=c>>>0}return t})();
function crc32(b){let c=~0;for(let i=0;i<b.length;i++)c=CRC[(c^b[i])&255]^(c>>>8);return ~c>>>0}
function zip(name,data){const nm=new TextEncoder().encode(name),c=crc32(data),N=nm.length,S=data.length;
  const lh=new Uint8Array(30+N),cd=new Uint8Array(46+N),ed=new Uint8Array(22);const L=new DataView(lh.buffer),C=new DataView(cd.buffer),E=new DataView(ed.buffer);
  L.setUint32(0,0x04034b50,true);L.setUint16(4,20,true);L.setUint32(14,c,true);L.setUint32(18,S,true);L.setUint32(22,S,true);L.setUint16(26,N,true);lh.set(nm,30);
  C.setUint32(0,0x02014b50,true);C.setUint16(4,20,true);C.setUint16(6,20,true);C.setUint32(16,c,true);C.setUint32(20,S,true);C.setUint32(24,S,true);C.setUint16(28,N,true);cd.set(nm,46);
  const off=lh.length+S;E.setUint32(0,0x06054b50,true);E.setUint16(8,1,true);E.setUint16(10,1,true);E.setUint32(12,cd.length,true);E.setUint32(16,off,true);
  const out=new Uint8Array(off+cd.length+22);out.set(lh);out.set(data,lh.length);out.set(cd,off);out.set(ed,off+cd.length);return out}
return {load,build,changed,zip,profile,inflate,deflate}})();
if(typeof module!=='undefined')module.exports=EDC;
