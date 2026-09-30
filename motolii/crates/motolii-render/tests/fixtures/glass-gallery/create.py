"""Create editable geometry, lighting and a Motolii material-study document."""
import ctypes, json, math, struct
from pathlib import Path
from PIL import Image, ImageDraw
out=Path(__file__).resolve().parent
repo=out.parents[3]
W,H=1600,1000
# Equirectangular studio lights in linear RGBE, not painted object highlights.
w,h=1024,512
with (out/'studio.hdr').open('wb') as f:
 f.write(f'#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {h} +X {w}\n'.encode())
 for y in range(h):
  v=y/h
  for x in range(w):
   u=x/w
   rgb=[.025,.045,.055]
   for ux,vy,sx,sy,color,power in [(.22,.30,.033,.22,(.42,1.,.88),4.0),(.65,.32,.07,.19,(1.,.62,.32),3.0),(.9,.23,.025,.13,(.6,.8,1.),5.),(.46,.66,.13,.055,(.1,.6,.65),1.0)]:
    dx=min(abs(u-ux),1-abs(u-ux));a=math.exp(-((dx/sx)**8+((v-vy)/sy)**8))
    rgb=[c+a*k*power for c,k in zip(rgb,color)]
   m=max(rgb);mant,e=math.frexp(m);sc=math.ldexp(256.,-e)
   f.write(bytes([min(255,int(c*sc)) for c in rgb]+[e+128]))
# Flat composition background.
im=Image.new('RGBA',(W,H)); px=im.load()
for y in range(H):
 for x in range(W):
  glow=math.exp(-(((x-1150)/650)**2+((y-440)/580)**2)*2)
  px[x,y]=(int(7+8*glow),int(15+23*glow),int(21+25*glow),255)
d=ImageDraw.Draw(im)
d.line((95,100,1505,100),fill=(70,98,105,255),width=1)
d.line((95,905,1505,905),fill=(50,72,80,255),width=1)
im.save(out/'background.png')
card=Image.new('RGBA',(512,172));ImageDraw.Draw(card).rounded_rectangle((1,1,510,170),radius=32,fill=(255,255,255,255));card.save(out/'card.png')

def mesh(name,nu,nv,fn):
 verts=[];norms=[]
 for j in range(nv+1):
  for i in range(nu+1):
   p,n=fn(i/nu,j/nv);verts.append(p);norms.append(n)
 with (out/name).open('w') as f:
  for p in verts:f.write('v %s %s %s\n'%p)
  for n in norms:f.write('vn %s %s %s\n'%n)
  for j in range(nv):
   for i in range(nu):
    a=j*(nu+1)+i+1;b=a+1;c=a+nu+1;dd=c+1
    f.write(f'f {a}//{a} {b}//{b} {dd}//{dd}\nf {a}//{a} {dd}//{dd} {c}//{c}\n')
def torus(u,v):
 a=u*math.tau;b=v*math.tau;r=1+.27*math.cos(b)
 return (r*math.cos(a),r*math.sin(a),.27*math.sin(b)),(math.cos(b)*math.cos(a),math.cos(b)*math.sin(a),math.sin(b))
def sphere(u,v):
 a=u*math.tau;b=v*math.pi
 p=(math.sin(b)*math.cos(a),math.cos(b),math.sin(b)*math.sin(a));return p,p
mesh('torus.obj',128,40,torus);mesh('sphere.obj',96,64,sphere)
lib=ctypes.CDLL(str(repo/'motolii/target/debug/libmotolii_ui.dylib'))
lib.motolii_probe_open.argtypes=[ctypes.c_char_p];lib.motolii_probe_open.restype=ctypes.c_void_p
lib.motolii_probe_request.argtypes=[ctypes.c_void_p,ctypes.c_char_p];lib.motolii_probe_request.restype=ctypes.c_char_p
c=lib.motolii_probe_open(b'');assert c
state={}
def req(op,**kw):
 s=json.loads(lib.motolii_probe_request(c,json.dumps(dict(op=op,**kw)).encode()));assert not s.get('error'),(op,s.get('error'));state.update(s);return s
def prop(id,name,value):req('setProperty',layer=id,property=name,value=value)
def place(name):
 path=out/name;req('import',paths=[str(path)]);a=next(a for a in state['assets'] if a['name'] in [path.name,path.stem]);req('placeAsset',id=a['id']);return state['selectedIds'][0]
def glass(id,rough,metal,trans=0,ior=1.5):
 req('select',ids=[id]);req('applyEffect',pluginId='motolii.glass');e=next(l for l in state['layers'] if l['id']==id)['effects'][-1]['id']
 for k,v in [('roughness',rough),('metallic',metal),('transmission',trans),('ior',ior)]:prop(id,f'effect.{e}.param.{k}',v)
def pose(id,x,y,scale,z=0,rx=0,ry=0,rz=0):
 for k,v in [('position',[x,y]),('scale',[scale,scale]),('position.z',z),('rotation.x',rx),('rotation.y',ry),('rotation',rz)]:
  prop(id,k,v)
def text(words,x,y,size,color):
 req('create',kind='text');id=state['selectedIds'][0];req('setText',layer=id,content=words);prop(id,'text_style.0.size',size);prop(id,'position',[x-W/2,y-H/2]);req('setColor',slot={'TextStroke':{'layer':id,'style':0}},rgba=[0,0,0,0]);req('setColor',slot={'TextFill':{'layer':id,'style':0}},rgba=color);return id
req('composition',width=W,height=H,durationFrames=180,background=[.02,.04,.06,1.])
env=place('studio.hdr');req('setAttrs',layers=[env],patch={'environment':True})
bg=place('background.png');prop(bg,'anchor',[W/2,H/2]);pose(bg,W/2,H/2,1+1000/(H/2/math.tan(math.radians(55/2))),1000)
ring=place('torus.obj');prop(ring,'anchor',[1.27,1.27]);pose(ring,1090,460,245,40,28,-24,-18);glass(ring,.065,1.)
pearl=place('sphere.obj');prop(pearl,'anchor',[1,1]);pose(pearl,1320,245,62,-40);glass(pearl,.24,.7)
ball=place('sphere.obj');prop(ball,'anchor',[1,1]);pose(ball,870,625,154,-200);glass(ball,.018,0.,.94,1.45)
slab=place('card.png');prop(slab,'anchor',[256,86]);pose(slab,1195,810,.78,0);glass(slab,.12,0.,.9,1.12)
text('MOTOLII     /     MATERIAL STUDY 001',315,153,16,[.60,.76,.78,1])
text('Light in',310,300,88,[.90,.95,.92,1])
text('form.',270,415,88,[.90,.95,.92,1])
text('Chrome, glass, and a little light.',320,555,22,[.65,.76,.77,1])
text('A real-time surface study.',266,600,18,[.44,.61,.65,1])
text('REFLECTION  /  REFRACTION',310,816,15,[.45,.69,.70,1])
text('SURFACE  /  02',1195,810,18,[.93,.98,.96,1])
text('01  /  LIGHT IN FORM',232,944,14,[.51,.67,.69,1])
text('MOTOLII  —  LIVE RENDER',1351,944,14,[.51,.67,.69,1])
req('select',ids=[]);req('save',path=str(out/'light-in-form.rrd'));print('saved',out)
