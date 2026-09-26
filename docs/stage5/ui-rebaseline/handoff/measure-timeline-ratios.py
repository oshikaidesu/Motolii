import numpy as np
from PIL import Image
from scipy import ndimage as ndi
def load(p): return np.asarray(Image.open(p).convert('RGB')).astype(float)
ROWS=['Transform','Scatter','Stagger','Along','Rotation','Scale']
XR={'Transform':(640,840),'Scatter':(640,1300),'Stagger':(640,1190),'Along':(640,1270),'Rotation':(640,910),'Scale':(700,1130)}
KEYS={'Transform':[632,850],'Scatter':[770,988],'Stagger':[617,778,867,1195],'Along':[632,816],'Rotation':[617,781,906],'Scale':[690]}
RAIL={'Transform':(760,830),'Scatter':(880,960),'Stagger':(900,1150),'Along':(660,780),'Rotation':(640,760)}
def run(path,centres,tag):
    I=load(path); L=I.sum(axis=2)/3; S=I.max(axis=2)-I.min(axis=2)
    pitch=np.polyfit(range(6),centres,1)[0]
    bars=[];keys=[];rails=[]
    for nm,c in zip(ROWS,centres):
        yc=int(round(c)); a,b=XR[nm]
        hs=[]
        for x in range(a,b,23):
            col=(S[yc-12:yc+13,x]>12)&(L[yc-12:yc+13,x]>70)
            if not col[12]: continue
            lo=hi=12
            while lo>0 and col[lo-1]: lo-=1
            while hi<24 and col[hi+1]: hi+=1
            hs.append(hi-lo+1)
        bars.append(float(np.median(hs)) if hs else np.nan)
        for x in KEYS[nm]:
            body=np.median(I[yc-6:yc-4,x-4:x+5].reshape(-1,3),axis=0)
            d=np.abs(I[yc-6:yc+7,x]-body).sum(axis=1)     # interior rows only: rim and edge excluded
            idx=[i for i in range(13) if d[i]>28 and abs(i-6)>=2]   # rows away from the connector
            up=[i for i in idx if i<6]; dn=[i for i in idx if i>6]
            if up and dn: keys.append((nm,x,max(dn)-min(up)+1))
        if nm in RAIL:
            body=np.median(L[yc-6:yc-4,RAIL[nm][0]:RAIL[nm][1]])
            for x in range(RAIL[nm][0],RAIL[nm][1],7):
                col=L[yc-5:yc+6,x]<body*0.86; best=0; i=0
                while i<11:
                    if col[i]:
                        j=i
                        while j+1<11 and col[j+1]: j+=1
                        if i<=6 and j>=4: best=max(best,j-i+1)
                        i=j+1
                    else: i+=1
                if best: rails.append(best)
    caps=[]
    for c in centres:
        yc=int(round(c)); reg=L[yc-8:yc+9,412:421]; ys=np.where((reg>120).any(axis=1))[0]
        if len(ys): caps.append(ys.max()-ys.min()+1)
    strip=(L[750:768,562:1520]>110); lab,n=ndi.label(strip,structure=np.ones((3,3)))
    rh=[sl[0].stop-sl[0].start for sl in ndi.find_objects(lab) if 5<=sl[0].stop-sl[0].start<=13 and sl[1].stop-sl[1].start<=8]
    m=(S[795:935,380:408]>60)&(L[795:935,380:408]>60); lab,n=ndi.label(m)
    ic=[(sl[0].stop-sl[0].start+sl[1].stop-sl[1].start)/2 for sl in ndi.find_objects(lab) if 12<=sl[0].stop-sl[0].start<=19 and 12<=sl[1].stop-sl[1].start<=19]
    B=np.nanmedian(bars); K=np.median([k[2] for k in keys]); T=np.median(rails); Ic=np.median(ic); Cp=np.median(caps); R=np.median(rh)
    print('==',tag)
    print('  row centres',centres,'-> pitch (regression) %.2f'%pitch)
    print('  bar heights per row',bars,' median %.1f'%B)
    print('  key outline extents',[(k[0][:3],k[1],k[2]) for k in keys],' median %.1f'%K)
    print('  connector thickness median %.1f  (n=%d)'%(T,len(rails)))
    print('  icon boxes %s median %.1f | cap heights %s median %.1f | ruler digits median %.1f (n=%d)'%(ic,Ic,caps,Cp,R,len(rh)))
    print('  RATIOS  bar/pitch %.2f | key/bar %.2f | connector/bar %.2f | icon/pitch %.2f | cap/pitch %.2f | ruler/pitch %.2f'%(B/pitch,K/B,T/B,Ic/pitch,Cp/pitch,R/pitch))
# row centres: Concept = icon squares (five) + the Transform diamond icon; Candidate = the layout's own icon squares
run('concept.png',[809.0,831.5,854.0,876.0,898.5,921.5],'CONCEPT')
run('pol6.png',[809.0,832.0,855.0,878.0,901.0,924.0],'CANDIDATE')
