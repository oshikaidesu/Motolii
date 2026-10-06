#!/bin/zsh
# usage: export.sh <start> <end> <fps> <spp> <out.mp4> [width height]
# The Forge renders each frame (accumulate <spp> samples, deterministic), ffmpeg muxes the picture with the source clip's audio.
START=$1; END=$2; FPS=$3; SPP=$4; OUT=$5; W=${6:-1280}; H=${7:-720}
SRC="/Users/member_ottoto/コラ素材meme/第１回【超底辺】YouTuber（高校生）の財布の中身を、いきなり、チェックしたら、やばすぎ、、。.mp4"
AT=${MOTOLII_VIDEO_AT:-12}
REL=/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/16_Raytracing/Bin/Release
WORK=/Users/member_ottoto/rust_ae/_ext/render_out
pkill -x 16_Raytracing; sleep 1
rm -rf $WORK; mkdir -p $WORK; cd $WORK
SHOTS=$REL/Screenshots   # the Forge writes captures next to the .app
rm -rf $SHOTS; mkdir -p $SHOTS
MOTOLII_NOUI=1 MOTOLII_VIDEO="$SRC" MOTOLII_VIDEO_AT=$AT MOTOLII_BATCH="$START,$END,$FPS,$SPP" \
  $REL/16_Raytracing.app/Contents/MacOS/16_Raytracing -ApplePersistenceIgnoreState YES -w $W -h $H > $WORK/run.out 2>&1
N=$(ls $SHOTS/*.png 2>/dev/null | wc -l | tr -d ' ')
echo "frames written: $N"
[ "$N" -gt 0 ] || { echo "no frames"; exit 1; }
DUR=$(python3 -c "print(($END-$START+1)/$FPS)")
T0=$(python3 -c "print($AT+$START/$FPS)")
# OIDN (existing owner of denoising): per frame png -> pfm -> denoise (CPU) -> png
DN=$WORK/dn; mkdir -p $DN
for f in $SHOTS/16_Raytracing_f*.png; do
  b=$(basename $f .png)
  ffmpeg -v error -y -i $f -pix_fmt rgbf32le $DN/$b.in.pfm
  oidnDenoise -d cpu --ldr $DN/$b.in.pfm --srgb -q high -o $DN/$b.out.pfm > /dev/null
  ffmpeg -v error -y -i $DN/$b.out.pfm -pix_fmt rgb24 $DN/$b.png
  rm -f $DN/$b.in.pfm $DN/$b.out.pfm
done
ffmpeg -v error -y -framerate $FPS -i $DN/16_Raytracing_f%06d.png -ss $T0 -t $DUR -i "$SRC" -map 0:v -map "1:a?" -c:v libx264 -crf 14 -pix_fmt yuv420p -c:a aac -b:a 192k -shortest -movflags +faststart "$OUT" && ls -la "$OUT" | awk '{print $5, $9}'
