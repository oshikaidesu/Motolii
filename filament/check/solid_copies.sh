#!/bin/zsh
# A solid takes Copies as any thing, and a .gltf with its parts in files beside it opens (Poly Haven hands them out so).
# A plane of two triangles, its numbers in a .bin of its own; then four of it, and four of the shelf's cube, each where its
# copy lies: lit across the picture, not only in one place.
#   ./check/solid_copies.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W <<'P'
import json,os,struct,subprocess,sys
from PIL import Image
W=sys.argv[1]
at=[(-.5,-.5,0),(.5,-.5,0),(.5,.5,0),(-.5,.5,0)]
turn=[(0,0,1)]*4
ix=[0,1,2,0,2,3]
data=b''.join(struct.pack('3f',*p) for p in at)+b''.join(struct.pack('3f',*n) for n in turn)+struct.pack('6H',*ix)
open(f'{W}/plane.bin','wb').write(data)
open(f'{W}/plane.gltf','w').write(json.dumps({'asset':{'version':'2.0'},'scene':0,'scenes':[{'nodes':[0]}],'nodes':[{'mesh':0}],
 'meshes':[{'primitives':[{'attributes':{'POSITION':0,'NORMAL':1},'indices':2}]}],'buffers':[{'uri':'plane.bin','byteLength':len(data)}],
 'bufferViews':[{'buffer':0,'byteLength':48},{'buffer':0,'byteOffset':48,'byteLength':48},{'buffer':0,'byteOffset':96,'byteLength':12}],
 'accessors':[{'bufferView':0,'componentType':5126,'count':4,'type':'VEC3','min':[-.5,-.5,0],'max':[.5,.5,0]},{'bufferView':1,'componentType':5126,'count':4,'type':'VEC3'},{'bufferView':2,'componentType':5123,'count':6,'type':'SCALAR'}]}))
def wide(name,said):
    r=subprocess.run(['zsh','./edit.sh',f'{W}/{name}.motolii'],input='\n'.join(said+[f'still {W}/{name}.png'])+'\n',capture_output=True,text=True,timeout=120,env={**os.environ,'SIZE':'1920x1080'})
    if 'NOT done' in r.stdout+r.stderr: return None
    b=Image.open(f'{W}/{name}.png').convert('L').point(lambda v:255 if v>20 else 0).getbbox()
    return b and b[2]-b[0]
one,four=wide('one',[f'add solid {W}/plane.gltf']),wide('four',[f'add solid {W}/plane.gltf','set 1 copies 4','set 1 copy_x 1.2'])
print('a .gltf with its parts beside it opens:','yes' if one else 'NO')
print('a file\'s copies each lie where theirs is:','yes' if one and four and four>one*2.5 else f'NO ({one} {four})')
cube=wide('cube',['add solid cube','set 1 copies 4','set 1 copy_x 1.5'])
print('a cube\'s copies too:','yes' if cube and cube>1000 else f'NO ({cube})')
P
rm -rf $W
