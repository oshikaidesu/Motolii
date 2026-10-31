#!/bin/zsh
# A document draws the same picture however it got there: two things with effects moved about, their pictures made again
# time and again, then what is on the stage against the same document opened anew (as GraphicsFuzz judges a shader by a
# picture it must equal: no picture known beforehand needed). Filament's handles come round after 16 ages
# (HandleAllocator.h HANDLE_AGE_BIT_COUNT), so what is kept by a handle reads an old picture only after many moves.
#   ./check/fx_moved.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W <<'P'
import math,os,subprocess,sys
from PIL import Image,ImageChops
W=sys.argv[1]; fx=os.getcwd()+'/effects/Bloom.fs'
calls=['add shape rect','say 1 fill ff3366',f'say 1 fx {fx}','add shape rect','say 2 fill 3366ff',f'say 2 fx {fx}']
for i in range(240):
    a=i*.7
    calls+=[f'set 1 x {1.5*math.cos(a):.3f}',f'set 1 y {2*math.sin(a*1.3):.3f}',f'set 2 x {1.5*math.sin(a*.8):.3f}',f'set 2 y {2*math.cos(a*1.1):.3f}',f'still {W}/moved.png']
def edit(said): return subprocess.run(['zsh','./edit.sh',f'{W}/d.motolii'],input='\n'.join(said)+'\n',capture_output=True,text=True,timeout=120)
r=edit(calls)
print('moved',r.stdout.count('still: done')+r.stderr.count('still: done'),'times')
edit([f'still {W}/anew.png'])
d=ImageChops.difference(*[Image.open(f'{W}/{n}.png').convert('RGB') for n in ('moved','anew')]).convert('L').point(lambda v:255 if v>24 else 0).histogram()
share=d[255]/sum(d)
print(f'differs from it opened anew {share*100:.2f} %')
print('moved or opened anew, the same picture:','yes' if share<.002 else 'NO')
P
rm -rf $W
