"""Offline geometry-coverage study, not a production renderer or GPU benchmark."""
import json,sys,gc,time
from pathlib import Path
import numpy as np
import trimesh
from trimesh.ray.ray_pyembree import RayMeshIntersector
ASSETS=Path(__file__).resolve().parent.parent/'2026-09-09-glass-gallery'
def obj(name):
 v=[];n=[];f=[]
 for line in (ASSETS/name).read_text().splitlines():
  s=line.split()
  if s[0]=='v':v.append(list(map(float,s[1:])))
  elif s[0]=='vn':n.append(list(map(float,s[1:])))
  elif s[0]=='f':f.append([int(x.split('/')[0])-1 for x in s[1:]])
 return np.array(v),np.array(f),np.array(n)
def scene(meta):
 vertices=[];faces=[];normals=[];ids=[];meshes=[];instances=[];mesh_keys={}
 for item in meta['inputs']:
  ident=item['index']
  if 'world_from_object' in item:
   key='sphere.obj' if item['model_bounds'][1][0]<1.1 else 'torus.obj'
   v,f,n=obj(key);world=np.array(item['world_from_object']).reshape(4,4,order='F')
  elif ident in [1,2,6] and item['bounds'] is not None:
   lo,hi=np.array(item['bounds']);key=f'plane-{ident}'
   v=np.array([[lo[0],lo[1],lo[2]],[hi[0],lo[1],lo[2]],[hi[0],hi[1],lo[2]],[lo[0],hi[1],lo[2]]]);f=np.array([[0,1,2],[0,2,3]]);n=np.tile([0,0,-1.],(4,1));world=np.eye(4)
  else:continue
  if key not in mesh_keys:
   mesh_keys[key]=len(meshes);meshes.append({'vertices':v.tolist(),'indices':f.ravel().tolist()})
  instances.append({'mesh':mesh_keys[key],'id':ident,'transform':world[:3].ravel().tolist()})
  offset=sum(len(a) for a in vertices);vertices.append(v@world[:3,:3].T+world[:3,3]);faces.append(f+offset)
  wn=n@np.linalg.inv(world[:3,:3]);wn/=np.linalg.norm(wn,axis=1)[:,None];normals.append(wn);ids.extend([ident]*len(f))
 m=trimesh.Trimesh(vertices=np.concatenate(vertices),faces=np.concatenate(faces),process=False)
 return m,np.concatenate(normals),np.array(ids),meshes,instances

def layers(inter,origins,directions,count):
 fi,ri,p=inter.intersects_id(origins,directions,multiple_hits=count>1,max_hits=count,return_locations=True)
 depths=np.linalg.norm(p-origins[ri],axis=1);order=np.lexsort((depths,ri));fi=fi[order];ri=ri[order];p=p[order];depths=depths[order]
 ranks=np.arange(len(ri))-np.repeat(np.flatnonzero(np.r_[True,np.diff(ri)!=0]),np.diff(np.r_[np.flatnonzero(np.r_[True,np.diff(ri)!=0]),len(ri)])) if len(ri) else np.array([],dtype=int)
 valid=ranks<count;ri=ri[valid];ranks=ranks[valid]
 out_ids=np.full((count,len(origins)),-1,dtype=int);out_p=np.zeros((count,len(origins),3));out_d=np.full((count,len(origins)),np.inf)
 out_ids[ranks,ri]=fi[valid];out_p[ranks,ri]=p[valid];out_d[ranks,ri]=depths[valid]
 return out_ids,out_p,out_d

def queries(mesh,normals,ids,pixel_stride=10):
 keep=np.isin(ids,[3,4,5]);receiver=trimesh.Trimesh(vertices=mesh.vertices,faces=mesh.faces[keep],process=False)
 ri=RayMeshIntersector(receiver);focal=500/np.tan(np.deg2rad(55/2));eye=np.array([800,500,-focal])
 x,y=np.meshgrid(np.arange(pixel_stride/2,1600,pixel_stride),np.arange(pixel_stride/2,1000,pixel_stride));target=np.c_[x.ravel(),y.ravel(),np.zeros(x.size)];d=target-eye;d/=np.linalg.norm(d,axis=1)[:,None]
 p,r,t=ri.intersects_location(np.tile(eye,(len(d),1)),d,multiple_hits=False)
 order=np.argsort(r);p=p[order];r=r[order];t=t[order];t=np.flatnonzero(keep)[t]
 bary=trimesh.triangles.points_to_barycentric(mesh.triangles[t],p);n=np.sum(normals[mesh.faces[t]]*bary[:,:,None],axis=1);n/=np.linalg.norm(n,axis=1)[:,None]
 incident=d[r];n*=np.where(np.sum(n*incident,axis=1)>0,-1,1)[:,None]
 reflected=incident-2*np.sum(incident*n,axis=1)[:,None]*n
 return p+n*.02,reflected

