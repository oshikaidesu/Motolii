# A poster in a riso "Shape Studies" manner, laid out in a 736x920 reference frame, written as edit calls.
import math
S = 1080 / 736          # reference px -> stage px
U = 595.6               # stage px per unit
TOP = (1920 - 920 * S) / 2
def X(rx): return (rx * S - 540) / U
def Y(ry): return -((ry * S + TOP) - 960) / U
def L(r): return r * S / U
out = ["say 0 background f2eee5"]
n = [0]
def thing(lines):
    n[0] += 1
    out.extend(l.replace("@@", str(n[0])) for l in lines)
    return n[0]
def rect(x0, y0, x1, y1, fill, round_=0.15, extra=()):
    w, h = x1 - x0, y1 - y0
    return thing(["add shape rect", f"say @@ fill {fill}", f"set @@ size {L(h):.4f}", f"set @@ wide {w / h:.4f}", f"set @@ round {round_}",
                  f"set @@ x {X((x0 + x1) / 2):.4f}", f"set @@ y {Y((y0 + y1) / 2):.4f}", *extra])
def circle(cx, cy, r, fill, extra=()):
    return thing(["add shape circle", f"say @@ fill {fill}", f"set @@ size {L(2 * r):.4f}", f"set @@ x {X(cx):.4f}", f"set @@ y {Y(cy):.4f}", *extra])
def grain(angle, size=1.3):
    return [f"set @@ gradient_angle {angle}", "say @@ fx ../../effects/grain.wgsl", f"set @@ js_grain_size {size}"]
Y_, R, B, P, G, K = "f6d740", "ef4b26", "2f63df", "f4a2cf", "1fb27a", "121212"
def fade(c, a=180): return f"{c}ff {c}00 {c}00"
# the yellow frame
rect(230, 133, 632, 333, Y_, 0.2); rect(62, 330, 237, 882, Y_, 0.12); rect(225, 690, 420, 882, Y_, 0.16); rect(200, 300, 300, 420, Y_, 0.0)
rect(560, 250, 632, 333, Y_, 0.2, grain(0) and [f"say @@ fill {fade(G)}", *grain(0)])
# top: red circle, blue to pink block, pink circle
circle(272, 135, 97, R); circle(272, 135, 97, fade(K), grain(225))
rect(230, 135, 400, 332, P, 0.1); rect(230, 135, 400, 332, fade(B), [*grain(270)][:0] or 0.1, grain(270))
circle(468, 268, 100, P); circle(468, 268, 100, fade(B), grain(90)); circle(468, 268, 100, f"{K}00 {R}ff", grain(0))
# green hourglass with blue and black at its foot
rect(52, 140, 130, 218, G, 0.3, ["set @@ rotate 45"]); rect(70, 210, 112, 300, G, 0.0); circle(90, 340, 62, G); circle(90, 340, 62, f"{B}00 {B}ff {K}ff", grain(90))
# the middle: red circle under a blue blob
circle(232, 500, 98, R); circle(232, 500, 98, fade(K), grain(0))
circle(325, 495, 95, B); circle(460, 480, 95, B); rect(240, 405, 552, 590, B, 0.25); circle(385, 690, 98, B); circle(385, 690, 98, f"{K}ff {K}00", grain(315))
rect(290, 690, 420, 790, P, 0.15); rect(290, 690, 420, 790, f"{P}00 {R}ff", 0.15, grain(0))
# right: pink panel with red at its foot, the green blob
rect(500, 465, 668, 702, P, 0.08); rect(500, 465, 668, 702, f"{P}00 {R}00 {R}ff", 0.08, grain(0)); circle(560, 520, 60, f"{G}ff {G}00", grain(270))
rect(595, 655, 708, 765, G, 0.15); rect(595, 655, 708, 765, f"{K}ff {K}00", 0.15, grain(270)); circle(548, 822, 60, G); rect(495, 740, 610, 862, G, 0.2); circle(548, 822, 60, f"{B}00 {B}ff", grain(0))
# green and black at the foot of the frame
rect(62, 640, 237, 882, f"{G}00 {G}ff", 0.12, grain(0)); rect(225, 760, 420, 882, f"{G}00 {G}ff", 0.16, grain(0))
# outlines
outlines = [(447, 85, 587, 226), (608, 377, 698, 537), (33, 557, 122, 646)]
# scribbles (points of the picture from the top left)
def px(rx, ry): return (rx * S, ry * S + TOP)
def pen(points):
    first = True
    for (rx, ry) in points:
        x, y = px(rx, ry)
        out.append(f"pen {0 if first else n[0] + 1} {x:.1f} {y:.1f}")
        first = False
    n[0] += 1
    out.extend([f"say {n[0]} fill none", f"say {n[0]} stroke 121212", f"set {n[0]} stroke_width 3"])
for (x0, y0, x1, y1) in outlines:
    pen([(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)])
pen([(598, 248), (690, 160), (640, 230), (660, 232), (650, 270), (690, 285)])
pen([(28, 488), (75, 445), (80, 470), (95, 450), (112, 465)])
pen([(282, 625), (430, 528), (440, 545), (460, 535), (472, 550)])
pen([(570, 860), (630, 820), (625, 840), (640, 830)])
# a burst of rays
cx, cy = 128, 800
for k in range(16):
    a = k * math.pi / 8
    out_n = n[0]
    out.append(f"pen 0 {px(cx, cy)[0]:.1f} {px(cx, cy)[1]:.1f}")
    out.append(f"pen {out_n + 1} {px(cx + 65 * math.cos(a), cy + 65 * math.sin(a))[0]:.1f} {px(cx + 65 * math.cos(a), cy + 65 * math.sin(a))[1]:.1f}")
    n[0] += 1
    out.extend([f"say {n[0]} fill none", f"say {n[0]} stroke 121212", f"set {n[0]} stroke_width 2.4"])
# words
def words(text, cx, cy, size, font="../arial_bold.ttf"):
    return thing([f"add text {font}", f"say @@ text {text}", f"say @@ fill 121212", f"set @@ size {L(size):.4f}", f"set @@ x {X(cx):.4f}", f"set @@ y {Y(cy):.4f}"])
words("Shape", 84, 38, 46); words("Studies", 628, 38, 46)
words("#086", 30, 884, 14); words("10/19/2026", 58, 900, 14); words("motolii", 680, 898, 15)
out += ["still out/poster_0.png"]
print("\n".join(out))
