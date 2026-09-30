# Technology Acquisition Gate と Capability / Technology Map（2026-09-30）

状態: **決定**（ゲート。自身を通した結果は §1）／**観察**（Map。置換は各粒の着手時にゲートを通してから）

原則: **Motolii owns meaning, not technology.** 目的は依存を増やすことでも自作を減らすことでもなく、After Effects 級の能力を持ちながら、
AE のように世界中の技術をHost内部で再発明し永久保守するソフトにならないこと。

## 1. ゲート自身を Technology Acquisition Gate に通す

**Capability**: 新しい技術責任をrepositoryへ持ち込む前に、既存技術を調べ、Reuse / Wrap / Adapt / Extend を Build より優先した証拠を残し、レビューで見落とさない。
（commit messageの書式や150行は要件ではない。）

**調べた既存技術と、見つかったこと**

| 候補 | 提供する物 | このCapabilityに対して |
|---|---|---|
| `git interpret-trailers` / `git stripspace`（git標準） | trailerの解釈・コメント除去。commit-msg hook から使える | **使う**（手書きの正規表現を置換） |
| `core.hooksPath` の `.githooks`（git標準、既に使用中） | hookの配置 | **使う**（継続） |
| lefthook / pre-commit framework | hookの導入・実行管理 | 規則は提供しない。既に `.githooks` で足りるので足さない |
| commitlint | conventional commits の検査。独自ruleはplugin | Nodeを持ち込み、必要なruleが1つ。足さない |
| Danger JS | PR本文・`created_files`・PRサイズからの警告 | 仕組みとしては最も近い（新規fileとPR本文を見る）。ただしNode＋PR運用が前提で、今のpush運用に合わない。PR運用へ移る時の置換候補 |
| require-checklist-action 等 | PR本文のchecklist強制 | PR欄の強制ならこれ。今はPR template（GitHub標準）に項目があるだけで足りる |
| GitHub rulesets / required status checks / CODEOWNERS（既に使用中） | 必須チェック・所有者承認 | **使う**: ゲート自身の改変をCODEOWNERSで保護（追加済み）。`stage5-structure` の必須化はowner設定 |
| dependency-review-action / cargo-deny / cargo-vet | **依存の追加**の脆弱性・license・供給元・監査 | 逆向き（Reuse側の責任）。`Build`（自作を増やす側）は見ない。cargo-denyは導入済み |
| ADR ツール（adr-tools / MADR / log4brains） | 決定の記録 | 記録だけで強制しない。記録先は既存の decision-index と reviews |

**結論**: 「大きな新規codeには調べた証拠が要る」を強制する既存toolは見つからなかった。80%は既存へ委ね、Motolii固有の20%だけ残す。

| 部品 | 判断 |
|---|---|
| trailerの解釈 | **WRAP EXISTING**: `git interpret-trailers --parse`（手書きregexを削除して置換済み） |
| hookの配置と実行 | **WRAP EXISTING**: `.githooks` + `core.hooksPath` |
| CIでの実行 | **WRAP EXISTING**: 既存の GitHub Actions job に1 step |
| ゲート改変の防止 | **WRAP EXISTING**: CODEOWNERS（追加済み） |
| 「150行以上の新規source file」という引き金 | **REDUCE TO CONFIG**: 変数2つ（`MIN_LINES`、対象dir）。既存toolに同等の物は無い。数値は経験則で、要件ではない |
| 「近道の語は理由にならない」語彙 | **KEEP（Motolii固有の意味）**: 何が理由になるかの方針で、1行のデータ。語を回避する言い換えは防げない（人とreviewが最後の砦） |
| `Acquisition:` / `Why-build:` の2 trailer | **KEEP**: 名前と値域だけが固有。形式はgit標準 |

自作scriptは約60行のglueまで縮んだ。**限界**: 申告の正しさは判定できない（嘘の `Reuse` は通る）。止めているのは無意識に書き始めることで、Buildは理由の形と利用者への返却で高くつく選択になる。

