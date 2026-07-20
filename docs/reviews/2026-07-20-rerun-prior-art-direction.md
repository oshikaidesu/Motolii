# Rerun先例調査・方向決定(2026-07-20)

ステータス: **先例調査+方向仮判定(反対側レビュー・ユーザー確認待ち)**。egui採用([判断](2026-07-18-m3-egui-selection.md))を受け、「実運用規模のegui appから何を・どう学ぶか」の方向を定める。[レビュー規律](README.md)に従い、本文書の結論を単独で設計根拠にしない。事実の台帳は[固定commit inventory](2026-07-20-rerun-pinned-commit-source-inventory.md)、実行手順は[学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)。

## 1. 調査質問

1. egui採用直後のMotoliiにとって、**同一構成(egui 0.35/wgpu 29)で実運用されている先例**はどれか
2. その先例から転移できるのはどの層か(構造/部品/運用)。転移できない目的差はどこか
3. 「学ぶ」の具体形は何か — 依存するのか、読むだけか、部品単位で借りるのか

## 2. 事実(要点。正本はinventory)

- Rerun 0.34.1(固定commit `4efb18f`、2026-07-07)は egui 0.35.0 / egui_tiles 0.16.0 / egui-wgpu 0.35.0 / wgpu 29.0 / winit 0.30.13 に**crates.io版のみで**依存する(fork patchなし)。[egui採用判断](2026-07-18-m3-egui-selection.md)の実測構成と同一
- blueprint: 「viewerで見た目を変える操作は全てblueprint(データ)の編集」であり、layout/view構成は専用データモデル(`.rbl`保存可)に正本化され、UIは毎frameそこから導出される。viewer Undo/Redoも0.21で実装済み
- 接合: viewportは`egui_wgpu::CallbackTrait`によるrender pass内compositeで描く(native texture登録方式ではない)
- 同梱fontはInterのみで、CJKは豆腐表示のままopen([#12770](https://github.com/rerun-io/rerun/issues/12770)、blocked/egui)
- H.264デコードにffmpeg-sidecar(CLI)を使用 — MotoliiのB-2第一候補と同一crate
- README: "We are in active development… _Expect breaking changes!_"。release はminor月次+随時patch、viewer系crateはcrates.io公開だがre_uiだけで累計278版
- egui_tiles(0.16.0)/egui_table(0.9.0)はrerun-io保守の**汎用ecosystem crate**として本体と分離配布

## 3. 判定の骨子

### 3.1 学習正本としてのRerun — 採用(仮判定)

Rerun 0.34.1固定commitを、M3期の「egui実運用パターンの第一学習源」とする。

転移条件の検査([規律](README.md)2に従う):

- **事実**: 上記§2は一次資料(固定commitのファイル・公式doc・issue)で再確認可能
- **転移条件**: Motoliiと同じtoolkit・同じversion組・同じ「wgpu renderer所有者とUI shellの分離」問題を持つ。ここは成立
- **因果**: 「Rerunが成功しているのはこれらの構造のおかげ」とまでは本調査では言えない(反例未探索)。よって各項目は「**仮説と整合する事例**」として扱い、Motolii側の既決の裏付け/検証質問の形でのみ使う
- **より小さい対策**: 依存せず読むだけ、が最小。以下§5で固定

### 3.2 blueprint構造 — Motolii既決の裏付けとして採用(仮判定)

「egui_tilesの生`Tree`/`TileId`/serdeを保存正本にせず、Motolii所有の安定layout modelから投影する」というMotolii既決([egui採用判断§1](2026-07-18-m3-egui-selection.md))は、Rerunのblueprint(データ正本→毎frame投影、`.rbl`portable、undo対象)と**独立に同型**である。既決の変更は不要。学習は「その正本をどう設計すると10 panel規模で破綻しないか」の具体化([転移計画](2026-07-20-rerun-learning-transfer-plan.md) Phase 1)に限定する。

### 3.3 ecosystem crateと viewer crateの区別 — 採用(仮判定)

- egui_tiles: 採用済み(既決)。保守元がRerun本体と同じ組織で、本体と同時にegui 0.35へ追従している事実は、egui_tiles採用のversion追従リスクを下げる材料
- egui_table: **未評価候補**として登録のみ(Browser大量行の部品候補)。採否は必要が生じたチケットで審査
- `re_ui` / `re_viewer` / `re_renderer`等のviewer系crate: **依存しない**(§5)

### 3.4 接合方式(CallbackTrait) — 移さない

RerunのCallbackTrait方式は「UI frameの中でdomain rendererを描く」構造であり、Motoliiの「UI threadでMotolii frameをrenderしない/最新値mailbox/generation破棄」境界([egui採用判断§5](2026-07-18-m3-egui-selection.md)、GR-UI 4)と両立しない。Motoliiはnative texture方式を維持する。ただしCallbackTrait方式は**実運用の対抗事例**なので、native texture方式が行き詰まった時の比較対象として記録を保持する(その時は仕様改訂を先に行う。黙って切り替えない)。

### 3.5 個別の裏付け(判断済み事項の補強。新決定なし)

- CJK font同梱必須(G0-6) — Rerunの未解決open issueが「egui任せでは解決しない」ことを示す
- ffmpeg-sidecar(B-2) — 実運用の独立収束事例
- テーマ実値のデータ化(U0e) — `.ron` themeの先行実例

## 4. 移さない部分(目的差の明示)

RerunはviewerでありMotoliiはauthoring toolである。次はRerunに**存在しない/解いていない**ため、先例として使わない。

- 作品Document・command journal・単一writer(Rerunのundoはblueprint=表示設定の編集に対するもので、作品データの編集ではない)
- キーフレーム編集・補間・タイムラインでのauthoring(time panelは閲覧・scrub UI)
- 色管理(OCIO-shaped一元化に相当する層は未確認 — 未調査項目として残す)
- 書き出し(export/encode)パイプライン
- 音声同期・音声主クロック(未確認。残課題)
- 毎frame in-RAM store queryを前提にした全体設計(Motoliiはrender workerと投影で分離する)

## 5. 方向(仮判定まとめ)

1. **採用(仮)**: Rerun 0.34.1固定commitを学習正本とし、[転移計画](2026-07-20-rerun-learning-transfer-plan.md)のPhase 1〜4を、対応するM3作業(layout model、timeline、U0e、PV系)の**着手前調査**として実行する
2. **採用(仮)**: 学習の出力は「Rerun解説」ではなく、Motolii側の既決文書・チケットへの追記の形で回収する(Rerun要約の二重管理をしない)
3. **停止線**: `re_*` viewer系crateへの依存追加、CallbackTrait方式への黙った切替、Rerun語彙(entity path/archetype等)のDocument/公開契約への持込、をSTOP条件とする
4. **棄却**: 「Rerunが動いているからeguiのCJK/性能問題は放置できる」という縮退(CJKはRerun自身が未解決)
5. **登録のみ**: egui_tableを未評価候補として[references.md](../references.md)へ登録

## 6. 調査の限界

- **他先例との横断比較を実施していない**。egui採用済み大規模appの網羅(比較母集団)を作らずにRerunを第一学習源としており、これは「同一version組・保守活発・ecosystem crate保守元」という実利による選定であって、最良である証明ではない。第二先例が必要になった時に別途調査する
- Rerunソースの精読は未着手で、本文書はREADME/公式doc/issue/ファイル所在の確認に基づく。Phase実行で事実が覆れば本文書を改訂する
- viewer undoの実装機構、blueprintのmigration、音声、色管理の各項目は未確認のまま「移さない/残課題」に置いた
- 本文書の仮判定はユーザー確認と反対側レビューを経ていない。[規律](README.md)6に従い、ゲート・仕様へ採用する時は判定語を併記して再判定する

## 7. 一次資料

[固定commit inventory §8](2026-07-20-rerun-pinned-commit-source-inventory.md)と同一(固定commit `4efb18f`基準)。