def cube_dirs(res):
 dirs=[];u,v=np.meshgrid((np.arange(res)+.5)/res*2-1,(np.arange(res)+.5)/res*2-1)
 forward=np.array([[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]])
 up=np.array([[0,-1,0],[0,-1,0],[0,0,1],[0,0,-1],[0,-1,0],[0,-1,0]])
 for f,a in zip(forward,up):
  d=f+u[:,:,None]*np.cross(f,a)-v[:,:,None]*a;d/=np.linalg.norm(d,axis=2)[:,:,None];dirs.append(d.reshape(-1,3))
 return np.concatenate(dirs)

def proxy_mesh(mesh,ids,inter,origins,res,count):
 all_p=[];all_f=[];all_id=[]
 grid=np.arange(res*res).reshape(res,res);a=grid[:-1,:-1].ravel();b=grid[:-1,1:].ravel();c=grid[1:,:-1].ravel();d=grid[1:,1:].ravel();cells=np.concatenate([np.stack([a,b,d],1),np.stack([a,d,c],1)])
 directions=cube_dirs(res)
 for origin in origins:
  fi,pos,_=layers(inter,np.tile(origin,(len(directions),1)),directions,count)
  for layer in range(count):
   for face in range(6):
    sel=slice(face*res*res,(face+1)*res*res);tri=fi[layer,sel];points=pos[layer,sel];safe=np.maximum(tri,0);objid=ids[safe];n=mesh.face_normals[safe]
    valid=np.all(tri[cells]>=0,axis=1)&np.all(objid[cells]==objid[cells[:,0]][:,None],axis=1)
    # A local smooth-patch heuristic. False acceptances are measured against the oracle.
    valid&=(np.sum(n[cells[:,0]]*n[cells[:,1]],axis=1)>.5)&(np.sum(n[cells[:,0]]*n[cells[:,2]],axis=1)>.5)
    f=cells[valid];offset=sum(len(p) for p in all_p);all_p.append(points);all_f.append(f+offset);all_id.extend(objid[f[:,0]])
 return trimesh.Trimesh(vertices=np.concatenate(all_p),faces=np.concatenate(all_f),process=False),np.array(all_id)

def run(path,out):
 meta=json.loads(path.read_text());m,n,ids,meshes,instances=scene(meta);inter=RayMeshIntersector(m);o,d=queries(m,n,ids);fi,p,t=layers(inter,o,d,1);fi=fi[0];p=p[0];t=t[0];has=fi>=0
 oracle_ids=np.where(has,ids[np.maximum(fi,0)],-1)
 # Oracle-aided upper bound: does either probe have line of sight to the *known* target?
 ideal=np.zeros((4,len(o)),dtype=bool)
 for origin in meta['origins']:
  q=p[has]-origin;dist=np.linalg.norm(q,axis=1);q/=dist[:,None]
  cf,cp,cd=layers(inter,np.tile(origin,(len(q),1)),q,4)
  for k in range(4):ideal[k,has]|=(cf[k]>=0)&(np.abs(cd[k]-dist)<.03)
 result={'case':path.stem,'rays':len(o),'oracle_hits':int(has.sum()),'ideal_depth_layer_coverage':{str(k):float(ideal[:k].any(axis=0)[has].mean()) for k in [1,2,4]},'proxies':[]}
 for res in [32,64,128]:
  for count in [1,2]:
   pm,pi=proxy_mesh(m,ids,inter,meta['origins'],res,count);pf,pp,pt=layers(RayMeshIntersector(pm),o,d,1);pf=pf[0];pred=pf>=0;predid=np.where(pred,pi[np.maximum(pf,0)],-1)
   err=np.full(len(t),np.inf);both=pred&has;err[both]=np.abs(pt[0,both]-t[both]);correct=pred&has&(predid==oracle_ids)&(err<.5)
   result['proxies'].append({'resolution':res,'layers':count,'triangles':len(pm.faces),'hit_coverage':float(correct[has].mean()),'false_hit_fraction':float((pred&~correct).mean()),'no_candidate_fraction':float((~pred).mean())})
   del pm;gc.collect()
 print(json.dumps(result),flush=True);out.mkdir(exist_ok=True,parents=True);(out/(path.stem+'-coverage.json')).write_text(json.dumps(result,indent=2)+'\n')
 # Same rays and geometry for the independent GPU query experiment.
 packet={'meshes':meshes,'instances':instances,'rays':np.c_[o,np.full(len(o),.001),d,np.full(len(o),1e6)].tolist(),'expected_ids':oracle_ids.tolist(),'expected_t':np.where(has,t,-1).tolist()}
 (out/(path.stem+'-packet.json')).write_text(json.dumps(packet,separators=(',',':')))
if __name__=='__main__':
 out=Path(sys.argv[1])
 for path in sys.argv[2:]:run(Path(path),out)
