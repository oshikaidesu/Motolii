#!/bin/zsh
# Code as it comes from the net, each site's form through one door (door.h, doc_door.h): nothing told what it is, each
# paints and moves, a three.js scene stands as solids of its own shapes and colours, a shader that reads a picture goes
# on the picked thing, and nothing on the shelf is mistaken for one of them. Hydra is a part apart, so here it only says so.
#   ./check/door.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
mkdir $W/s
cat > $W/s/dwitter.txt <<'J'
c.width=1920;for(i=0;i<300;i++)x.fillStyle=R(i,99,255-i,.9),x.fillRect(960+S(i/9+t)*i*2,540+C(i/7+t)*i,24,24)
J
printf 'sin(t-hypot(x-7.5,y-7.5))' > $W/s/tixy.txt
cat > $W/s/turtletoy.txt <<'J'
Canvas.setpenopacity(1);
const turtle = new Turtle();
function walk(i) { turtle.forward(30); turtle.right(91 + i / 2000); return i < 60000; }
J
cat > $W/s/two.txt <<'J'
var two = new Two({ width: 1280, height: 720 }).appendTo(document.body);
var r = two.makeRectangle(640, 360, 300, 300);
r.fill = '#ff8000';
two.bind('update', function (f) { r.rotation = f / 30; r.translation.x = 300 + f * 4; }).play();
J
cat > $W/s/paper.txt <<'J'
var path = new Path.Circle({ center: view.center, radius: 200, fillColor: 'tomato' });
function onFrame(event) { path.position.y = view.center.y + Math.sin(event.time * 2) * 150; }
J
cat > $W/s/canvas.txt <<'J'
const cv = document.getElementById('c'), g = cv.getContext('2d');
cv.width = 1280; cv.height = 720;
let f = 0;
function loop() { requestAnimationFrame(loop); g.fillStyle = '#001'; g.fillRect(0, 0, 1280, 720); g.fillStyle = '#7ff'; g.beginPath(); g.arc(640 + Math.cos(f / 20) * 400, 360, 60, 0, 7); g.fill(); f++; }
loop();
J
cat > $W/s/shadertoy.txt <<'J'
void mainImage(out vec4 o, in vec2 p){ vec2 u=(p*2.-iResolution.xy)/iResolution.y; o=vec4(.5+.5*cos(6.*length(u)-iTime*3.+vec3(0,2,4)),1); }
J
cat > $W/s/sandbox.txt <<'J'
#ifdef GL_ES
precision mediump float;
#endif
uniform float time;
uniform vec2 resolution;
void main(void){ vec2 p=gl_FragCoord.xy/resolution; gl_FragColor=vec4(p, .5+.5*sin(time*4.+p.x*20.),1.0); }
J
cat > $W/three.js <<'J'
import * as THREE from 'three';
const scene = new THREE.Scene(), camera = new THREE.PerspectiveCamera(50, 16 / 9, 0.1, 100);
camera.position.z = 8;
const renderer = new THREE.WebGLRenderer();
const knot = new THREE.Mesh(new THREE.TorusKnotGeometry(1, 0.3, 80, 12), new THREE.MeshStandardMaterial({ color: 0xff3010 }));
scene.add(knot);
const shape = new THREE.BoxGeometry(0.4, 0.4, 0.4), blue = new THREE.MeshStandardMaterial({ color: 0x1040ff }), boxes = [];
for (let i = 0; i < 12; i++) { boxes.push(new THREE.Mesh(shape, blue)); scene.add(boxes[i]); }
renderer.setAnimationLoop(t => {
  boxes.forEach((b, i) => b.position.set(3 * Math.cos(i + t / 1000), 1.5 * Math.sin(i * 2), 3 * Math.sin(i + t / 1000)));
  renderer.render(scene, camera);
});
J
print -r -- "<!doctype html><html><head><script type=\"importmap\">{}</script></head><body><script type=\"module\">$(<$W/three.js)</script></body></html>" > $W/page.html
cat > $W/form.cpp <<'C'
#include "door.h"
#include <fstream>
#include <iostream>
#include <sstream>
int main(int n, char** a) { int wrong = 0; for (int i = 1; i < n; ++i) { std::ifstream in(a[i]); std::stringstream t; t << in.rdbuf(); if (!door_form(t.str()).empty()) ++wrong, std::cout << a[i] << "\n"; } std::cout << wrong << " of " << n - 1 << "\n"; }
C
c++ -std=c++20 -I. $W/form.cpp -o $W/form
python3 - $W <<'P'
import glob,os,subprocess,sys
from PIL import Image,ImageChops
W=sys.argv[1]
def run(name,*said):
    r=subprocess.run(['zsh','./edit.sh',f'{W}/{name}.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=280,env={**os.environ,'SIZE':'960x540','MOTOLII_OWN':f'{W}/own'})
    return r.stdout+r.stderr
g=lambda n:Image.open(f'{W}/{n}.png').convert('RGB')
for f in sorted(glob.glob(f'{W}/s/*.txt')):
    n=os.path.basename(f)[:-4]
    said=run(n,f'later door 0 {f}','at 1',f'still {W}/{n}1.png','at 2.2',f'still {W}/{n}2.png')
    colours=len(g(n+'2').resize((48,27)).getcolors(9999))
    moved=ImageChops.difference(g(n+'1'),g(n+'2')).getbbox() is not None
    print(f'{n} is read, paints and moves:','yes' if 'NOT done' not in said and colours>1 and moved else f'NO ({colours} {moved} {said[-240:]})')
for name,file in (('a three.js scene',f'{W}/three.js'),('and as a whole page',f'{W}/page.html')):
    k=name[:3]
    said=run(k,f'later door 0 {file}','list','add camera camera','at 1',f'still {W}/{k}1.png','set 4 x -5','set 4 z 3.3','set 4 rotate_y -56',f'still {W}/{k}2.png')
    picture=g(k+'1').resize((96,54)).getdata()
    red=sum(1 for r,gr,b in picture if r>120 and r>2*b); blue=sum(1 for r,gr,b in picture if b>120 and b>2*r)
    print(f'{name} stands as a group of solids, its colours its own:','yes' if '3 kind at 0.0: solid' in said and '2 kind at 0.0: solid' in said and '1 shape at 0.0: null' in said and red>20 and blue>3 else f'NO ({red} {blue} {said[-240:]})')
    print('  the world\'s own camera walks round it:','yes' if ImageChops.difference(g(k+'1'),g(k+'2')).getbbox() is not None else 'NO')
said=run('h','later door 0 osc(20, 0.1, 0.8).rotate(0.5).out()')
print('Hydra, not there, says how to get it:','yes' if 'extensions/hydra/get.sh' in said else f'NO ({said[-240:]})')
said=run('f',f'add film {os.getcwd()}/check/beats.mp4',"later door 1 void mainImage(out vec4 o, in vec2 p){ o=texture(iChannel0, p/iResolution.xy).bgra; }",'list','cards 1')
print('a shader that reads a picture goes on the picked thing:','yes' if 'NOT done' not in said and '2 kind at' not in said and 'Shadertoy' in said else f'NO ({said[-240:]})')
shelf=[f for k in ('effects','recipes','animators') for f in glob.glob(k+'/**',recursive=True) if f.endswith(('.fs','.js'))]
out=subprocess.run([f'{W}/form']+shelf,capture_output=True,text=True).stdout
print('nothing on the shelf is mistaken for code from the net:','yes' if out.startswith('0 of ') else 'NO '+out[-300:],f'({out.split()[-1]} files)')
P
