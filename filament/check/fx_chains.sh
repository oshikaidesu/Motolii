#!/bin/zsh
# Effects laid on effects, in a minute: not every pair of them (345,156 pairs take 45 minutes), but as ClusterFuzzLite
# fuzzes a change: what was changed first, each changed effect beside every other, both ways round; then, for what time
# is left, chains of 20 at random as Skia's FuzzAPIImageFilter stacks its filters (depth 20), drawn from this commit,
# so another commit tries others and the same one tries the same. A chain breaks if it falls, stands still or is refused.
#   ./check/fx_chains.sh [seconds]
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W ${1:-60} <<'P'
import glob,os,random,subprocess,sys,time
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
    open_=['add shape rect','say 1 fill ff3366 3366ff','set 1 scale_x 3','at 1']
    for c in part: open_+=['say 1 fx '+' '.join(c),f'still {W}/s.png']
    if os.path.exists(f'{W}/d.motolii'): os.remove(f'{W}/d.motolii')
    try:
        r=subprocess.run(['zsh','./edit.sh',f'{W}/d.motolii'],input='\n'.join(open_)+'\n',capture_output=True,text=True,timeout=60)
    except subprocess.TimeoutExpired as e:
        return 'stood still',(e.stdout or b'').decode(errors='replace')
    o=r.stdout+r.stderr
    if r.returncode: return f'fell ({r.returncode})',o
    if 'NOT done' in o: return 'refused',o
    return None,o
t=time.time(); done=pairs=0; broke=None
while not broke and time.time()-t<limit:
    part=[chains.pop(0) for _ in range(min(40,len(chains)))] or [random.sample(fx,20) for _ in range(40)]
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
