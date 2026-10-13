# 資産台帳

**作る前にここを見る。「無い」と思ったら、ここと過去の倉庫を探してから言う。**
最終更新：2026-10-14。足した物・消した物があったら、その時にここも直す。

状態の言葉：
- **今の土台で動く** ＝ Filament を Dawn（WebGPU）で動かす今の作りで、検査か動画で確かめてある。
- **前の土台で動いた** ＝ 作って動かした事はあるが、今の土台（Dawn）では確かめていない。
- **読んだだけ** ＝ 中身は読んだが、動かしていない。
- **未確認** ＝ 在る事だけ分かっている。

場所は `/Users/motolii_maker/motolii`（今）と `/Users/motolii_maker/rust_ae/Motolii`（過去の倉庫。**読むだけ。中の物を書き換えない・走らせない**。決まりは `~/rust_ae/HANDOFF.md`）。

## 1. 動く絵をガワに渡す（コピーなし）

**もう在る。** Filament が GPU の上の絵の置き場（CVPixelBuffer）に直接描いて、Flutter が同じ置き場をテクスチャとして貼る。CPU を通さない。

| 何 | どこ | 状態 |
|---|---|---|
| OS ごとに違う所だけの口（絵の置き場3枚と「描き終わった」の知らせ） | `filament/glass_door.h` | 前の土台で動いた |
| その Mac 版：Filament の出口を CVPixelBuffer に差し替える（Metal） | `filament/glass_door_mac.mm` | 前の土台で動いた |
| C の顔：描く係を始める、数を1個渡す、最後の絵を渡す（`glass_start`、`glass_env`、`glass_front`、`glass_frames`） | `filament/glass_live.h`、`glass_live.mm` | 前の土台で動いた |
| ライブラリにするビルド（`out/libglass_live.dylib`） | `filament/build_live.sh` | 前の土台で動いた |
| ガワ無しで確かめる机上の検査 | `filament/glass_live_test.mm` | 前の土台で動いた |
| Flutter 側：その置き場をテクスチャとして登録して貼る（`GlassTexture`、`FlutterTexture`、`copyPixelBuffer`、`textureFrameAvailable`、チャンネル `motolii/glass`） | `skin/book/macos/Runner/MainFlutterWindow.swift` | 前の土台で動いた |
| 貼る部品：絵を出して、ドラッグとホイールと数を送る | `skin/book/lib/sets/glass_live.dart` | 前の土台で動いた |
| 開く：`skin/book/glass.sh`（Widgetbook の Glass › Live） | `skin/book/glass.sh` | 前の土台で動いた |
| Windows の道筋（古い土台の物と、今の土台の物の両方） | `filament/WINDOWS.md` | 読んだだけ（何もビルドしていない） |

今の土台に持ってくる時に要る事：描いているのは古いガラスの場面（`glass_scene.mm`、Metal）。今の舞台（`kit.h`、Dawn）の絵をこの置き場に描かせる所がまだ無い。Debug ビルドの Mac だけ。

## 2. ガワ（Flutter）

| 何 | どこ | 状態 |
|---|---|---|
| Flutter 本体 | `~/rust_ae/Motolii/.tools/flutter` | 前の土台で動いた（`glass.sh` がここを使う） |
| 部品の棚（Widgetbook）：19 コンポーネント、約 166 の使い方、約 1.8 万行。数値の井戸・スイッチ・スライダー・色・タブ・タイムライン（行、バー、キー、再生ヘッド、定規）・ブラウザ・舞台のふち・窓 | `skin/book/lib/`（`tokens.dart`、`kit.dart`、`sets/`、`parts/`） | 前の土台で動いた（見た目の部品。中身にはつながっていない） |
| 見本の窓（製品ではない）：机（`desk.dart`、`desk_ease.dart`、`desk_depth.dart`）、ドック（ファイル、フォント、エフェクト、素材）、舞台 | `skin/book/lib/workspace/` | 前の土台で動いた |
| 間の定義を触る習作（イージング、ウィグル） | `skin/book/lib/sets/studies/`（`panel_inspector_a_ease.dart`、`op_04_wiggle.dart`、`pop_04_wiggle.dart` ほか 160 本） | 未確認（在る事だけ） |
| 設計の紙 | `skin/DESIGN.md`、`skin/OVERVIEW.md`、`skin/PROCESS.md` | — |
| **過去の製品の窓**：橋、セッション、タイムライン、インスペクタ、舞台、机（ease、blend、depth、history、relations、notes） | `~/rust_ae/Motolii/motolii/ui/lib/` | 読んだだけ |
| 過去のスキンと中身の間の約束：命令 JSON 約 90 個、返事の「様子」の JSON、絵のテクスチャ番号 | `~/rust_ae/Motolii/motolii/ui/lib/bridge/protocol.dart`、`ui/native/src/port.rs`、`snapshot.rs` | 読んだだけ。対応表は `reports/ae_rubric/past_document_map.md` |
| 過去の「Ease の机」と、間の種類のいつもの数 | `~/rust_ae/Motolii/motolii/ui/lib/desks/ease/`、`ui/native/src/editor/ease_kinds.rs` | 読んだだけ。数は `filament/interp.h` に持ってきた |

