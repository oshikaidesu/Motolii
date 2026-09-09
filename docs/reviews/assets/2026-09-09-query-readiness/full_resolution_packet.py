"""Mesh-only primary surface workload at 1600x1000, one mirror query per surface pixel."""
import json,sys
from pathlib import Path
import numpy as np
import coverage_lab as lab
root=Path(__file__).resolve().parent
m,n,ids,meshes,instances=lab.scene(json.loads((root/'cases/full-03.json').read_text()))
o,d=lab.queries(m,n,ids,pixel_stride=1)
r=np.c_[o,np.full(len(o),.001),d,np.full(len(o),1e6)].astype(np.float32);o=r[:,:3].astype(float);d=r[:,4:7].astype(float)
def expected(mesh,ident):
 fi,p,t=lab.layers(lab.RayMeshIntersector(mesh),o+d*.001,d,1);fi=fi[0];t=t[0]+.001
 return np.where(fi>=0,ident[np.maximum(fi,0)],-1).tolist(),np.where(fi>=0,t,-1).tolist()
ei,et=expected(m,ids)
a,an,ai,_,ainstances=lab.scene(json.loads((root/'cases/full-02.json').read_text()));aei,aet=expected(a,ai)
packet={'meshes':meshes,'instances':instances,'rays':r.tolist(),'expected_ids':ei,'expected_t':et,'alternate_instances':ainstances,'alternate_expected_ids':aei,'alternate_expected_t':aet}
Path(sys.argv[1]).write_text(json.dumps(packet,separators=(',',':')))
print(json.dumps({'unique_rays':len(r),'hits':sum(i>=0 for i in ei)}))
