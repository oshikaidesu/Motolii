#!/bin/zsh
# What an effect draws is not cut off at the thing's frame: every effect alone on a thing, none may end in a straight edge
# there (the frame, 128 points round it: reports/alight_checklist.md). Then effects laid on effects, in the time left of a
# minute: not every pair of them (345,156 pairs take 45 minutes), but as ClusterFuzzLite fuzzes a change: what was changed
# first, each changed effect beside every other, both ways round; then chains of 20 at random as Skia's
# FuzzAPIImageFilter stacks its filters (depth 20), drawn from this commit, so another commit tries others and the same
# one tries the same. A chain breaks if it falls, stands still or is refused.
#   ./check/fx_chains.sh [seconds]
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W ${1:-60} <<'P'
import glob,os,random,subprocess,sys,time
from PIL import Image
W,limit=sys.argv[1],float(sys.argv[2])
fx=sorted(glob.glob(os.getcwd()+'/effects/*.fs')+glob.glob(os.getcwd()+'/effects/*.wgsl'))
def git(*a): return subprocess.run(['git',*a],capture_output=True,text=True).stdout
touched={os.path.basename(p) for p in git('diff','--name-only','-z','HEAD~1','HEAD','--','effects').split('\0')}
touched|={os.path.basename(p[3:]) for p in git('status','--porcelain','-z','--','effects').split('\0')}
changed=[f for f in fx if os.path.basename(f) in touched]
head=git('rev-parse','HEAD').strip()
chains=[]
for f in changed:
    rest=[o for o in fx if o!=f]
    for i in range(0,len(rest),10): chains.append([x for o in rest[i:i+10] for x in (f,o)]+[f])
random.seed(head)
def draw(part):
    open_=['add shape rect','say 1 fill ff3366 3366ff','at 1',f'still {W}/plain.png']
    for i,c in enumerate(part): open_+=['say 1 fx '+' '.join(c),f'still {W}/{i}.png']
    if os.path.exists(f'{W}/d.motolii'): os.remove(f'{W}/d.motolii')
    try:
        r=subprocess.run(['zsh','./edit.sh',f'{W}/d.motolii'],input='\n'.join(open_)+'\n',capture_output=True,text=True,timeout=60)
    except subprocess.TimeoutExpired as e:
        subprocess.run(['pkill','-f',W])
        return 'stood still',(e.stdout or b'').decode(errors='replace')
    o=r.stdout+r.stderr
    if r.returncode: return f'fell ({r.returncode})',o
    if 'NOT done' in o: return 'refused',o
    return None,o
def share(im,b):
    h=im.crop(b).histogram(); return sum(h[7:])/max(sum(h),1)
def cut(p,F):   # (a side of the frame lit just within it and dark just past it)
    im=Image.open(p).convert('L'); w,h=im.size; x0,y0,x1,y1=F; sides=[]
    if x0>4: sides.append((share(im,(x0+1,y0,x0+4,y1)),share(im,(x0-4,y0,x0-1,y1))))
    if x1<w-4: sides.append((share(im,(x1-4,y0,x1-1,y1)),share(im,(x1+1,y0,x1+4,y1))))
    if y0>4: sides.append((share(im,(x0,y0+1,x1,y0+4)),share(im,(x0,y0-4,x1,y0-1))))
    if y1<h-4: sides.append((share(im,(x0,y1-4,x1,y1-1)),share(im,(x0,y1+1,x1,y1+4))))
    return any(i>.3 and o<.02 for i,o in sides)
t=time.time(); done=pairs=0; broke=None
known={'God Rays.fs','Pixellate.fs','Triangles.fs'}   # (Pixellate and Triangles clamp to 0..1 in their own lines; God Rays not read yet)
alone=changed+random.sample(fx,len(fx)); seen=0; trimmed=[]
while not broke and alone and time.time()-t<limit/2:   # (half the time: what was changed, then others this commit draws)
    part=[[alone.pop(0)] for _ in range(min(40,len(alone)))]
    why,o=draw(part)
    if why: broke=(why,part[min(o.count('still: done')-1,len(part)-1)],[]); break
    b=Image.open(f'{W}/plain.png').convert('L').point(lambda v:255 if v>6 else 0).getbbox(); F=(b[0]-128,b[1]-128,b[2]+128,b[3]+128)
    trimmed+=[os.path.basename(c[0]) for i,c in enumerate(part) if cut(f'{W}/{i}.png',F)]; seen+=len(part)
print('alone',seen,'cut at the frame',len(trimmed),*sorted(set(trimmed)-known))
print('no effect is cut at the frame:','yes' if set(trimmed)<=known else 'NO')
while not broke and time.time()-t<limit:
    part=[chains.pop(0) for _ in range(min(8,len(chains)))] or [random.sample(fx,20) for _ in range(8)]   # (8 at a time: a slow CI Mac draws 40 of them in more than draw()'s minute, and that read as standing still)
    why,o=draw(part)
    if why:
        at=min(o.count('still: done'),len(part)-1)
        broke=(why,part[at],[l for l in o.split('\n') if 'NOT done' in l][:2])
        break
    done+=len(part); pairs+=sum(len(c)-1 for c in part)
print(f'changed {len(changed)}, chains {done}, pairs {pairs}, commit {head[:9]}, {time.time()-t:.0f} s')
if broke:
    print(broke[0],*[os.path.basename(x) for x in broke[1]],sep=' | ')
    print(*broke[2],sep='\n')
print('effects on effects hold:','NO' if broke else 'yes')
P
rm -rf $W