## 3. 中身（今の土台：Filament を Dawn で）

| 何 | どこ | 状態 |
|---|---|---|
| 舞台と道具箱（物を置く、ガラス、型、時間、絵と動画の書き出し、測る） | `filament/kit.h` | 今の土台で動く |
| 動画（GStreamer。どの形式でも、切り出し・速さ・逆再生、色、音、コピーなしで板に） | `filament/video.h` | 今の土台で動く（Mac だけ） |
| AEの絵・写真・打った文字（ThorVG） | `filament/drawn.h` | 今の土台で動く |
| 3Dのファイル（Filament の gltfio） | `filament/solid.h` | 今の土台で動く |
| AEの動きを物に渡す（3Dレイヤー、カメラ） | `filament/motion.h`、ThorVG へのパッチ | 今の土台で動く |
| 間の定義9種類（過去の倉庫の写し） | `filament/interp.h` | 今の土台で動く（倉庫の式と差 0） |
| 文書（SQLite に事実を並べる）、取り消し、キーと間 | `filament/doc.h` | 今の土台で動く。**形と名前を過去の倉庫に揃える途中** |
| 編集の口（C の関数）と、その客（行を読む道具、素の C） | `filament/motolii.h`、`motolii.mm`、`edit.cpp`、`check/api.c`、`edit.sh` | 今の土台で動く |
| 板の材質（色抜き、型、重ね方4つ、動画の色） | `filament/flat.mat`、`matte.mat`、`matte.glsl` | 今の土台で動く |
| 場面の見本：民衆、人形劇、マスク、重ね方、壁、AE再生、AEの動き、文書 | `filament/neko.mm`、`puppet.mm`、`mask.mm`、`lay.mm`、`wall.mm`、`ae.mm`、`led.mm` | 今の土台で動く。動画は `reports/show/` |
| ビルド：今の土台用 | `filament/dawn_probe/kit_wgpu.sh`、`tvg_on_dawn.sh` | 今の土台で動く |
| 人の物への手入れ：Filament（計算の口、機械を借りる口、外の絵を受ける口、動画の共有の機能）、ThorVG（3つ目の数 z） | `filament/dawn_probe/filament_dawn_compute.patch`、`thorvg_z.patch`、`gpu_door.*` | 今の土台で動く。ThorVG の物は本家に送らない |

## 4. ガラスと光（残すと決まっている）

| 何 | どこ | 状態 |
|---|---|---|
| 厚いガラス（二面の屈折、縁の虹、後ろの絵と動画を曲げて映す） | `filament/thick_glass.mat`、`far_look.mat`、`far_side.mat`、`studio.glsl`、`kit.h` の `thick_glass` | 今の土台で動く |
| 紙の上の影と、どんな形でも落ちる虹の光 | `filament/glasslight.mat`、`paper.mat`、`floor_shade.mat`、`kit.h`（`paper_and_sun`、`glass_light`、`shadow_of`）、`blocks.mm`、`shapes.mm`、`dance.mm` | 今の土台で動く（`led.mm` が使う）。1個に 6〜10 ms |
| 輪、ポスター、踊り、砕いたガラスのポスター | `filament/ring.mm`、`poster.mm`、`dance.mm`、`acid.mm`、`own_glass.mat` | 前の土台で動いた（Dawn でも同じ絵、と当時確かめた物もある） |
| **気に入りの絵**：X-cube のオーロラ | `filament/xcube_gpu.mm`（`try 11 56`）、`xcube_drop.mm`、`xcube_*.mat` | 前の土台で動いた |
| 平らな面のガラスの、正確で鋭い床の光（光の束を割る） | `filament/beam_glass.h`、`beam_split.h`、`beam_spectrum.h`、`beam_extrude.h`、`glass_beam.mm`、`glass_scene.mm`、`BEAM_GLASS.md` | 前の土台で動いた。**これ以上の最適化はしない、と止めてある** |
| 虹のコースティクス（Filament だけで） | `filament/caustic_tri.h`、`caustic_tri.mm`、`filament/caustic_tri/` | 前の土台で動いた |
| 3Dの道のガラス、プリズム、ほかの習作 | `filament/beauty.mm`、`hero.mm`、`trio.mm`、`prism*.mm`、`collage.mm`、`link.mm`、`torture.mm`、`spatial.mm`、`gpu_tube.mm` ほか | 前の土台で動いた（古い。材質は `filament/*.mat` の 93 本のうち約 84 本がこの山の物） |

