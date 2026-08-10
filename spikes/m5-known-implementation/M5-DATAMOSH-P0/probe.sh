#!/bin/sh
set -eu

fail() {
    printf '%s\n' "$1" >&2
    cleanup
    exit 1
}

cleanup() {
    if [ -n "${work_dir:-}" ] && [ -d "$work_dir" ]; then
        rm -rf -- "$work_dir"
    fi
}

ffmpeg_bin=${FFMPEG_BIN:-ffmpeg}
ffprobe_bin=${FFPROBE_BIN:-ffprobe}
command -v "$ffmpeg_bin" >/dev/null 2>&1 || fail "DATAMOSH_TOOL_MISSING: ffmpeg"
command -v "$ffprobe_bin" >/dev/null 2>&1 || fail "DATAMOSH_TOOL_MISSING: ffprobe"

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/motolii-datamosh-p0.XXXXXX")
trap 'cleanup; exit 1' HUP INT TERM
source_mp4="$work_dir/source.mp4"
source_copy="$work_dir/source-copy.mp4"
output_a="$work_dir/datamosh-a.mp4"
output_b="$work_dir/datamosh-b.mp4"

# 二つ目のGOP先頭だけを落とし、後続P-frameを一つ目の参照鎖へ残す。
"$ffmpeg_bin" -hide_banner -loglevel error -y \
    -f lavfi -i "testsrc2=s=96x64:r=10:d=2" \
    -vf "hue=h='if(lt(t,1),0,90)'" \
    -c:v mpeg4 -q:v 4 -g 10 -bf 0 -threads 1 \
    -map_metadata -1 -metadata creation_time=1970-01-01T00:00:00Z \
    "$source_mp4"
cp "$source_mp4" "$source_copy"

source_packets="$work_dir/source-packets.csv"
"$ffprobe_bin" -v error -select_streams v:0 \
    -show_entries packet=pts_time,flags -of csv=p=0 "$source_mp4" > "$source_packets"
[ "$(wc -l < "$source_packets" | tr -d ' ')" = 20 ] || fail "DATAMOSH_FIXTURE_INVALID: packet count"
[ "$(awk -F, '$2 ~ /K/ { print NR - 1 }' "$source_packets")" = "$(printf '0\n10')" ] || fail "DATAMOSH_FIXTURE_INVALID: key packet layout"

run_datamosh() {
    output=$1
    "$ffmpeg_bin" -hide_banner -loglevel error -y -i "$source_mp4" \
        -map 0:v:0 -c:v copy -bsf:v "noise=drop='eq(n,10)*key'" \
        -map_metadata -1 -metadata creation_time=1970-01-01T00:00:00Z \
        "$output"
}

run_datamosh "$output_a"
run_datamosh "$output_b"
cmp "$source_mp4" "$source_copy" >/dev/null || fail "DATAMOSH_SOURCE_MUTATED"
cmp "$output_a" "$output_b" >/dev/null || fail "DATAMOSH_NONDETERMINISTIC_OUTPUT"

output_packets="$work_dir/output-packets.csv"
"$ffprobe_bin" -v error -select_streams v:0 \
    -show_entries packet=pts_time,flags -of csv=p=0 "$output_a" > "$output_packets"
[ "$(wc -l < "$output_packets" | tr -d ' ')" = 19 ] || fail "DATAMOSH_PACKET_ORACLE_FAILED: packet count"
! awk -F, '$1 == "1.000000" { found = 1 } END { exit !found }' "$output_packets" || fail "DATAMOSH_PACKET_ORACLE_FAILED: target key packet survived"
[ "$(awk -F, '$2 ~ /K/ { count++ } END { print count + 0 }' "$output_packets")" = 1 ] || fail "DATAMOSH_PACKET_ORACLE_FAILED: unexpected key packet count"

"$ffmpeg_bin" -hide_banner -loglevel error -i "$output_a" -f null -
decoded_frames=$("$ffprobe_bin" -v error -count_frames -select_streams v:0 \
    -show_entries stream=nb_read_frames -of default=nw=1:nk=1 "$output_a")
[ "$decoded_frames" = 19 ] || fail "DATAMOSH_DECODE_FAILED: decoded_frames=$decoded_frames"

"$ffmpeg_bin" -hide_banner -loglevel error -i "$source_mp4" -f framemd5 - > "$work_dir/source.md5"
"$ffmpeg_bin" -hide_banner -loglevel error -i "$output_a" -f framemd5 - > "$work_dir/output.md5"
source_frame=$(awk -F, '$1 !~ /^#/ && $3 + 0 == 11 { gsub(/ /, "", $6); print $6 }' "$work_dir/source.md5")
output_frame=$(awk -F, '$1 !~ /^#/ && $3 + 0 == 11 { gsub(/ /, "", $6); print $6 }' "$work_dir/output.md5")
[ -n "$source_frame" ] && [ -n "$output_frame" ] || fail "DATAMOSH_FRAME_ORACLE_FAILED: missing pts 1.1"
[ "$source_frame" != "$output_frame" ] || fail "DATAMOSH_FRAME_ORACLE_FAILED: reference loss had no decoded effect"

printf 'DATAMOSH_PROBE_OK packets=19 decoded_frames=19 changed_pts=1.1\n'
cleanup
