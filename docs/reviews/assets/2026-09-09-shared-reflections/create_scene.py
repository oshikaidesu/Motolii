import ctypes,json,sys,tempfile
from pathlib import Path
from PIL import Image,ImageDraw
repo=Path(__file__).resolve().parents[4]
out=Path(sys.argv[1] if len(sys.argv)>1 else tempfile.mkdtemp(prefix='motolii-reflection-'))
out.mkdir(parents=True,exist_ok=True)
lib=ctypes.CDLL(str(repo/'motolii/target/debug/libmotolii_ui.dylib'))
lib.motolii_probe_open.argtypes=[ctypes.c_char_p];lib.motolii_probe_open.restype=ctypes.c_void_p
lib.motolii_probe_request.argtypes=[ctypes.c_void_p,ctypes.c_char_p];lib.motolii_probe_request.restype=ctypes.c_char_p
c=lib.motolii_probe_open(b'');assert c
state={}
def req(op,**kw):
 s=json.loads(lib.motolii_probe_request(c,json.dumps(dict(op=op,**kw)).encode()))
 assert not s.get('error'),(op,s.get('error'));state.update(s);return s
def prop(id,name,value):req('setProperty',layer=id,property=name,value=value)
def create(kind):
 req('create',kind=kind);return state['selectedIds'][0]
def place(path):
 req('import',paths=[str(path)]);a=next(a for a in state['assets'] if a['name'] in [path.name,path.stem]);req('placeAsset',id=a['id']);return state['selectedIds'][0]
def text(words,x,y,size):
 id=create('text');req('setText',layer=id,content=words);prop(id,'text_style.0.size',size);prop(id,'position',[x-640,y-360]);req('setColor',slot={'TextStroke':{'layer':id,'style':0}},rgba=[0,0,0,0]);return id
req('composition',width=1280,height=720,durationFrames=120,background=[0.015,0.02,0.04,1.0])
im=Image.new('RGBA',(1024,1024),'#13375b');draw=ImageDraw.Draw(im)
for y in range(0,1024,128):
 for x in range(0,1024,128):
  draw.rectangle([x,y,x+127,y+127],fill=['#1167cc','#f1bd43','#bc2858','#00a699'][(x//128+y//128)%4])
  draw.ellipse([x+32,y+32,x+95,y+95],outline='white',width=6)
im.save(out/'offscreen-pattern.png')
p=place(out/'offscreen-pattern.png');prop(p,'position',[-1800,-1800]);prop(p,'scale',[4.0,4.0]);prop(p,'position.z',-1600)
a=create('cube');prop(a,'position',[370,415]);prop(a,'scale',[1.0,1.0]);prop(a,'rotation.x',12);prop(a,'rotation.y',-25)
b=create('rectangle');prop(b,'position',[760,235]);prop(b,'scale',[2.0,2.0])
for id in [a,b]:
 req('select',ids=[id]);req('applyEffect',pluginId='motolii.glass');e=next(l for l in state['layers'] if l['id']==id)['effects'][-1]['id']
 for k,v in [('roughness',0.02),('metallic',1.0),('transmission',0.0)]:prop(id,f'effect.{e}.param.{k}',v)
text('SHARED SCENE REFLECTION',640,90,44)
text('MESH',380,655,26);text('2D SURFACE',940,655,26)
text('The colored pattern is behind the camera.',640,145,23)
req('select',ids=[a]);req('save',path=str(out/'shared-reflection.rrd'))
# An HD mesh Repeater case with the same offscreen source and authored camera.
req('select',ids=[b]);req('delete')
req('select',ids=[a]);req('applyEffect',pluginId='motolii.repeat');e=next(l for l in state['layers'] if l['id']==a)['effects'][-1]['id']
prop(a,'position',[40,210]);prop(a,'scale',[0.04,0.04]);prop(a,'rotation.x',0);prop(a,'rotation.y',0)
for k,v in [('mode',2),('columns',40),('position_each',[30,16])]:prop(a,f'effect.{e}.param.{k}',v)
for count in [10,100,1000]:
 prop(a,f'effect.{e}.param.count',count);req('save',path=str(out/f'repeater-{count}.rrd'))
print('saved',out)
