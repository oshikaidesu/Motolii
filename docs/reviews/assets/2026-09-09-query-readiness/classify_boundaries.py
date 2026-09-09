"""Independent f64 local-space triangle check for dense-workload discrepancies."""
import json,sys
from pathlib import Path
import numpy as np
packet=json.loads(Path(sys.argv[1]).read_text());details=json.loads(Path(sys.argv[2]).read_text());unique=list({(x['index'],x['alternate']):x for x in details}.values())
def trace(ray,alternate):
 o=ray[:3];d=ray[4:7];hits=[]
 for inst in packet['alternate_instances'] if alternate else packet['instances']:
  mesh=packet['meshes'][inst['mesh']];v=np.asarray(mesh['vertices'],dtype=np.float32).astype(float);f=np.asarray(mesh['indices']).reshape(-1,3);w=np.eye(4);w[:3]=np.asarray(inst['transform'],dtype=np.float32).reshape(3,4)
  inv=np.linalg.inv(w);ro=(inv@np.r_[o,1])[:3];rd=inv[:3,:3]@d;tri=v[f];a=tri[:,0];e1=tri[:,1]-a;e2=tri[:,2]-a
  q=np.cross(np.broadcast_to(rd,e2.shape),e2);det=np.einsum('ij,ij->i',e1,q);ok=np.abs(det)>1e-12;iv=np.divide(1,det,out=np.zeros_like(det),where=ok);tv=ro-a;u=np.einsum('ij,ij->i',tv,q)*iv;q2=np.cross(tv,e1);vv=(q2@rd)*iv;t=np.einsum('ij,ij->i',e2,q2)*iv;ok&=(u>=0)&(vv>=0)&(u+vv<=1)&(t>=ray[3])&(t<=ray[7])
  for i in np.flatnonzero(ok):hits.append({'id':inst['id'],'primitive':int(i),'t':float(t[i]),'bary':[float(u[i]),float(vv[i])]})
 return sorted(hits,key=lambda x:x['t'])
result={'coplanar_ties':[],'other':[]}
for x in unique:
 if {x['expected_id'],x['gpu_id']}=={2,6} and abs(x['expected_t']-x['gpu_t'])<.001:
  # Both objects are explicit quads on z=0; retain this as a semantic tie, not a match.
  result['coplanar_ties'].append(x);continue
 r=np.asarray(packet['rays'][x['index']],dtype=np.float32).astype(float);neighbors=[]
 for k in [0,1,2,4,5,6]:
  for sign in [-1,1]:
   q=r.copy();q[k]=float(np.nextafter(np.float32(r[k]),np.float32(sign*np.inf)));h=trace(q,x['alternate']);neighbors.append({'component':k,'sign':sign,'nearest':h[:1]})
 result['other'].append({'gpu':x,'f64_nearest':trace(r,x['alternate'])[:1],'one_ulp_neighbors':neighbors})
Path(sys.argv[3]).write_text(json.dumps(result,indent=2)+'\n')
print('coplanar ties',len(result['coplanar_ties']))
for x in result['other']:print(x['gpu']['index'],x['gpu']['alternate'],'f64',x['f64_nearest'],'neighbor ids',[(a['component'],a['sign'],a['nearest'][0]['id'] if a['nearest'] else -1) for a in x['one_ulp_neighbors']])
