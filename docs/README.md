# docs/ 読み方ガイド

このディレクトリが**現在の設計の唯一の情報源**。コードを読む前にここを読む。
矛盾する記述を見つけたら、それはバグとして扱い修正する(旧仕様の混在は許容しない)。
仕様・モック・コードを触る前に、対象主題を[決定逆引き台帳](decision-index.md)で検索し、既決の正本を読んでから着手する。docsを触る変更は`scripts/check-docs.sh`を通してから終える。

> 整理履歴(2026-07-08): 初期検討資料 `design-memo.md`(2026-07-05) と `discussion-log-2026-07-06.md` は、現決定と矛盾する旧仕様(Tauri+WebView採用、OpenCut Reactコード流用等)を含むため削除した。生きた決定はすべて [concept.md](concept.md) に移植済み。経緯が必要ならgit履歴を参照。

## 30秒サマリ

- **何を作るか**: MV(ミュージックビデオ)制作のための、モーショングラフィック指向のコンポジットツール。AEの重さへの構造的な回答。3〜5分の動画を書き出せたら完成
- **長期の北極星**: 映像表現を、時刻・入力・型付きparameterから決まる再利用可能な単位として実行・保存・配布できる共通環境にする。制作者と開発者を固定身分にせず、利用→調整→構成→inspection→fork→authoring→共有を一つの経路にする。多数のcreator-authorが公開境界の上で独立して表現を増やせることを成長力とする。「映像制作におけるVST」はHostと拡張単位を分ける構造の類比に限り、音楽中心の製品像やDAW化は目標ではない([concept.md](concept.md#長期の北極星-映像表現を実行再利用配布できる単位にする)、[連続体決定](reviews/2026-07-22-creator-developer-continuum-decision.md))
- **技術スタック**: Rust + wgpu(レンダコア、VRAM常駐) / ffmpegサイドカープロセス / Cargo workspaceは確定。UIは[React / WebView chrome + native Rust/wgpu Stage/Timeline](ui-runtime-architecture.md)へ責任分割し、通常windowは[1 top-level wgpu Surface + 2 native viewport + opaque child WebView islands](reviews/2026-07-21-ui-surface-topology-decision.md)へ固定した。native操作はrenderer非依存のheadless kernelへ置く。OS window、surface runtime、Core／Host module／plugin、first／third-partyの信頼境界は[軸分離決定](reviews/2026-07-22-m3-surface-extension-axis-separation.md)に従って別判定する
- **開発方式**: 仕様書駆動の並列AIエージェント開発。[M2基盤再締結](reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。歴史から再採択したD1n external revisionは独立follow-up・未実装で、external change検出/cloud-safeは未達。M3はU0a〜U0e-1、U1a-1/2、U1b-1/2、U2a-0/1、U2b-1、U2c-1/4までmain到達済み。歴史からU2b-2 Place、U4b-0 Add Position Key、U2h-1 primary selection、U3a-1 headless Timelineを決定済み・未実装follow-upとして再採択した。G0-9中もU3a-1等のheadless layout/hit-testは論理上進行可だが、現在のSelected U series順は別に守り、WebView/native製品統合とU3a-2 windowed rendererはplatform合格まで停止する。plugin UI公開契約はG0-3 / GAP-13の別審判まで停止する
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
| [decision-index.md](decision-index.md) | 決定逆引き台帳: 主題キーワード→既決の正本へのポインタ(状態語彙固定・機械検証対象) | **運用正本**(2026-07-19新設。作業前の逆引き入口) |
| [performance-model.md](performance-model.md) | 性能の設計根拠と規律 | 現行 |
| [memory-model.md](memory-model.md) | メモリ階層(VRAM/RAM/ディスク)の役割分担と容量疑念の台帳 | 現行 |
| [simulation-model.md](simulation-model.md) | 時間軸の自由度モデル: 物理シミュレーション(SimulationPlugin+StateTrack)と前後フレーム参照(宣言的時間窓)の設計 | 現行(2026-07-10。口の予約段階、実装v1.x) |
| [pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) | 落とし穴カタログ+ロードマップ+凍結ゲート | 現行 |
| [plugin-authoring.md](plugin-authoring.md) | プラグイン作者向け規約(LLM/人間共通。static first-party公開façadeと未実装distributionを分離) | 現行(2026-07-23歴史回収で状態訂正) |
| [reviews/2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md](reviews/2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md) | M1全28版からFrameDesc／TextureRefの生存意味、歴史的signature、現行安全性gapを分離 | **Unit 3C縮小採用／GAP-17未実装** |
| [reviews/2026-07-23-historical-public-capability-provenance-lineage-recovery.md](reviews/2026-07-23-historical-public-capability-provenance-lineage-recovery.md) | A1公開crate、surface/provenance、creator連続体からbundled first-party source実証と未成立third-party runtimeを分離 | **Unit 3B-runtime-B2-A縮小採用** |
| [reviews/2026-07-23-historical-vism-kit-distribution-lineage-recovery.md](reviews/2026-07-23-historical-vism-kit-distribution-lineage-recovery.md) | Vism／Kit／実装計画29版を処分し、構成、導入集合、再現lock、catalog、hostless配布を分離 | **Unit 9A縮小採用** |
| [reviews/2026-07-23-historical-plugin-ecosystem-lineage-recovery.md](reviews/2026-07-23-historical-plugin-ecosystem-lineage-recovery.md) | 旧plugin ecosystemの未処分11版からcommunity politics、User library、look/primitiveと危険な旧schemaを分離 | **Unit 9B縮小採用** |
| [reviews/2026-07-23-historical-audio-generalization-lineage-recovery.md](reviews/2026-07-23-historical-audio-generalization-lineage-recovery.md) | 音声一般化全6版からcomponent／mix意味を維持し、旧Transport varispeed、製品mixed再生／UI未到達を分離 | **Unit 5B設計維持／GAP-28未実装** |
| [plugin-resources.md](plugin-resources.md) | プラグインのリソースライフサイクル・アセット境界・時間参照(F-10/F-11) | **縮小採用**(PipelineCache/AssetRef/予約型は実装済み、GpuAssetCache/Importer/Feedback実行は未実装・未凍結) |
| [references.md](references.md) | 依存候補・参考リポジトリ(ライセンス区分) | 現行 |
| [ae-pain-points.md](ae-pain-points.md) | AEユーザー不満の体系化+我々の解決タグ(プラグイン窓口仮説の検証) | 現行 |
| [dev-experience.md](dev-experience.md) | 開発体験(DX): プラグイン/シェーダのホットリロードはしご(AE再起動地獄の予防) | 現行(2026-07-13。設計ノート、契約変更なし) |
| [plugin-ui-model.md](plugin-ui-model.md) | プラグインUIモデル: 宣言語彙 vs 自由描画。M3着手前決定で縮小採用 | **採否済み分析**(v1はHost自動生成panel、自由UIは延期) |
| [interaction-simplicity-model.md](interaction-simplicity-model.md) | 操作単純化モデル: Direct/Tool/Advanced正規化、plugin昇格、PP-Gate、M0〜M5割当 | 現行(2026-07-14。凍結済み公開契約は変更しない) |
| [extensible-core-model.md](extensible-core-model.md) | 小さなコアと探索可能な拡張: Core kernel／bundled Host module／first-party／third-partyの分界、壊れない探索、編集pluginの責任寿命、Documentを増やさないアドレス可能な個体、表現domainを列挙しない能力境界、性能上限を焼かない原則 | **設計原則**(2026-07-17。`motolii-core` crateやUI runtimeの分類表ではなく、未凍結APIの実装許可でもない) |
| [vism-package-concept.md](vism-package-concept.md) | Vism (`.vism`): Project・内部plugin kind・Host UIから分離して保存/共有/再利用する映像表現の配布単位。Motoliiは最初のHost、container/loaderは未決 | **コンセプト・名称・拡張子決定／ファイル形式未決**(2026-07-17。v1実装許可ではない) |
| [vism-kit-model.md](vism-kit-model.md) | Core=文法、Vism=小さな表現、Kit=provider選択と型付き接続、Project=作品。BPM/Beatを例に、Vism直接依存を避けるmaterialize構成とfork能力の境界を定義 | **設計原則決定／schema・形式未決**(2026-07-17) |
| [community-distribution-model.md](community-distribution-model.md) | 中央人気／dedupeを持たず、分散地図、User library、Plugin Set、Project Lockで多数作者と複数界隈をつなぐcommunity運用 | **運用・ガバナンス原則決定／protocol・schema・製品UI未決**(2026-07-23) |
| [generative-user-boundary.md](generative-user-boundary.md) | ジェネラティブ表現とユーザー拡張の境界: Shape/SVG、p5.js型入力、Materialize/Live/Feedback/Simulation、Host責務 | **設計決定**(2026-07-15。未凍結runtimeの実装許可ではない) |
| [ui-interaction-language.md](ui-interaction-language.md) | M3のUI操作言語: 既知の外殻、可視の因果、Parameter Panelを表現のホームにするUI力学、共通component契約、Simple/Advanced、漏れ実装の拒否 | **設計決定**(2026-07-16、Parameter Panel力学を2026-07-18追補) |
| [ui-visual-language.md](ui-visual-language.md) | M3の視覚言語: 高密度一覧、意味色、既存UIへの馴染み、contrast、token規約、参照範囲 | 設計基準(具体token値はM3視覚確定(G0-6)待ち) |
| [ui-score-model.md](ui-score-model.md) | 時間面UI構成モデル: 固定Laneを所有者にしない時間投影、選択コンテキスト、Group関係ラベル、回帰審判 | **設計決定**(2026-07-17、2026-07-22用語訂正。公開API・schemaの実装許可ではない) |
| [ui-runtime-architecture.md](ui-runtime-architecture.md) | React/DOM chrome、native Stage/Timeline、headless interaction、React asset直接移管、1 surface/2 viewport/WebView islandsの責任境界 | **責任境界・surface topology決定**(React package移管可。platform受入とrenderer採否はG0-9実機spike待ち) |
| [mocks/](mocks/README.md) | M3高密度メインUI(基準)+timeline/interaction/UI力学の比較モック台帳 | 視覚構成の基準モック |
| [ui-reference-map.md](ui-reference-map.md) | M3 UI参照地図: 規範/prototype/採否台帳/移行互換/証拠/履歴の参照順位と、React移行の実状態・既知の未統一 | **運用正本**(2026-07-19。`codex/m3-mock-components`側から回収) |
| [ui-concept.md](ui-concept.md) | UIコンセプト: 表現をすぐ画にする制作面、最初の結果、五本柱 | **設計方針**(2026-07-22に音楽メタファーを撤回。契約・M3ステータス変更なし) |
| [implementation-ledger.md](implementation-ledger.md) | 現場向け実装進行台帳: M0〜M5のNOW/NEXT/WAIT、依存、Issue昇格順 | **日々の発注入口**(意味・完了条件は各specが正本。M3は段階発注可) |
| [backlog.md](backlog.md) | イシュー候補台帳(現在地サマリ+横断/新規ギャップ/v2バックログ) | 現行 |
| [specs/](specs/README.md) | マイルストーン仕様書(エージェントへの発注書)。確定/ドラフトのステータスはspecs/README.md参照 | M0/M1確定、M2基盤再締結済み(D5は別レーン)、M3はG0-9中でtoolkit非依存とReact asset直接移管R0〜R6だけ段階実装可、M4/M5ドラフト |
| [reviews/](reviews/README.md) | レビュー規律+**全review文書の索引**(この表は現役参照の抜粋。全量はreviews/README.md側が正本で、`scripts/check-docs.sh`が抜けを検証) | 運用正本 |
| [spikes/](spikes/) | スパイク結果報告(S1: Slint統合、S2: デコード、[S3(R8): Vello採否](spikes/s3-vello.md)、[G0-9: UI runtime部分比較](spikes/g0-9-ui-runtime.md)、[wgpu 29 surface host](spikes/g0-9-surface-host.md)、[native Timeline外観first pass](spikes/g0-9-timeline-visual-parity.md)、[native Easing popup core縦切り](spikes/g0-9-native-easing-popup.md)) | 個別文書の状態に従う |
| [reviews/2026-07-12-m2-permanence-prevention.md](reviews/2026-07-12-m2-permanence-prevention.md) | M2恒久焼き込みの**予防手順**(やること5手)。運用正本 | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-prevention.md](reviews/2026-07-14-m3-ui-boundary-prevention.md) | M3でUI都合をDocument・レンダ・公開契約へ逆流させない**予防手順**(規律8本) | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-counter-review.md](reviews/2026-07-14-m3-ui-boundary-counter-review.md) | M3 UI境界規約の反対側レビュー。R1〜R9を採用/縮小/延期で再判定 | 現行(判定反映済み) |
| [reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md](reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md) | React/WebView、Host/community同一kit、native surface統合のG0-9証拠 | **責任境界・surface topology決定 / platform受入比較中** |
| [reviews/2026-07-22-m3-react-product-asset-promotion-contract.md](reviews/2026-07-22-m3-react-product-asset-promotion-contract.md) | Reactモックを製品packageへ直接所有移管し、維持／交換境界、diagnostic route、発注・検収STOPを固定 | **決定 / 発注停止線**(明示再開まで発注しない) |
| [reviews/2026-07-22-m3-native-easing-popup-acceptance.md](reviews/2026-07-22-m3-native-easing-popup-acceptance.md) | React trigger、native wgpu popup全内容、Host popup lifecycle/User settingsの境界と実機審判 | **決定**(G0-9 isolated spikeを実行、製品U4b接続は停止) |
| [reviews/2026-07-22-m3-surface-extension-axis-separation.md](reviews/2026-07-22-m3-surface-extension-axis-separation.md) | OS window、native/React surface、Core/Host module/plugin、first/third-party信頼境界を独立判定 | **決定**(G0-9製品surfaceとG0-3 plugin UIを分離) |
| [reviews/2026-07-22-creator-developer-continuum-decision.md](reviews/2026-07-22-creator-developer-continuum-decision.md) | 利用→調整→構成→fork→authoring→共有を一つの作者経路にし、React・Vism・first-party参照実装を多数作者の成長戦略へ統合 | **決定**(参加資格は薄くし、trust／sandbox／Host責任は維持) |
| [reviews/2026-07-21-ui-surface-topology-decision.md](reviews/2026-07-21-ui-surface-topology-decision.md) | 1 top-level wgpu Surface、Stage/Timeline viewport、opaque child WebView islands | **topology決定 / platform受入継続** |
| [reviews/2026-07-16-m3-preflight-decisions.md](reviews/2026-07-16-m3-preflight-decisions.md) | M3着手前決定: input/状態寿命、plugin UI、性能測定、操作文法を固定し、見た目とresource実値を証拠待ちへ分離 | **設計決定**(G0-2/4/7完了。G0-3は2026-07-21再評価中) |
| [reviews/2026-07-20-m3-keymap-codec-contract.md](reviews/2026-07-20-m3-keymap-codec-contract.md) | U0d-2 keymap JSON wire・原本保全・migration境界 | **決定**(2026-07-20) |
| [reviews/2026-07-16-m3-ui-concept-to-tickets.md](reviews/2026-07-16-m3-ui-concept-to-tickets.md) | UIコンセプトを1 Issue=1 commitの実装粒へ分解。状態、入力、視覚、preview、共通操作、最初のEffect panelの依存と拒否条件 | **条件付き発注の正本**(U0b〜U4aの枝番。各行依存に従い発注可) |
| [reviews/2026-07-16-ui-update-forensics.md](reviews/2026-07-16-ui-update-forensics.md) | Figma/Ableton/AE/Blender/Godot/Home AssistantとLinux GUIの公式更新・fork履歴から、UI失敗、不安定platformの隔離、user拡張をMotoliiのcomponent審判へ変換 | **調査と採用審判**(AF-1〜17) |
| [reviews/2026-07-17-non-video-workspace-asset-ui-prior-art.md](reviews/2026-07-17-non-video-workspace-asset-ui-prior-art.md) | 写真管理、3D／ゲーム制作、CAD、IDEから、外部素材探索、task別Workspace、自由配置、視線handoffを再調査。SourcesのTray／Drawer／Dock仮説とFocus Contract、比較モック審判へ翻訳 | **先例調査・翻訳仮説**(M3製品実装・公開APIの許可ではない) |
| [reviews/2026-07-17-aviutl2-comment-voices.md](reviews/2026-07-17-aviutl2-comment-voices.md) | AviUtl2動画の公開コメント34件+表示返信から、軽さ/重さ、統合/分業、拡張/管理、移行/旧資産等の統一できない一次声を保存 | **一次声の観察台帳**(反対側レビュー前。設計根拠ではない) |
| [reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md](reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md) | VSM-A0: 現行pluginの登録・保存・評価・migration境界をコード事実で分類 | **調査完了** |
| [reviews/2026-07-17-vism-a7-bpm-datatrack-spike.md](reviews/2026-07-17-vism-a7-bpm-datatrack-spike.md) | VSM-A7: 現行BPM→DataTrack→DocParamの最小意味fixture | **spike完了** |
| [reviews/2026-07-17-vism-a0d-contract-migration-ownership-decision.md](reviews/2026-07-17-vism-a0d-contract-migration-ownership-decision.md) | VSM-A0D: Document、plugin作者、Host catalog、executorの所有分離 | **設計決定** |
| [reviews/2026-07-17-vism-a0s-contract-catalog-spec.md](reviews/2026-07-17-vism-a0s-contract-catalog-spec.md) | VSM-A0S: Contract Catalog、prepared resolution、runtime公開境界 | **A0I-1〜3 + D1m保存/open所有を実装済み** |
| [reviews/2026-07-17-vism-a1-public-crate-boundary-spec.md](reviews/2026-07-17-vism-a1-public-crate-boundary-spec.md) | VSM-A1S: Opacity外部crate化のfaçade、依存allowlist、first-party組み立て、必須capability、移動前pixel gate | **A1-3完了** |
| [reviews/2026-07-17-vism-a2-legacy-project-migration-decision.md](reviews/2026-07-17-vism-a2-legacy-project-migration-decision.md) | VSM-A2S: Sine外部crate化時の旧CLI ProjectV1 migration処分と公開façadeレビュー | **設計決定／A2実装可** |
| [reviews/2026-07-18-vism-a3-external-expression-survey.md](reviews/2026-07-18-vism-a3-external-expression-survey.md) | VSM-A3R: AE Expression／Script／Effect、aescripts、Blender Driver／Geometry Nodes／Simulation／Add-onを責任分類し、Parameter Panel中心のA3候補へ翻訳 | **調査完了**（採用決定は[A3D](reviews/2026-07-18-vism-a3d-radial-repeater-decision.md)） |
| [reviews/2026-07-18-vism-a3d-radial-repeater-decision.md](reviews/2026-07-18-vism-a3d-radial-repeater-decision.md) | VSM-A3D: 決定論的2D Radial Repeater LayerSource（`core.layer_source.radial_repeater` v1）のidentity・正準意味・parameter閉集合・UI投影要求・非目標 | **設計決定・VSM-A3実装完了** |
| [reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md](reviews/2026-07-18-vism-a3s-layersource-lowering-spec.md) | VSM-A3S: 一般LayerSource lowering（prepared→`RenderStep::Plugin`）、clear一般化、拒否分類、rect分離、画素契約、U4a handoff、A3分割発注表。[F1](reviews/2026-07-17-vism-implementation-plan.md)でHost cache GAPを訂正し、`VSM-A3-0`〜`VSM-A3-4`まで実装済み | **仕様・VSM-A3完了** |
| [reviews/2026-07-14-unified-stage-camera-design.md](reviews/2026-07-14-unified-stage-camera-design.md) | 2D/3Dを分けない単一カメラ、Stage、Output Frame、枠外表示の意味と実装順 | **決定**(2026-07-14) |
| [reviews/2026-07-14-recent-concept-propagation-audit.md](reviews/2026-07-14-recent-concept-propagation-audit.md) | 直近の根幹決定を意味・Document・評価・UI・依存・コードの6面で逆引きした未反映台帳 | 横断監査(2026-07-14) |
| [reviews/2026-07-14-motion-foundation-known-tech-disposition.md](reviews/2026-07-14-motion-foundation-known-tech-disposition.md) | Relative Move、Bounds/ROI、Effect Scope、Instance/Elementを既知技術で再判定した最小契約 | **決定**(2026-07-14) |
| [reviews/2026-07-15-relative-scope-duplicator-decision.md](reviews/2026-07-15-relative-scope-duplicator-decision.md) | modifier+drag、透過Stage、Explicit Definition/Use、Cavalry型Duplicator、stable seedの具体化 | **決定**(2026-07-15) |
| [reviews/2026-07-15-prior-art-complaint-boundary-audit.md](reviews/2026-07-15-prior-art-complaint-boundary-audit.md) | 先例が収束した固定契約と、Null/Group/Crop等の日曜大工帯を分離 | **調査第一陣**(2026-07-15) |
| [reviews/2026-07-15-implementation-readiness-ledger.md](reviews/2026-07-15-implementation-readiness-ledger.md) | M2〜M5のREADY/SPIKE/WAIT/BLOCKED分類とIssue昇格順 | **運用正本**(2026-07-15) |
| [reviews/2026-07-12-rework-prior-art.md](reviews/2026-07-12-rework-prior-art.md) | 出戻りの先人調査(予防側/失敗後の対比)。設計根拠ではない | 仮説メモ |
| [reviews/2026-07-12-pathop-ae-cavalry-comparison.md](reviews/2026-07-12-pathop-ae-cavalry-comparison.md) | PathOp語彙のAE/Lottie×Cavalry比較。意味【決定】前の材料(採択後は参考) | 調査メモ(未採用) |
| [reviews/2026-07-13-undecided-critical-path-confirm.md](reviews/2026-07-13-undecided-critical-path-confirm.md) | 友人レビュー確認: 未決の追跡先・クリティカルパス補正・B⑤コード確認 | 確認メモ |
| [reviews/2026-07-13-decision-pack-adoption.md](reviews/2026-07-13-decision-pack-adoption.md) | #103/#100/残小項目の**【決定】採択**(AE/Lottie・OTIO・DAW・Qt) | 現行(決定) |
| [reviews/2026-07-14-m2-core-closure.md](reviews/2026-07-14-m2-core-closure.md) | M2コア締結宣言(**撤回**・単独再宣言を廃止し再締結ゲートへ移行) | 撤回(2026-07-14) |
| [reviews/2026-07-15-m2-foundation-reclosure-gate.md](reviews/2026-07-15-m2-foundation-reclosure-gate.md) | M2恒久面の再締結条件とM3製品実装の停止線 | **M2基盤再締結解除・main発効済み**(PR #218。M3はU0a入場完了後に段階発注可) |
| [reviews/2026-07-15-m2-foundation-reclosure-counter-review.md](reviews/2026-07-15-m2-foundation-reclosure-counter-review.md) | M2基盤再締結ゲートの反対側レビューと採否 | **P0/P1=0・発効merge可** |
| [reviews/2026-07-15-shared-effect-lifecycle-decision.md](reviews/2026-07-15-shared-effect-lifecycle-decision.md) | Shared Effectの削除/Unlink/Copy Local/orphan lifecycle（GAP-14） | **決定**(2026-07-15 / #166) |
| [reviews/2026-07-15-d1l-copylocal-remint-counter-review.md](reviews/2026-07-15-d1l-copylocal-remint-counter-review.md) | D1l Copy Local内部ID契約の反対側レビュー、journal/counter指摘と採否 | **P0/P1=0・merge可**(PR #196) |
| [reviews/2026-07-15-d1l-journal-revert-boundary-decision.md](reviews/2026-07-15-d1l-journal-revert-boundary-decision.md) | D1lのJournalEdit v1→v2互換、Undo等価、Writer採番単一路の追補 | **決定・merge済み**(PR #197) |
| [reviews/2026-07-15-d1l-journal-revert-boundary-counter-review.md](reviews/2026-07-15-d1l-journal-revert-boundary-counter-review.md) | PR #197の反対側レビュー、採番/閉集合/orphan指摘と採否 | **P0/P1=0・merge可** |
| [reviews/2026-07-16-d1l-current-document-constructor-decision.md](reviews/2026-07-16-d1l-current-document-constructor-decision.md) | 新規Documentをv4で作る製品constructorと、legacy `new_v1`/D1e/D1l Commandの版境界 | **決定**(lint機構は下記追補) |
| [reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md](reviews/2026-07-16-d1l-new-v1-lint-conflict-decision.md) | `new_v1` deprecated属性とprotected semantic/clippyの三律背反を、`doc(hidden)`+AST gateへ一本化 | **決定追補** |
| [reviews/2026-07-17-d1i4-semantic-oracle-boundary-decision.md](reviews/2026-07-17-d1i4-semantic-oracle-boundary-decision.md) | D1i-4/S16の保護単位をtest harness全体から意味の期待値oracleへ訂正し、API配線と作品意味を分離 | **決定追補／BlendModeから段階移行** |
| [reviews/2026-07-16-d1l-current-document-constructor-counter-review.md](reviews/2026-07-16-d1l-current-document-constructor-counter-review.md) | 新規Document v4生成契約の版/構造検証/allowlist指摘と採否 | **P0/P1=0・merge可** |
| [reviews/2026-07-15-p5-generative-pattern-disposition.md](reviews/2026-07-15-p5-generative-pattern-disposition.md) | p5.js系ジェネ表現をone-shot/純関数/Feedback/Simulation/記録入力へ分類 | **調査・配置案**(2026-07-15) |
| [reviews/2026-07-16-m3-ui-gap-survey.md](reviews/2026-07-16-m3-ui-gap-survey.md) | M3前UIギャップ調査: U1〜U8に席が無いUI領域(書き出し/保存/エラー表示等)とコア側前提の欠落(状態購読/ParamDefメタデータ/Transport等) | **調査メモ**(2026-07-16。各項目の採否は個別M3チケット／依存充足後の裁定で決める) |
| [reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md](reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md) | すぐに受け入れられたUIの先例集: 第一部=プロダクト単位の受容(界隈の期待リスト)、第二部=業界収斂した操作語彙+UX原理の一次資料(M3転移の本線)、第三部=後発の勝ち筋「どの操作も直感的」(Ableton→AEカウンター)。設計根拠ではない | 仮説メモ(2026-07-16) |
| [reviews/2026-07-18-m3-egui-selection.md](reviews/2026-07-18-m3-egui-selection.md) | M3 UI基盤をSlintからeguiへ変更した時点の既存wgpu device/native texture、lifecycle、日本語IME、可変panel証拠 | **歴史的採否決定**(完了済みegui基準を保持。現行採否はG0-9再評価中) |
| [reviews/2026-07-20-rerun-learning-transfer-plan.md](reviews/2026-07-20-rerun-learning-transfer-plan.md) | RerunのUI、時間面、GPU viewport、selection、実行系、試験系をRR-0〜9へ分解し、M1〜M5の関与、転移順、停止線、発注の強制動線を規定 | **方向決定／学習・発注運用正本**(source監査は可。依存・vendoring・移植はM3入場後。§9の順序と6ラベルは無視禁止) |
| [reviews/2026-07-21-m3-u1a-1-static-viewport-contract.md](reviews/2026-07-21-m3-u1a-1-static-viewport-contract.md) | U1a-1の単一Document→display閉路、register-once、event-loop前setup、製品window lifecycle、中央Stage境界 | **実装完了**(旧night差分は直接統合せず、本契約から再実装。実monitor DPI移動はU1e) |
| [reviews/2026-07-21-m3-u1a-2-layout-projection-contract.md](reviews/2026-07-21-m3-u1a-2-layout-projection-contract.md) | U1a-2の固定5 role layout intent、runtime proposal権限、局所input adapter、Stage/Status境界 | **実装完了**(Grok反対側レビュー ACCEPT、P0/P1=0。保存codecはU1a-3、自由dock実機はU1e) |
| [reviews/2026-07-20-rerun-source-asset-inventory.md](reviews/2026-07-20-rerun-source-asset-inventory.md) | 固定commitの139 package、非コード資産、拡張example、Importer、Viewer MCP、試験基盤等を全体棚卸し | **観察**(package-levelは全量、file/API-levelは重点候補。候補分類は採用裁定ではない) |
| [reviews/2026-07-20-rerun-re-ui-module-inventory.md](reviews/2026-07-20-rerun-re-ui-module-inventory.md) | `re_ui`をfile-levelへ分解し、React安定ID、M3 task、CJK/IME、転移候補、次のMotolii oracleへ対応付け | **観察／比較中**(一括DEPENDは棄却候補。個別分類は反対側レビュー前で、実装・発注許可ではない) |
| [reviews/2026-07-20-perceptual-expression-translation-decision.md](reviews/2026-07-20-perceptual-expression-translation-decision.md) | 工業系の厳密な境界と、軽量な知覚表現、Draft / Final、Vism、Rerunの役割をMotolii Hostの翻訳命題へ統合 | **決定**(公開API・Document schema・Rerun SDK依存の追加許可ではない) |
| [reviews/2026-07-20-local-worktree-publication-audit.md](reviews/2026-07-20-local-worktree-publication-audit.md) | GitHubへ公開した正典候補・M3分岐・WIP保全と、吸収済みまたは旧契約として公開しなかったdirty worktreeの比較 | **観察／外部再開地図**(branch存在は採択根拠ではない) |
| [reviews/2026-07-17-extensible-core-prior-art-translation.md](reviews/2026-07-17-extensible-core-prior-art-translation.md) | extensible-core §7(個体性)・§9(遊び)未決部の先例翻訳: 四段の個体性、選択≠Object化、宣言的介入(Pin/Impulse/Exclude)、集合所有の状態、上限非焼き込み、Preview縮退、遊びの観察を一次資料で確認しMotolii語彙へ翻訳。「既知で埋まる部分」と「埋まらない残り(介入正本の逆転・四段の利用者文法・遊びの判定)」を分離 | **調査第二陣**(2026-07-17。反対側レビュー待ち、設計根拠ではない) |
| [reviews/2026-07-17-vism-implementation-plan.md](reviews/2026-07-17-vism-implementation-plan.md) | Vismを静的pluginの公開境界実証→typed provider/Kit→package意味→container/trust spike→loader/install→UI/headless互換Hostへ分けた実装順。自動完了条件、依存、LLM発注規律、STOP線つき | **実装ロードマップ案**(2026-07-17。package実装は未許可) |
| [reviews/2026-07-17-vism-ready-counter-review-disposition.md](reviews/2026-07-17-vism-ready-counter-review-disposition.md) | 既存pluginのVism-ready化提案を実コードで反対側審判。A0復帰、consumer API不在、Sine migration／doc既知表、Macro非atomicを採用し、A0→A7→A0D→A0S→A0I→A1/A2→B0/B1/B2へ修正 | **採否決定**(2026-07-17。実装許可ではない) |

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
- **プラグインパネル**: `NodeDesc.params`自動生成panelは全保存paramを操作できる必須fallbackとして決定済みだが、製品U4aは未実装。plugin所有egui/native/Web/wgpu UIはG0-3 / GAP-13の公開・sandbox・互換・配布審判まで公開しない。標準製品surfaceのG0-9合格だけでは解除しない
