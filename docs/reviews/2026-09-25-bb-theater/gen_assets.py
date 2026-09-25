"""BB theater assets: two character standees (PNG, alpha), a poster, and a room light map (HDR)."""
import math, os
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
SS = 4  # supersampling

def canvas(w, h):
    return Image.new("RGBA", (w * SS, h * SS), (0, 0, 0, 0))

def s(v):  # scale coords
    return [c * SS for c in v]

def finish(img, w, h, name):
    img = img.resize((w, h), Image.LANCZOS)
    img.save(os.path.join(HERE, name))
    print(name, img.size)

OUT = (58, 40, 38, 255)
def blob(d, box, fill, width=5):
    d.ellipse(s(box), fill=fill, outline=OUT, width=width * SS)
def poly(d, pts, fill, width=5):
    d.polygon([c * SS for p in pts for c in p], fill=fill, outline=OUT, width=width * SS)
def rrect(d, box, r, fill, width=5):
    d.rounded_rectangle(s(box), radius=r * SS, fill=fill, outline=OUT, width=width * SS)

def eyes(d, cx, cy, gap, iris):
    for sx in (-1, 1):
        x = cx + sx * gap
        d.ellipse(s([x - 17, cy - 24, x + 17, cy + 24]), fill=(255, 255, 255, 255), outline=OUT, width=3 * SS)
        d.ellipse(s([x - 12, cy - 16, x + 12, cy + 20]), fill=iris)
        d.ellipse(s([x - 6, cy - 8, x + 6, cy + 10]), fill=(30, 22, 30, 255))
        d.ellipse(s([x - 9, cy - 14, x - 1, cy - 5]), fill=(255, 255, 255, 255))
        d.arc(s([x - 22, cy - 34, x + 22, cy - 6]), 200, 340, fill=OUT, width=4 * SS)

# ---- standee A: a girl, long navy hair, white blouse, red ribbon, navy skirt -----------------
W, H = 520, 1200
img = canvas(W, H); d = ImageDraw.Draw(img)
hair = (46, 58, 110, 255); skin = (255, 226, 205, 255)
poly(d, [(95, 250), (425, 250), (470, 820), (50, 820)], hair)                       # back hair
rrect(d, [150, 900, 215, 1150], 20, skin); rrect(d, [305, 900, 370, 1150], 20, skin)  # legs
rrect(d, [135, 1120, 225, 1178], 22, (70, 50, 48, 255)); rrect(d, [295, 1120, 385, 1178], 22, (70, 50, 48, 255))
poly(d, [(150, 640), (370, 640), (440, 930), (80, 930)], (40, 52, 96, 255))          # skirt
rrect(d, [140, 430, 380, 670], 40, (246, 244, 238, 255))                            # blouse
rrect(d, [90, 450, 150, 760], 28, (246, 244, 238, 255)); rrect(d, [370, 450, 430, 760], 28, (246, 244, 238, 255))
blob(d, [85, 735, 145, 795], skin); blob(d, [375, 735, 435, 795], skin)
poly(d, [(260, 470), (205, 440), (210, 520)], (205, 40, 50, 255)); poly(d, [(260, 470), (315, 440), (310, 520)], (205, 40, 50, 255))
blob(d, [245, 455, 275, 490], (205, 40, 50, 255))
rrect(d, [238, 380, 282, 440], 12, skin)                                            # neck
blob(d, [120, 120, 400, 420], skin)                                                 # face
poly(d, [(110, 250), (140, 120), (260, 80), (380, 120), (410, 250), (330, 190), (260, 230), (180, 190)], hair)  # bangs
eyes(d, 260, 300, 58, (70, 110, 190, 255))
d.arc(s([240, 345, 280, 375]), 20, 160, fill=OUT, width=4 * SS)
for sx in (-1, 1):
    d.ellipse(s([260 + sx * 95 - 22, 335, 260 + sx * 95 + 22, 355]), fill=(255, 170, 170, 150))
finish(img, W, H, "standee_a.png")

# ---- standee B: a boy, short brown hair, orange hoodie, grey trousers ------------------------
W, H = 540, 1260
img = canvas(W, H); d = ImageDraw.Draw(img)
hair = (110, 70, 45, 255)
rrect(d, [155, 820, 255, 1200], 26, (96, 100, 112, 255)); rrect(d, [285, 820, 385, 1200], 26, (96, 100, 112, 255))
rrect(d, [135, 1165, 265, 1235], 26, (235, 235, 235, 255)); rrect(d, [275, 1165, 405, 1235], 26, (235, 235, 235, 255))
rrect(d, [120, 450, 420, 880], 70, (236, 128, 52, 255))                             # hoodie
rrect(d, [60, 480, 140, 820], 34, (236, 128, 52, 255)); rrect(d, [400, 480, 480, 820], 34, (236, 128, 52, 255))
blob(d, [62, 795, 138, 865], skin); blob(d, [402, 795, 478, 865], skin)
rrect(d, [190, 700, 350, 800], 30, (214, 108, 40, 255))                             # pocket
d.line(s([235, 470, 225, 600]), fill=(250, 240, 230, 255), width=6 * SS); d.line(s([305, 470, 315, 600]), fill=(250, 240, 230, 255), width=6 * SS)
rrect(d, [245, 400, 295, 470], 12, skin)
blob(d, [125, 130, 415, 440], skin)
poly(d, [(115, 290), (120, 170), (200, 100), (330, 100), (420, 170), (425, 290), (380, 210), (300, 230), (240, 180), (170, 230)], hair)
eyes(d, 270, 320, 62, (90, 60, 40, 255))
d.arc(s([240, 360, 300, 400]), 20, 160, fill=OUT, width=4 * SS)
finish(img, W, H, "standee_b.png")

