#!/bin/zsh
# A p5.js sketch that paints in 2D, with no browser: what it paints is ThorVG's shapes on a picture (p5_stand.h,
# drawn_sketch.h). One that wipes moves; one that never wipes builds up, as its canvas does; a frame comes the same
# however it is reached; its random() is the same each time; its camera sees the film under it (film_read.h); and each
# of its lines that draw is a knob, however many that line draws (script.h sketch_marked).
#   ./check/sketch_flat.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
cat > $W/wipes.js <<'J'
let x = 50;
function setup() { createCanvas(400, 300); noStroke(); }
function draw() { background(0); fill(255, 0, 0); circle(x, 150, 40); x += 3; }
J
cat > $W/lines.js <<'J'
function setup() { createCanvas(400, 300); noStroke(); }
function draw() { background(0, 0, 40);
  for (let i = 0; i < 8 + frameCount / 10; i++) { fill(255, 0, 0); rect(10 + i * 40, 120, 30, 30); }
}
J
cat > $W/builds.js <<'J'
let x = 50;
function setup() { createCanvas(400, 300); background(0); noStroke(); }
function draw() { fill(255, 0, 0); circle(x, 150, 40); x += 3; }
J
cat > $W/dice.js <<'J'
function setup() { createCanvas(400, 300); noStroke(); }
function draw() { background(0); fill(random(255), random(255), random(255)); rect(random(300), random(200), 100, 100); }
J
cat > $W/sees.js <<'J'
let cam;
function setup() { createCanvas(400, 300); cam = createCapture(VIDEO); cam.size(40, 30); cam.hide(); noStroke(); }
function draw() { cam.loadPixels(); let most = 0; for (let i = 0; i < cam.pixels.length; i += 4) most = max(most, cam.pixels[i + 1]); background(0, most, 0); }
J
cat > $W/grown.js <<'J'
let g;
function setup() { createCanvas(800, 400); noStroke(); g = createGraphics(50, 50); g.background(255); g.fill(0); g.noStroke(); g.textAlign(CENTER, CENTER); g.textSize(30); g.text('H', 25, 25); }
function draw() { background(255); image(g, 0, 0, 400, 400); image(g, 500, 0, 50, 50); }
J
python3 - $W <<'P'
import os,subprocess,sys
from PIL import Image,ImageChops
W=sys.argv[1]
def run(name,*said):
    r=subprocess.run(['zsh','./edit.sh',f'{W}/{name}.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=280,env={**os.environ,'SIZE':'800x600'})
    return r.stdout+r.stderr
at=lambda n:f'{W}/{n}.png'
said=run('a',f'add picture {W}/wipes.js','set 1 size 3','at 0.2',f'still {at("w0")}','at 1',f'still {at("w1")}','at 0.2',f'still {at("w2")}')
print('a 2D sketch is taken as it is:','yes' if 'NOT done' not in said else 'NO '+said[-300:])
run('b',f'add picture {W}/builds.js','set 1 size 3','at 0.2',f'still {at("b0")}','at 1',f'still {at("b1")}','at 0.5',f'still {at("b2")}')
run('c',f'add picture {W}/builds.js','set 1 size 3','at 0.5',f'still {at("b3")}')
run('d',f'add picture {W}/dice.js','set 1 size 3','at 1',f'still {at("d0")}')
run('e',f'add picture {W}/dice.js','set 1 size 3','at 1',f'still {at("d1")}')
run('f',f'add film {os.getcwd()}/check/beats.mp4',f'add picture {W}/sees.js','set 2 size 3','at 0.5',f'still {at("s0")}')
said=run('l',f'add film {os.getcwd()}/check/beats.mp4',f'add picture {W}/lines.js','set 2 size 3','at 0.5',f'still {at("l0")}','set 2 js_line_3 0',f'still {at("l1")}','set 2 js_line_3 1','set 2 js_line_2 0',f'still {at("l2")}','set 2 js_line_2 1','key 2 js_line_3 0 0','key 2 js_line_3 1 1',f'still {at("l3")}')
reds=lambda n:Image.open(at(n)).convert('RGB').split()[0].point(lambda v:255 if v>128 else 0).getbbox()
g=lambda n:Image.open(at(n)).convert('RGB')
w0,w1,b0,b1=reds('w0'),reds('w1'),reds('b0'),reds('b1')
wide=lambda b:b[2]-b[0]
print('a sketch that wipes moves, nothing left behind:','yes' if w0 and w1 and w1[0]>w0[2] and abs(wide(w0)-wide(w1))<=2 else f'NO ({w0} {w1})')
print('one that never wipes builds up:','yes' if b0 and b1 and wide(b1)>2*wide(b0) and b1[0]==b0[0] else f'NO ({b0} {b1})')
print('going back shows the same frame:','yes' if ImageChops.difference(g('w0'),g('w2')).getbbox() is None else 'NO')
d=ImageChops.difference(g('b2'),g('b3')).convert('L').point(lambda v:255 if v>24 else 0).getbbox()
print('a frame built up is the same however it is reached:','yes' if d is None else f'NO ({d})')
print('its random() is the same each time:','yes' if ImageChops.difference(g('d0'),g('d1')).getbbox() is None else 'NO')
print('its camera sees the film under it:','yes' if max(g('s0').split()[1].getdata())>40 else 'NO')
red=lambda n:sum(1 for r,gr,b in g(n).resize((80,60)).getdata() if r>180 and gr<90 and b<90)
print('a line that draws is a knob, however many it draws:','yes' if 'NOT done' not in said and red('l0')>20 and red('l1')==0 else f'NO ({red("l0")} {red("l1")} {said[-200:]})')
print('its background line off, what is under it shows:','yes' if red('l2')>20 and max(g('l2').split()[1].getdata())>40 else 'NO')
run('g',f'add picture {W}/grown.js','set 1 size 3','at 0.2',f'still {at("g0")}')
ink=lambda n,box:sum(1 for r,gr,b in g(n).crop(box).getdata() if r<100)
full=g('g0').size[0]/800; big,small=ink('g0',tuple(int(v*full) for v in (0,0,400,400))),ink('g0',tuple(int(v*full) for v in (500,0,550,50)))
print('words on another canvas grow as it grows, once:','yes' if small and 40<big/small<90 else f'NO ({big} {small})')
print('and it takes keys:','yes' if 0<g('l3').getextrema()[0][1] and red('l3')<red('l0') else f'NO ({red("l3")} {red("l0")})')
P
