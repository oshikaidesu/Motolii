"""A coordinate-frame change and an explicit coplanar surface-group experiment."""
import json,sys
from pathlib import Path
import numpy as np
p=json.loads(Path(sys.argv[1]).read_text());mode=sys.argv[3] if len(sys.argv)>3 else 'both';origin=np.array([0,0,0.] if mode=='group' else [800,500,0.],dtype=np.float32)
# This scene has two explicitly planar 2D inputs on z=0. IDs 2 and 6 become
# one visibility surface group; resolving its ordered layers is a separate task.
for key in ['instances','alternate_instances']:
 for item in p.get(key,[]):
  mat=np.asarray(item['transform'],dtype=np.float32).reshape(3,4);mat[:,3]-=origin;item['transform']=mat.ravel().tolist()
  if mode!='rebase' and item['id'] in [2,6]:item['id']=100
r=np.asarray(p['rays'],dtype=np.float32);r[:,:3]-=origin;p['rays']=r.tolist()
for key in ['expected_ids','alternate_expected_ids']:
 if mode!='rebase':p[key]=[100 if x in [2,6] else x for x in p[key]]
Path(sys.argv[2]).write_text(json.dumps(p,separators=(',',':')))