## 2. 方法と限界

6領域をread-onlyで調べ、それぞれ外部（crates.io、公式docs、GitHub）を検索した。各領域の報告にある「未確認」はこの文書にも引き継ぐ。
**私が独立に確認した主張**: `ARC_SAMPLES = 24`（`motolii-doc/src/vector/geom.rs:252-257`）、Offsetの自前実装（`picture/shapes_ops/ops.rs:217`）、encodeが `tool_command` を使う素の `Command`（`media/encode.rs:56`）、
`kurbo` は `motolii-doc` のみ依存（render側に無い）、`FluidBoard` に仮想化が無い（`media_fluid.dart` に `ListView.builder` / sliver が0件。実測: 1,845件のfolderでThumbnailの最悪フレーム1.2秒）、
raw GPU API の自前天井超過（`owned_budget` が赤）、決定索引の矛盾（§5）。
**訂正**: 調査報告が「Merge Pathsをアルファpixelで行う再発明」とした `shapes_ops/coverage.rs` は、実際は**マスク合成**（`MaskMode` の Add / Subtract 等。`engine/mask.rs`）で、AEでもラスタのアルファ演算。再発明ではなくMotoliiの意味に分類した。
ベクターのboolean（Merge Paths）の実装は見つからなかった（必要かどうかはINVESTIGATE）。

## 3. Capability / Technology Map

分類: BORROWED（ほぼそのまま借りる）／WRAPPED（Motolii固有の意味だけを薄く接続）／MOTOLII（外へ委ねない意味）／SUSPECT（借りているが周囲の一般技術が多い）／REINVENTED（成熟した既存があるのに一般技術を自作）。

