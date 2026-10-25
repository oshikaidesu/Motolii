#!/bin/zsh
# Every check there is, run again, what each says kept as a file a judge can read:  ../reports/ae_rubric/records/
#   ./records.sh          (about ten minutes; each run is given up after a set time, so none hangs the rest)
cd "$(dirname "$0")"
R=$PWD/../reports/ae_rubric/records
E="$(pkg-config --cflags --libs gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0 gstreamer-pbutils-1.0)"
mkdir -p $R
within() { perl -e 'alarm shift; exec @ARGV' "$@" }   # (seconds, then the command)
quiet() { grep -v "FEngine\|WebGPU adapter\|Backend feature\|libvulkan\|BackendVk\|^$" }
{ date "+%Y-%m-%d %H:%M"; git log --oneline | head -1; sw_vers -productVersion; sysctl -n machdep.cpu.brand_string } > $R/when.txt

within 300 ./motion_test.sh > $R/motion.txt 2>&1
within 900 ./ae_test.sh 2>&1 | grep "films compared" > $R/ae_files.txt; cp out/wgpu_kit/out/ae_test.tsv $R/ae_files.tsv
within 300 ./check/video_colour.sh > $R/video_colour.tsv 2>/dev/null
within 300 ./check/doc.sh > $R/doc.txt 2>&1
within 300 ./check/sound.sh > $R/sound.txt 2>&1
within 120 ./check/interp_twin.sh > $R/interp_twin.txt 2>&1   # (interp.h against the past repository's own formulas)
(cd out/wgpu_kit && rm -f ../api.motolii && BACKEND=webgpu within 60 ../api_check ../api.motolii out/api.png 2>&1 | quiet) > $R/api.txt   # (motolii.h called from plain C)

KIT_EXTRA="$E" KIT_SRC=check/video_time.mm BUILDONLY=1 ./dawn_probe/kit_wgpu.sh time_check
for f in out/vid_odd/*; do
    echo "== ${f:t}"; (cd out/wgpu_kit && BACKEND=webgpu within 120 ../time_check_wgpu ../vid_odd/${f:t} 2>&1 | quiet)
done > $R/video_time.txt

for n in wall mask puppet neko lay; do
    KIT_EXTRA="$E" BUILDONLY=1 ./dawn_probe/kit_wgpu.sh $n
    echo "== $n"; (cd out/wgpu_kit && BACKEND=webgpu within 120 ../${n}_wgpu time 2>&1 | quiet)
done > $R/speed.txt
(cd out/wgpu_kit && BACKEND=webgpu within 700 ../neko_wgpu soak ${SOAK:-600} 2>&1 | quiet) > $R/soak_neko.txt
