# Technology Acquisition Gate と再発明候補の監査（2026-09-30）

状態: **決定**（ゲート）／**観察**（候補。置換は各粒の着手時にゲートを通してから）

## 1. ゲートをどこへ置いたか

方針は既にあった — [既知実装採択モデル](../known-implementation-adoption-model.md)（2026-08-02）と[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)。
効かなかった理由は方針の不足ではなく**位置**だった: 10欄・7 routeの票は「M3〜M5の採択地図」を作る計画者の道具で、日々の実装が最初に読む
`motolii/AGENTS.md` には1文しか無く、止める点が無かった。新しい制度は足さず、次の4点だけを動かした。

| 置き場 | 変更 | 行数 |
|---|---|---|
| `motolii/AGENTS.md` | 「作ってよいのは…」の2行を、同じ2行のまま Technology Acquisition Gate（6項目）に置き換え | 25 → 25 |
| 採択モデル §0 | 通常の実装が使う6行の短票、Buildの理由にならない物、要件定義の順。10欄は採択地図用に残す | +25 |
| commit（`.githooks/commit-msg`、CI） | 新しい150行以上のソースfileを加える commit は `Acquisition: Reuse\|Wrap\|Adapt\|Extend\|Semantics\|Build — …`。Build は `Why-build:` も要り、近道の言葉は通らない | script 1本 |
| PR template | 既存の「Known implementation / thin seam」欄をゲート項目へ差し替え | 1 → 1 |

Decision: `Reuse → Wrap → Adapt → Extend → Build`。Motoliiが持つのは作品の意味（identity・時間・parameter接続・scope・Undo・persistence・依存/無効化・
preview/exportの意味・provider契約）で、一般技術は借りる。`Semantics` は「一般技術は作らず意味だけを書いた」時の申告。

限界（正直に）: hook は「申告の有無と形」を強制するだけで、申告が正しいかは判定できない。嘘の `Reuse` は通る。止めているのは
**無意識に書き始めること**で、Build は理由の形（30字以上・近道の語を含まない）と利用者への返却で高くついた選択になる。
履歴400 commit に当てると34 commit（約8%）が対象で、最新は今日の Explore（自前の force layout を含む394行）だった。

## 2. 現行codeの再発明候補（置換はしない。各粒の着手時にゲートを通す）

「確認済」は現在のcodeを読んだ事実、「未確認」は名前や規模からの推測。

| # | 自作している物 | 既存の何へ委ねられそうか | Motolii側に残す意味 | 置換価値 | 置換リスク |
|---|---|---|---|---|---|
| 1 | 長い一覧の描画と配置: `FluidBoard` が全faceをwidgetにし、`MediaLibraryBody` も全tileを作る（確認済。1,845件のfolderで Thumbnail の最悪フレーム1.2秒） | Flutter の遅延構築（sliver / `ListView.builder`）、必要なら staggered grid の既存package。数はcatalogの paging | face の identity が view 間で続くこと、選択 ring、置く操作 | **高**（実害が出た） | 中（view間の滑らかな移動は見える範囲だけに限る） |
| 2 | Exploreのforce layout（`explore_graph.dart` の `_settle`、Fruchterman–Reingold をDartで O(n²)、UI thread。確認済） | force-directed の既存実装（Barnes–Hut 付きのRust crate、d3-forceの手法）を native 側で。Relations Graph も同じ需要 | 近さ（similarity）の意味、地図の記憶、選択で動かない契約 | 中〜高（300件超で地図を出せない原因） | 中（記憶と連続性を保つ写像が要る） |
| 3 | thumbnail / face の生成（`editor/thumbnail.rs` 218行、catalog の `faces`、ffmpeg-sidecar。確認済） | OSのサムネイル機構（macOS の QuickLook thumbnail 等）、ffmpeg の thumbnail filter と標準の cache | face key（ファイルが変われば変わる）、previewが出す物 | 中 | 中（3 OS の対応） |
| 4 | Rust→Swift→Dart の status 同期（手書きJSON、`snapshot_cache`、known id。確認済。document size で遅くなる根。2026-09-30の遅延作業で転送は簡素化済み、delta は未決） | Rerun の store event / `ChunkStoreSubscriber`、既存の差分同期の仕組み、FFI の typed data | snapshot の意味、reference id、Undo の見え方 | 高 | 高（wire の変更） |
| 5 | store の読みcache（`RecordCache` / `TrackCache`、snapshot の行cache、`frame_graph/cache.rs`。確認済） | `re_query::QueryCache` + `ChunkStoreSubscriber` | WorkKey と時間の意味、preview の上書き | 中 | 中〜高 |
| 6 | texture pool（`EffectScratch`。ledger 6番。確認済） | `re_renderer` の `GpuTexturePool` | effect の中間 target の寿命 | 低〜中 | 中（同一frame内の再利用） |
| 7 | 選択範囲のGPU縮約と staging の状態機械（`selection_bounds.rs` 216行。確認済） | `re_renderer` の picking / outline mask と `GpuReadbackBelt`（fork は texture の読み戻しだけ。buffer 読み戻しは fork へ Extend） | 選択 bounds の意味、Stage との同じコマ | 中 | 中 |
| 8 | GPU compute の土台（`block_program/` と `engine/blocks.rs`。生の pipeline / shader module / bind group layout を持つ。確認済で `owned_budget` が赤） | `re_renderer` の pool 経由。solver は `rapier3d`（既に依存） | 箱 block の意味、効果のデータ | 高（自前天井の違反） | 高（各passの中身は**未確認**。置換前に棚卸し） |
| 9 | path の trim / 演算（`shapes_ops/ops/trim.rs` など526行。規模のみ。未確認） | `lyon_geom` / `lyon_algorithms` の既存操作 | 効果のデータとしての意味 | 低 | 低〜中 |

決裁済みで再評価しない物: `salsa` による評価正本の反転、`Tauri` 全面導入、別の full UI framework、外部DB／WALへの journal 移管（[references.md](../references.md) の「責任委譲候補」）。
**すでに借りている物**: `taffy`、`cosmic-text`、`tiny-skia`、`lyon_geom`、`rubato`、`cpal`、`ffmpeg-sidecar`、`re_video`、`rapier3d`、`rusqlite`、`walkdir`、`notify` — 方針が効いている所。

## 3. 検証: 今後、LLMが新機能を実装しようとしたらどう止まるか

例「Motion Tracking」。LLMが `motolii/crates/motolii-render/src/tracking.rs`（220行）を足そうとして commit する。

1. `Acquisition:` 行なし → hook が止める。（`new code without an Acquisition line`）
2. `Acquisition: Build — simpler than adding a dependency` と `Why-build: simpler, small code, avoid a dependency` → 止める。（`that Why-build is not a reason`）
3. `Acquisition: Reuse — ffmpeg-sidecar で得た動き解析（または OpenCV 等の既存 tracker）をWrapし、結果を Track として Document に持つ` → 通る。
   この時点で LLM は Capability・既存技術・Motolii の意味（Track 結果の identity / 時間 / parameter 接続 / Undo / persistence）を書いている。

実際の出力は同日のcommitメッセージと検証ログを参照（scripts/check-acquisition.sh を一時worktreeで実行）。
