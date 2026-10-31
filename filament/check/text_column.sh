#!/bin/zsh
# A text's size is a line's, however many lines it makes (After Effects' and Alight Motion's Font Size): its letters are as
# large stood in a column as in one line, and the column three letters tall is about three times as tall.
#   ./check/text_column.sh
cd "$(dirname "$0")/.."
W=$(mktemp -d)
python3 - $W <<'P'
import subprocess,sys
from PIL import Image
W=sys.argv[1]
r=subprocess.run(['zsh','./edit.sh',f'{W}/d.motolii'],input='\n'.join(['add shape rect','say 1 fill ffffff','set 1 size 20','add text /System/Library/Fonts/Supplemental/Impact.ttf','say 2 text HHH','say 2 fill 111111','set 2 size 0.3','at 0.5',f'still {W}/line.png','say 2 wrap vertical',f'still {W}/column.png'])+'\n',capture_output=True,text=True,timeout=60)
def box(n):
    a,b,c,d=Image.open(f'{W}/{n}.png').convert('L').point(lambda v:255 if v<128 else 0).getbbox(); return c-a,d-b
(lw,lh),(cw,ch)=box('line'),box('column')
print(f'a line {lw} by {lh}, a column {cw} by {ch}')
print('a letter is as large in a column:','yes' if abs(cw-lw/3)<.12*lw/3 else 'NO')
print('a column of three is three letters tall:','yes' if 2.6*lh<ch<3.6*lh else 'NO')
P
rm -rf $W
