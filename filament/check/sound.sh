#!/bin/zsh
# Is a film's own sound in the written film, in time with its picture? The puppet film (two clips with sound, one of
# them shorter than the film, so it goes round), and a document whose film comes on 1.5 seconds in, cut from its second
# second, twice as fast: the written sound against the files' own, by where the two match best.   check/sound.sh
cd "$(dirname "$0")/.."
W=out/wgpu_kit/out
rm -f $W/puppet.mp4 $W/sound_doc.mp4 out/sound_check.motolii
mkdir -p out/vid_two   # (two clips with sound, always the same two, whatever else is in out/vid)
ln -sf ../vid/happy.mp4 out/vid_two/happy.mp4; ln -sf ../vid/mohsen.mp4 out/vid_two/mohsen.mp4
FILMS=../vid_two ./puppet.sh film > /dev/null 2>&1
printf 'add film ../vid/mohsen.mp4\nset 1 start 1.5\nset 1 begin 2\nset 1 speed 2\nwrite out/sound_doc.mp4\n' | ./edit.sh out/sound_check.motolii > /dev/null 2>&1
for n in $W/puppet.mp4 $W/sound_doc.mp4 out/vid/happy.mp4 out/vid/mohsen.mp4; do ffmpeg -hide_banner -loglevel error -y -i $n -vn -ac 1 -ar 8000 -f f32le $W/${n:t:r}.f32 || echo "no sound in $n"; done
ffmpeg -hide_banner -loglevel error -y -ss 2 -i out/vid/mohsen.mp4 -vn -af atempo=2 -ac 1 -ar 8000 -f f32le $W/mohsen_cut.f32
python3 - $W $(ffprobe -v error -show_entries format=duration -of csv=p=0 out/vid/mohsen.mp4) <<'PY'
import sys, numpy as np
W, R, round_ = sys.argv[1], 8000, float(sys.argv[2])
o, d, h, m, c = (np.fromfile(f'{W}/{n}.f32', dtype=np.float32) for n in ('puppet', 'sound_doc', 'happy', 'mohsen', 'mohsen_cut'))
def off(a, b, span=4000):   # where b lies in a: milliseconds, and how alike they are there
    n = min(len(a), len(b)); a = a[:n]; b = b[:n]
    at = lambda k: float(np.dot(a[max(0, k):n + min(0, k)], b[max(0, -k):n - max(0, k)]))
    best = max(range(-span, span + 1, 4), key=at)
    return best / R * 1000, at(best) / (np.linalg.norm(a) * np.linalg.norm(b) + 1e-9)
rows = [('puppet film, against happy.mp4', off(o, h)), ('puppet film, against mohsen.mp4', off(o, m)),
        ('puppet film where mohsen.mp4 has gone round, against its start', off(o[int(round_ * R):], m)),
        ('document: film on at 1.5 s, cut from 2 s, twice as fast, against that cut', off(d[int(1.5 * R):], c)),
        ('document: before the film comes on (loudest, of 1)', (0.0, float(np.abs(d[:int(1.45 * R)]).max())))]
for name, (ms, alike) in rows[:4]: print('%-74s off by %5.1f ms, alike %.2f: %s' % (name, ms, alike, 'in time' if abs(ms) < 20 and alike > 0.2 else 'NOT in time'))
print('%-74s %.4f: %s' % (rows[4][0], rows[4][1][1], 'silent' if rows[4][1][1] < 0.001 else 'NOT silent'))
PY
ffmpeg -hide_banner -loglevel error -y -f lavfi -i "sine=frequency=440:duration=3" -b:a 128k out/sound_check.mp3
printf 'add film out/sound_check.mp3\n' | ./edit.sh out/sound_check_mp3.motolii 2>&1 | grep -q 'add: done' && echo 'an .mp3 is placed though it says no length of its own: yes' || echo 'an .mp3 is placed though it says no length of its own: NO'
rm -f out/sound_check.mp3 out/sound_check_mp3.motolii