| Capability | Current implementation | Existing technology available | Motolii-owned semantics | Class | Action |
|---|---|---|---|---|---|
| Document store | `motolii-doc` store 505行、`re_entity_db` / `re_chunk_store` 上。`read.rs` に RecordCache / TrackCache | Rerun の store、`re_query::QueryCache` + `ChunkStoreSubscriber` | layer・property・slot の identity、revision の意味 | BORROWED + WRAPPED（cacheは SUSPECT） | KEEP（cache は INVESTIGATE: 上流のQueryCacheへ） |
| Command / Undo | `edit/src/document.rs` 609行（Rerunの chunk共有snapshot）。`editor/history.rs` 284行は手書きJSON履歴（上限200） | Rerun blueprint undo、`undo` crate（History tree） | 編集の意図、Undoの区切り、scope | WRAPPED + `history.rs` SUSPECT | KEEP。`history.rs` は INVESTIGATE |
| Persistence | `persist.rs` 219行、RRD | Rerun RRD | 保存・復旧・journalの意味 | WRAPPED（ファイル構成からの推定、詳細未読） | KEEP |
| Animation / easing | `eval/track.rs` 647、`eval/bezier.rs` 121（根の求解は `kurbo::solve_cubic`） | `keyframe` / `splines` / CSS easing 系crate | easing の種類、時間の意味、Hold / Random の意味 | WRAPPED + MOTOLII | KEEP |
| Timeline UI | `timeline_core/` 1,925行 + `hf/shell/timeline.dart` 562 + `hf/desk/ease.dart` 798 | 製品級のFlutter timeline packageは見つからず | clip・時間・keyframe の意味 | 自作（正当：既製品なし） | KEEP |
| Stage | `panels/stage/*` 1,459 + `editor/stage.rs` 768。描画はRerun側 | Rerun `re_view_spatial`（egui依存で直接は使えない） | gesture → intent の写像 | WRAPPED | KEEP |
| UI toolkit（`hf/`） | 12,062行。dock・menu・metrics を自前。packageは `docking` のみ | Flutter標準、既存package | 製品の見た目 | SUSPECT（規模は未計測。製品のchromeが多い可能性） | INVESTIGATE: 汎用widgetとchromeの仕分け |
| Media decode | `engine/texture.rs` 518行、`re_video`（fork）。1GiBのGPU frame cache（自前LRU） | `re_video`、`ffmpeg-next`、VideoToolbox | source時間→pts、色空間の選択 | BORROWED + 自前cache | KEEP |
| Media encode / mux | `media/encode.rs` 247行。素の `Command`（libx264・aac固定）。thumbnail側は `ffmpeg-sidecar` を使う | `ffmpeg-sidecar`、VideoToolbox / AVAssetWriter | 書き出しの色の意味（bt709 tv-range等） | WRAPPED + SUSPECT（同じ道具を2通りで呼ぶ） | THIN: sidecarへ寄せる。ハードウェアencodeは INVESTIGATE（計測してから） |
| Export orchestration | `export.rs` 430、`jobs/export.rs`、`jobs/freeze.rs`、Lottie出力 約1,440行 | ffmpeg（mux）、Lottieは形式固有 | 書き出しの意味（取消・snapshot・拒否）、Lottieへの写像 | MOTOLII | KEEP |
| Media Browser（一覧の描画） | `FluidBoard` 全faceをwidget化、`MediaLibraryBody` も全tile | Flutterの sliver / `ListView.builder`、staggered grid、catalogのpaging | faceの identity が view 間で続く、選択ring、置く操作 | **REINVENTED**（実害：1.2秒フレーム） | **REPLACE candidate（最優先）**: 可視範囲だけ構築。移動の滑らかさは見える範囲に限る |
| Explore（layout） | `explore_graph.dart` 394行、O(n²)のFruchterman–ReingoldをUIスレッドで | `forceatlas2` / `fdg` / `grapher`（Barnes–Hut）、Dartなら `graphview` 系 | 近さ（similarity）の意味、地図の記憶、選択で動かない契約 | **REINVENTED** | **REPLACE candidate**: 解く部分を native（Barnes–Hut）へ。UIスレッドの外。Relations Graphも同じ需要 |
| Explore（similarity） | 手製の重み和（名前・folder・時間・形・kind・長さ）。kNNは全対ループ（Dart） | 近傍索引（HNSW等）。内容特徴が出来た時にUMAP系 | 重みと「同一bytesは重複」の規則 | MOTOLII（重み）+ kNNの全対ループはSUSPECT | THIN: 重みは残し、近傍探索だけ索引へ |
| Asset database / search | `catalog/*`（rusqlite・WAL・FTS5 trigram・walkdir） | SQLite FTS5、Tantivy | asset uid と path の分離、past_places、relink方針 | BORROWED + MOTOLII | KEEP |
| Asset watching | `catalog/watch.rs` 111行、`notify`（FSEvents） | notify、watchexec | 変更の合図だけ（真実は索引） | BORROWED | KEEP |
| Thumbnails | `editor/thumbnail.rs` 218行。data-URIをprocess内HashMapに入れ、退避も永続化も無い | macOS QuickLook（`QLThumbnailGenerator`）、freedesktop thumbnail cache | face key（ファイルが変われば変わる） | WRAPPED + cacheが REINVENTED | INVESTIGATE: OS機構＋上限付きcache。3 OSの対応 |
| File dialogs | `MainFlutterWindow.swift` の `NSOpenPanel` 直接 | `file_selector`、`rfd` | 許す種類 | BORROWED（OS API） | KEEP |
| Content fingerprint / identity | `fingerprint.rs` 214行（SHA-256、edges）、`file-id` | `blake3`、`file-id` | fingerprint文字列は永続契約、証拠の強さの順位 | BORROWED + MOTOLII | KEEP（形式は保存物の契約） |
| Audio decode | `audio/decode.rs` 76行、ffmpeg CLIを通す。全体をメモリに読む（4時間上限） | Symphonia、ffmpeg | 正準形式、stream番号 | WRAPPED | KEEP |
| Audio resample / device / ring | rubato、cpal、rtrb の薄いglue | 同左 | `source_frame_to_device` | WRAPPED | KEEP |
| Audio mix | `audio/mix.rs` 301行。1サンプルごとに `RationalTime` を評価、倍速は線形補間 | miniaudio node graph、Kira mixer（ただし有理時間に結べない） | clip内時間でのgain・pan・fade、範囲外のLoop/Silence | MOTOLII（手書きDSP） | THIN: ブロック単位の評価へ。倍速の補間は INVESTIGATE |
| Audio clock / TimeMap / waveform | `clock.rs` 201、`time_map.rs` 106、`waveform.rs` 316（`waveform-data`） | Kira / Rodio の時計はtick基準で合わない | 音声マスター→映像frame、遅延規則、TimeMap | MOTOLII + waveformは BORROWED | KEEP |
| Text shaping | `picture/shaping.rs` 580行、`cosmic-text` | `parley`（HarfRust・Skrifa・ICU4X） | text と span の意味 | WRAPPED | KEEP（parleyは bidi/fallbackで困った時に再評価） |
| 日本語組版規則 | `shaping.rs:244-277`（autospace・ぶら下げ・約物詰め）。文字種の表は手書き | 相当するcrateなし | 編集の意味そのもの | MOTOLII（手製） | KEEP。文字種表は unicode 由来データとの突き合わせを INVESTIGATE |
| Layout | `picture/flow.rs` 772行、`taffy` | taffy | Display・子の順・identity・時間 | BORROWED + MOTOLII | KEEP |
| Vector: 図形の構築 | `doc/vector/geom.rs` 403行（rect・ellipse・polystar・arc） | `kurbo`（既に `motolii-doc` に依存。`Arc`・`Ellipse`・`RoundedRect`） | 頂点モデル（in/out tangent）は文書の意味 | MOTOLII + 小さな手書き数学 | THIN（価値小・危険小） |
| Path: bezier評価・弧長・分割 | `geom.rs:197-355`。24サンプルの累積弦長＋線形逆引き（誤差の保証なし） | `kurbo` の `ParamCurveArclen`（`inv_arclen`）、`subsegment` | なし | **SUSPECT** | **REPLACE candidate**: Trim・Chop・Resample・on-path motion が全部これに乗る。価値高・危険低 |
| Path effects: Trim / Zigzag / Pucker / Wiggle 等 | `shapes_ops/ops.rs` 587、`ops/shaping.rs` 343、`ops/trim.rs` 183 | 既製品なし（AE固有の演算子）。`kurbo` が弧長・法線・細分の部品 | AE互換の演算子と引数の意味 | MOTOLII | KEEP（弧長だけ借り直す） |
| Path: Offset Paths | `ops.rs:217-360`。多角形に標本化して辺をずらす。開いたpathを拒否（`:227`） | `kurbo::offset`、`i_overlay`（offset / buffering / boolean）、clipper2 | join・miter limit の意味 | **REINVENTED** | **REPLACE candidate**: 価値高 |
| Path: boolean / Merge Paths | 見つからない（`coverage.rs` はマスク合成） | `i_overlay`、clipper2、`geo` | Merge の意味 | 未実装 | INVESTIGATE（製品要求かどうか） |
| Glyph stroke 分割 | `shapes_ops/strokes.rs` 216 + `engine/strokes.rs` 59。192px固定格子のdilate/erode/marching squares | 骨格・medial axis のcrateは未確認 | 文字の一画ごとのアニメーション | **REINVENTED**（研究寄り） | INVESTIGATE（孤立していて実害なし。精度の上限あり） |
| Stroking / fill（CPU） | `tiny-skia`（`raster.rs`） | `tiny-skia`、`vello_cpu` | 線・塗りの引数の意味 | BORROWED | KEEP |
| Tessellation / GPU path draw | re_renderer fork（`compositor/paths.rs` 238） | lyon、`vello_gpu` | 決定済み（索引28） | BORROWED | KEEP |
| 2D rendering（組み立て） | `compositor/` 10.5k行。re_renderer の `ViewBuilder`・`RectangleDrawData` で描く | re_renderer | 2D / 2.5D / 3D の run を切る法則 | WRAPPED | KEEP |
| 3D rendering | 読み込みは `gltf`・`tobj`・`stl_io`、描画は re_renderer の mesh。押し出し等は自前（`compositor/extrude`） | re_renderer mesh、`parry3d`（CPU bounds） | 3Dの意味（カメラ・投影・札） | BORROWED + 一部未確認 | INVESTIGATE（この監査では個別に読んでいない） |
| Compositor（blend / matte） | blend式は vello から借用（`blend.wgsl`）、matte は luma式を vello から | vello | どのmodeを保存するか、mask scope・順序 | BORROWED | KEEP（模範例） |
| Masks | `engine/mask.rs`、`shapes_ops/coverage.rs` 99行（アルファ合成） | なし（AEと同じラスタ演算） | MaskMode、順序（mask→effect） | MOTOLII | KEEP |
| Effects（kernel） | `vism/*.wgsl / .fs`（blur・glow・kuwahara・chroma_key 等） | OpenFX / ISF（形式） | 効果の identity・引数・時間・永続化 | 効果のデータ | KEEP（kernelは棚の中身） |
| Shader runtime / plugin（Vism） | `effects/vism.rs` 591、`isf/mod.rs` 746、`wesl =0.4.2`、naga | ISF、WESL、naga。OpenFXは別ABI | manifest・引数面・provider契約 | WRAPPED | KEEP。OpenFX host は INVESTIGATE（生態系の価値。規模大） |
| Optical flow | `vism/pixel_motion_blur.fs` 103行。ISFのpyramid Lucas–Kanade | OpenCV（DIS / Farneback / PyrLK）、RIFE / RAFT（ML） | 「動きの場」の意味と、その estimator を替えても作品が変わらない契約 | 効果のデータ（SUSPECT as technology） | KEEP（棚の効果）。estimator を provider として替えられるかを INVESTIGATE。OpenCVはCPU読み戻しが要り、再生中 readback 0 の規則と衝突 |
| Tracking / CV | `media/blob.rs` 362行（閾値・輪郭・面積・ID継続をCPUで）、`analysis_program.rs` 381 | `imageproc`（輪郭・連結成分）、OpenCV CSRT / KCF、kornia-rs | Track結果の identity・時間・parameter接続・Undo・persistence | **REINVENTED**（CV核）+ MOTOLII | REPLACE candidate（輪郭・ラベル）。本物の物体追跡は INVESTIGATE |
| Physics | `engine/physics.rs` 636行。Rapier3d へ意図を写すだけ | Rapier | 意図・lies・wells・links、前フレームを覚える逐次状態 | WRAPPED | KEEP（決定性の feature flag が有効かは未確認） |
| Particles | `store/particles.rs` 288 + `particle_program.rs` 302。CPUの決定的な発生＋弾道 | `bevy_hanabi`（Bevy ECS前提で合わない） | 発生のパラメータを時間の純関数にする（scrubの再現） | MOTOLII | KEEP |
| Color management | `moxcms` はingestのICC→sRGBのみ（`texture.rs:393-400`） | moxcms、OCIO（`ocio-rs`）、lcms2 | working space の選択 | WRAPPED（ingestのみ） | KEEP。AE風の linear / ACES パイプラインは INVESTIGATE |
| FrameGraph（評価） | `frame_graph/` 9.3k行。核（cache・scheduler・compiler 約1.8k）は `WorkKey` 索引のCPU memoグラフ＋世代で取消。残りは領域ごとの `*_program.rs` 約7.4k | `salsa` は決定済みで不採用（references.md）。bevy / zenfg は合わない | node の key と時間依存、評価の意味 | 核は REINVENTED（ただし決定済み）+ `*_program` は MOTOLII | KEEP（再評価の引き金なしには動かさない） |
| Render graph / pass scheduling | 独自のrender graphは無い（re_renderer側にも無い。`draw_phases` に「formalize したい」の注記） | zenfg（v0.1・新しすぎる）、Bevy render graph（ECS前提） | なし | 存在しない | KEEP（借りる物が無い） |
| GPU scheduling（submit / poll） | re_renderer の submit上限4と `begin_frame` を使う。`texture.rs:311`・`blocks.rs:1147` に直接 `queue.submit`、`compositor/device.rs` に `wait_for_gpu` | re_renderer | なし | 大部分 BORROWED + SUSPECT | THIN（直接submitを `before_submit` 経路へ） |
| GPU resource / cache | `EffectScratch`（texture pool）、`block_program` の `Pool`（buffer pool）、動画frame cache、生のcreate 24箇所 | re_renderer の `GpuTexturePool` / buffer pool | 効果の中間targetの寿命 | SUSPECT | THIN（ledger 6番。同一frame内再利用の保持が条件） |
| GPU compute の土台 | `block_program/passes.rs` 687 + `wgsl.rs` 357 + `selection_bounds.rs` 216。生の pipeline / shader module / bind group layout（自前天井は0、`owned_budget` が赤） | re_renderer の pool。solverはRapier | 箱 block の意味 | SUSPECT | **REPLACE candidate**: 自前天井の違反。選択範囲は re_renderer の picking / `GpuReadbackBelt`（fork に buffer 読み戻しを足す = Extend） |
| Status sync（Rust→Swift→Dart） | `snapshot_cache.rs` 509 + `snapshot.rs` 625。手書きJSON、known id、2026-09-30にbytes転送へ簡素化済み | Rerun の store event / `ChunkStoreSubscriber`、flutter_rust_bridge、flatbuffers / protobuf | snapshotが運ぶ物の意味、reference id | SUSPECT | INVESTIGATE（delta は wire の変更） |
| Plugin / provider contract | `extensions/*` | OpenFX（業界標準ABI） | provider契約、作品互換 | WRAPPED | KEEP |

