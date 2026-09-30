# A Downloads-like folder for integration_test/heavy_folder_test.dart: ~3,800 files, ~1,850 of them media (pictures, gifs, sounds,
# clips, exact duplicates), nested, plus documents. Usage: python3 scripts/make-heavy-media-folder.py /tmp/heavy
import os,random,struct,sys,zlib
random.seed(7)
root=sys.argv[1] if len(sys.argv)>1 else '/tmp/heavy'
def png(w,h):
    raw=b''.join(b'\x00'+bytes(random.randrange(256) for _ in range(w*3)) for _ in range(h))
    def chunk(t,d): c=struct.pack('>I',len(d))+t+d; return c+struct.pack('>I',zlib.crc32(t+d)&0xffffffff)
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')
def gif():
    pal=bytes(random.randrange(256) for _ in range(12))
    return b'GIF89a'+struct.pack('<HH',4,4)+b'\x81\x00\x00'+pal+b',\x00\x00\x00\x00\x04\x00\x04\x00\x00\x02\x02\x44\x01\x00;'
def wav(n=4000):
    data=bytes(random.randrange(256) for _ in range(n))
    return b'RIFF'+struct.pack('<I',36+n)+b'WAVEfmt '+struct.pack('<IHHIIHH',16,1,1,8000,8000,1,8)+b'data'+struct.pack('<I',n)+data
dirs=['','Screenshots','Screenshots/2026','Work/clients/a','Work/clients/b','Music','Music/loops','Reference','old/2024','old/2025','Videos','Videos/raw']
def path(d,name):
    p=os.path.join(root,d); os.makedirs(p,exist_ok=True); return os.path.join(p,name)
for i in range(850):
    d=random.choice(dirs); nm=random.choice(['IMG_%04d.png','Screenshot %d.png','untitled %d.png','design_v%d.png','texture_%d.png'])%random.randrange(99999)
    open(path(d,nm),'wb').write(png(random.choice([8,12,16]),random.choice([8,12,16])))
for i in range(400):
    d=random.choice(dirs); open(path(d,'anim_%d.gif'%i),'wb').write(gif())
for i in range(370):
    d=random.choice(dirs); open(path(d,random.choice(['rec %d.wav','loop_%d.wav','voice_%d.wav'])%i),'wb').write(wav())
for i in range(60):
    d=random.choice(dirs); open(path(d,'track_%d.mp3'%i),'wb').write(b'ID3'+bytes(random.randrange(256) for _ in range(3000)))
src=os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','motolii','ui','explorer','fixtures','media')+'/'
vids=[open(src+f,'rb').read() for f in ['coast-drift.mp4','sky-tilt-vertical.mp4','skyline-pan.mov']]
for i in range(140):
    d=random.choice(dirs); v=random.choice(vids); ext='mov' if v is vids[2] else 'mp4'
    open(path(d,'clip_%d.%s'%(i,ext)),'wb').write(v+bytes(random.randrange(256) for _ in range(64)))
dup=png(8,8)
for i in range(25):
    open(path(random.choice(dirs),'IMG_dup (%d).png'%i),'wb').write(dup)
for i in range(2000):
    open(path(random.choice(dirs),'doc_%d.pdf'%i),'wb').write(b'%PDF-1.4 '+bytes(200))
