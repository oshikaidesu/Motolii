# Rerun学習・転移計画(2026-07-20)

ステータス: **学習計画(観察。転移判定は項目ごとに仮判定を併記)**。対象は[固定commit inventory](2026-07-20-rerun-pinned-commit-source-inventory.md)(Rerun 0.34.1 / `4efb18f`)。本計画は読む順序と検証質問の台帳であり、**依存追加・コード移植・製品実装の許可ではない**。採否の方向は[先例調査・方向決定](2026-07-20-rerun-prior-art-direction.md)を正本とする。

## 0. なぜRerunを学習源にするか(前提)

1. [egui採用判断](2026-07-18-m3-egui-selection.md)直後であり、egui 0.35 / egui_tiles 0.16 / wgpu 29 / winit 0.30.13という**同一version組で動く実運用規模のegui appはRerunが最大級**。version読み替えなしで実装事実を観察できる
2. Motoliiが既決の境界(egui_tiles生Treeを正本にしない、UI状態の分類、CJK font同梱、ffmpeg-sidecar)と**独立に同型へ収束した実例**を複数持ち、既決の裏付け/反例探索の両方に使える
3. dual license(MIT OR Apache-2.0)で、[references.md](../references.md)の区分上「コードを読める」対象である

同時に、RerunはMotoliiと**目的が違う**(可視化viewer。作品Documentの編集ツールではない)。学習は「同じ問題を解いた箇所」に限定し、目的差を無視した輸入をしない(§3)。

## 1. 転移対象マップ

判定語: **転移候補**(Motolii側の既決/チケットへ翻訳して使う)/ **参考**(設計判断の材料に留める)/ **移さない**(Motolii境界と矛盾)。いずれも本計画では仮判定であり、確定は各チケット/仕様側で行う。

