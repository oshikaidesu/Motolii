#!/bin/zsh
# The window's working eye and the camera's, two eyes on one world (After Effects' working 3D views, AbsentMotion's
# cameraMode): the working eye moves with no camera, shows no 2D thing, no fog of the camera's, and never reaches the
# document or a film.
#   ./check/world_view.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
shot() { printf "$2\n" | ./edit.sh $W/$1.motolii > /dev/null 2>&1 }
world="add solid cube\nsay 1 fill 3366ff\nadd shape rect\nsay 2 fill ff3366\nsay 2 space 3d\nset 2 x 0.9\nset 2 z -3\nadd shape rect\nsay 3 fill 33ff66\nset 3 x -0.5\nset 3 y 0.3\nadd camera eye\nset 4 rotate_y 40"
shot both "$world\nstill $W/camera.png\nlater eye 0 0 20 1\nstill $W/eye.png\nlater eye 0\nstill $W/again.png"
shot moved "$world\nset 4 rotate_y -30\nstill $W/camera2.png\nlater eye 0 0 20 1\nstill $W/eye2.png"
shot faces "add shape rect\nsay 1 fill ff3366\nsay 1 space 2.5d\nadd camera eye\nlater eye 0 0 0 1\nstill $W/face.png\nset 2 rotate_y 80\nstill $W/edge.png"
shot turned "$world\nlater eye 0 50 -10 1.5\nstill $W/eye3.png"
shot fog "$world\nset 4 fog 1\nset 4 fog_near 0\nset 4 fog_far 8\nstill $W/fog.png\nlater eye 0 0 20 1\nstill $W/fog_eye.png"
printf "add solid cube\nadd camera eye\nlater eye 0 90 0 1\nstill $W/x.png\nlater hold 1\ncarry place 120 0\nstill $W/y.png\nvalue 1 x 0\nvalue 1 z 0\n" | ./edit.sh $W/side.motolii > $W/side.txt 2>&1
shot none "add shape rect\nsay 1 fill 33ff66\nstill $W/plain.png\nlater eye 0 40 20 1\nstill $W/plain_eye.png"
shot film "$world\nset 0 duration 0.1\nwrite $W/film_camera.mp4\nlater eye 0 40 20 1\nwrite $W/film_eye.mp4"
shot kept "$world\nlater eye 0 40 20 1\nlater eye 0"; shot plain "$world"; sqlite3 $W/kept.motolii .dump > $W/kept.sql; sqlite3 $W/plain.motolii .dump > $W/plain.sql
python3 -c "
from PIL import Image, ImageChops
def changed(a, b):
    return sum(ImageChops.difference(Image.open('$W/' + a + '.png').convert('L'), Image.open('$W/' + b + '.png').convert('L')).histogram()[64:])
def green(a):
    return sum(1 for r, g, b in Image.open('$W/' + a + '.png').convert('RGB').getdata() if g > 200 and r < 120 and b < 150)
def say(what, ok, how): print(what + ': ' + ('yes' if ok else 'NO') + ' (' + how + ')')
n = changed('camera', 'again'); say('let go, the camera\'s picture is back', n == 0, '%d pixels' % n)
n = changed('camera', 'camera2'); say('the camera turned, its own picture turns', n > 2000, '%d pixels' % n)
n = changed('eye', 'eye2'); say('the camera turned, the working eye\'s picture stays', n == 0, '%d pixels' % n)
def red(a):
    return sum(1 for r, g, b in Image.open('$W/' + a + '.png').convert('RGB').getdata() if r > 200 and g < 120)
a, b = red('face'), red('edge'); say('a 2.5D sheet goes on facing the camera, seen by the working eye', a > b + 500, '%d and %d pixels' % (a, b))
n = changed('eye', 'eye3'); say('the working eye turned, its picture turns', n > 2000, '%d pixels' % n)
a, b = green('camera'), green('eye'); say('a 2D thing is in the camera\'s picture and not the world\'s', a > 500 and b == 0, '%d and %d pixels' % (a, b))
n = changed('eye', 'fog_eye'); m = changed('camera', 'fog'); say('a camera\'s fog is in its picture alone', n == 0 and m > 500, '%d in the eye, %d in the camera' % (n, m))
n = changed('plain', 'plain_eye'); say('with no camera there is one eye', n == 0, '%d pixels' % n)"
cmp -s $W/film_camera.mp4 $W/film_eye.mp4 && echo "a film is the camera's whichever eye the window has: yes" || echo "a film is the camera's whichever eye the window has: NO"
diff -q <(sed -E 's/,[0-9.]+\);$/);/' $W/kept.sql) <(sed -E 's/,[0-9.]+\);$/);/' $W/plain.sql) > /dev/null && echo "the working eye is not in the document: yes" || echo "the working eye is not in the document: NO"
cat $W/side.txt | tail -2 | tr '\n' ' ' | python3 -c "
import re, sys
n = [float(x) for x in re.findall(r'-?\d+\.?\d*(?:e-?\d+)?', sys.stdin.read())]
print('a hand moves a thing the way the working eye sees it: ' + ('yes' if len(n) >= 2 and abs(n[0]) < 1e-6 and n[1] != 0 else 'NO') + ' (x %s, z %s)' % tuple(n[:2]))"
rm -rf $W
