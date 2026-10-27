# filament/ — Motolii の中身（エンジン）

*Motolii's engine (C++ / Objective-C++). Named after Filament for historical reasons; Google's Filament itself is fetched into `third_party/` at build time. The entry points are `motolii.h` (the C calls the window uses), `motolii_hand.h` (the thread that owns the document and draws) and `edit.cpp` (a shell that takes one call per line). Of the ~300 files at this level, 63 are what the current build reaches (listed below); the rest are earlier probes kept as they were.*

窓（`../skin/book/`）は、ここが組む `out/libmotolii.dylib` を C の呼び出しで使います。作りの全体は [`../ARCHITECTURE.md`](../ARCHITECTURE.md)。

## 入口

| ファイル | 何か |
|---|---|
| `motolii.h` | 窓が使う C の呼び出しの全部（書類を開く・物を足す・値を変える・キー・取り消し・書き出し・絵を受け取る） |
| `motolii.mm` | その実装（`libmotolii.dylib` になる）。中で `motolii_hand.h`（書類を持って描くスレッド）と `motolii_calls.h` を読む |
| `edit.cpp` | 1 行ずつの命令で書類をいじるシェル。`./edit.sh 書類.motolii` で組んで開く（`help` で一覧） |
| `doc.h` と `doc_*.h` | 書類（SQLite）：物・値・キー・取り消し（変更を SQLite の表に積む）・場面への反映 |
| `kit.h`・`kit_*.h`・`drawn*.h` | Filament と ThorVG で描く所 |

## 今の組み立てが使う物

`edit.sh` → `dawn_probe/kit_wgpu.sh` が組む物。`#include` を辿ると、この階層では次の 63 本です。

- 書類：`doc.h`、`doc_edit.h`、`doc_kinds.h`、`doc_looks.h`、`doc_match.h`、`doc_moment.h`、`doc_moment_fx.h`、`doc_paths.h`、`doc_place.h`、`doc_recipes.h`、`doc_relations.h`、`doc_values.h`、`interp.h`、`ways.h`
- 描く：`kit.h`、`kit_draw.h`、`kit_stage.h`、`kit_things.h`、`drawn.h`、`drawn_flat.h`、`drawn_kept.h`、`drawn_made.h`、`screen.h`、`stage_kept.h`、`picture.h`、`points.h`、`solid.h`、`live.h`、`lottie_slots.h`、`bend.cpp`、`skpath.cpp`
- 効果：`thing_fx.h`、`isf.cpp`・`isf.h`（ISF）、`aviutl.cpp`・`aviutl.h`・`aviutl_fx.h`（AviUtl2）、`splatfx.cpp`・`splatfx.h`（点群）、`three_isf.cpp`・`three_isf.h`、`solid_three.cpp`・`solid_three.h`（three.js の形）、`mosh.h`
- 動き・音・動画：`motion.h`、`physics.cpp`・`physics.h`（Jolt）、`script.cpp`・`script.h`（QuickJS のレシピ）、`godot_easing.h`、`video.h`、`gst_read.h`、`sound.h`、`heard.h`、`midi.h`、`midi_roll.cpp`・`midi_roll.h`
- 入口：`motolii.h`、`motolii.mm`、`motolii_calls.h`、`motolii_hand.h`、`motolii_write.h`、`edit.cpp`

材質（`.mat`）は `kit_wgpu.sh` の一覧の 22 本（`paper`・`flat`・`floor_lit`・`solid_image` など）と、それが読む `.glsl`。

## それ以外

この階層の残り（`ring.mm`・`poster.mm`・`neko.mm`・`beam_glass_*.h`・`glass_scene_*.inc` など）は、前の試し（Metal の Filament の SDK を使った物、ガラスや光の試し）です。消さずに置いてあり、`ring.sh` などで今も動かせます（組み立て済みの SDK は `sdk.sh` が取る）。何がどこまで動いたかは [`../ASSETS.md`](../ASSETS.md)。

| フォルダ | 中身 |
|---|---|
| `dawn_probe/` | Filament を WebGPU（Dawn）で組むスクリプトと、Filament・ThorVG・Dawn に当てる差分（`*.patch`） |
| `check/` | 中身の試験。`check/all.sh` でまとめて |
| `effects/`・`recipes/`・`animators/`・`ways/`・`looks/`・`solids/` | 同梱の効果・レシピ・動き・イーズ・ルック・形（利用者のフォルダ `~/Library/Application Support/Motolii/` と同じ形） |
| `aviutl_scripts/` | 同梱の AviUtl2 のスクリプト |
| `tsl/` | TSL と shaders.com の効果を取ってきて WGSL にする道具 |
| `third_party/` | 取ってくる依存（git の外）と、同梱の小さな物 |
