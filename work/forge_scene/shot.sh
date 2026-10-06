#!/bin/zsh
# usage: shot.sh <out.png> [seconds]   launches the Forge sample, waits, captures only its window, quits it
OUT=$1; WAIT=${2:-25}
REL=/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/16_Raytracing/Bin/Release
pkill -x 16_Raytracing; sleep 1
cd $REL && rm -f 16_Raytracing.log
( MOTOLII_NOUI=1 ./16_Raytracing.app/Contents/MacOS/16_Raytracing -ApplePersistenceIgnoreState YES > /tmp/forge_run.out 2>&1 & )
sleep $WAIT
WID=$(osascript -l JavaScript -e 'ObjC.import("CoreGraphics"); var l=ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo($.kCGWindowListOptionOnScreenOnly,0))); var best=null; for (var w of l){ if(w.kCGWindowOwnerName=="16_Raytracing"){ if(!best||w.kCGWindowBounds.Width>best.kCGWindowBounds.Width) best=w; } } best? best.kCGWindowNumber : ""')
if [ -z "$WID" ]; then echo "NO WINDOW"; else screencapture -x -o -l $WID "$OUT"; echo "captured window $WID"; fi
pgrep -x 16_Raytracing >/dev/null && echo "alive" || echo "DEAD"
grep -E "ERR" $REL/16_Raytracing.log | cut -c1-200 | head -3
pkill -x 16_Raytracing
