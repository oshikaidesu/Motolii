# 資産台帳

**作る前にここを見る。「無い」と思ったら、ここと過去の倉庫を探してから言う。**
最終更新：2026-10-14。足した物・消した物があったら、その時にここも直す。
**忘れても見張りが止める**：コミットの前に `.githooks/assets_check.py` が走って、git の知っているファイルのうち台帳に場所の無い物と、台帳に在るのにもう無い場所を数える。1つでもあればコミットは通らない。
（場所は `` ` `` で囲んで書く。フォルダを書けば中身は全部載った事になる。同じ行で前の場所と同じフォルダなら、名前だけでいい。）

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
| Flutter 側：その置き場をテクスチャとして登録して貼る（`GlassTexture`、`FlutterTexture`、`copyPixelBuffer`、`textureFrameAvailable`、チャンネルの名前は motolii/glass） | `skin/book/macos/Runner/MainFlutterWindow.swift` | 前の土台で動いた |
| 貼る部品：絵を出して、ドラッグとホイールと数を送る | `skin/book/lib/sets/glass_live.dart` | 前の土台で動いた |
| 開く：`skin/book/glass.sh`（Widgetbook の Glass › Live） | `skin/book/glass.sh` | 前の土台で動いた |
| 開く：`skin/book/live.sh`（Workspace › Live、.dart を保存すると Flutter のホットリロード） | `skin/book/live.sh` | 今の土台で動く（保存から2秒） |
| Windows の道筋（古い土台の物と、今の土台の物の両方） | `filament/WINDOWS.md` | 読んだだけ（何もビルドしていない） |

| **今の舞台（Dawn）の絵を、同じ形の置き場に描く口**。置き場3枚を Dawn に見せて Filament の描き先にする。描き終わった最後の1枚を渡す | `filament/live.h`、口は `filament/motolii.h` の `motolii_live`、`motolii_front`、`motolii_frames`、`motolii_play`、`motolii_now` | 今の土台で動く（検査 `filament/check/live.mm`：60コマ毎秒、置き場の絵と止め絵の差 0） |

上の「前の土台で動いた」物は、古いガラスの場面（`filament/glass_scene.mm`、Metal）を描いている。Flutter 側の受け（置き場をもらって貼る所）は、どちらの絵でも同じ。Debug ビルドの Mac だけ。

## 2. ガワ（Flutter）

| 何 | どこ | 状態 |
|---|---|---|
| Flutter 本体 | `~/rust_ae/Motolii/.tools/flutter/` | 前の土台で動いた（ガラスを開くスクリプトがここを使う） |
| 部品の棚（Widgetbook）：19 コンポーネント、約 166 の使い方、約 1.8 万行。数値の井戸・スイッチ・スライダー・色・タブ・タイムライン（行、バー、キー、再生ヘッド、定規）・ブラウザ・舞台のふち・窓 | `skin/book/lib/`（`skin/book/lib/tokens.dart`、`kit.dart`、`skin/book/lib/sets/`、`skin/book/lib/parts/`） | 前の土台で動いた（見た目の部品。中身にはつながっていない） |
| **見本の窓を本物の中身につないだ物**：Widgetbook の Workspace › Live。Flutter から中身の C の関数を直接呼ぶ束ね、Mac の窓の側の絵の受け（`StageTexture`）、舞台の席に絵を貼る所 | `skin/book/lib/motolii.dart`、`skin/book/macos/Runner/MainFlutterWindow.swift`、`skin/book/lib/workspace/workspace.dart`、`stage_view.dart`、`ws.dart`、`skin/book/lib/main.dart` | 今の土台で動く（舞台の絵と再生まで。ほかの席はまだ作り物の文書） |
| ウィジェットブックの土台：Mac 用の窓の作り、部品の検査、設定 | `skin/book/`（`skin/book/macos/`、`skin/book/test/`、`skin/book/pubspec.yaml`）、`skin/README.md` | 前の土台で動いた |
| 見本の窓（製品ではない）：机、ドック（ファイル、フォント、エフェクト、素材）、舞台 | `skin/book/lib/workspace/`（机は `desk.dart`、`desk_ease.dart`、`desk_depth.dart`） | 前の土台で動いた |
| 間の定義を触る習作（イージング、ウィグル） | `skin/book/lib/sets/studies/`（`panel_inspector_a_ease.dart`、`op_04_wiggle.dart`、`pop_04_wiggle.dart` ほか 160 本） | 未確認（在る事だけ） |
| 設計の紙 | `skin/DESIGN.md`、`skin/OVERVIEW.md`、`skin/PROCESS.md` | — |
| **過去の製品の窓**：橋、セッション、タイムライン、インスペクタ、舞台、机（ease、blend、depth、history、relations、notes） | `~/rust_ae/Motolii/motolii/ui/lib/` | 読んだだけ |
| 過去のスキンと中身の間の約束：命令 JSON 約 90 個、返事の「様子」の JSON、絵のテクスチャ番号 | `~/rust_ae/Motolii/motolii/ui/lib/bridge/protocol.dart`、`~/rust_ae/Motolii/motolii/ui/native/src/port.rs`、`snapshot.rs` | 読んだだけ。対応表は `reports/ae_rubric/past_document_map.md` |
| 過去の「Ease の机」と、間の種類のいつもの数 | `~/rust_ae/Motolii/motolii/ui/lib/desks/ease/`、`~/rust_ae/Motolii/motolii/ui/native/src/editor/ease_kinds.rs` | 読んだだけ。数は `filament/interp.h` に持ってきた |

## 3. 中身（今の土台：Filament を Dawn で）

| 何 | どこ | 状態 |
|---|---|---|
| 舞台と道具箱（物を置く、ガラス、型、時間、絵と動画の書き出し、測る） | `filament/kit.h` | 今の土台で動く |
| 動画（GStreamer。どの形式でも、切り出し・速さ・逆再生、色、音、コピーなしで板に） | `filament/video.h` | 今の土台で動く（Mac だけ） |
| AEの絵・写真・打った文字（ThorVG） | `filament/drawn.h` | 今の土台で動く |
| **形を三角形にして Filament で描く**（試作）：libtess2（SGI Free B）で輪郭を三角形に、材質で色と頂点の曲げ、View の MSAA。星40個の波で 11.56 → 3.03 ms／コマ | `filament/libtess2.sh`、`filament/shape.mat`、`filament/check/shape_mesh.mm`、`filament/check/shape_mesh.sh` | 書類の形はこれで描く（`filament/drawn.h` の `flat_mesh`：塗りは libtess2、線は ThorVG の stroker を `filament/bend.cpp` の tvg_edge から。グラデ・効果つきは今まで通り ThorVG の絵） |
| **形の曲げ（AE のシェイプ操作）**：形の値 round（どの形にも、キー可）・bloat（パンク・膨張）・zigzag／zigzag_ridges／zigzag_smooth・offset（パスのオフセット）。中身は ThorVG の Lottie 読みの部品そのまま。三角形の形の点を動かす曲げ（warp・bend・twist・pinch・skew・squash・wiggle）は `filament/flat.mat` の頂点の段、ThorVG の絵の warp は ThorVG の頂点シェーダーへのパッチ `filament/dawn_probe/thorvg_bend.patch`（`WgCanvas::bend`）、`filament/drawn.h` の `warp_at`。Skia の道まわりは `filament/skia.sh`（未接続） | `filament/bend.cpp`、`filament/drawn.h` の `bend_at`、部品は `filament/third_party/thorvg_src/src/loaders/lottie/tvgLottieModifier.cpp` | 今の土台で動く（4つ毎コマ曲げて +1.4 ms） |
| 3Dのファイル（Filament の gltfio） | `filament/solid.h` | 今の土台で動く |
| AEの動きを物に渡す（3Dレイヤー、カメラ） | `filament/motion.h`、ThorVG へのパッチ | 今の土台で動く |
| 間の定義9種類（過去の倉庫の写し） | `filament/interp.h` | 今の土台で動く（倉庫の式と差 0） |
| 文書（SQLite に事実を並べる）、取り消し、キーと間 | `filament/doc.h` | 今の土台で動く。**形と名前を過去の倉庫に揃える途中** |
| 編集の口（C の関数）と、その客（行を読む道具、素の C）。エンジンを触る係は中の1本の糸で、時計も中身が持つ | `filament/motolii.h`、`motolii.mm`、`edit.cpp`、`edit.sh`、`filament/check/api.c` | 今の土台で動く |
| **物理の試し：踊る猫が玉を弾く（2D、Box2D）、ハリネズミのボールプール（3D、Jolt）**。緑を抜いた動画の輪郭を毎コマ OpenCV で焼き、玉をその輪郭に当てて、キーに焼いて書類にする | `filament/catball/` | 今の土台で動く（Box2D 3.1.1 は Homebrew。どの物理に決めるかは TASKS の 0 と 8） |
| **物理エンジンの測り台**：同じ4場面（かけらの山・関節の格子・ロープ・猫の壁と球）を Rapier（Rust）と Jolt（C++）で測る。エンジン本体は `filament/third_party/physics/`（リポジトリ外、取ってきた物） | `filament/physics_bench/`（Rapier の測り台と Jolt の測り台） | 測り終えた（Jolt に決めた、TASKS の 0） |
| **書類の中の物理（Jolt）**：物の値 physics（none・moves・still）・bounce・friction、段の gravity。最後のキーまではキー通り、そこから物理（速さを引き継ぐ）。1/60 秒刻みで回して毎コマ貯める、書類が変わったら回し直す | `filament/physics.h`、`filament/physics.cpp`（Jolt は `filament/third_party/physics/`、ビルドは `filament/edit.sh`） | 今の土台で動く |
| **レシピの器（スクリプトの層）**：レシピ1個＝1ファイル。頭に書類への行と AviUtl2 式のつまみ（`--track`・`--check`・`--select`）、下に JS（`each(i, n, t, k)` が写し1個ずつの場所を返す、QuickJS-ng）。つまみは物の値 `js_名前` になり札に出る。置けばすぐ棚に出る | `filament/script.h`、`script.cpp`、`filament/recipes/*.js`（44個、うち Canvalry の並べ方24本は `filament/recipes/from_canvalry.py` が頭の行を足して作る）、検査 `filament/check/script.c`、エンジンの測り台 `filament/js_bench/` | 今の土台で動く（15個は前の表と画素まで同じ） |
| **出来合いの動き（Cavalry の Animators の形）と名前のイージング**：分けた文字の1個ずつを、ある値（y・x・X回転・不透明度・出る数）を一方から他方へ、名前のイージングで、順に（または乱数の順に）ずらして動かす。In は物の始まりから、Out は終わる直前に終わる。名前のイージング（SlowIn・SpringOut・BounceOut ほか9本）は1つの表で、キーの区間（SQLite）とスクリプトの `ease()`（JS）が同じ式を走らせる | `filament/animators/*.js`（13本、各1行で `animate()` を呼ぶ）、`animate()` は `filament/script.cpp` の前置き、`filament/ways.h`（名前と式） | 今の土台で動く（文字13個 1080p 2.65 ms、キーと JS の値が一致） |
| **利用者のプリセット（Ease の動き方）**：1個＝1ファイル（1行目が名前、残りが式：a・b・u・p1..p4、`bezier(u, x1, y1, x2, y2)`）。Ease の desk にどの書類でも出て、キーに当てると書類に書き込まれる（1歩）。利用者のフォルダ（Mac は Application Support の Motolii、Windows は APPDATA の Motolii、MOTOLII_OWN で差し替え。その中の ways/。recipes/・effects/・animators/ も同じ扱い）と本体の見本 `filament/ways/`（`snappy.way`）、フォルダは `filament/doc.h` の doc_folders、読むのは doc_own_ways | 今の土台で動く（結合テスト「a preset of your own」） |
| **作業中に鳴る音**：生の舞台が再生している間、各物の音（書き出しと同じ Stage::sounding：切り出し・速さ・回り・始まり）を GStreamer の playbin（音だけ）で、舞台の時刻が置く所へ探して鳴らす。止まれば止まる、0.1秒ずれたら探し直す。MOTOLII_MUTE で誰にも聞こえない（試験用） | `filament/heard.h`、`motolii_heard`、試験 `filament/check/heard.c`・`heard.sh` | 今の土台で動く（ずれ 2〜14 ms、2倍速で1秒に 2.00 秒） |
| **画面全体の効果（WGSL）**：段の `script` に .wgsl を1本。vgpu の書き方（フラグメントだけ、`fs_main`、絵の texture と sampler、つまみの uniform 構造体）、頭に AviUtl2 式のつまみ。段の絵を画面外に描き、Filament 自身の描く係の口（全画面の三角）で効果をかけて画面へ | `filament/screen.h`、`filament/dawn_probe/screen_door.h`、`screen_door.cpp`、`filament/effects/*.wgsl`（46本：vgpu から借りた CRT、three.js の10本、ISF-Files の33本 `isf_*.wgsl`、前のコマを使う物を含む、keijiro/SlitScanCam の時間置換 `time_displacement.wgsl`、newomp4/phosphor のデータモッシュ `datamosh.wgsl`）、変換 `filament/effects/from_three.py`・`from_isf.py`（naga） | 今の土台で動く（素通しは効果なしと画素まで同じ、CRT で +0.8 ms） |
| 板の材質（色抜き、型、重ね方4つ、動画の色） | `filament/flat.mat`、`matte.mat`、`matte.glsl` | 今の土台で動く |
| 場面の見本：民衆、人形劇、マスク、重ね方、壁、AE再生、AEの動き、文書 | `filament/neko.mm`、`puppet.mm`、`mask.mm`、`lay.mm`、`wall.mm`、`ae.mm`、`led.mm`（開くのは `filament/neko.sh`、`puppet.sh`、`lay.sh`、`led.sh`。led の動きは `filament/led.json`、3Dシェイプレイヤーつきは `filament/led_card.json`） | 今の土台で動く。動画は `reports/show/` |
| ビルド：今の土台用。Filament と ThorVG を Dawn で作る手順、Filament の中に仕事を入れる口、計算の口の確かめ | `filament/dawn_probe/`（場面を作るのは `kit_wgpu.sh`、ThorVG は `tvg_on_dawn.sh`） | 今の土台で動く |
| 人の物への手入れ：Filament（計算の口、機械を借りる口、外の絵を受ける口、動画の共有の機能）、ThorVG（3つ目の数 z、4 MiB を超える書き込みを分けて書く） | `filament/dawn_probe/filament_dawn_compute.patch`、`thorvg_z.patch`、`thorvg_wg_upload.patch`、`gpu_door.h`、`gpu_door.cpp` | 今の土台で動く。ThorVG の物は本家に送らない |

## 4. ガラスと光（残すと決まっている）

| 何 | どこ | 状態 |
|---|---|---|
| 厚いガラス（二面の屈折、縁の虹、後ろの絵と動画を曲げて映す） | `filament/thick_glass.mat`、`far_look.mat`、`far_side.mat`、`studio.glsl`、`kit.h` の `thick_glass` | 今の土台で動く |
| 紙の上の影と、どんな形でも落ちる虹の光 | `filament/glasslight.mat`、`paper.mat`、`floor_shade.mat`、`kit.h`（`paper_and_sun`、`glass_light`、`shadow_of`）、`blocks.mm`、`shapes.mm`、`dance.mm` | 今の土台で動く（`led.mm` が使う）。1個に 6〜10 ms |
| 輪、ポスター、踊り、砕いたガラスのポスター | `filament/ring.mm`、`poster.mm`、`dance.mm`、`acid.mm`、`acid_flat.svg`、`own_glass.mat`（開くのは `filament/ring.sh`、`poster.sh`、`dance.sh`、`blocks.sh`、`shapes.sh`、`show.sh`） | 前の土台で動いた（Dawn でも同じ絵、と当時確かめた物もある） |
| **気に入りの絵**：X-cube のオーロラ | `filament/xcube_gpu.mm`（`try 11 56`）、`xcube_drop.mm`、`xcube.mm`、`xcube_*.mat` | 前の土台で動いた |
| 平らな面のガラスの、正確で鋭い床の光（光の束を割る） | `filament/beam_glass.h`、`beam_split.h`、`beam_spectrum.h`、`beam_extrude.h`、`glass_beam.mm`、`glass_scene.mm`、`BEAM_GLASS.md`、`beam_glass_test.cpp`、`extrude_check.cpp`、`flat_pop_test.cpp`、`beam_test.sh`、`build_one.sh` | 前の土台で動いた。**これ以上の最適化はしない、と止めてある** |
| 光の束を割るのを GPU の上で（Filament の計算の口を通して） | `filament/cbeam.h`、`cbeam.wgsl` | 前の土台で動いた |
| 虹のコースティクス（Filament だけで） | `filament/caustic_tri.h`、`caustic_tri.mm`、`filament/caustic_tri/` | 前の土台で動いた |
| 3Dの道のガラス、プリズム、ほかの習作 | `filament/beauty.mm`、`hero.mm`、`trio.mm`、`prism*.mm`、`collage.mm`、`link.mm`、`torture.mm`、`spatial.mm`、`gpu_tube.mm`、`caustics.mm`、`fresh.mm`、`holes.mm`、`layers.mm`、`probe.mm`、`robust.mm`、`filament/prism.sh`、`dichro.sh`、`link.sh`、材質の部品 `filament/*.glsl` | 前の土台で動いた（古い。材質 `filament/*.mat` は 93 本あって、約 84 本がこの山の物） |

## 5. 文書と編集（過去の倉庫）

| 何 | どこ | 状態 |
|---|---|---|
| 文書の形：層、種類、値、番号、並び、時間（コマと分数）、カメラ、文字、素材 | `~/rust_ae/Motolii/motolii/crates/motolii-doc/src/store/`、`~/rust_ae/Motolii/motolii/crates/motolii-doc/src/core/` | 読んだだけ |
| キーと間の定義、値の種類 | `~/rust_ae/Motolii/motolii/crates/motolii-doc/src/eval/track.rs`、`bezier.rs`、`value.rs` | 読んだだけ。式は `filament/interp.h` に写した |
| GSAP の名前の表 | `~/rust_ae/Motolii/motolii/crates/motolii-doc/src/eval/gsap.rs` | **持ってこない**（負の遺産） |
| 編集の操作、まとめ（グループ）、取り消し、確かめ、保存 | `~/rust_ae/Motolii/motolii/crates/motolii-edit/src/` | 読んだだけ |
| ThorVG や OS とのつなぎ | `~/rust_ae/Motolii/motolii/crates/motolii-glue/src/macos/` | 未確認 |
| 描く側、Lottie への書き出し | `~/rust_ae/Motolii/motolii/crates/motolii-render/` | 未確認 |
| 決め事の紙 | `~/rust_ae/Motolii/docs/`、`~/rust_ae/Motolii/MANIFESTO.ja.md`、`VISION.ja.md`、`~/rust_ae/Motolii/v1/PLAN/` | ほぼ未読 |

## 6. 検査と記録

| 何 | どこ | 状態 |
|---|---|---|
| 検査を全部走らせ直して記録に書く | `filament/records.sh` → `reports/ae_rubric/records/` | 今の土台で動く（約20分） |
| 文書（置く・取り消し・間違い35個・間の行き方・開き直し・色） | `filament/check/doc.sh`、`doc_calls.txt`、`doc_wrong.txt`、`doc_ways.txt` | 今の土台で動く |
| 音、動画の色、動画の時間、C からの呼び出し、間の式の突き合わせ | `filament/check/`（`sound.sh`、`video_colour.sh`、`video_time.mm`、`api.c`、`interp_twin.sh`、`solid_probe.mm`） | 今の土台で動く |
| AEのファイル103本、AEの動き | `filament/ae_test.sh`、`ae_ref.cpp`、`motion_test.sh`、`motion_test.mm`、`motion_test.json`、`motion_test3d.json`、`motion_test3d.inc` | 今の土台で動く |
| 採点表と採点4回 | `AE_RUBRIC.md`、`reports/ae_rubric/round_0*.md`、`evidence.md`、`try.md` | — |
| タスク表 | `TASKS.md` | — |
| この台帳と、その見張り（コミットの前に走る） | `ASSETS.md`、`.githooks/assets_check.py`、`.githooks/pre-commit` | 今の土台で動く |
| 入口の紙 | `README.md` | — |

## 7. 実験と調べ物（結果だけ使う。作り直さない）

| 何 | どこ | 状態 |
|---|---|---|
| 動画40本の読み比べ（GStreamer と FFmpeg） | `filament/probe_gst/` | 今の土台で動く |
| 文書の入れ物の確かめ（SQLite の取り消し、GES、式の計算） | `filament/probe_edit/` | 今の土台で動く |
| Filament に出来るかの確かめ（GPU の切り取り、1回で複数の値を書く、頂点の無い描き方） | `filament/clip_probe.mm`、`mrt_probe.mm`、`proc_probe.mm` | 前の土台で動いた |
| 床の虹のコースティクス（wgpu + WGSL） | `caustics/` | 前の土台で動いた（別の作り） |
| ハードウェアの光線（wgpu の ray query） | `rayq/` | 前の土台で動いた（別の作り） |
| ThorVG のマスクを WebGPU の上で測った物 | `melt/` | 前の土台で動いた（git には入っていない） |
| 動く Filament を Mac の窓に出した最初の確かめ | `bridge/` | 前の土台で動いた（git には入っていない） |
| `filmask/`、`thorvg/`（Rust の小さい物） | `filmask/`、`thorvg/` | 未確認 |
| `filament_base/`、`filament_holes/`、`filament_premerge/`、`filament_walls/` | 同名のフォルダ | 未確認（10/12 の filament フォルダの写しに見える） |
| 調べた紙：編集の中身、Flutter のガワの作り、過去の倉庫との対応、ガラス、マスク、二つのカメラ | `reports/ae_rubric/research_*.md`、`past_document_map.md`、`reports/`、`research_notes/` | — |

## 8. 素材（git には入れていない物が多い）

| 何 | どこ |
|---|---|
| 猫ミーム8本（ユーザーが置いた物。リポジトリに入れない。全部 H.264・音つき・緑背景：`happy`、`mohsen`、`waving`、`swinging`、`dog`（暗い緑）、`vest`、`popcat`、`girlfriend`）と、うちで作った緑背景の猫 `cat_green` | `filament/out/vid/` |
| 作りたい画の手本7枚（ガラスの見た目6枚と、ロケットを四角い窓で切って並べた青いコラージュ） | `research_notes/refs/`（git の外。青いコラージュは `research_notes/refs/G_rocket_collage_blue.jpg`） |
| 変な動画14本（この Mac で作った試し用） | `filament/out/vid_odd/` |
| AE のファイルの見本103本 | `filament/third_party/thorvg_example/res/lottie/` |
| 3Dファイルの見本（ヘルメット、狐ほか） | `filament/third_party/filament_src/third_party/models/` |
| 部屋の光 | `filament/out/small_studio/`、`filament/out/lightroom/`、`filament/out/graffiti_shelter_2k/` |
| フォントへの近道 | `filament/out/arial_black.ttf`、`arial_unicode.ttf` |