| # | Rerun資産 | 学ぶこと | Motolii対応先 | 仮判定 |
|---|---|---|---|---|
| L1 | blueprint(`re_viewport_blueprint`、`.rbl`、公式doc) | **layout/view構成を専用データモデルに正本化し、UI(egui_tiles)は毎frameの投影とする**構造。保存・読込・portable化。「viewerで見た目を変える操作=blueprint編集」という一元化 | 「Motolii所有の安定layout model→egui_tiles投影」既決([egui採用判断§1](2026-07-18-m3-egui-selection.md)、[M3仕様](../specs/M3-ui-integration.md))の具体化。Workspace-session状態の設計 | 転移候補 |
| L2 | viewer Undo/Redo([#3135](https://github.com/rerun-io/rerun/issues/3135)、0.21〜) | UI側編集(layout/表示設定)のundoを、egui状態でなく**データモデル(blueprint)上の履歴**として実装した構造。regression [#10304](https://github.com/rerun-io/rerun/issues/10304)の原因分類 | MotoliiのUI状態4分類のうちWorkspace-session候補のundo扱い(D2 command/単一writerとは別系)。GR-UI 1/3の検討材料 | 参考(機構未読) |
| L3 | `gpu_bridge/re_renderer_callback.rs` | `egui_wgpu::CallbackTrait`でrender passへ直接composite する接合。viewport clamp回避、`prepare`/`paint`二相、`Mutex<ViewBuilder>`共有 | Motoliiは`register_native_texture`+最新値mailbox+worker renderを採用済み。**UI frame内でdomain renderを回す構造は境界違反**([egui採用判断§5](2026-07-18-m3-egui-selection.md)) | 移さない(比較記録として保持) |
| L4 | `re_time_panel`(`data_density_graph.rs`ほか) | 大量エンティティ×時間軸UIをeguiで成立させる分割(axis/selection/control/density)。密度描画・購読(`recursive_chunks_per_timeline_subscriber`)の構造 | Motolii timelineは「toolkit非依存のlayout/hit-test/render model+単一wgpu面」既決。**widget木で行を作る方式は採らない**が、モジュール分割・密度表現・時間軸snapは参考になる | 参考 |
| L5 | `re_ui/data/*.ron`(dark/light theme、color_table) | テーマ実値をcode外のデータとして持ち、生成/検証する運用。icon体系、Inter単一font運用 | U0e(DTCG token JSON→Rust/egui adapter決定的生成)の先例。[ui-visual-language.md](../ui-visual-language.md) | 転移候補 |
| L6 | CJK未解決([#12770](https://github.com/rerun-io/rerun/issues/12770)) | 同梱fontがInterのみのegui大規模実運用appでも、CJKは豆腐表示のままopen(blocked/egui)である事実 | 「CJK font同梱またはOS font resolver必須」([egui採用判断§4](2026-07-18-m3-egui-selection.md)、G0-6)を**待っても解決しない**裏付け。egui任せの縮退案を棄却する材料 | 転移候補(判断の裏付け) |
| L7 | `re_video`(ffmpeg-sidecar / dav1d / WebCodecs) | H.264をffmpeg-sidecar(CLI)で解く構成が実運用されている事実。AV1のdav1d(ARM64除外・nasm)、containerパースの分離 | B-2(ffmpeg-sidecar第一候補)の**独立収束事例**。コーデック別方針の比較材料 | 参考(裏付け) |
| L8 | `egui_table` 0.9.0 | 大量行("millions of rows")・sticky header・可変行高のtable widget。rerun-io保守、egui 0.35対応 | Browser list表示・大量asset行の候補部品。ただし現時点で必要性未確定 | 参考(未評価候補) |
| L9 | `re_renderer` | standalone可のwgpu renderer(WebGL tier、hot shader reload、resource cache) | Motoliiはrenderコア自前+Vello採用済み。rendererの輸入はしない。resource cache/hot reloadの設計だけ[dev-experience.md](../dev-experience.md)の参考 | 移さない(設計参考) |
| L10 | immediate mode運用(ARCHITECTURE.md)+`re_tracing`/`re_memory` | 「毎frame in-RAM storeへquery」を成立させる最適化文化と計測基盤の置き方 | Motoliiは毎frame Document全走査を前提にしない(mailbox/投影)。ただし「UI更新CPUを常時計測する」規律はG0-4系へ参考 | 参考 |

## 2. 読む順序(phase制。各phaseに検証質問を固定)

読み方の規律: 固定commit `4efb18f`のみを読む。1 phase = 1つの検証質問セットに答えたら、結果を該当既決文書/チケットの側へ登録して閉じる(Rerun側の要約を増殖させない)。

### Phase 1 — blueprint系(最優先。L1/L2)

対象: `crates/viewer/re_viewport_blueprint/` → `crates/viewer/re_blueprint_tree/` → viewer undo実装(re_viewer_context周辺)。

検証質問:

1. blueprintデータモデルとegui_tiles Treeの間の**投影・逆写像**はどこで行い、tab drag等のUI操作はどの粒度でblueprint編集になるか
2. blueprint編集のundo単位は何か。egui memory(開閉・scroll等のtransient)とblueprint(永続)を**どう線引き**しているか
3. `.rbl`の互換方針(旧blueprintを開く時の挙動)はあるか
4. panel開閉・サイズはblueprintに入るか、egui側に残るか — Motoliiの「Workspace-session候補」との対応表を作る

### Phase 2 — time panel(L4)

対象: `crates/viewer/re_time_panel/src/`一式。

検証質問:

1. 行(エンティティ)数が多い時、描画とhit-testをどう間引くか(可視範囲外の扱い、`data_density_graph`の集約粒度)
2. 時間軸のzoom/pan/snapとplayheadの状態はどこが持つか(blueprint/transientの別)
3. Motoliiの「単一wgpu面timeline」決定に対し、egui widgetで組んだRerunが**どの規模で足りているか**(限界の兆候: 専用culling、frame落ち対策コードの有無)

### Phase 3 — re_ui / theme(L5/L6)

対象: `crates/viewer/re_ui/`(`data/*.ron`、theme読込コード、icon管理)。

検証質問:

1. `.ron`→egui `Style`/独自tokenへの変換は生成か手書きか。実値の検証(contrast等)はあるか
2. componentカタログ(`re_ui_example`)の構成 — U0eのreference screen審判の参考
3. font登録経路(`FontDefinitions`)と、CJK fallbackを入れる場合の差し込み点

### Phase 4 — 接合と計測(L3/L7/L10)

対象: `gpu_bridge/`一式、`re_video/src`、`re_tracing`の使われ方。

検証質問:

1. CallbackTrait方式の制約(pass内で何ができないか、resize/DPI時の挙動)を、Motolii native texture方式の負例表として整理
2. re_videoのcolor space/転送(decode→GPU texture)の置き場所 — MotoliiのB-2/色変換一元化と比較
3. puffin等どの計測を常設し、どこにspanを置いているか

## 3. 転移の停止線

1. **依存しない**: `re_*` viewer系crateをMotolii workspaceの依存に加えない(ecosystem crateであるegui_tiles採用済み、egui_tableは未評価候補として別途審査)。理由は[方向決定§5](2026-07-20-rerun-prior-art-direction.md)
2. **UI frame内renderを輸入しない**: CallbackTrait方式・毎frame store query方式を、mailbox/worker境界([egui採用判断§5](2026-07-18-m3-egui-selection.md))の代替として持ち込まない
3. **Rerunの意味論を持ち込まない**: entity path/ECS archetype/blueprint timelineの語彙・構造をMotolii Document/公開契約へ写さない。学ぶのは「layout正本の分離」という構造だけ
4. **コード移植時はライセンス手続きを踏む**: dual licenseで移植自体は可能だが、行単位の流用が発生する場合は出典commit・ライセンス表記を残し、[references.md](../references.md)の区分を更新してから行う。既定は「読んで設計を学ぶ」に留める
5. **timelineをwidget木で作り始めない**: Phase 2はあくまで観察。Motoliiの単一wgpu面決定([egui採用判断§5](2026-07-18-m3-egui-selection.md))の再審は、観察結果を持って別文書で行う

## 4. 深掘り残課題(未調査の明示)

- chunk store / `re_query`のcache構造と無効化(毎frame queryの実コスト内訳)
- time panelのculling/仮想化の実装有無(Phase 2で確定)
- blueprintのversioning・migration(公式docに記載なし。実装側の確認要)
- viewer undoの実装機構(L2。issue完了の事実のみ確認済み)
- 音声の扱い(re_videoは映像のみに見えるが未確認)
- egui 0.35のmulti-pass等、Rerunが上流eguiへ入れた変更の系譜
- 他の大規模egui先例との横断比較([方向決定§6](2026-07-20-rerun-prior-art-direction.md)の限界と同じ)

## 5. 一次資料

[固定commit inventory §8](2026-07-20-rerun-pinned-commit-source-inventory.md)の一覧を共有する(同一固定commit)。本文書からの追加はなし。