## 4. 分類

**A. KEEP（今の借り方が理想的）**: Document store・Persistence・Animation/easing・Stage・Layout・Text shaping・CPU stroking/fill・GPU tessellation・Blend/matte・Masks・Effects kernel・Physics・Particles・
Catalog/索引/監視/fingerprint・File dialogs・Audio の decode/resample/device/clock/TimeMap・Timeline UI・Export・Plugin/provider・2D composite。

**B. THIN（借りているが、Motolii側を薄くできる）**: Media encode（sidecarへ）・Audio mix（ブロック評価）・GPU の直接submit・EffectScratch とPool・Explore の kNN 探索・図形の構築（kurbo の型）。

**C. REPLACE（一般技術を再発明している可能性が高い）**:
1. **Media Browser の一覧描画**（実害が出ている。最優先）
2. **Explore の force layout**（UIスレッドのO(n²)）
3. **Path の弧長**（24サンプル。Trim系の土台）
4. **Offset Paths**（多角形近似）
5. **Tracking のCV核**（`blob.rs`）
6. **GPU compute の土台**（自前天井の違反）

**D. INVESTIGATE（証拠不足）**: UI toolkit `hf/`・thumbnails（QuickLook）・ハードウェアencode・Undo履歴 `history.rs`・Status sync・Optical flow の estimator 交換・OCIO・OpenFX host・日本語組版の文字種表・
Glyph stroke 分割・ベクターboolean・3D rendering（個別に未読）・Audio の倍速補間・Document cache（QueryCache へ）。