## 5. 文書と編集（過去の倉庫）

| 何 | どこ（`~/rust_ae/Motolii/motolii/crates/`） | 状態 |
|---|---|---|
| 文書の形：層、種類、値、番号、並び、時間（コマと分数）、カメラ、文字、素材 | `motolii-doc/src/store/`、`core/` | 読んだだけ |
| キーと間の定義、値の種類 | `motolii-doc/src/eval/track.rs`、`bezier.rs`、`value.rs` | 読んだだけ。式は `filament/interp.h` に写した |
| GSAP の名前の表 | `motolii-doc/src/eval/gsap.rs` | **持ってこない**（負の遺産） |
| 編集の操作、まとめ（グループ）、取り消し、確かめ、保存 | `motolii-edit/src/` | 読んだだけ |
| ThorVG や OS とのつなぎ | `motolii-glue/src/macos/` | 未確認 |
| 描く側、Lottie への書き出し | `motolii-render/` | 未確認 |
| 決め事の紙 | `docs/`、`MANIFESTO.ja.md`、`VISION.ja.md`、`v1/PLAN/` | ほぼ未読 |

## 6. 検査と記録

| 何 | どこ | 状態 |
|---|---|---|
| 検査を全部走らせ直して記録に書く | `filament/records.sh` → `reports/ae_rubric/records/` | 今の土台で動く（約20分） |
| 文書（置く・取り消し・間違い35個・間の行き方・開き直し・色） | `filament/check/doc.sh`、`doc_calls.txt`、`doc_wrong.txt`、`doc_ways.txt` | 今の土台で動く |
| 音、動画の色、動画の時間、C からの呼び出し、間の式の突き合わせ | `filament/check/sound.sh`、`video_colour.sh`、`video_time.mm`、`api.c`、`interp_twin.sh` | 今の土台で動く |
| AEのファイル103本、AEの動き | `filament/ae_test.sh`、`motion_test.sh` | 今の土台で動く |
| 採点表と採点4回 | `AE_RUBRIC.md`、`reports/ae_rubric/round_0*.md`、`evidence.md`、`try.md` | — |
| タスク表 | `TASKS.md` | — |

## 7. 実験と調べ物（結果だけ使う。作り直さない）

| 何 | どこ | 状態 |
|---|---|---|
| 動画40本の読み比べ（GStreamer と FFmpeg） | `filament/probe_gst/` | 今の土台で動く |
| 文書の入れ物の確かめ（SQLite の取り消し、GES、式の計算） | `filament/probe_edit/` | 今の土台で動く |
| 床の虹のコースティクス（wgpu + WGSL） | `caustics/` | 前の土台で動いた（別の作り） |
| ハードウェアの光線（wgpu の ray query） | `rayq/` | 前の土台で動いた（別の作り） |
| ThorVG のマスクを WebGPU の上で測った物 | `melt/` | 前の土台で動いた（git には入っていない） |
| 動く Filament を Mac の窓に出した最初の確かめ | `bridge/` | 前の土台で動いた（git には入っていない） |
| `filmask/`、`thorvg/`（Rust の小さい物） | `filmask/`、`thorvg/` | 未確認 |
| `filament_base/`、`filament_holes/`、`filament_premerge/`、`filament_walls/` | 同名のフォルダ | 未確認（10/12 の `filament/` の写しに見える） |
| 調べた紙：編集の中身、Flutter のガワの作り、過去の倉庫との対応、ガラス、マスク、二つのカメラ | `reports/ae_rubric/research_*.md`、`past_document_map.md`、`reports/`、`research_notes/` | — |

## 8. 素材（git には入れていない物が多い）

| 何 | どこ |
|---|---|
| 猫ミーム2本（ユーザーの物。リポジトリに入れない）と緑背景の猫 | `filament/out/vid/` |
| 変な動画14本（この Mac で作った試し用） | `filament/out/vid_odd/` |
| AE のファイルの見本103本 | `filament/third_party/thorvg_example/res/lottie/` |
| 3Dファイルの見本（ヘルメット、狐ほか） | `filament/third_party/filament_src/third_party/models/` |
| 部屋の光 | `filament/out/small_studio/`、`lightroom/`、`graffiti_shelter_2k/` |
| フォントへの近道 | `filament/out/arial_black.ttf`、`arial_unicode.ttf` |
