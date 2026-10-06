#!/bin/zsh
# usage: post.sh <frame> <stage_dir>   denoise (OIDN) then the community effect (wgsl-fx); writes <stage_dir>/out_<frame>.png
N=$1; S=$2
SHOT=/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/16_Raytracing/Bin/Release/Screenshots/16_Raytracing_stage.png
FXHOST=/Users/member_ottoto/rust_ae/_ext/wgsl-fx-target/release/wgsl-fx
EFFECT=/Users/member_ottoto/rust_ae/Motolii/work/forge_scene/fx/glow.wgsl
ffmpeg -v error -y -i $SHOT -pix_fmt rgbf32le $S/in.pfm || exit 1
oidnDenoise -d cpu --ldr $S/in.pfm --srgb -q high -o $S/dn.pfm > /dev/null || exit 1
ffmpeg -v error -y -i $S/dn.pfm -pix_fmt rgb24 $S/dn.png || exit 1
set -- $(awk -F, -v n=$N '$1==n{print $2, $3, $4}' $S/fx.csv)
$FXHOST $EFFECT $S/dn.png $S/out_$N.png threshold=$1 intensity=$2 radius=$3