## 5. Rerun の決定からの逸脱と、FrameGraph の切り分け

**決定**: 2026-08-10「Motolii は Rerun Spatial Viewer の creator-facing wrapper。scene / view / query / camera / picking / visualizer / GPU draw を再実装しない」。
2026-08-20 に「direct `re_renderer` scene の禁止」部分は撤回済み（viewer層が egui 依存で、`re_renderer` を直接使うしか道が無いため）。**評価経路を2本にしない**趣旨だけが現行。
決定索引の230・413・415行はこの撤回を反映していなかったので、注記を足した（89行が正）。

**Picture → Engine → FrameGraph → Compositor → re_renderer → wgpu** の切り分け:

| 層 | 性質 |
|---|---|
| picture/（8.1k） | Motoliiの評価の意味（layerがtで何か）。KEEP |
| engine/（14k） | 再生の持ち主。意味＋直接submit等の SUSPECT |
| frame_graph/ 核（約1.8k） | 一般の「メモ化＋世代で取消」。**第二のGPU schedulerではなくCPUのmemo層**。`salsa` は決定済みで不採用 |
| frame_graph/ `*_program`（約7.4k） | 領域ごとの意味。KEEP |
| compositor/ | re_renderer を呼ぶ薄い層＋run切りの法則。KEEP |
| re_renderer / wgpu | 借りている |

