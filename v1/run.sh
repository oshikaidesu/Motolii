#!/bin/zsh
pkill -f Motolii.app; sleep 0.5
export MOTOLII_SKIN=/Users/member_ottoto/rust_ae/Motolii/v1/skin/build/macos/Build/Products/Debug/stage_skin.app/Contents/Frameworks/App.framework
export MOTOLII_SVG=/Users/member_ottoto/rust_ae/Motolii/work/forge_scene/assets/sunface.svg
export MOTOLII_VIDEO=/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/probe/avf/intra.mov
(/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/Motolii/Bin/Release/Motolii.app/Contents/MacOS/Motolii -ApplePersistenceIgnoreState YES > /private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/run.log 2>&1 &)
sleep ${1:-9}
grep -E "MOTOLII|svg:|xception|dyld|rash" /private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/run.log | tail -5
pgrep -f Motolii.app >/dev/null && echo RUNNING || tail -6 /private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/run.log
W=$(osascript -l JavaScript -e 'ObjC.import("CoreGraphics");var l=ObjC.deepUnwrap(ObjC.castRefToObject($.CGWindowListCopyWindowInfo(1,0)));l.filter(w=>w.kCGWindowOwnerName=="Motolii"&&w.kCGWindowBounds.Width>1000).map(w=>w.kCGWindowNumber)[0]')
screencapture -l $W -x /private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/m.png; sips -Z 1100 /private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/39997739-b037-4310-b366-cc28b56b68f1/scratchpad/m.png >/dev/null
echo "elapsed: $(( ($(date +%s) - $(date -j -f %H:%M:%S 16:09:54 +%s)) / 60 )) min of 30"
