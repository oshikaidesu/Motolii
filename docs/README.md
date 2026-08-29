# docs/ 読み方ガイド

このディレクトリが**設計の意味(コンセプト・裁定・審判)の唯一の情報源**。世界には属さない共有地であり、実装の現在地は各世界のコード(Cargo.toml・その場のコメント)が正本。コードを読む前にここを読む。
矛盾する記述を見つけたら、それはバグとして扱い修正する(旧仕様の混在は許容しない)。
仕様・モック・コードを触る前に、対象主題を[決定逆引き台帳](decision-index.md)で検索し、既決の正本を読んでから着手する。docsを触る変更は`scripts/check-docs.sh`を通してから終える。

> 整理履歴(2026-07-08): 初期検討資料 `design-memo.md`(2026-07-05) と `discussion-log-2026-07-06.md` は、現決定と矛盾する旧仕様(Tauri+WebView採用、OpenCut Reactコード流用等)を含むため削除した。生きた決定はすべて [concept.md](concept.md) に移植済み。経緯が必要ならgit履歴を参照。

## 30秒サマリ

- **stage4(2026-08-29裁定・最優先で読む)**: 世界は`motolii/`(四代目)へ切り替わった。入口は[motolii/AGENTS.md](../motolii/AGENTS.md)——一行の憲法「rerunを軸に持ったAE(Lottie)インタフェースのソフト」+三本柱+背骨3本の1ページ。front=Dioxus Native(Blitz)、技術層=Rerunフォークの部品(再発明は罪)、意味=Lottieが審判。これにより下記「リセット」「技術スタック」「shell現在地」「M3現在地」および「読む順序」のM3〜M5系は**stage4以前の記述**になった(歴史記録として残す)。stage4で生きて引き続けるのは意味の正本([concept.md](concept.md)・[CANON.md](CANON.md)・[決定逆引き台帳](decision-index.md)・[references.md](references.md))。**UIを触る前は`motolii/AGENTS.md`「UIを触る前の3つの問い」から、審判=[motolii-deltas.md](motolii-deltas.md)・在庫=[刻まれた文法のギャップ地図](ui-inherited-grammar-gap.md)・合否=[「普通に使える」品質バー](ui-quality-bar.md)の3本を引く。**正本=[stage4裁定](decision-index.md)
- **リセット(2026-08-20裁定・stage4以前)**: ドリフトの累積を1度リセットし、軸を1本にする。rerun の `crates/store/*` と `re_renderer` を pin fork から引いて **Document と合成器の実体**にし(AE の意味は `re_types_core` の custom component として Motolii 側に建てる)、front は **iced のみ**で store 投影に徹し、拡張口を **trait 1本**へ収束させる。器は**新 workspace**。これにより下記「技術スタック」「shell現在地」「M3現在地」の各行は**リセット前の記述**になった(歴史記録として残す)。正本=[リセット裁定](reviews/2026-08-20-reset-to-one-axis.md)
- **何を作るか**: MV(ミュージックビデオ)制作のための、モーショングラフィック指向のコンポジットツール。AEの重さへの構造的な回答。3〜5分の動画を書き出せたら完成
- **長期の北極星**: 映像表現を、時刻・入力・型付きparameterから決まる再利用可能な単位として実行・保存・配布できる共通環境にする。制作者と開発者を固定身分にせず、利用→調整→構成→inspection→fork→authoring→共有を一つの経路にする。多数のcreator-authorが公開境界の上で独立して表現を増やせることを成長力とする。「映像制作におけるVST」はHostと拡張単位を分ける構造の類比に限り、音楽中心の製品像やDAW化は目標ではない([concept.md](concept.md#長期の北極星-映像表現を実行再利用配布できる単位にする)、[連続体決定](reviews/2026-07-22-creator-developer-continuum-decision.md))
- **技術スタック**: iced（現行host／layout／input）/ Rerun Spatial Viewer（Stage島／spatial runtime）/ Rust Host（Document、D2、media、resource policy）/ FFmpeg（media／export）。MotoliiはRerunのcreator-facing wrapperとして、永続意味と薄いidentity／time／asset翻訳だけを持ち、scene／view／query／camera／picking／rendererを再構築しない。全surfaceは同じrevision付きsnapshotを読む
- **開発原則 — 既知実装優先、新設前に探索・採択**: Motoliiは作品意味、製品policy、admission、acceptance oracle、絶対規律を所有するが、一般機構を第一原理から発明しない。M3〜M5は利用者成果→機構class→既知実装調査→`REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL`裁定→採択地図→薄い接続→同一oracleで旧route退役、の順で進める。既存task列や投入工数を実装・維持理由にせず、M4/M5も採択地図が閉じる前に独自cache／scheduler／3D engine／scene framework等を作らない。正本は[既知実装採択・置換開発モデル](known-implementation-adoption-model.md)
- **開発原則 — 発注はreturn後の再選定まで**: 利用者成果の背骨から一契約境界をclosed orderへcompileする。成果は実装だけでなく、検索場所、候補、採否、不適合理由、exact gap、再入場条件を持つ調査返却も正規出口とする。主担当は返却後に古い`next`へ戻らず、current codeから次edgeを選び直す。正本は[発注コンパイルと調査返却loop](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)
- **M3 baseline-required自走**: 一般的なdesktop動画編集ソフトで欠落すると購入候補から外れるuser-visible outcomeは、独立した非OpenAI調査と別family反例監査を通過後に`BASELINE_REQUIRED`として必要性を自動承認する。総監督Codexはfeature listを独力抽出せず、evidence packet、current authorityへの写像、scope／oracle、最終採否だけを所有する。現時点の採用itemは0。Claude directのempty-workspace 1-query Web capability probeは通過したが、baseline本調査、challenge、mappingは未実施である。正本は[M3 baseline-required自走checkpoint](reviews/2026-08-07-m3-baseline-required-autonomy-checkpoint.md)
- **shell現在地(2026-08-19)**: `motolii-shell-iced`が現行製品hostと新規機能target。`motolii-blitz-shell`とegui製品UIはlegacy/referenceで、Timelineの参照実装とRerun Stage島内のeguiは用途を分けて残す。既定bin名やlauncherの機械的残余はhost authorityを戻さない。各面の正本、能力残余、起動・撮影器具は[CANON.md](CANON.md)、裁定は[iced移行決定](reviews/2026-08-18-iced-host-migration-decision.md)
- **M3現在地(2026-08-19)**: iced shellへBrowser、Inspector、Timeline、Stage島と`UiIntent` gatewayが統合済み。これは製品routeの現在地であり、全能力・視覚・実機の完成宣言ではない。RN製品面は撤去済みで、`ui/motolii-rn/src`は移植参照としてのみ残る。詳細は[実装台帳](implementation-ledger.md)
- **開発方法**: 制作意図→Motoliiの型付き意味→Rerun／Skia／FFmpeg→Stage結果の薄い一本をmainへ積む。PRは一成果を運ぶlanding envelopeで、Issue候補一覧は第二ledgerではない。詳細は[叩き台PR統合決定](reviews/2026-08-10-creator-translation-working-draft-pr-integration-decision.md)
- **M3の実装分解**: [M3仕様](specs/M3-ui-integration.md#10-実装wave)のR0〜R4を成果wave、[M3 RN runtime実行地図](m3-rn-runtime-execution-map.md)を施工node・依存・oracleのdispatch正本とする。[旧既知技術採択地図](m3-parallel-implementation-map.md)と[旧実行可能地図](m3-executable-dispatch-map.md)はsemantic oracle、既存owner、未閉鎖gapの検索資料であり、旧ID／rendererを新runtimeへ自動継承しない
- **設計目標の代表値**: 1080p動画レイヤー40本同時で破綻しない / プロセス強制終了しても編集を失わない(コマンドジャーナル) / フレーム並列(マルチコア)を構造で保証

## 読む順序(初見向け)

1. [concept.md](concept.md) — 何であって何でないか。**全決定事項の台帳**(スコープ、プラグイン境界、座標系、並行性、音声方針)
2. [performance-model.md](performance-model.md) — 「なぜAEより軽くできるか」の物理(メモリ帯域モデル)、品質モード(Draft/Final)、並列性、40レイヤー目標の試算。**容量・VRAM上限への疑念は[memory-model.md](memory-model.md)(疑念台帳)へ**
3. [pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) — **最重要・最大**。落とし穴カタログ(A〜H、先行プロジェクト死因分析+LLM開発規律込み)とロードマップ(M0〜M5)、凍結ゲート
4. M3〜M5の全体並列campaignを始める時: [統一並列開始baseline](reviews/2026-08-09-unified-parallel-start-baseline-decision.md)（開始履歴と候補状態）→ [cold-replaceable監督と停止封じ込め](reviews/2026-08-09-cold-replaceable-supervision-failure-containment-decision.md)（一つのtop seat、停止・復旧、failure injection）を先に読む
5. M3〜M5の計画・発注・実装に着手する時: [known-implementation-adoption-model.md](known-implementation-adoption-model.md)(利用者成果→既知実装調査→採択地図→接続→退役)→ [発注コンパイルと調査返却loop](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)(closed order→実装／調査return→次edge再選定)→ 外部LLMを使う時だけ[発注観測・実行・可変配分runbook](llm-dispatch-observation-and-allocation-runbook.md)(CLI引数、途中log、主要model、利用枠profile)
6. M3実装に着手する時: [M3完成像](m3-completion-image.md)（何のために繋ぐのか。発注前にownerが同じ絵を持つため）→ [M3仕様](specs/M3-ui-integration.md)（R0〜R4、意味、gate）→ [UI runtime責任境界](ui-runtime-architecture.md) → [現行UIとRerunの対応表](m3-rerun-ui-correspondence.md)（surfaceごとの採択／残余／接続状態）→ [M3 RN runtime実行地図](m3-rn-runtime-execution-map.md)（施工node、依存、oracle）→ [実装台帳](implementation-ledger.md)の一意な`DO`。旧[m3-parallel-implementation-map.md](m3-parallel-implementation-map.md)／[m3-executable-dispatch-map.md](m3-executable-dispatch-map.md)は既存ownerとoracleの検索にだけ使う
7. M4に着手する時: [M4既知実装調査](reviews/2026-08-02-m4-known-implementation-survey.md)→ [M4既知実装採択・並列実装地図](m4-known-implementation-adoption-map.md)→ [implementation-ledger.md](implementation-ledger.md)(一意な`DO`)→ [specs/M4-cache-and-analysis.md](specs/M4-cache-and-analysis.md)(意味と実装ガード)。M5に着手する時: [Rerun Spatial Viewer採択再締結](reviews/2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md)→[M5採択地図](m5-known-implementation-adoption-map.md)→ledger→M5仕様の順で、current Stageへ接続する一契約を選ぶ
8. UIを表示・起動・比較する時: [CANON.md](CANON.md)(面ごとの視覚正本・Timeline実装・製品shell・撮影器具の現在地を1枚で確認)→ [ui-artifact-terminology.md](ui-artifact-terminology.md)(要求名→成果物種別→実装状態。未実装のPreviewをMock/baseline/spikeで代替しない。ただし`motolii_ui_shell`関連の記述は撤去済みの旧shellを指すので同文書冒頭の訂正注記を先に読む)→ [ui-reference-map.md](ui-reference-map.md)(対象surfaceの正本と実体)
9. プラグインを書く/量産させる時: [plugin-authoring.md](plugin-authoring.md)(LLM/人間共通の契約・禁止事項・型紙)
10. 依存・参考リポジトリを調べる時: [references.md](references.md)(ライセンス区分つき。GPL系はコードを読むことすら禁止)

## ファイルマップ

| ファイル | 役割 | 状態 |
|---|---|---|
| [concept.md](concept.md) | コンセプト定義・決定事項の台帳 | 現行(決定はここに追記される) |
| [CANON.md](CANON.md) | 「今どれが正本か」の1枚索引(視覚/Timeline実装/token/製品shell/撮影器具、各行に最終更新日) | 現行(2026-08-19新設。正本が動いたら追記する索引で、新しい設計判断は書かない) |
| [reviews/2026-08-09-unified-parallel-start-baseline-decision.md](reviews/2026-08-09-unified-parallel-start-baseline-decision.md) | 製品main、現行authority、直列核、UI配置逃げ道、仮コード調査、未commit設計資料を一つの開始履歴へ収束し、候補状態を固定する | **決定／candidate branch収束済み・main統合とcampaign未実施** |
| [reviews/2026-08-09-cold-replaceable-supervision-failure-containment-decision.md](reviews/2026-08-09-cold-replaceable-supervision-failure-containment-decision.md) | 一つのtop seat、cold replacement、下位seatの権限上限、停止・復旧・採用gateとfailure injectionを固定する | **決定／failure injectionとfresh closure review待ち** |
| [known-implementation-adoption-model.md](known-implementation-adoption-model.md) | M3〜M5共通の既知実装調査、採択地図、薄い接続、独自負債置換・退役の開発順序 | **確定運用／非凍結の横断開発原則**(2026-08-02。M3適用済み、M4/M5採択地図確定。反証と実測で改訂可) |
| [reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md) | 利用者成果の背骨からclosed orderを作り、実装または調査返却を検収してcurrent codeから次edgeを再選定する横断発注loop | **決定／全計画・発注・STOP/RETURN・次粒再選定の正本** |
| [reviews/2026-08-07-m3-baseline-required-autonomy-checkpoint.md](reviews/2026-08-07-m3-baseline-required-autonomy-checkpoint.md) | M3のbaseline必要性自動承認、非OpenAI抽出、別family challenge、Codexの整理・写像・採否責任、Web research再入場条件を分離する | **決定／baseline採用0、Web capability probe通過、R0再検収とfresh本調査が次lane** |
| [llm-dispatch-observation-and-allocation-runbook.md](llm-dispatch-observation-and-allocation-runbook.md) | closed orderをprovider CLI、途中stream、生log、主要model候補、利用枠別allocation profileへ接続する | **運用正本／CLI snapshotは起動前更新** |
| [reviews/2026-08-07-terra-grok-composer-role-reallocation-decision.md](reviews/2026-08-07-terra-grok-composer-role-reallocation-decision.md) | Terraをbounded order compile、Sparkを極小施工、Grok 4.6 medium／high／xhighを通常・複雑・長時間agenticなbounded施工へ置き、Composer 2.5を明示理由のある代替施工へ置く | **現行model役割決定／固定pipelineではない** |
| [m3-completion-image.md](m3-completion-image.md) | M3が完成したとき利用者が触れるソフトの1枚。完成条件、waveごとの操作列、既決の触り心地、M5/3Dの委託境界、完成条件に含まないものを既存正本から導出して集約する | **参照(2026-08-09)。新規決定を作らない。現行正本と食い違えば正本が勝つ。acceptance条件・oracleにしない** |
| [m3-rerun-ui-correspondence.md](m3-rerun-ui-correspondence.md) | 現行`ui/motolii-rn/`の各surfaceをRerun機構、Motolii固有責任、実接続状態へ対応付ける | **現行コード対応表／新規決定を作らない**(2026-08-11) |
| [m3-rn-runtime-execution-map.md](m3-rn-runtime-execution-map.md) | 現行RN + rust-skia + wgpu M3のR0〜R4を実在target、単一owner、非LLM oracle、依存、cutoverへ分解し、ordinary editor viabilityを既存nodeへ重ねる施工地図 | **現行dispatch地図／R0候補READY-RECHECK、R1〜R4事前compile済み、baseline採用0**(2026-08-07。旧M3地図は検索・oracle履歴) |
| [利用者成果の背骨と調査不足粒の自律再接続決定](reviews/2026-08-04-outcome-spine-autonomous-gap-research-decision.md) | 大きな利用者成果を進捗軸に保ち、一契約境界の粒を既知実装へ自律再接続する規律 | **決定**(2026-08-04。M3 HUMAN最終集約を含む) |
| [m4-known-implementation-adoption-map.md](m4-known-implementation-adoption-map.md) | M4の13親、既知実装route、詳細子、採択probe、並列wave、旧負債の退役境界 | **初版採択地図／runtime未発注**(2026-08-02) |
| [m5-known-implementation-adoption-map.md](m5-known-implementation-adoption-map.md) | M5のRerun Spatial Viewer subsystem採択、Motolii固有residual、private検証、接続再選定 | **Rerun採択再締結／private検証DONE／製品runtime未接続**(2026-08-10) |
| [vism-known-implementation-adoption-map.md](vism-known-implementation-adoption-map.md) | Vism全入口の既知解、採用方式、private境界、probe、cutover、retirement | **採択地図決定／依存・runtime実装は未許可**(2026-08-02) |
| [M4既知実装調査](reviews/2026-08-02-m4-known-implementation-survey.md) | M4のcache／resource／disk artifact／区間／background job／proxy／SVG候補を具体APIまで比較 | **比較中**(採択probe前。候補比較だけFable助言を再照合済み) |
| [M4 disk artifact store再検索](reviews/2026-08-02-m4-disk-artifact-store-resurvey.md) | dormantなcacache後の候補を再検索し、global CASを過剰仕様としてverified recipe file storeへ縮小 | **縮小採用**(2026-08-02。tempfile採択probe前) |
| [storage-to-GPU direct I/O観察](reviews/2026-08-06-storage-to-gpu-direct-io-design-observation.md) | cuFile／xio-sig一次資料から、artifact identityをCore、storage→GPU到達経路をHost private policyへ分離する既決境界の補強材料と再入場条件を固定 | **観察／BUILD FORBIDDEN**(2026-08-06。新task・API・設定UIなし) |
| [M5既知実装調査](reviews/2026-08-02-m5-known-implementation-survey.md) | M5の3D math／import／depth／bounds／text／identity候補を既存ownerへ割り当てる比較 | **比較中**(反対側レビューと採択probe前) |
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
| [reviews/2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md](reviews/2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md) | wgpu課題／先例全4版から同期readbackとcold pipeline捕捉面を再照合し、計測前の方式固定を拒否 | **Unit 5C延期維持／GAP-29・30未実装** |
| [reviews/2026-07-23-historical-d5-transport-lineage-recovery.md](reviews/2026-07-23-historical-d5-transport-lineage-recovery.md) | D5 Transport全4版からaudio clock主、video drop、DRS縮退、device wait／D4-FU境界を現行コードへ再照合 | **Unit 5D決定維持／製品統合pending** |
| [reviews/2026-07-23-historical-color-export-lineage-recovery.md](reviews/2026-07-23-historical-color-export-lineage-recovery.md) | 色変換／GPU export先例1版を現行コードへ再照合し、重複GAP-14をGAP-31へ正規化、TRC／readback責任を分離 | **Unit 5E採択維持／GAP-31未実装** |
| [reviews/2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md](reviews/2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md) | メディア可搬性／GPUベンダ差の未実施再調査計画1版を、GAP-3／7・K4とINF-3の狭い再入場gateへ再配置 | **Unit 5F計画維持／調査未実施** |
| [reviews/2026-07-23-historical-vello-adoption-lineage-recovery.md](reviews/2026-07-23-historical-vello-adoption-lineage-recovery.md) | Vello採否レビュー／spike結果2版を現行局所renderer判断へ再照合し、成立性とK6／P6／U3a-2製品統合を分離 | **Unit 5G採択維持／製品未統合** |
| [reviews/2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md](reviews/2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md) | R9実素材／書き出し受入4版を再照合し、M1歴史sign-offと現行製品release受入を分離 | **Unit 5H歴史完了維持／GAP-32** |
| [reviews/2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md](reviews/2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md) | M0-S2 decode 6版を再照合し、採択済み自前pipe／CFR seekとVFR／process lifecycle未成立を分離 | **Unit 5I採択維持／K4・GAP-26** |
| [reviews/2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md](reviews/2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md) | M4 cache／analysis仕様20版を再照合し、Host専権cache、StateTrack、敗北枝、未実装境界を再締結 | **Unit 5J決定維持／K0契約凍結(test-only)／K1〜K8未実装** |
| [reviews/2026-07-23-historical-performance-model-lineage-recovery.md](reviews/2026-07-23-historical-performance-model-lineage-recovery.md) | performance model 21版を再照合し、liveness-aware target poolを復元、性能仮説と実装事実を分離 | **Unit 5K規律維持／pool実装済み** |
| [reviews/2026-07-23-historical-memory-model-lineage-recovery.md](reviews/2026-07-23-historical-memory-model-lineage-recovery.md) | memory model 6版を再照合し、VRAM／RAM／disk責任、hard budget、capacity／deadline境界を再締結 | **Unit 5L決定維持／K1・K7・K8未実装** |
| [reviews/2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md](reviews/2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md) | R3/DataTrack統合review 3版を再照合し、当時完了と後続半開総尺／helper driftを分離 | **Unit 5M採択維持／後続意味優先** |
| [plugin-resources.md](plugin-resources.md) | プラグインのリソースライフサイクル・アセット境界・時間参照(F-10/F-11) | **縮小採用**(PipelineCache/AssetRef/予約型は実装済み、GpuAssetCache/Importer/Feedback実行は未実装・未凍結) |
| [references.md](references.md) | 依存候補・参考リポジトリ(ライセンス区分) | 現行 |
| [ae-pain-points.md](ae-pain-points.md) | AEユーザー不満の体系化+我々の解決タグ(プラグイン窓口仮説の検証) | 現行 |
| [dev-experience.md](dev-experience.md) | 開発体験(DX): WGSL差し替え→journal復元付き再起動→将来WASM交換のはしご。hot reloadとcrash recoveryをHost所有状態からの同じinstance交換路へ畳む | 現行(2026-07-25。runtime／ABI契約は未決) |
| [plugin-ui-model.md](plugin-ui-model.md) | プラグインUIモデル: 宣言語彙 vs 自由描画。M3着手前決定で縮小採用 | **採否済み分析**(v1はHost自動生成panel、自由UIは延期) |
| [interaction-simplicity-model.md](interaction-simplicity-model.md) | 操作単純化モデル: Direct/Tool/Advanced正規化、plugin昇格、PP-Gate、M0〜M5割当 | 現行(2026-07-14。凍結済み公開契約は変更しない) |
| [extensible-core-model.md](extensible-core-model.md) | 小さなコアと探索可能な拡張: Controlled Core／admitted Host capability module／presentation module／非信頼first-party／third-party pluginの分界、締結後の並列化、編集pluginの責任寿命、Documentを増やさないアドレス可能な個体、表現domainを列挙しない能力境界 | **設計原則**(2026-07-17、2026-07-25 microkernel・信頼境界一般化。未凍結APIの実装許可ではない) |
| [vism-package-concept.md](vism-package-concept.md) | Vism (`.vism`): Project・内部plugin kind・Host UIから分離して保存/共有/再利用する映像表現の配布単位。Motoliiは最初のHost、container/loaderは未決 | **コンセプト・名称・拡張子決定／ファイル形式未決**(2026-07-17。v1実装許可ではない) |
| [vism-kit-model.md](vism-kit-model.md) | Core=文法、Vism=小さな表現、Kit=provider選択・型付き接続・初期値・公開controlを持つRack型の作者成果、Project=作品。Vism直接依存を避けるmaterialize構成とfork能力の境界を定義 | **設計原則決定／schema・形式未決**(2026-07-17、2026-07-23用語統合) |
| [reviews/2026-07-23-vism-kit-rack-unification-decision.md](reviews/2026-07-23-vism-kit-rack-unification-decision.md) | 独立Plugin Setを廃止し、接続済み一式をRack型Vism Kitへ、無関係な推薦集合をcurator list／feedへ分離 | **用語・責任統合決定／形式未決** |
| [community-distribution-model.md](community-distribution-model.md) | 中央人気／dedupeを持たず、分散地図、User library、Rack型Vism Kit、外部curator list／feed、Project Lockで多数作者と複数界隈をつなぐcommunity運用 | **運用・ガバナンス原則決定／protocol・schema・製品UI未決**(2026-07-23) |
| [reviews/2026-07-26-third-party-sustainable-economy-decision.md](reviews/2026-07-26-third-party-sustainable-economy-decision.md) | 作者が無料／有料、OSS／proprietary等を選べる経済圏と、Motoliiが市場を所有しない責任境界 | **決定／commerce protocol未決** |
| [reviews/2026-07-26-vism-malware-containment-contract-decision.md](reviews/2026-07-26-vism-malware-containment-contract-decision.md) | ambient authority 0、hard budget、typed failure、bounded recovery等の悪性Vism封じ込め意味論 | **意味論決定／runtime・schema・安全保証未締結** |
| [reviews/2026-07-27-vism-authoring-journey-decision.md](reviews/2026-07-27-vism-authoring-journey-decision.md) | v1 source forkと将来local Vism、作者入口、typed closure、Kit接続を分ける作者journey | **比較中／A4Sへ接続** |
| [reviews/2026-07-31-authoring-continuity-capsule-goal-contract.md](reviews/2026-07-31-authoring-continuity-capsule-goal-contract.md) | Inspect→Fork→preflight→atomic adoptionを一変更カプセルへ閉じ、初心者には一つの作者面を示す契約 | **決定** |
| [reviews/2026-08-01-vism-authoring-language-boundary-decision.md](reviews/2026-08-01-vism-authoring-language-boundary-decision.md) | 一般creator-authorの公式sourceをTypeScriptとし、WGSL／Rustの席、MTS-1、非目標を固定 | **言語方針決定／engine・package・payload未決** |
| [reviews/2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md](reviews/2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md) | Cavalry先例をPath／Instance等の意味値、純粋operation、明示capability、Host責任へ翻訳 | **意味SDK決定／SDK-S0未実装** |
| [reviews/2026-08-01-sdk-s0-path2d-semantic-fixture-spec.md](reviews/2026-08-01-sdk-s0-path2d-semantic-fixture-spec.md) | Cavalry型の意味連続性を既存PathOpへ接続する最初の`Path2D → Path2D` fixture責任仕様 | **SDK-S0S仕様・独立review完了／SDK-S0I未実装** |
| [reviews/2026-08-01-vism-inspector-source-automation-boundary-decision.md](reviews/2026-08-01-vism-inspector-source-automation-boundary-decision.md) | Vismを通常単位、Inspectorを意味の第一面、sourceを外部IDEへ段階開示する境界 | **決定／製品統合未実装** |
| [reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md](reviews/2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md) | Radial Repeater fork、公開façade、Host conformanceを閉じる外部crate作者scaffold仕様 | **仕様案／独立review待ち** |
| [generative-user-boundary.md](generative-user-boundary.md) | ジェネラティブ表現とユーザー拡張の境界: Shape/SVG、p5.js型入力、Materialize/Live/Feedback/Simulation、Host責務 | **設計決定**(2026-07-15。未凍結runtimeの実装許可ではない) |
| [ui-interaction-language.md](ui-interaction-language.md) | M3のUI操作言語: 既知の外殻、可視の因果、Parameter Panelを表現のホームにするUI力学、共通component契約、Simple/Advanced、漏れ実装の拒否 | **設計決定**(2026-07-16、Parameter Panel力学を2026-07-18追補) |
| [ui-visual-language.md](ui-visual-language.md) | M3の視覚言語: 高密度一覧、意味色、既存UIへの馴染み、contrast、token規約、参照範囲 | 設計基準(具体token値はM3視覚確定(G0-6)待ち) |
| [ui-score-model.md](ui-score-model.md) | 時間面UI構成モデル: 固定Laneを所有者にしない時間投影、選択コンテキスト、Group関係ラベル、回帰審判 | **設計決定**(2026-07-17、2026-07-22用語訂正。公開API・schemaの実装許可ではない) |
| [ui-runtime-architecture.md](ui-runtime-architecture.md) | React Native shell、rust-skia Timeline／Curve、wgpu + rust-skia Stage、headless interaction、platform adapterの責任境界 | **2026-08-07再基線化／製品移行未完了** |
| [reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md](reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) | UI選定、隔離probe、旧route処分、macOS先行／Windows gate、M3-R0〜R4への再基線決定 | **決定／R0未統合候補あり、main未到達** |
| [ui-artifact-terminology.md](ui-artifact-terminology.md) | Motolii Studio / Mock / Preview、baseline、spike、source assetと製品結合段階を分離するUI成果物の命名正本 | **運用正本**(Previewは結合済みnative desktop実行物だけ。現時点では未実装) |
| [mocks/](mocks/README.md) | M3高密度メインUI(基準)+timeline/interaction/UI力学の比較モック台帳 | 視覚構成の基準モック |
| [mocks-ui/](mocks-ui/README.md) | React/Viteで動く固定source asset。hash fixture、Storybook、Playwright、component map | **現行prototype / 製品直接移管のsource**（一部surfaceはlegacy bridge） |
| [ui-reference-map.md](ui-reference-map.md) | M3 UI参照地図: 規範/prototype/採否台帳/移行互換/証拠/履歴の参照順位と、React移行の実状態・既知の未統一 | **運用正本**(2026-07-19。`codex/m3-mock-components`側から回収) |
| [ui-concept.md](ui-concept.md) | UIコンセプト: 表現をすぐ画にする制作面、最初の結果、五本柱 | **設計方針**(2026-07-22に音楽メタファーを撤回。契約・M3ステータス変更なし) |
| [implementation-ledger.md](implementation-ledger.md) | 現場向け実装進行台帳: M0〜M5のNOW/NEXT/WAIT、依存、Issue昇格順 | **日々の発注入口**(意味・完了条件は各specが正本。M3は段階発注可) |
| [m3-parallel-implementation-map.md](m3-parallel-implementation-map.md) | 旧M3 routeの既知技術供給、owner、oracle、gap | **履歴化した実装検索地図**(新runtimeの実装waveはM3仕様を正とする) |
| [m3-executable-dispatch-map.md](m3-executable-dispatch-map.md) | 旧route 33子のtyped state、exact target、利用者出口 | **履歴化したdispatch snapshot**(未閉鎖gapとoracle検索用。新規dispatch authorityではない) |
| [backlog.md](backlog.md) | イシュー候補台帳(現在地サマリ+横断/新規ギャップ/v2バックログ) | 現行 |
| [specs/](specs/README.md) | マイルストーン仕様書(エージェントへの発注書)。確定/ドラフトのステータスはspecs/README.md参照 | M0/M1確定、M2基盤再締結済み(D5は別レーン)、M3はG0-9中でtoolkit非依存とReact asset直接移管R0〜R6だけ段階実装可、M4/M5ドラフト |
| [reviews/](reviews/README.md) | レビュー規律+**全review文書の索引**(この表は現役参照の抜粋。全量はreviews/README.md側が正本で、`scripts/check-docs.sh`が抜けを検証) | 運用正本 |
| [reviews/2026-07-31-repository-validation-topology-decision.md](reviews/2026-07-31-repository-validation-topology-decision.md) | `cargo test`をRust laneへ限定し、task oracle、repository lane、外部審判を分離 | **決定** |
| [spikes/](spikes/) | スパイク結果報告(S1: Slint統合、S2: デコード、[S3(R8): Vello採否](spikes/s3-vello.md)、[G0-9: UI runtime部分比較](spikes/g0-9-ui-runtime.md)、[wgpu 29 surface host](spikes/g0-9-surface-host.md)、[native Timeline外観first pass](spikes/g0-9-timeline-visual-parity.md)、[native Easing popup](spikes/g0-9-native-easing-popup.md)、[native Graph View](spikes/g0-9-native-graph-view.md)、[native Depth Rail](spikes/g0-9-native-depth-rail.md)、[multi-Surface window](spikes/g0-10-multi-surface-window.md)、[M4-K0領域契約凍結](spikes/m4-k0-region-contract.md)) | 個別文書の状態に従う |
| [reviews/2026-07-12-m2-permanence-prevention.md](reviews/2026-07-12-m2-permanence-prevention.md) | M2恒久焼き込みの**予防手順**(やること5手)。運用正本 | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-prevention.md](reviews/2026-07-14-m3-ui-boundary-prevention.md) | M3でUI都合をDocument・レンダ・公開契約へ逆流させない**予防手順**(規律8本) | 現行 |
| [reviews/2026-07-14-m3-ui-boundary-counter-review.md](reviews/2026-07-14-m3-ui-boundary-counter-review.md) | M3 UI境界規約の反対側レビュー。R1〜R9を採用/縮小/延期で再判定 | 現行(判定反映済み) |
| [reviews/2026-07-22-m3-comfortable-use-work-map.md](reviews/2026-07-22-m3-comfortable-use-work-map.md) | 製品外殻からLocal Alpha、日常操作、Distribution Readyまでを制作経路で並べるM3大地図 | **粒度化・Fable全粒レビュー完了／2026-07-25 Wave 0同期済み** |
| [reviews/2026-07-22-m3-comfortable-use-granulation.md](reviews/2026-07-22-m3-comfortable-use-granulation.md) | 快適利用大地図を仕様判断・実装・E2E・実機審判へ分けた旧152行 | **履歴snapshot／oracle来歴**（現行分解は[m3-parallel-implementation-map.md](m3-parallel-implementation-map.md)、機械dispatchは[implementation-ledger.md](implementation-ledger.md)のみ） |
| [reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md](reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md) | 旧React/WebView、Host/community kit、native surface統合の調査 | **履歴／2026-08-07再基線で標準runtimeから外れた** |
| [reviews/2026-07-22-m3-react-product-asset-promotion-contract.md](reviews/2026-07-22-m3-react-product-asset-promotion-contract.md) | Reactモックを製品packageへ直接所有移管し、維持／交換境界、diagnostic route、発注・検収STOPを固定 | **決定 / 発注停止線**(明示再開まで発注しない) |
| [reviews/2026-07-22-m3-native-easing-popup-acceptance.md](reviews/2026-07-22-m3-native-easing-popup-acceptance.md) | 旧React trigger／native wgpu popupのinteraction・lifecycle受入 | **旧route oracle**(Curve Editor標準はrust-skia native componentへ改訂) |
| [reviews/2026-07-22-m3-native-depth-rail-acceptance.md](reviews/2026-07-22-m3-native-depth-rail-acceptance.md) | native Depth Railの同一Z stack、scope、distributionとDocument境界 | **決定**(isolated core合格、製品P2R接続は停止) |
| [reviews/2026-07-22-m3-detachable-panel-window-contract.md](reviews/2026-07-22-m3-detachable-panel-window-contract.md) | Timeline/Graphから全製品panelへ一般化したdetach/re-dock、multi-window、単一snapshot契約 | **決定**(headless placementとmulti-Surface lifecycle合格、製品結合は停止) |
| [reviews/2026-07-22-m3-surface-extension-axis-separation.md](reviews/2026-07-22-m3-surface-extension-axis-separation.md) | OS window、native/React surface、Core/Host module/plugin、first/third-party信頼境界を独立判定 | **決定**(G0-9製品surfaceとG0-3 plugin UIを分離) |
| [reviews/2026-07-22-creator-developer-continuum-decision.md](reviews/2026-07-22-creator-developer-continuum-decision.md) | 利用→調整→構成→fork→authoring→共有を一つの作者経路にし、React・Vism・first-party参照実装を多数作者の成長戦略へ統合 | **決定**(参加資格は薄くし、trust／sandbox／Host責任は維持) |
| [reviews/2026-07-21-ui-surface-topology-decision.md](reviews/2026-07-21-ui-surface-topology-decision.md) | 旧1 top-level wgpu Surface、Stage/Timeline viewport、opaque child WebView islands | **旧route決定／移行oracle。新規標準は2026-08-07再基線決定** |
| [reviews/2026-07-16-m3-preflight-decisions.md](reviews/2026-07-16-m3-preflight-decisions.md) | M3着手前決定: input/状態寿命、plugin UI、性能測定、操作文法を固定し、見た目とresource実値を証拠待ちへ分離 | **設計決定**(G0-2/4/7完了。G0-3は2026-07-21再評価中) |
| [reviews/2026-07-20-m3-keymap-codec-contract.md](reviews/2026-07-20-m3-keymap-codec-contract.md) | U0d-2 keymap JSON wire・原本保全・migration境界 | **決定**(2026-07-20) |
| [reviews/2026-07-16-m3-ui-concept-to-tickets.md](reviews/2026-07-16-m3-ui-concept-to-tickets.md) | UIコンセプトを1 Issue=1 commitの実装粒へ分解。状態、入力、視覚、preview、共通操作、最初のEffect panelの依存と拒否条件 | **条件付き発注の正本**(U0b〜U4aの枝番。各行依存に従い発注可) |
| [reviews/2026-07-19-am-keyframe-graph-observation.md](reviews/2026-07-19-am-keyframe-graph-observation.md) | AMのCurve Editor公式事実、Motoliiへの採否、現行React fixtureとの差分、legacy bridge停止線 | **観察・差分台帳**（React-native置換待ち） |
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
| [reviews/2026-07-14-unified-stage-camera-design.md](reviews/2026-07-14-unified-stage-camera-design.md) | 2D/3Dを分けない単一active camera、Stage、Output Frame、枠外表示の意味。将来の具体camera所有だけ2026-07-24決定へ置換 | **決定**(2026-07-14、将来所有改訂) |
| [reviews/2026-07-24-replaceable-semantic-seat-decision.md](reviews/2026-07-24-replaceable-semantic-seat-decision.md) | Host semantic seat、換装可能Provider、Effect／Filter分類、Content-Aware Scale候補の一般則 | **決定**(2026-07-24、Fable ACCEPT) |
| [reviews/2026-07-24-camera-object-provider-decision.md](reviews/2026-07-24-camera-object-provider-decision.md) | Cameraをタイムライン上の換装可能Object／Providerとし、点群以後のrendererとrepresentation非依存Observation Contractで接続する | **決定**(2026-07-24、既存Planar互換不変) |
| [reviews/2026-07-25-controlled-microkernel-host-module-parallelism-decision.md](reviews/2026-07-25-controlled-microkernel-host-module-parallelism-decision.md) | Coreをauthorityとtyped protocolへ細くし、Host capabilityを並列化する一般則。TCBをCore＋admitted Host moduleへ限定し、公開pluginはfirst/third-partyを問わず非信頼とする | **決定**(2026-07-25、[Fable反対側レビュー](reviews/2026-07-25-controlled-microkernel-fable-counter-review.md)訂正後ACCEPT、現行API／schema変更なし) |
| [reviews/2026-07-25-parallel-human-response-frontier-execution-decision.md](reviews/2026-07-25-parallel-human-response-frontier-execution-decision.md) | 締結済みcontract上のlaneを全体barrierから解放し、通常製品routeの人間応答地点へrolling waveで返す。Fableを共有境界reviewへ限定する | **実行決定**(2026-07-25、既存未決／人間感覚gateは維持) |
| [reviews/2026-07-25-parallel-lane-readiness-map.md](reviews/2026-07-25-parallel-lane-readiness-map.md) | Wave 0の製品資産、人間審判、Vism仕様、M4/M5 contract spike、M2修復をlane化し、変更pathとSTOPを固定 | **実行決定**(2026-07-25、[Fableレビュー](reviews/2026-07-25-parallel-lane-readiness-fable-review.md)の初回P1二件を訂正後ACCEPT) |
| [reviews/2026-07-25-cu-0a05a-interrupted-worktree-restart-disposition.md](reviews/2026-07-25-cu-0a05a-interrupted-worktree-restart-disposition.md) | CU-0A05Aの旧隔離差分を証拠カプセルへ固定し、fresh baseへの縮小再適用、全試験・visual・Grok再取得を要求 | **停止線**(CU-0A05A自体は現行`DO`、旧差分の直接採用だけ禁止) |
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
| [reviews/2026-07-18-m3-egui-selection.md](reviews/2026-07-18-m3-egui-selection.md) | M3 UI基盤をSlintからeguiへ変更した時点の既存wgpu device/native texture、lifecycle、日本語IME、可変panel証拠と2026-07-24の処分 | **歴史的採用決定／製品runtime採用は撤回**(既存egui基準は比較・診断baseline限定) |
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
- **Camera / Observation**: 全Compositionに単一active cameraの席が常在し、2D=`z=0`を含む全objectが同じ観測を共有する。既存`CompCameraDoc::PlanarOrthographic`は互換baseline、将来の具体modelは換装可能Camera Object／Provider。Output Frameはその投影開口で、Stage Viewのpan/zoomはDocument外
- **凍結ゲート**: M1完了後、実際に動いたインターフェースだけを凍結して並列開発を解禁する関門。[宣言](reviews/2026-07-10-freeze-gate-declaration.md)済み(2026-07-10)。改訂は解凍手続き(理由+migrate+ゴールデン)を通す
- **グループ仮出力(ベイク)**: プリコンポの代替。グループ出力を時間範囲でキャッシュし、編集で自動無効化
- **SimulationPlugin / StateTrack**: 逐次状態シミュレーション(布・液体・パーティクル)のプラグイン境界と、そのベイク結果(チェックポイント列の区間キャッシュ)。状態はホストが所有し、`render_frame(t)`はベイク結果を読む純関数のまま(落とし穴F-12、[simulation-model.md](simulation-model.md)。口の予約段階)
- **TemporalFootprint(時間窓)**: エコー/モーションブラー等が前後フレーム/サブフレームサンプルを読むための、`NodeDesc`への静的宣言(予約。任意時刻アクセスAPIは不採用)
- **プラグインパネル**: `NodeDesc.params`自動生成panelは全保存paramを操作できる必須fallbackとして決定済みだが、製品U4aは未実装。plugin所有egui/native/Web/wgpu UIはG0-3 / GAP-13の公開・sandbox・互換・配布審判まで公開しない。標準製品surfaceのG0-9合格だけでは解除しない
- **UI配置保留**: 操作意味とtyped routeが閉じ、最終surfaceだけが未決のcontrolは[Host-owned staging surface](reviews/2026-08-09-ui-placement-deferral-staging-surface-decision.md)へ一時配置して並列接続を進められる。値／保存ownerを移さず、final assignmentで退役する。空間interaction、未決意味、公開UI frameworkの逃げ道にはしない