**第二renderer・第二scene engineは見当たらない**（描画は re_renderer、選択・outlineも借りている）。**第二になっているのは資源の寿命**（`EffectScratch`・`Pool`・生のGPU create）と、一部の直接submit。
render graph と pass scheduler は、そもそも re_renderer に無く、借りる物が無い。

## 6. 次の一手（実装はしない。各粒でゲートを通す）

1. Media Browser の一覧描画を遅延構築へ（実害。`heavy_folder_test` が赤から緑になることを審判にする）
2. Explore の layout を native の Barnes–Hut へ（UIスレッドの外）
3. 弧長を `kurbo` へ（Trim系の土台。誤差の保証が付く）
4. GPU compute の土台を re_renderer の pool 経由へ（`owned_budget` を緑に戻す）
5. `blob.rs` の輪郭・ラベルを `imageproc` へ

## 出典

ゲート: [git-interpret-trailers](https://git-scm.com/docs/git-interpret-trailers)、[Danger JS](https://danger.systems/js/)、[require-checklist-action](https://github.com/mheap/require-checklist-action)、
[dependency-review-action](https://github.com/actions/dependency-review-action)、[cargo-vet](https://mozilla.github.io/cargo-vet/)、[log4brains](https://github.com/thomvaill/log4brains)。
Map: [kurbo](https://crates.io/crates/kurbo)、[i_overlay](https://github.com/iShape-Rust/iOverlay)、[vello](https://github.com/linebender/vello)、[parley](https://github.com/linebender/parley)、[lyon](https://crates.io/crates/lyon)、
[forceatlas2](https://crates.io/crates/forceatlas2)、[fdg](https://github.com/grantshandy/fdg)、[graphview](https://pub.dev/packages/graphview)、[two_dimensional_scrollables](https://pub.dev/packages/two_dimensional_scrollables)、
[flutter_staggered_grid_view](https://pub.dev/packages/flutter_staggered_grid_view)、[imageproc](https://crates.io/crates/imageproc)、[OpenCV](https://opencv.org/license/)、[RIFE](https://github.com/nihui/rife-ncnn-vulkan)、
[Rapier](https://github.com/dimforge/rapier)、[OpenFX](https://github.com/AcademySoftwareFoundation/openfx)、[ocio-rs](https://github.com/shaloong/ocio-rs)、[Symphonia](https://lib.rs/crates/symphonia)、[Kira](https://lib.rs/crates/kira)、
[ffmpeg-sidecar](https://lib.rs/crates/ffmpeg-sidecar)、[QLThumbnailGenerator](https://docs.rs/objc2-quick-look-thumbnailing/latest/objc2_quick_look_thumbnailing/struct.QLThumbnailGenerator.html)、
[freedesktop thumbnail spec](https://specifications.freedesktop.org/thumbnail/latest-single/)、[salsa](https://github.com/salsa-rs/salsa)、[zenfg](https://docs.rs/zenfg/latest/zenfg/)、[flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge)。
ライセンス・最終release日・性能は、ここに挙げた物の多くで**未確認**（採択する粒で確認する）。
