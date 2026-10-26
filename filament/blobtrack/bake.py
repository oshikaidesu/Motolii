# The regions a film's pictures are tracked by (k1sh0r/blob-tracker's BlobTracker, MIT, used as its README's Python
# module: frames in order, its last_tracks each picture), each track a Motolii shape: a rectangle of edge only, its
# place and size keyed each picture it is seen, unseen (opacity 0) the rest. Its edge as blob-tracker draws it
# (kit/blobtrack.py draw: 235 of 255 white, here white at 235 / 255 opacity, _lw(2), _lw(3) past 230 * k; here the widest it is, the width not keyed). The film filling a 1920 x 1080 stage, as
# catball's (sim.c: a unit of the stage 335.885 of its points, y up).
#   python3 -I blobtrack/bake.py <film> <start s> <seconds> > calls.txt   (edit.sh calls, the film thing 1)
import sys, os, math
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'kit'))
from contextlib import closing
from blobtrack import BlobTracker, probe, read_frames

WIDE, HIGH, POINT = 1920, 1080, 335.885
film, start, seconds = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
info = probe(film)
W, H, fps = info['width'], info['height'], info['fps']
class Tracker(BlobTracker):   # (its process as it is, but its overlay not drawn: only last_tracks is read here; draw was 8 of its 14 ms a picture)
    def draw(self, rgb, tracks, pts, frame_no):
        return None
bt = Tracker(W, H)
seen = {}   # (a track's id: [(t, x, y, w, h), ...])
with closing(read_frames(film, info, start=start, duration=seconds)) as frames:
    for n, rgb in enumerate(frames):
        bt.process(rgb)
        for tr in bt.last_tracks:
            seen.setdefault(tr['id'], []).append((n / fps, tr['x'], tr['y'], tr['w'], tr['h']))

print('together on')   # (all of it one step, the stage matched once at the end: 4 minutes otherwise, 38 s so, for 162 tracks of 3 s)
print(f'add film {os.path.abspath(film)}\nset 1 size {HIGH / POINT:.4f}')
# (each track cut where it was lost, and the pieces dealt to as few shapes as are ever seen at once, the earliest first
#  to the first shape free (an interval graph coloured greedily): 750 tracks of the 14 s film, 750 shapes and 32 pictures
#  a second otherwise; the large ones apart, their edge wider and not keyed)
pieces = []
for tid, at in seen.items():
    run = [at[0]]
    for p in at[1:]:
        if p[0] - run[-1][0] > 1.5 / fps: pieces.append(run); run = [p]
        else: run.append(p)
    pieces.append(run)
shapes = {False: [], True: []}
for piece in sorted(pieces, key=lambda r: r[0][0]):
    big = max(math.sqrt(w / W * WIDE * h / H * HIGH) for _, _, _, w, h in piece) > 230
    free = next((one for one in shapes[big] if one[-1][-1][0] + 1.5 / fps < piece[0][0]), None)
    (free.append(piece) if free is not None else shapes[big].append([piece]))
thing = 1
for big, ones in shapes.items():
    for one in ones:
        thing += 1
        print(f'add shape rect\nsay {thing} fill none\nsay {thing} stroke ffffff\nset {thing} stroke_width {3 if big else 2}')
        print(f'key {thing} opacity 0 0\nway {thing} opacity 0 Hold')
        keys = {}
        for piece in one:
            print(f'key {thing} opacity {piece[0][0]:.4f} {235 / 255:.4f}\nway {thing} opacity {piece[0][0]:.4f} Hold')
            for t, x, y, w, h in piece:
                u, v = (x + w / 2) / W * WIDE, (y + h / 2) / H * HIGH
                tall, wide = max(h / H * HIGH, 1.0), max(w / W * WIDE, 1.0)
                for name, value in (('x', (u - WIDE / 2) / POINT), ('y', (HIGH / 2 - v) / POINT), ('size', tall / POINT), ('scale_x', wide / tall)):
                    keys.setdefault(name, []).append(f'{t:.4f} {value:.4f}')
            off = piece[-1][0] + 1 / fps
            print(f'key {thing} opacity {off:.4f} 0\nway {thing} opacity {off:.4f} Hold')
        for name, said in keys.items():   # (each value's keys in one call: motolii_key_many, one statement made ready once)
            print(f'keys {thing} {name} ' + ' '.join(said))
print('together off')
print(f'# {len(seen)} tracks, {len(pieces)} pieces, {thing - 1} shapes', file=sys.stderr)
