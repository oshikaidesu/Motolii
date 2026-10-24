# A picture that is not like the ones either side of it (a panel drawn black for one picture, a flash): found in a
# recording of the screen. Two neighbours alike and the one between unlike both is a flicker; a moving picture is
# unlike its neighbours too, but they are unlike each other as much, so it is not one.
#   python3 -I tool/flicker.py rec.mp4 [out_dir]        (record:  python3 -I tool/flicker.py --record 5 rec.mov)
# Says each flicker (its picture's number and how far out it is) and writes it, with the pictures either side, as one
# .png into out_dir.
import os, statistics, subprocess, sys

W, H = 192, 120


def record(seconds, to):   # (the whole screen by the system's own recorder: near 60 a second, where ffmpeg's took 16)
    subprocess.run(['screencapture', '-v', f'-V{int(seconds)}', '-x', to], check=True)


def pictures(film):
    raw = subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', film, '-vf', f'fps=60,scale={W}:{H},format=gray', '-f', 'rawvideo', '-'],
                         capture_output=True, check=True).stdout
    n = W * H
    return [raw[i:i + n] for i in range(0, len(raw) - n + 1, n)]


def apart(a, b):   # (how unlike two pictures are: the share of their points that moved by more than a little)
    return sum(abs(x - y) > 24 for x, y in zip(a, b)) / len(a)


def main():
    if sys.argv[1] == '--record':
        record(float(sys.argv[2]), sys.argv[3])
        sys.argv[1:] = sys.argv[3:]
    film, out = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else os.path.splitext(sys.argv[1])[0] + '_flicker'
    all_ = pictures(film)
    kept = [0]   # (a picture the capture repeated is one picture: kept once)
    for i in range(1, len(all_)):
        if all_[i] != all_[kept[-1]]:
            kept.append(i)
    seen = [all_[i] for i in kept]
    out_by = []
    for k in range(1, len(seen) - 1):
        out_by.append((min(apart(seen[k - 1], seen[k]), apart(seen[k], seen[k + 1])) - apart(seen[k - 1], seen[k + 1]), k))
    if not out_by:
        return print('too short')
    usual = statistics.median(abs(o) for o, _ in out_by)
    found = [(o, k) for o, k in out_by if o > max(0.02, 8 * usual)]   # (a share of the screen unlike both sides, far past the usual)
    print(f'{len(all_)} pictures recorded, {len(seen)} different; usual {usual:.4f}; flickers {len(found)}')
    os.makedirs(out, exist_ok=True)
    for o, k in sorted(found, reverse=True):
        n = kept[k]
        print(f'  picture {n}: {o:.3f} of the screen unlike both sides')
        sel = '+'.join(f'eq(n,{kept[j]})' for j in (k - 1, k, k + 1))
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-i', film, '-vf', f"fps=60,select='{sel}',scale=980:-1,tile=3x1",   # (counted as pictures() counts them: 60 a second)
                        '-frames:v', '1', f'{out}/flicker_{n:05d}.png'], check=True)


main()