# ---- poster: a framed picture ---------------------------------------------------------------
W, H = 600, 420
img = Image.new("RGBA", (W * SS, H * SS), (236, 230, 216, 255)); d = ImageDraw.Draw(img)
d.rectangle(s([28, 28, W - 28, H - 28]), fill=(120, 170, 200, 255))
d.ellipse(s([380, 70, 480, 170]), fill=(252, 236, 180, 255))
d.polygon([c * SS for p in [(28, 330), (200, 170), (330, 300), (430, 210), (572, 330), (572, 392), (28, 392)] for c in p], fill=(70, 110, 90, 255))
d.rectangle(s([0, 0, W - 1, H - 1]), outline=(90, 70, 55, 255), width=14 * SS)
finish(img, W, H, "poster.png")

# ---- room light map (HDR): a warm key light up-left-front, a dim room around -------------
# re_renderer equirect: phi=(u-0.5)*2pi, theta=v*pi, dir=(sin th sin phi, cos th, -sin th cos phi); +y up, +z toward the camera.
W, H = 1024, 512
S = np.array([-0.62, 0.55, 0.56]); S /= np.linalg.norm(S)
HALF = math.radians(4.0)
v = (np.arange(H) + 0.5) / H; u = (np.arange(W) + 0.5) / W
th = v[:, None] * math.pi; ph = (u[None, :] - 0.5) * 2 * math.pi
dd = np.stack([np.sin(th) * np.sin(ph), np.cos(th) * np.ones_like(ph), -np.sin(th) * np.cos(ph)], -1)
room = np.array([0.050, 0.046, 0.042])
img = np.broadcast_to(room, dd.shape).copy().astype(np.float32)
img *= (0.7 + 0.3 * np.clip(dd[..., 1:2], 0, 1))
cosang = dd @ S
key = np.array([1.0, 0.80, 0.58]) * 60.0
img[cosang >= math.cos(HALF)] = key
def rgbe(rgb):
    m = rgb.max(-1)
    e = np.where(m > 1e-32, np.floor(np.log2(np.maximum(m, 1e-32))) + 1, 0)
    sc = np.where(m > 1e-32, 256.0 / np.exp2(e), 0)
    out = np.zeros(rgb.shape[:-1] + (4,), np.uint8)
    out[..., :3] = np.clip(rgb * sc[..., None], 0, 255).astype(np.uint8)
    out[..., 3] = np.where(m > 1e-32, e + 128, 0).astype(np.uint8)
    return out
with open(os.path.join(HERE, "room.hdr"), "wb") as f:
    f.write(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n" + f"-Y {H} +X {W}\n".encode())
    f.write(rgbe(img).tobytes())
print("room.hdr")

# ---- left wall with a window (a PNG whose hole lets the light in) ------------------------------
W, H = 1400, 1100
img = Image.new("RGBA", (W * SS, H * SS), (232, 226, 214, 255)); d = ImageDraw.Draw(img)
wx0, wy0, wx1, wy1 = 430, 260, 1010, 820
d.rectangle(s([wx0 - 26, wy0 - 26, wx1 + 26, wy1 + 40]), fill=(168, 140, 110, 255))   # frame
d.rectangle(s([wx0, wy0, wx1, wy1]), fill=(0, 0, 0, 0))                                 # the hole
mx = (wx0 + wx1) / 2; my = (wy0 + wy1) / 2
d.rectangle(s([mx - 9, wy0, mx + 9, wy1]), fill=(168, 140, 110, 255))                  # muntins
d.rectangle(s([wx0, my - 9, wx1, my + 9]), fill=(168, 140, 110, 255))
d.rectangle(s([wx0 - 40, wy1 + 30, wx1 + 40, wy1 + 58]), fill=(150, 122, 95, 255))     # sill
d.rectangle(s([0, H - 44, W, H]), fill=(138, 111, 88, 255))                             # skirting
finish(img, W, H, "left_wall.png")

# ---- the outside seen through the window: a soft garden in daylight ---------------------------
W, H = 900, 700
im = Image.new('RGB', (W, H), (196, 220, 236)); d = ImageDraw.Draw(im)
for i in range(H):
    t = i / H; d.line([(0, i), (W, i)], fill=(int(200 - 40 * t), int(222 - 20 * t), int(240 - 60 * t)))
d.ellipse([-200, 380, 600, 900], fill=(120, 160, 100)); d.ellipse([350, 420, 1200, 950], fill=(100, 145, 90))
d.ellipse([120, 220, 420, 520], fill=(90, 130, 80)); d.rectangle([255, 480, 285, 640], fill=(110, 85, 60))
im.filter(ImageFilter.GaussianBlur(6)).save(os.path.join(HERE, "outside.png")); print("outside.png")
