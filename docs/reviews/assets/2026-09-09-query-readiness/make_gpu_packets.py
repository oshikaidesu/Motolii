import json,sys,importlib.util
from pathlib import Path
import numpy as np
import coverage_lab as lab
out=Path(sys.argv[1]);p=json.loads((out/'full-03-packet.json').read_text());m,n,ids,meshes,instances=lab.scene(json.loads((Path(__file__).resolve().parent/'cases'/'full-02.json').read_text()));r=np.asarray(p['rays']);o=r[:,:3];d=r[:,4:7];fi,points,t=lab.layers(lab.RayMeshIntersector(m),o+d*.001,d,1);fi=fi[0];t=t[0]+.001;p['alternate_instances']=instances;p['alternate_expected_ids']=np.where(fi>=0,ids[np.maximum(fi,0)],-1).tolist();p['alternate_expected_t']=np.where(fi>=0,t,-1).tolist();(out/'moving-packet.json').write_text(json.dumps(p,separators=(',',':')))
v=[[-1,-1,0],[1,-1,0],[1,1,0],[-1,1,0]];f=[0,1,2,0,2,3];a=np.eye(4)[:3];b=a.copy();b[2,3]=1
x,y=np.meshgrid(np.linspace(-.975,.975,40),np.linspace(-.975,.975,40));uv=np.c_[(x.ravel()+1)/2,(y.ravel()+1)/2];hole=((uv-.5)**2).sum(axis=1)<.09
r=np.c_[x.ravel(),y.ravel(),np.full(x.size,-2),np.full(x.size,.001),np.zeros((x.size,2)),np.ones(x.size),np.full(x.size,100)]
p={'meshes':[{'vertices':v,'indices':f,'alpha_hole':True},{'vertices':v,'indices':f}],'instances':[{'mesh':0,'id':10,'transform':a.ravel().tolist()},{'mesh':1,'id':11,'transform':b.ravel().tolist()}],'rays':r.tolist(),'expected_ids':np.where(hole,11,10).tolist(),'expected_t':np.where(hole,3,2).tolist()};(out/'alpha-packet.json').write_text(json.dumps(p,separators=(',',':')))

q=json.loads((out/'alpha-packet.json').read_text());a=q['instances'][0];back=q['instances'][1];q['instances']=[]
for i in range(20):
 b=dict(a);b['transform']=a['transform'].copy();b['transform'][11]=i;q['instances'].append(b)
back['transform'][11]=20;q['instances'].append(back);q['expected_t']=[-1 if i==11 else 2 for i in q['expected_ids']];q['expected_ids']=[-2 if i==11 else 10 for i in q['expected_ids']];(out/'budget-packet.json').write_text(json.dumps(q,separators=(',',':')))
