---
name: motolii-export
description: Export a Motolii document to a movie (range, audio mixed in, ffmpeg/libx264), a still, or Lottie JSON — from the UI ops or offline with zz_watch + ffmpeg. Use when someone wants an mp4/png out, or an export fails, or wants to verify one with ffprobe.
---

# Motolii export (2026-09-19)

## Contract

- **Entry points** (`motolii/crates/motolii-render/src/export.rs`): `export` (70, whole comp), `export_range` (88),
  `export_range_with_progress(engine, view, job, range, cancel, on_progress)` (118) — the one everything calls; `export_still(engine, view,
  frame, path)` (177, PNG via `image::save_buffer`). `ExportJob { out_path, qp0 }` (36). `Cancel` (54) is an `AtomicBool` checked every
  frame (152) and every audio chunk (321).
- **Per frame** (151-166): `engine.render_frame(view, t)` → RGBA8 sRGB (`FrameDesc::try_packed` 133-140) → `Encoder::write_frame`.
  Progress = `{ frames_done, frames_total }`.
- **Encoder** (`src/media/encode.rs:45-101`): ffmpeg on PATH via `media::tool_command` (`ffmpeg_bin()`), rawvideo rgba on stdin, `-r num/den`,
  `-c:v libx264`; `qp0 = false` → `-crf 18 -pix_fmt yuv420p`, `qp0 = true` → `-qp 0 -pix_fmt yuv444p`; always
  `-vf scale=out_color_matrix=bt709:out_range=tv` + bt709 primaries/trc/colorspace, `-color_range tv`. Container = the out path's extension
  (ffmpeg decides; `.mp4` / `.mov` both work with libx264). No ffmpeg → `MediaError::ToolNotFound("ffmpeg")` (media.rs:79).
  `finish` (133) closes stdin, reads stderr, fails on a non-zero exit; a broken pipe surfaces ffmpeg's stderr (117-128).
- **Audio** (`render_audio_input` 295-339): `AudioProgram::from_view_blocking` (`src/audio/program.rs:202`) collects every layer whose file can
  carry audio (`file_source_can_have_audio` 126: asset type `audio/*` or `video/*`, unknown extensions are tried); if there is none,
  no audio track. Otherwise it mixes the range in 48 000-frame chunks with `program.mix_audio` (228) into a temp `f32le` file
  (`$TMPDIR/motolii-export-audio-<pid>-<n>.f32le`, 341-362) at the canonical 48 kHz stereo (`audio/convert.rs:4-5`), and the encoder adds
  `-f f32le -ar 48000 -ac 2 -i <tmp> -map 1:a:0 -c:a aac -b:a 192k -shortest` (encode.rs:61-67). The temp file is deleted on drop (221-225).
  Layer `Level`, `Pan`, `Fade In/Out`, `Time Remap`, `Speed` (names.rs:25-30) are the mix's inputs — export hears what playback hears.
- **Temp output** (`TempOutput` 227-293): writes `.<stem>.export-<pid>-<n>.<ext>` next to the destination, renames on success, removes it on
  cancel/failure — a half movie never sits at the destination path.
- **UI ops** (`motolii/ui/native/src/port.rs:235-236`, controller `ui/extensions/jobs/src/export.rs`): `{"op":"export","path":…,"start":f,"end":f}`
  (end exclusive, must be inside the comp and non-empty, 18); snapshot = `document.flattened()` (19) rendered on a thread named
  `motolii-port-export` with a fresh `Engine` and `qp0: false` (25). `exportStatus` (read from `status`) → `{ phase: idle | running |
  cancelling | complete | cancelled | failed, done, total, path, error }` (13); `cancelExport` (14). One export at a time (16).
- **Lottie** (`src/export/lottie.rs`): `export_lottie(view) -> LottieExport { json, unsupported: Vec<UnsupportedForLottie> }` (64). No UI op
  and no CLI call it today — it is exercised by `ui/native/src/editor/text_format.rs:113` and `motolii/tests/matte_relationship.rs`.
  Unrepresentable easings return `LottieExportError::UnrepresentableEasing` (47-50).

## Limits

- Frame format is fixed RGBA8 (encode.rs:53-55 refuses anything else); resolution = comp; fps = comp's rational fps.
- Odd sizes: libx264 + yuv420p needs even width/height — set an even comp (the probe's hint at `media/probe.rs:402` says the same for inputs).
- Audio is `-shortest`: a soundtrack longer than the range is cut, shorter is padded by nothing (silence in the mix, since the mix covers the range).
- Video inputs must be probe-supported (`media/probe.rs:249-276`, audio codec list); anything else fails at `AudioProgram` build.
- Blocks, ropes and physics render exactly as in playback because the same `Engine::render_frame` is used; RopePass state continues only for
  consecutive frames (`block_program.rs:770`), which an export range always is.

## Offline alternative (no UI, frames on disk)

```sh
cargo build --profile watch -p motolii-render --example zz_watch          # once, after any Rust change
MOTOLII_LAST=299 MOTOLII_STEP=1 motolii/target/watch/examples/zz_watch /abs/shot.rrd /abs/frames   # 0000.png … 0299.png, then poll `shot:` and pkill
ffmpeg -y -framerate 30 -i /abs/frames/%04d.png -i /abs/music.wav -map 0:v -map 1:a -c:v libx264 -crf 18 -pix_fmt yuv420p -c:a aac -b:a 192k -shortest /abs/out.mp4
```
`MOTOLII_SHRINK` must be 1 (or omitted) for a full-size movie (zz_watch.rs:37,47). This path does not mix layer Level/Pan/Fade — pass the
audio file yourself. For a still: `export_still` or just take one `NNNN.png`.

## Verification

```sh
ffprobe -v error -show_entries stream=codec_type,codec_name,width,height,r_frame_rate,pix_fmt,color_space,sample_rate,channels -of default=nw=1 out.mp4
ffprobe -v error -show_entries format=duration -of csv=p=0 out.mp4        # = (end − start) / fps
```
Expect `h264`, `yuv420p`, `bt709`, and with audio `aac`, `48000`, `2`. Tests: `cargo test -p motolii-render --lib export`,
`cargo test -p motolii-render --lib encode` (uses `Encoder::open_with_command` with a fake ffmpeg, encode.rs:35).
