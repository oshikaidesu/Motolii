from pathlib import Path
import json, numpy as np
from PIL import Image
root=Path(__file__).resolve().parent.parent/'2026-09-09-glass-gallery'
def mesh(name):
 v=[];n=[];f=[]
 for l in (root/name).read_text().splitlines():
  s=l.split()
  if s[0]=='v':v.append(list(map(float,s[1:])))
  elif s[0]=='vn':n.append(list(map(float,s[1:])))
  elif s[0]=='f':f.append([int(x.split('/')[0])-1 for x in s[1:]])
 v=np.asarray(v);n=np.asarray(n);f=np.asarray(f)
 tri=v[f];return tri[:,0],tri[:,1]-tri[:,0],tri[:,2]-tri[:,0],n[f].mean(axis=1)
meshes={'sphere':mesh('sphere.obj'),'torus':mesh('torus.obj')}
def hit_stats(shape,world,origin):
 # Moller-Trumbore; nearest positive triangle hit, no face culling.
 inv=np.linalg.inv(world);o=(inv@np.r_[origin,1])[:3];v,e1,e2,n=meshes[shape];tv=o-v
 back=0;hits=0;dist=[]
 for j in range(64):
  y=1-2*(j+.5)/64;phi=j*np.pi*(3-np.sqrt(5));r=np.sqrt(1-y*y)
  d=inv[:3,:3]@np.array([r*np.cos(phi),y,r*np.sin(phi)])
  p=np.cross(np.broadcast_to(d,e2.shape),e2);det=np.einsum('ij,ij->i',e1,p)
  ok=np.abs(det)>1e-10;idet=np.divide(1,det,out=np.zeros_like(det),where=ok)
  u=np.einsum('ij,ij->i',tv,p)*idet;q=np.cross(tv,e1);vv=q@d*idet;t=np.einsum('ij,ij->i',e2,q)*idet
  ok&=(u>=0)&(vv>=0)&(u+vv<=1)&(t>1e-6)
  if ok.any():
   k=np.argmin(np.where(ok,t,np.inf));hits+=1;back+=int(n[k]@d>0);dist.append(float(t[k]))
 return {'rays':64,'hits':hits,'backface_hits':back,'nearest_distance':min(dist) if dist else None,'local_origin':o.tolist()}
def analyze(folder):
 out=[];prev_atlas=None
 for i in range(33):
  m=json.loads((folder/f'full-{i:02}.json').read_text());models=[x for x in m['inputs'] if 'world_from_object'in x]
  row={'step':i,'origins':m['origins'],'receivers':m['receivers'],'queries':[]}
  for pi,origin in enumerate(m['origins']):
   for model in models:
    shape='sphere' if model['model_bounds'][1][0]<1.1 else 'torus'
    world=np.array(model['world_from_object']).reshape(4,4,order='F')
    row['queries'].append({'probe':pi,'input':model['index'],'shape':shape,'excluded':model['index']==m['receivers'][pi],**hit_stats(shape,world,origin)})
  atlas=np.asarray(Image.open(folder/f'full-{i:02}-atlas.png'),dtype=np.int16)
  if prev_atlas is not None:
   h=atlas.shape[0]//2;row['atlas_delta']=[int(np.abs(atlas[k*h:(k+1)*h]-prev_atlas[k*h:(k+1)*h]).sum()) for k in range(2)]
  prev_atlas=atlas;out.append(row)
 (folder/'geometry-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
 for i in [0,2,3,4,28,29,32]:
  row=out[i];print(folder.name,i,'atlas',row.get('atlas_delta'),[(q['probe'],q['input'],q['shape'],q['hits'],q['backface_hits']) for q in row['queries'] if not q['excluded']])
if __name__=='__main__':
 import sys
 analyze(Path(sys.argv[1]))
