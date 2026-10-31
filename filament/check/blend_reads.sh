#!/bin/zsh
# A way of lying that reads what is under it draws what the W3C's Compositing and Blending says, worked out here apart from
# flat.mat: each of the 21 over one colour, the answer read off the picture. Then what is read is what lies under it: not what
# lies over it, one such over another reads it as laid, and a fill of what is behind it takes the effect on it (the
# Background Copy). They were all drawn as normal for a week and nothing said so: their pictures were gone with the views.
#   ./check/blend_reads.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W <<'P'
import subprocess,sys,math,os
from PIL import Image
W=sys.argv[1]
b=(0xcc/255,0x66/255,0x33/255); s=(0x4d/255,0x99/255,0xe6/255)
lum=lambda c:.3*c[0]+.59*c[1]+.11*c[2]
sat=lambda c:max(c)-min(c)
def clip(c):
    l,n,x=lum(c),min(c),max(c)
    if n<0: c=[l+(v-l)*l/(l-n) for v in c]
    if x>1: c=[l+(v-l)*(1-l)/(x-l) for v in c]
    return c
def setlum(c,l): return clip([v+l-lum(c) for v in c])
def setsat(c,v):
    x,n=max(c),min(c)
    return [(k-n)*v/(x-n) if x>n else 0 for k in c]
def each(f): return [f(x,y) for x,y in zip(b,s)]
hard=lambda x,y:2*x*y if y<=.5 else 1-2*(1-x)*(1-y)
def soft(x,y):
    d=((16*x-12)*x+4)*x if x<=.25 else math.sqrt(x)
    return x-(1-2*y)*x*(1-x) if y<=.5 else x+(2*y-1)*(d-x)
dodge=lambda x,y:0 if x==0 else 1 if x>=1-y else x/(1-y)
burn=lambda x,y:1 if x>=1 else 0 if 1-x>=y else 1-(1-x)/y
ways={'darken':each(min),'lighten':each(max),'overlay':each(lambda x,y:hard(y,x)),'soft_light':each(soft),'hard_light':each(hard),
      'color_dodge':each(dodge),'color_burn':each(burn),'difference':each(lambda x,y:abs(x-y)),'exclusion':each(lambda x,y:x+y-2*x*y),
      'hue':setlum(setsat(s,sat(b)),lum(b)),'saturation':setlum(setsat(b,sat(s)),lum(b)),'color':setlum(s,lum(b)),'luminosity':setlum(b,lum(s)),
      'linear_burn':each(lambda x,y:x+y-1),'linear_light':each(lambda x,y:x+2*y-1),
      'vivid_light':each(lambda x,y:burn(x,2*y) if y<=.5 else dodge(x,2*y-1)),'pin_light':each(lambda x,y:min(x,2*y) if y<=.5 else max(x,2*y-1)),
      'darker_color':list(s if lum(s)<lum(b) else b),'lighter_color':list(s if lum(s)>lum(b) else b),'subtract':each(lambda x,y:x-y),'divide':each(lambda x,y:x/y)}
def run(doc,calls):
    r=subprocess.run(['zsh','./edit.sh',f'{W}/{doc}.motolii'],input='\n'.join(calls)+'\n',capture_output=True,text=True,timeout=120)
    return r.stdout+r.stderr
def at(name,x=.5,y=.5):
    im=Image.open(f'{W}/{name}.png').convert('RGB'); return im.getpixel((int(im.width*x),int(im.height*y)))
calls=['add shape rect','say 1 fill cc6633','add shape rect','say 2 fill 4d99e6']
for w in ways: calls+=[f'say 2 blend {w}',f'still {W}/{w}.png']
o=run('ways',calls)
bad=[]
for w,want in ways.items():
    if not os.path.exists(f'{W}/{w}.png'): bad.append(w+' (no picture)'); continue
    got=at(w); want=[round(255*min(1,max(0,v))) for v in want]
    if max(abs(g-h) for g,h in zip(got,want))>4: bad.append(f'{w} {got} wants {tuple(want)}')
print('each way draws what the W3C says:','yes (21)' if not bad else 'NO '+'; '.join(bad))
run('over',['add shape rect','say 1 fill ffffff','add shape rect','say 2 fill ffffff','say 2 blend difference','add shape rect','say 3 fill ff0000','set 3 size 0.4',f'still {W}/over.png'])
print('what lies over it is not read:','yes' if at('over')[0]>250 and at('over')[1]<5 and sum(at('over',.5,.6))<15 else f"NO {at('over')} {at('over',.5,.6)}")
run('two',['add shape rect','say 1 fill ffffff','add shape rect','say 2 fill ffffff','say 2 blend difference','add shape rect','say 3 fill ffffff','say 3 blend difference',f'still {W}/two.png'])
print('one over another reads it as laid:','yes' if sum(at('two'))>750 else f"NO {at('two')}")
run('copy',['add shape rect','say 1 fill 3399cc','add shape rect','say 2 fill behind',f'say 2 fx {os.getcwd()}/effects/Color Invert.fs',f'still {W}/copy.png'])
print('a fill of what is behind takes the effect on it:','yes' if abs(at('copy')[0]-0xcc)<6 and abs(at('copy')[1]-0x66)<6 else f"NO {at('copy')}")
P
rm -rf $W
