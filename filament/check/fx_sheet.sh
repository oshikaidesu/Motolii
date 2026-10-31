#!/bin/zsh
# A thing's sheet lies where the thing is: a rect given an effect that changes nothing stands where it stood, on a stage
# whose sides are no multiple of 16 (Filament pads its picture to one and tells a camera's projection: Renderer.cpp; a
# device sheet hears of it by vertexDomainDeviceJittered: surface_main.vs).
#   ./check/fx_sheet.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
cat > $W/nothing.fs <<'FS'
/*{ "INPUTS": [{ "NAME": "inputImage", "TYPE": "image" }] }*/
void main() { gl_FragColor = IMG_THIS_PIXEL(inputImage); }
FS
for way in plain effect; do
    extra=''; [[ $way == effect ]] && extra="later fx_file 1 $W/nothing.fs\n"
    printf "add shape rect\nsay 1 fill ffffff\n$extra" | ./edit.sh $W/$way.motolii > /dev/null 2>&1
    sqlite3 $W/$way.motolii "insert into fact(thing, name, at, value) values(0, 'wide', 0.0, 1000), (0, 'high', 0.0, 1000) on conflict do update set value = excluded.value;"
    printf "still $W/$way.png\n" | ./edit.sh $W/$way.motolii > /dev/null 2>&1
done
python3 -c "
from PIL import Image
def edges(path):
    im = Image.open(path).convert('L'); w, h = im.size
    across = lambda v: [i + (127.5 - v[i]) / (v[i + 1] - v[i]) for i in range(len(v) - 1) if (v[i] - 127.5) * (v[i + 1] - 127.5) < 0]
    a, b = across([im.getpixel((x, h // 2)) for x in range(w)]), across([im.getpixel((w // 2, y)) for y in range(h)])
    return a[0], a[-1], b[0], b[-1]
off = max(abs(p - e) for p, e in zip(edges('$W/plain.png'), edges('$W/effect.png')))
print('a sheet lies where its thing is: ' + ('yes' if off < 1 else 'NO') + ' (%.2f pixels off at most)' % off)"
rm -rf $W
