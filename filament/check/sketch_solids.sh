#!/bin/zsh
# A p5.js sketch, its file as it is, with no browser: its box() a real solid in the world (p5_stand.h). As tall as the
# sketch says, going where the sketch goes as its frames run, and the same again on going back (a sketch only goes
# forward: a frame once run is kept). Each copy is the colour the sketch fills it with.
#   ./check/sketch_solids.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
cat > $W/sketch.js <<'J'
let x = -100;
function setup() { createCanvas(400, 400, WEBGL); }
function draw() { translate(x, 0, 0); box(40, 200, 40); x += 2; }
J
cat > $W/colours.js <<'J'
function setup() { createCanvas(400, 400, WEBGL); noStroke(); }
function draw() { fill(230, 20, 20); translate(-80, 0, 0); box(60); fill(20, 60, 230); translate(160, 0, 0); box(60); }
J
cat > $W/mesh.js <<'J'
function setup() { createCanvas(600, 400, WEBGL) }
function draw() {
  background(20, 30, 60)
  noStroke()
  fill(230, 60, 40)
  ellipse(-200, 0, 150, 150)
  beginShape(TRIANGLE_STRIP)
  for (let i = 0; i < 6; i++) { fill(20, 220, 60); vertex(-50 + i * 20, -60 + frameCount, 0); vertex(-50 + i * 20, 60 + frameCount, 0) }
  endShape()
  stroke(255, 255, 0)
  strokeWeight(6)
  line(150, -150, 280, 150)
}
J
cat > $W/pic.js <<'J'
let pg, sound
function preload() { sound = loadSound('none.mp3') }
function setup() {
  createCanvas(600, 400, WEBGL)
  document.getElementById('panel').style.display = 'none'
  textureMode(NORMAL)
  textureWrap(REPEAT)
  pg = createGraphics(64, 64)
  pg.noStroke()
}
function draw() {
  background(10, 20, 40)
  pg.background(255, 200, 0)
  pg.fill(200, 0, 100)
  pg.rect(0, 0, 32, 32 + frameCount)
  noStroke()
  texture(pg)
  beginShape(TRIANGLE_STRIP)
  vertex(-260, -180, 0, 0, 0)
  vertex(-260, 180, 0, 0, 2)
  vertex(260, -180, 0, 2, 0)
  vertex(260, 180, 0, 2, 2)
  endShape()
}
J
cat > $W/words.js <<'J'
function setup() { createCanvas(600, 400, WEBGL) }
function draw() {
  background(250)
  fill(200, 0, 60)
  textSize(80)
  textAlign(CENTER, CENTER)
  text('Motolii', 0, 0)
  stroke(0, 200, 0)
  strokeWeight(8)
  line(-300, 170, 300, 170)
}
J
cat > $W/light.js <<'J'
function setup() { createCanvas(600, 400, WEBGL); noStroke() }
function draw() {
  background(0)
  ambientLight(0, 0, 128)
  directionalLight(255, 0, 0, 0, 0, -1)
  fill(255)
  sphere(100)
}
J
cat > $W/paint.js <<'J'
const V = `attribute vec3 aPosition; attribute vec3 aNormal; attribute vec2 aTexCoord;
uniform mat4 uModelViewMatrix; uniform mat4 uProjectionMatrix; uniform mat3 uNormalMatrix;
varying vec3 vNormal; varying vec2 vTexCoord;
void main() { vTexCoord = aTexCoord; vNormal = uNormalMatrix * aNormal; gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aPosition, 1.0); }`
const F = `precision highp float;
varying vec3 vNormal; varying vec2 vTexCoord; uniform vec3 uColor;
const float PI = 3.14159265;
float stripes(vec2 p) { return step(0.5, fract(p.x * 3.0)); }
void main() { gl_FragColor = vec4(mix(uColor, vec3(1.0, 1.0, 0.0), stripes(vTexCoord)), 1.0); }`
let sh
function setup() { createCanvas(600, 400, WEBGL); noStroke(); sh = createShader(V, F) }
function draw() {
  background(30)
  shader(sh)
  sh.setUniform('uColor', [1, 0, 0])
  push(); translate(-160, 0, 0); box(150); pop()
  sh.setUniform('uColor', [0, 0, 1])
  push(); translate(160, 0, 0); sphere(90); pop()
}
J
cat > $W/wide.js <<'J'
function setup() { createCanvas(600, 400, WEBGL); noStroke() }
function draw() {
  background(0)
  ortho(-width, width, -height, height, -10000, 10000)
  camera(0, 0, 1000, 0, 0, 0, 0, 1, 0)
  fill(0, 255, 0)
  push(); translate(0, 0, -300); box(200); pop()
}
J
cat > $W/deep.js <<'J'
function setup() { createCanvas(600, 400, WEBGL) }
function draw() {
  background(0)
  noStroke()
  fill(0, 255, 0)
  push(); translate(0, 0, -1500); box(300); pop()
  stroke(255, 0, 0)
  line(-280, 180, 280, 180)
}
J
python3 - $W <<'P'
import os,subprocess,sys
from PIL import Image,ImageChops
W=sys.argv[1]
said=['add solid cube',f'say 1 script {W}/sketch.js','add light sun','add camera camera','set 3 z 9',
      'at 0',f'still {W}/a.png','at 1',f'still {W}/b.png','at 0',f'still {W}/c.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/s.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'1920x1080'})
print('a sketch\'s file is taken as it is:','yes' if 'NOT done' not in r.stdout+r.stderr else 'NO '+(r.stdout+r.stderr)[-300:])
said=['add solid cube',f'say 1 script {W}/mesh.js','list','at 0',f'still {W}/m0.png','at 1',f'still {W}/m1.png','at 0',f'still {W}/m2.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/m.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
g=lambda n:Image.open(f'{W}/{n}.png').convert('RGB').resize((96,54))
count=lambda n,f:sum(1 for p in g(n).getdata() if f(*p))
line=sum(1 for p in Image.open(f'{W}/m0.png').convert('RGB').getdata() if p[0]>200 and p[1]>200 and p[2]<100)
print('what it draws is its own shapes, one mesh:','yes' if '1 shape at 0.0: sketch' in r.stdout and count('m0',lambda r,gr,b:r>150 and gr<110)>30 and count('m0',lambda r,gr,b:gr>150 and r<90)>30 and line>500 else f"NO {count('m0',lambda r,gr,b:r>150 and gr<110)} {count('m0',lambda r,gr,b:gr>150 and r<90)} {line}")
print('  what it clears with is behind it:','yes' if count('m0',lambda r,gr,b:b>r+15 and b<90)>1000 else 'NO')
print('  it moves as its frames run, and going back is the same:','yes' if ImageChops.difference(g('m0'),g('m1')).getbbox() and not ImageChops.difference(g('m0'),g('m2')).getbbox() else 'NO')
said=['add solid cube',f'say 1 script {W}/pic.js','list','at 0',f'still {W}/p0.png','at 0.3',f'still {W}/p1.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/p.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
near=lambda p,k:all(abs(p[i]-k[i])<3 for i in range(3))
yellow,red=count('p0',lambda r,gr,b:near((r,gr,b),(255,200,0))),count('p0',lambda r,gr,b:near((r,gr,b),(200,0,100)))
print('what it paints on a createGraphics is on what it draws, its wrap as it says:','yes' if yellow>300 and red>300 else f'NO {yellow} {red}')
print('  that picture goes on with its frames:','yes' if ImageChops.difference(g('p0'),g('p1')).getbbox() else 'NO')
said=['add solid cube',f'say 1 script {W}/words.js','at 0',f'still {W}/w0.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/w.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
red=[x for x,p in enumerate(Image.open(f'{W}/w0.png').convert('RGB').crop((0,200,960,340)).getdata()) if p[0]>150 and p[1]<80]
print('words stand in WEBGL, in its fill, where it aligns them:','yes' if len(red)>2000 and abs(sum(x%960 for x in red)/len(red)-480)<25 else f'NO {len(red)}')
said=['add solid cube',f'say 1 script {W}/light.js','add light sun','at 0',f'still {W}/l0.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/l.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
p=Image.open(f'{W}/l0.png').convert('RGB').getpixel((480,270))
print('what it lights it lights by its own lights, by p5\'s own sums (0.73 of its red, all of its blue):','yes' if near(p,(186,0,128)) else f'NO {p}')
said=[f'later door 0 {W}/paint.js','at 0',f'still {W}/s0.png']
subprocess.run(['zsh','./edit.sh',f'{W}/z.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'MOTOLII_OWN':W,'SIZE':'960x540'})
h=lambda a,f:sum(1 for c in Image.open(f'{W}/s0.png').convert('RGB').crop(a).getdata() if f(*c))
red,blue,yellow=h((0,0,480,540),lambda r,gr,b:r>200 and gr<60),h((480,0,960,540),lambda r,gr,b:b>200 and r<60),h((0,0,960,540),lambda r,gr,b:r>200 and gr>200 and b<60)
print('its own shader runs on what it draws, each told its own:','yes' if min(red,blue,yellow)>1000 and not h((0,0,960,540),lambda r,gr,b:min(r,gr,b)>200) else f'NO {red} {blue} {yellow}')
said=['add solid cube',f'say 1 script {W}/wide.js','at 0',f'still {W}/o0.png']
subprocess.run(['zsh','./edit.sh',f'{W}/o.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
o=Image.open(f'{W}/o0.png').convert('L').point(lambda v:255 if v>20 else 0).getbbox()
print('ortho is as wide as its box says, however deep:','yes' if o and abs((o[3]-o[1])-135)<4 else f'NO {o}')
said=['add solid cube',f'say 1 script {W}/deep.js','at 0',f'still {W}/e0.png']
subprocess.run(['zsh','./edit.sh',f'{W}/e.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'960x540'})
far=sum(1 for c in Image.open(f'{W}/e0.png').convert('RGB').getdata() if c[1]>200 and c[0]<60)
print('what stands far back is in front of what it clears with:','yes' if far>2000 else f'NO {far}')
box=lambda n:Image.open(f'{W}/{n}.png').convert('L').point(lambda v:255 if v>20 else 0).getbbox()
a,b=box('a'),box('b')
tall=a and (a[3]-a[1])>3*(a[2]-a[0])
print('its box() stands as tall as it says:','yes' if tall else f'NO ({a})')
moved=a and b and (b[0]+b[2])-(a[0]+a[2])
print('it goes where the sketch goes as its frames run:','yes' if moved and moved>2*(a[2]-a[0]) else f'NO ({a} {b})')
same=ImageChops.difference(Image.open(f'{W}/a.png').convert('RGB'),Image.open(f'{W}/c.png').convert('RGB')).getbbox() is None
print('going back shows the same frame:','yes' if same else 'NO')
said=[f'later door 0 {W}/colours.js','add light sun','at 0',f'still {W}/d.png']
subprocess.run(['zsh','./edit.sh',f'{W}/t.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'MOTOLII_OWN':W,'SIZE':'960x540'})
p=Image.open(f'{W}/d.png').convert('RGB')
n=[sum(1 for c in p.crop(a).getdata() if c[i]>c[j]+60) for a,i,j in (((0,0,480,540),0,2),((480,0,960,540),2,0))]
print('each copy is the colour its sketch fills it with:','yes' if min(n)>500 else f'NO ({n})')
P
