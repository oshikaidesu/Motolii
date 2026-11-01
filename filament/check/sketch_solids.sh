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
python3 - $W <<'P'
import os,subprocess,sys
from PIL import Image,ImageChops
W=sys.argv[1]
said=['add solid cube',f'say 1 script {W}/sketch.js','add light sun','add camera camera','set 3 z 9',
      'at 0',f'still {W}/a.png','at 1',f'still {W}/b.png','at 0',f'still {W}/c.png']
r=subprocess.run(['zsh','./edit.sh',f'{W}/s.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=200,env={**os.environ,'SIZE':'1920x1080'})
print('a sketch\'s file is taken as it is:','yes' if 'NOT done' not in r.stdout+r.stderr else 'NO '+(r.stdout+r.stderr)[-300:])
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
