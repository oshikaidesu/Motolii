# docs/ 読み方ガイド

このディレクトリが**現在の設計の唯一の情報源**。コードを読む前にここを読む。
矛盾する記述を見つけたら、それはバグとして扱い修正する(旧仕様の混在は許容しない)。

> 整理履歴(2026-07-08): 初期検討資料 `design-memo.md`(2026-07-05) と `discussion-log-2026-07-06.md` は、現決定と矛盾する旧仕様(Tauri+WebView採用、OpenCut Reactコード流用等)を含むため削除した。生きた決定はすべて [concept.md](concept.md) に移植済み。経緯が必要ならgit履歴を参照。

## 30秒サマリ

- **何を作るか**: MV(ミュージックビデオ)制作のための、モーショングラフィック指向のコンポジットツール。AEの重さへの構造的な回答。3〜5分の動画を書き出せたら完成
- **長期の北極星**: 映像表現を、時刻・入力・型付きparameterから決まる再利用可能な単位として演奏・保存・配布できる共通実行環境にする。「映像制作におけるVST」は構造の比喩であり、VST互換やDAW化は目標ではない([concept.md](concept.md#長期の北極星-映像表現を演奏再利用配布できる単位にする))
- **技術スタック(確定)**: Rust + wgpu(レンダコア、VRAM常駐) / Slint(UI、wgpuゼロコピー統合。WebView/Tauri案は廃止) / ffmpegサイドカープロセス(デコード・エンコード) / Cargo workspace(`crates/motolii-*`)
- **開発方式**: 仕様書駆動の並列AIエージェント開発。凍結ゲート宣言済みだが、現在は[M2基盤再締結ゲート](reviews/2026-07-15-m2-foundation-reclosure-gate.md)を優先し、M3製品実装を停止中。M2再締結は別PRのA〜C証跡が必要
- **設計目標の代表値**: 1080p動画レイヤー40本同時で破綻しない / プロセス強制終了しても編集を失わない(コマンドジャーナル) / フレーム並列(マルチコア)を構造で保証

## 読む順序(初見向け)

1. [concept.md](concept.md) — 何であって何でないか。**全決定事項の台帳**(スコープ、プラグイン境界、座標系、並行性、音声方針)
2. [performance-model.md](performance-model.md) — 「なぜAEより軽くできるか」の物理(メモリ帯域モデル)、品質モード(Draft/Final)、並列性、40レイヤー目標の試算。**容量・VRAM上限への疑念は[memory-model.md](memory-model.md)(疑念台帳)へ**
3. [pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) — **最重要・最大**。落とし穴カタログ(A〜H、先行プロジェクト死因分析+LLM開発規律込み)とロードマップ(M0〜M5)、凍結ゲート
4. 実装に着手する時: [implementation-ledger.md](implementation-ledger.md)(NOW/NEXT/WAIT)→ [specs/README.md](specs/README.md)(プロセスとステータス表)→ 対象`specs/M*.md`(タスク表と**末尾の「実装ガード」節**の両方を読む)
5. プラグインを書く/量産させる時: [plugin-authoring.md](plugin-authoring.md)(LLM/人間共通の契約・禁止事項・型紙)
6. 依存・参考リポジトリを調べる時: [references.md](references.md)(ライセンス区分つき。GPL系はコードを読むことすら禁止)

## ファイルマップ

| ファイル | 役割 | 状態 |
|---|---|---|
| [concept.md](concept.md) | コンセプト定義・決定事項の台帳 | 現行(決定はここに追記される) |
| [performance-model.md](performance-model.md) | 性能の設計根拠と規律 | 現行 |
| [memory-model.md](memory-model.md) | メモリ階層(VRAM/RAM/ディスク)の役割分担と容量疑念の台帳 | 現行 |
| [simulation-model.md](simulation-model.md) | 時間軸の自由度モデル: 物理シミュレーション(SimulationPlugin+StateTrack)と前後フレーム参照(宣言的時間窓)の設計 | 現行(2026-07-10。口の予約段階、実装v1.x) |
| [pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) | 落とし穴カタログ+ロードマップ+凍結ゲート | 現行 |
| [plugin-authoring.md](plugin-authoring.md) | プラグイン作者向け規約(LLM並列量産の契約書) | 現行(2026-07-10) |
| [plugin-resources.md](plugin-resources.md) | プラグインのリソースライフサイクル・アセット境界・時間参照(F-10/F-11) | **凍結ゲートで確定**(実装残はM2) |
| [references.md](references.md) | 依存候補・参考リポジトリ(ライセンス区分) | 現行 |
| [ae-pain-points.md](ae-pain-points.md) | AEユーザー不満の体系化+我々の解決タグ(プラグイン窓口仮説の検証) | 現行 |
| [dev-experience.md](dev-experience.md) | 開発体験(DX): プラグイン/シェーダのホットリロードはしご(AE再起動地獄の予防) | 現行(2026-07-13。設計ノート、契約変更なし) |
| [plugin-ui-model.md](plugin-ui-model.md) | プラグインUIモデル: 宣言語彙 vs 自由描画。M3着手前決定で縮小採用 | **採否済み分析**(v1はHost自動生成panel、自由UIは延期) |
| [interaction-simplicity-model.md](interaction-simplicity-model.md) | 操作単純化モデル: Direct/Tool/Advanced正規化、plugin昇格、PP-Gate、M0〜M5割当 | 現行(2026-07-14。凍結済み公開契約は変更しない) |
| [generative-user-boundary.md](generative-user-boundary.md) | ジェネラティブ表現とユーザー拡張の境界: Shape/SVG、p5.js型入力、Materialize/Live/Feedback/Simulation、Host責務 | **設計決定**(2026-07-15。未凍結runtimeの実装許可ではない) |
| [ui-interaction-language.md](ui-interaction-language.md) | M3のUI操作言語: 既知の外殻、可視の因果、Parameter Panelを表現のホームにするUI力学、共通component契約、Simple/Advanced、漏れ実装の拒否 | **設計決定**(2026-07-16、Parameter Panel力学を2026-07-18追補) |
| [ui-visual-language.md](ui-visual-language.md) | M3の視覚言語: 高密度一覧、意味色、既存UIへの馴染み、contrast、token規約、参照範囲 | 設計基準(具体token値はM3視覚確定(G0-6)待ち) |
| [mocks/](mocks/README.md) | M3高密度メインUI(基準)+timeline/interaction/UI力学の比較モック台帳 | 視覚構成の基準モック |
| [ui-concept.md](ui-concept.md) | UIコンセプト: 体験の北極星(譜面台・First Beat・五本柱)。散在するUI文書の層地図つき | **設計仮説・反対側レビュー待ち**(2026-07-16。契約・M3ステータス変更なし) |
| [implementation-ledger.md](implementation-ledger.md) | 現場向け実装進行台帳: M0〜M5のNOW/NEXT/WAIT、依存、Issue昇格順 | **日々の発注入口**(意味・完了条件は各specが正本。M3行は再締結ゲート優先) |
| [backlog.md](backlog.md) | イシュー候補台帳(現在地サマリ+横断/新規ギャップ/v2バックログ) | 現行 |
| [specs/](specs/README.md) | マイルストーン仕様書(エージェントへの発注書)。確定/ドラフトのステータスはspecs/README.md参照 | M0/M1確定、M2段階発注可(基盤再締結ゲート発効中)、M3ドラフト/製品実装停止、M4/M5ドラフト |
| [spikes/](spikes/) | スパイク結果報告(S1: Slint統合、S2: デコード、[S3(R8): Vello採否](spikes/s3-vello.md)) | 完了報告(歴史的記録、更新しない) |
| [reviews/2026-07-12-m2-permanence-prevention.md](reviews/2026-07-12-m2-permanence-prevention.md) | M2恒久焼き込みの**予防手順**(やること5手)。運用正本 | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-prevention.md](reviews/2026-07-14-m3-ui-boundary-prevention.md) | M3でUI都合をDocument・レンダ・公開契約へ逆流させない**予防手順**(規律8本) | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-counter-review.md](reviews/2026-07-14-m3-ui-boundary-counter-review.md) | M3 UI境界規約の反対側レビュー。R1〜R9を採用/縮小/延期で再判定 | 現行(判定反映済み) |
| [reviews/2026-07-16-m3-preflight-decisions.md](reviews/2026-07-16-m3-preflight-decisions.md) | M3着手前決定: input/状態寿命、plugin UI、性能測定、操作文法を固定し、見た目とresource実値を証拠待ちへ分離 | **設計決定**(G0-2/3/4/7完了) |
| [reviews/2026-07-16-m3-ui-concept-to-tickets.md](reviews/2026-07-16-m3-ui-concept-to-tickets.md) | UIコンセプトを1 Issue=1 commitの実装粒へ分解。状態、入力、視覚、preview、共通操作、最初のEffect panelの依存と拒否条件 | **実装発注の正本**(U0b〜U4aの枝番) |
| [reviews/2026-07-16-ui-update-forensics.md](reviews/2026-07-16-ui-update-forensics.md) | Figma/Ableton/AE/Blender/Godot/Home AssistantとLinux GUIの公式更新・fork履歴から、UI失敗、不安定platformの隔離、user拡張をMotoliiのcomponent審判へ変換 | **調査と採用審判**(AF-1〜17) |
| [reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md](reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md) | VSM-A0: 現行pluginの登録・保存・評価・migration境界をコード事実で分類 | **調査完了** |
| [reviews/2026-07-17-vism-a7-bpm-datatrack-spike.md](reviews/2026-07-17-vism-a7-bpm-datatrack-spike.md) | VSM-A7: 現行BPM→DataTrack→DocParamの最小意味fixture | **spike完了** |
| [reviews/2026-07-17-vism-a0d-contract-migration-ownership-decision.md](reviews/2026-07-17-vism-a0d-contract-migration-ownership-decision.md) | VSM-A0D: Document、plugin作者、Host catalog、executorの所有分離 | **設計決定** |
| [reviews/2026-07-17-vism-a0s-contract-catalog-spec.md](reviews/2026-07-17-vism-a0s-contract-catalog-spec.md) | VSM-A0S: Contract Catalog、prepared resolution、runtime公開境界 | **A0I-1〜3実装完了** |
| [reviews/2026-07-17-vism-a1-public-crate-boundary-spec.md](reviews/2026-07-17-vism-a1-public-crate-boundary-spec.md) | VSM-A1S: Opacity外部crate化のfaçade、依存allowlist、first-party組み立て、必須capability、移動前pixel gate | **A1-3完了** |
| [reviews/2026-07-17-vism-a2-legacy-project-migration-decision.md](reviews/2026-07-17-vism-a2-legacy-project-migration-decision.md) | VSM-A2S: Sine外部crate化時の旧CLI ProjectV1 migration処分と公開façadeレビュー | **設計決定／A2実装可** |
| [reviews/2026-07-18-vism-a3-external-expression-survey.md](reviews/2026-07-18-vism-a3-external-expression-survey.md) | VSM-A3R: AE Expression／Script／Effect、aescripts、Blender Driver／Geometry Nodes／Simulation／Add-onを責任分類し、Parameter Panel中心のA3候補へ翻訳 | **調査完了**（採用決定は[A3D](reviews/2026-07-18-vism-a3d-radial-repeater-decision.md)） |
| [reviews/2026-07-18-vism-a3d-radial-repeater-decision.md](reviews/2026-07-18-vism-a3d-radial-repeater-decision.md) | VSM-A3D: 決定論的2D Radial Repeater LayerSource（`core.layer_source.radial_repeater` v1）のidentity・正準意味・parameter閉集合・UI投影要求・非目標 | **設計決定** |
| [reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md](reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md) | VSM-A3S: 一般LayerSource lowering（prepared→`RenderStep::Plugin`）、clear一般化、拒否分類、rect分離、画素契約、U4a handoff、A3分割発注表。[F1](reviews/2026-07-17-vism-implementation-plan.md)でHost cache GAPを訂正し、`VSM-A3-0`〜`VSM-A3-4`まで実装済み | **仕様・VSM-A3完了** |
| [reviews/2026-07-14-unified-stage-camera-design.md](reviews/2026-07-14-unified-stage-camera-design.md) | 2D/3Dを分けない単一カメラ、Stage、Output Frame、枠外表示の意味と実装順 | **決定**(2026-07-14) |
| [reviews/2026-07-14-recent-concept-propagation-audit.md](reviews/2026-07-14-recent-concept-propagation-audit.md) | 直近の根幹決定を意味・Document・評価・UI・依存・コードの6面で逆引きした未反映台帳 | 横断監査(2026-07-14) |
| [reviews/2026-07-14-motion-foundation-known-tech-disposition.md](reviews/2026-07-14-motion-foundation-known-tech-disposition.md) | Relative Move、Bounds/ROI、Effect Scope、Instance/Elementを既知技術で再判定した最小契約 | **決定**(2026-07-14) |
| [reviews/2026-07-15-relative-scope-duplicator-decision.md](reviews/2026-07-15-relative-scope-duplicator-decision.md) | modifier+drag、透過Stage、常時Effect接続線、Cavalry型Duplicator、stable seedの具体化 | **決定**(2026-07-15) |
| [reviews/2026-07-15-prior-art-complaint-boundary-audit.md](reviews/2026-07-15-prior-art-complaint-boundary-audit.md) | 先例が収束した固定契約と、Null/Group/Crop等の日曜大工帯を分離 | **調査第一陣**(2026-07-15) |
| [reviews/2026-07-15-implementation-readiness-ledger.md](reviews/2026-07-15-implementation-readiness-ledger.md) | M2〜M5のREADY/SPIKE/WAIT/BLOCKED分類とIssue昇格順 | **運用正本**(2026-07-15) |
| [reviews/2026-07-12-rework-prior-art.md](reviews/2026-07-12-rework-prior-art.md) | 出戻りの先人調査(予防側/失敗後の対比)。設計根拠ではない | 仮説メモ |
| [reviews/2026-07-12-pathop-ae-cavalry-comparison.md](reviews/2026-07-12-pathop-ae-cavalry-comparison.md) | PathOp語彙のAE/Lottie×Cavalry比較。意味【決定】前の材料(採択後は参考) | 調査メモ(未採用) |
| [reviews/2026-07-13-undecided-critical-path-confirm.md](reviews/2026-07-13-undecided-critical-path-confirm.md) | 友人レビュー確認: 未決の追跡先・クリティカルパス補正・B⑤コード確認 | 確認メモ |
| [reviews/2026-07-13-decision-pack-adoption.md](reviews/2026-07-13-decision-pack-adoption.md) | #103/#100/残小項目の**【決定】採択**(AE/Lottie・OTIO・DAW・Qt) | 現行(決定) |
| [reviews/2026-07-14-m2-core-closure.md](reviews/2026-07-14-m2-core-closure.md) | M2コア締結宣言(**撤回**・単独再宣言を廃止し再締結ゲートへ移行) | 撤回(2026-07-14) |
| [reviews/2026-07-15-m2-foundation-reclosure-gate.md](reviews/2026-07-15-m2-foundation-reclosure-gate.md) | M2恒久面の再締結条件とM3製品実装の停止線 | **発効宣言**(別の再締結PRでA〜C証跡待ち) |
| [reviews/2026-07-15-m2-foundation-reclosure-counter-review.md](reviews/2026-07-15-m2-foundation-reclosure-counter-review.md) | M2基盤再締結ゲートの反対側レビューと採否 | **P0/P1=0・発効merge可** |
| [reviews/2026-07-14-motion-foundation-known-tech-disposition.md](reviews/2026-07-14-motion-foundation-known-tech-disposition.md) | Relative Move、Bounds/ROI、Effect Scope、Instance/Elementを既知技術で再判定した最小契約 | **決定**(2026-07-14) |
| [reviews/2026-07-15-relative-scope-duplicator-decision.md](reviews/2026-07-15-relative-scope-duplicator-decision.md) | modifier+drag、透過Stage、Explicit Definition/Use、Cavalry型Duplicator、stable seedの具体化 | **決定**(2026-07-15) |
| [reviews/2026-07-15-prior-art-complaint-boundary-audit.md](reviews/2026-07-15-prior-art-complaint-boundary-audit.md) | 先例が収束した固定契約と日曜大工帯の分離 | **調査第一陣**(2026-07-15) |
| [reviews/2026-07-15-implementation-readiness-ledger.md](reviews/2026-07-15-implementation-readiness-ledger.md) | M2〜M5のREADY/SPIKE/WAIT/BLOCKED分類とIssue昇格順 | **運用正本**(2026-07-15) |
| [reviews/2026-07-15-shared-effect-lifecycle-decision.md](reviews/2026-07-15-shared-effect-lifecycle-decision.md) | Shared Effectの削除/Unlink/Copy Local/orphan lifecycle（GAP-14） | **決定**(2026-07-15 / #166) |
| [reviews/2026-07-15-d1l-copylocal-remint-counter-review.md](reviews/2026-07-15-d1l-copylocal-remint-counter-review.md) | D1l Copy Local内部ID契約の反対側レビュー、journal/counter指摘と採否 | **P0/P1=0・merge可**(PR #196) |
| [reviews/2026-07-15-d1l-journal-revert-boundary-decision.md](reviews/2026-07-15-d1l-journal-revert-boundary-decision.md) | D1lのJournalEdit v1→v2互換、Undo等価、Writer採番単一路の追補 | **決定・merge済み**(PR #197) |
| [reviews/2026-07-15-d1l-journal-revert-boundary-counter-review.md](reviews/2026-07-15-d1l-journal-revert-boundary-counter-review.md) | PR #197の反対側レビュー、採番/閉集合/orphan指摘と採否 | **P0/P1=0・merge可** |
| [reviews/2026-07-16-d1l-current-document-constructor-decision.md](reviews/2026-07-16-d1l-current-document-constructor-decision.md) | 新規Documentをv4で作る製品constructorと、legacy `new_v1`/D1e/D1l Commandの版境界 | **決定**(lint機構は下記追補) |
| [reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md](reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md) | `new_v1` deprecated属性とprotected semantic/clippyの三律背反を、`doc(hidden)`+AST gateへ一本化 | **決定追補** |
| [reviews/2026-07-17-d1i4-semantic-oracle-boundary-decision.md](reviews/2026-07-17-d1i4-semantic-oracle-boundary-decision.md) | D1i-4/S16の保護単位をtest harness全体から意味の期待値oracleへ訂正し、API配線と作品意味を分離 | **決定追補／BlendModeから段階移行** |
| [reviews/2026-07-16-d1l-current-document-constructor-counter-review.md](reviews/2026-07-16-d1l-current-document-constructor-counter-review.md) | 新規Document v4生成契約の版/構造検証/allowlist指摘と採否 | **P0/P1=0・merge可** |
| [reviews/2026-07-15-p5-generative-pattern-disposition.md](reviews/2026-07-15-p5-generative-pattern-disposition.md) | p5.js系ジェネ表現をone-shot/純関数/Feedback/Simulation/記録入力へ分類 | **調査・配置案**(2026-07-15) |
| [reviews/2026-07-16-m3-ui-gap-survey.md](reviews/2026-07-16-m3-ui-gap-survey.md) | M3前UIギャップ調査: U1〜U8に席が無いUI領域(書き出し/保存/エラー表示等)とコア側前提の欠落(状態購読/ParamDefメタデータ/Transport等) | **調査メモ**(2026-07-16。採否はM3入場PRで) |
| [reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md](reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md) | すぐに受け入れられたUIの先例集: 第一部=プロダクト単位の受容(界隈の期待リスト)、第二部=業界収斂した操作語彙+UX原理の一次資料(M3転移の本線)、第三部=後発の勝ち筋「どの操作も直感的」(Ableton→AEカウンター)。設計根拠ではない | 仮説メモ(2026-07-16) |

## 全体で守る規律(コードレビュー最重視項目)

どれか1つ破るだけでプロジェクトの根拠が崩れる、という種類のもの。番号は重要度順ではない。

1. **VRAM常駐**: ピクセルはwgpuテクスチャとしてGPUに置いたまま処理する。安易なCPU処理の混入1箇所で「AEより軽い」根拠が消える([performance-model.md](performance-model.md))。確定出力の非同期コピーアウトによるキャッシュ充填は例外([memory-model.md](memory-model.md) P1)
2. **色変換の一元化(OCIO-shaped)**: 色変換はレンダ直前の1箇所のみ。散らばった瞬間にOliveの二の舞(全書き直し)(落とし穴F-5)
3. **プラグイン純関数契約**: プラグインの出力は時刻tと入力だけで決まる。隠れた可変状態の禁止。これがフレーム並列(マルチコア)の前提で、破るとAEと同じ「後付け不能」になる([performance-model.md](performance-model.md)§6)。第一選択は常にf(t)の安い力(「馬鹿正直にシミュレートしない」[concept.md](concept.md))。それで書けない逐次状態表現だけ、この契約を破らずに**レンダ経路の外のベイク境界**で扱う([simulation-model.md](simulation-model.md))
4. **単一writer+不変スナップショット**: ドキュメントを書き換えるのは編集スレッド(コマンド適用)だけ。他は全員`Arc<Document>`の読み手。Natronの死因(race/deadlock)の構造的排除(落とし穴F-2)
5. **正準座標系**: 空間パラメータは単位なし・原点中央・Y-up・高さ基準正規化で持ち、px変換はレンダ直前1箇所。Draft/Finalの見た目一致の前提(落とし穴F-1)
6. **プレビューと書き出しは同一関数**: 両者は`render_frame(t, Quality)`の引数が違うだけ。別コードパスを作らない(落とし穴B-4)
7. **プラグイン契約にベンダー/OS固有APIを出さない**: 見せるGPUはwgpu/WGSL抽象のみ。CUDA/Metal/DX等を契約に露出するとAEプラグイン圏と同じOS分断を再輸入する(落とし穴F-9。母数根拠はE章、出典は[references.md](references.md))

これらは個別の最適化規則ではなく、「映像制作におけるVST」型の共通実行環境を成立させる下部構造でもある。新しい公開境界は、表現単位・再現性・可搬性・作品の持続性・Host一貫性・作者体験・制作者体験の[7審判](concept.md#設計と実装の審判)を通す。

## 用語の最短定義

- **Document**: プロジェクト状態の単一の純データ構造(serde可能)。コマンド(差分)適用でのみ変更され、コマンドは追記ジャーナルに記録される(常時保存)
- **Quality (Draft/Final)**: 同一レンダ関数に渡す品質パラメータ。Draft=1/2解像度(重い時1/4へ自動降格)・fp16。Finalのみ厳密
- **DataTrack / ParamDriver**: 解析プラグインが生成する時系列データと、それでパラメータを駆動する仕組み(「解析→生成」がこのツールの長期的な強み)
- **TimeMap**: クリップのソース時刻写像。v1は恒等+定数速度のみ実装、スキーマは初日から予約(落とし穴F-4)
- **CompCamera**: 全Compositionに常在し、2D=`z=0`を含む全objectが共有する単一カメラ。Output Frameはその投影開口。Stage Viewのpan/zoomはDocument外で、別cameraではない
- **凍結ゲート**: M1完了後、実際に動いたインターフェースだけを凍結して並列開発を解禁する関門。[宣言](reviews/2026-07-10-freeze-gate-declaration.md)済み(2026-07-10)。改訂は解凍手続き(理由+migrate+ゴールデン)を通す
- **グループ仮出力(ベイク)**: プリコンポの代替。グループ出力を時間範囲でキャッシュし、編集で自動無効化
- **SimulationPlugin / StateTrack**: 逐次状態シミュレーション(布・液体・パーティクル)のプラグイン境界と、そのベイク結果(チェックポイント列の区間キャッシュ)。状態はホストが所有し、`render_frame(t)`はベイク結果を読む純関数のまま(落とし穴F-12、[simulation-model.md](simulation-model.md)。口の予約段階)
- **TemporalFootprint(時間窓)**: エコー/モーションブラー等が前後フレーム/サブフレームサンプルを読むための、`NodeDesc`への静的宣言(予約。任意時刻アクセスAPIは不採用)
- **プラグインパネル(v1)**: `NodeDesc.params`からの自動生成プロパティパネルのみが公開UI境界。`.slint`/wgpuカスタムUIは延期([判定](reviews/2026-07-12-plugin-ui-v1-boundary.md))
