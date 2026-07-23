# 全歴史価値回収 coverage台帳（2026-07-23）

状態: **観察**

対象: Motolii repositoryのcutoff refsから到達できる全`docs/**/*.md`履歴。価値の処分規則は[「負けた仕様」の価値回収](2026-07-23-losing-specification-value-recovery.md)を正本とする。

## 1. 目的

全履歴を一度に要約せず、小さな回収単位を反復しても、二重監査・未監査・「読んだが処分していない」を区別できるようにする。

本台帳は製品仕様を決めない。各Git blobの処分状況、回収単位、証跡だけを管理する。現行正本、公開API、Document、plugin契約を変更する判断は、各単位の独立文書へ置く。

## 2. Cutoff母集団

2026-07-23に`refs/archive/*`と作業checkpointを含む全Git refのref→SHAを固定し、そのcommit集合から到達できるMarkdown blobを列挙した。

| 項目 | 固定値 |
|---|---:|
| cutoff refs | 326 |
| unique Markdown paths | 199 |
| unique Markdown blobs | 1,797 |
| blob bytes合計 | 41,545,462 |
| 処分済み（Unit 1 + 2A + 2B + 2C + 3A + 3B-UI + 3B-runtime-A/B1/B2-A + 3C + 4A + 4B + 4C-1/2/3 + 4D + 4E + 4F + 4G + 4H + 4I + 4J + 4K + 4L + 4M + 4N + 4O + 4P + 4Q + 5A + 5B + 5C + 5D + 5E + 9A + 9B + 9C + 9D + 9E） | 357 |
| 未処分 | 1,440 |

証跡は[evidence/historical-value-recovery](evidence/historical-value-recovery/README.md)に置く。この数字は「現在のbranch一覧」を毎回数え直す値ではなく、committed `cutoff-refs.tsv`と`corpus.tsv`の固定値である。

| 固定ファイル | SHA-256 |
|---|---|
| `cutoff-refs.tsv` | `98d4d859ff0ac9e5346fb55c242e442fe733f523a42927a3baa0004b22171dc0` |
| `corpus.tsv` | `426701d4536bc523ec47d4b9b3a61d9c53d5f373a6be21e8f89d91f965dfed5d` |
| `paths.tsv` | `49d05004a6a40f6348b654901ba5eaa7fdce59571e1201e9c883414791552517` |

cutoff後に追加・更新されたdocsは最後のdelta単位へ分離する。進行中に母集団を黙って増やさない。

## 3. 状態の区別

| 状態 | 意味 | 完了計算に入るか |
|---|---|---|
| INVENTORIED | blob/pathが母集団に入った | いいえ |
| READ | 全文または明示topicを読んだが、価値の最終処分は未完 | いいえ |
| DISPOSITIONED | 現行規範・成立理由・再入場候補・負例・archiveのみへ分類し、必要な回収先とSTOP線を記録した | はい |

「現行文書を一度読んだ」「path一覧に載った」だけで歴史回収済みにしない。最終完了は`disposition-receipts/*.tsv`のunique blob集合だけで計算する。

## 4. 一単位の固定手順

1. receipt未登録blobを、同じ主題または同じ文書lineageから選ぶ。
2. 最初の版を全文で読み、以後の版は親版との差分と変更後の節を読む。分岐した版は別枝として確認する。
3. 現行正本、現行コード事実、既存の撤回・訂正・停止線と照合する。
4. 生き残る主張を5分類し、復活させない旧field・技術・modeも記す。
5. 回収文書、`decision-index.md`（新決定がある場合）、`reviews/README.md`を同じ変更で更新する。
6. 処理したblobを一つの`disposition-receipts/<unit>.tsv`へ登録する。
7. `git diff --check`、`scripts/check-docs.sh`、`scripts/check-historical-docs-recovery.sh`を通す。
8. 1コミット・1 draft PRで公開してから次単位へ進む。

同じ文面が多数の版へ反復されても、差分を読まず最新だけで代表させない。一方、全版を毎回全文再読して41 MBの重複を作業量として水増しせず、Gitの親子差分で「追加・削除・意味変更」を読む。

## 5. 回収単位

単位境界は進行中に細分化してよいが、複数境界を一つのPRへ再結合しない。

| Unit | 主対象 | 状態 |
|---|---|---|
| 0 | 現行docs棚卸しと歴史path inventory（PR #268）、現行root/specs/mocks読了（PR #269） | READ / disposition未完 |
| 1 | 初期設計、single camera、2.5D、旧plugin Kit（PR #270） | DISPOSITIONED（3 blobs） |
| 2A | historical-only基盤7 path（M2/M3 gate、keymap、external revision、preview、workspace、Graph） | DISPOSITIONED（11 blobs） |
| 2B | historical-only React mock / Browser / WebView製品移管6 path | DISPOSITIONED（17 blobs） |
| 2C | historical-only D2 / selection / headless Timeline契約5 path | DISPOSITIONED（8 blobs） |
| 3A | 最小Coreの意味、M1 plugin境界、M2締結撤回のlineage | DISPOSITIONED（12 blobs） |
| 3B-UI | plugin UI比較、v1自動panel fallback、G0-3/GAP-13 lineage | DISPOSITIONED（15 blobs） |
| 3B-runtime-A | PipelineCache、AssetRef、GpuAssetCache/Importer候補、lookbehind/Feedback lineage | DISPOSITIONED（5 blobs） |
| 3B-runtime-B1 | plugin authoring 41版、static/first-party/scaffold/native/WASM語彙 | DISPOSITIONED（41 blobs） |
| 3B-runtime-B2-A | 公開façade、first-party composition、surface/provenance、creator連続体 | DISPOSITIONED（11 blobs） |
| 3B-runtime-B2-B | native/WASM payload、install/load、third-party runtimeの横断残余 | 一部処分（payload意味はUnit 9A、runtime横断残余はUnit 9B以後） |
| 3C | FrameDescとplugin-facing共有型のlineage | DISPOSITIONED（28 blobs） |
| 4A | D1m project sidecar identity、session所有、legacy migration診断、D1n分岐 | DISPOSITIONED（6 blobs） |
| 4B | GR-PV恒久焼き込み予防5手とPathOp／dependency更新lineage | DISPOSITIONED（9 blobs） |
| 4C-1 | D1仕様穴、TimeMap、Asset指紋、編集時Generator先例lineage | DISPOSITIONED（12 blobs） |
| 4C-2 | 第二D1コード監査S1〜S18の全4版。実装済み群、DataTrack identity比較、OTIO loss report未実装を現行コードで再判定 | DISPOSITIONED（4 blobs） |
| 4C-3 | M2前第一コード監査の全2版。M2入場で解消した群と、公開runtime／M4／M5に残るPB/TM/GR/CQ/LG群を再分離 | DISPOSITIONED（2 blobs） |
| 4C-4 | Document、schema、migration、journal、Undoの残余lineage | 未着手 |
| 4D | M2E-7 RenderCtx解凍手続き全2版。三点セット、Quality転送追補、予約fieldの非証明範囲 | DISPOSITIONED（2 blobs） |
| 4E | M2E-2 test oracle保護ruleset有効化ログ。歴史的実地証拠と2026-07-23 live設定再確認 | DISPOSITIONED（1 blob） |
| 4F | M2入口ゲート全43版。対象taskの限定、A→B→C順序、棄却label gate、D1-prelude、完了再開、歴史的達成の範囲を回収 | DISPOSITIONED（43 blobs） |
| 4G | M2基盤再締結ゲート全14版。A/B/C証明、D1m/camera/持越し境界、D1n分岐、解除とM3入場の分離を回収 | DISPOSITIONED（14 blobs） |
| 4H | M2独立追補レビュー全3版。初回P1、製品経路を通る修復、再審査、local／remote／解除証拠とP2の現行処分を回収 | DISPOSITIONED（3 blobs） |
| 4I | M2 planar camera決定3版＋runtime解凍2版。semantic core／runtime／実UIの分離、oracle分割、数値評価順、Spatial再入場条件を回収 | DISPOSITIONED（5 blobs） |
| 4J | Shared Effect lifecycle全3版。Delete／Unlink／Copy Local／orphan、内部ID再採番、予約区間、Undo watermark、UI分離を回収 | DISPOSITIONED（3 blobs） |
| 4K | D1l独立反対側レビュー3本。Copy Local予約閉包、journal／Undo／Writer、現行Document生成口の見落としと、timeout／非実在pathを証拠へ数えない規律を回収 | DISPOSITIONED（3 blobs） |
| 4L | D1l current constructor＋legacy lint決定全4版。v4→現行v5への一般化、deprecated三律背反、doc-hidden＋AST採択、live suppression driftを回収 | DISPOSITIONED（4 blobs。lint実装修復はGAP-23） |
| 4M | D1l journal／Undo／Writer全2版。Effect 6 reservation、v1/v2 adapter、3 prepareと、未実装Position Add Key追補、snapshot fallback driftを回収 | DISPOSITIONED（2 blobs。fallback修復はGAP-24） |
| 4N | Param Pipeline／Element Domain／Constraint Graph全2版。現行single source、PP/ED/CG解凍条件、UI・one-shot Generatorとの非同一性とtask ID衝突を回収 | DISPOSITIONED（2 blobs。三能力は未実装、M2側IDはM2-GAP-15） |
| 4O | D1i-4 semantic oracle境界訂正全1版。oracle／harness分離、段階移行、現行分類とgate自己保護不足を回収 | DISPOSITIONED（1 blob。意味契約は完了、自己保護はGAP-25） |
| 4P | M2再締結gate反対側レビュー全1版。gate契約自体の事前検収、authority fact-check、発効／解除／landing／entry分離、timeout非証拠を回収 | DISPOSITIONED（1 blob。当時の発効可否だけを証明） |
| 4Q | 統一Stage／Camera UI全2版。旧M2 schema案と現行planar実装を分別し、Camera／Stage View／object owner、off-frame同一world、native/ReactとCore/pluginの直交を回収 | DISPOSITIONED（2 blobs。M2 camera実装済み、U1f/U2d未実装） |
| 5A | R1 export／GPU safety全5版。初期10所見、再監査漏れ、`[x]`非証拠、現行G1〜G8とGPU health driftを回収 | DISPOSITIONED（5 blobs。process/artifactはGAP-26、GPU healthはGAP-27） |
| 5B | 音声一般化設計全6版。恒久component／mix意味、段階的進捗表示、D5とのTransport衝突、製品接続／UI未到達を回収 | DISPOSITIONED（6 blobs。mixer coreは成立、製品Transport接続はGAP-28、AG-3 UIは未実装） |
| 5C | wgpu課題反対側レビュー3版＋readback／cold compile先例1版。計測前優先度の撤回、同期readback、pipeline捕捉面を回収 | DISPOSITIONED（4 blobs。readback採択gateはGAP-29、product cold compileはGAP-30） |
| 5D | D5 Transport先例全4版。旧adaptive resampling帰属の反証、audio clock主、video drop、DRS縮退、device wait／D4-FU責任境界を回収 | DISPOSITIONED（4 blobs。骨格成立、本番Preview／GPU計測／実機E2EとGAP-28は未完了） |
| 5E | 色変換／GPU export先例全1版。採択済みinverse変換、TRC、readbackを別責任へ分離し、重複GAP-14をGAP-31へ正規化 | DISPOSITIONED（1 blob。decode GPU化済み、export inverseはGAP-31） |
| 5F〜 | render、GPU、media、cache、analysis、color、export残lineage | 未着手 |
| 6 | UI runtime、window/surface、Stage/Preview、Browser/Inspector、workspace lineage | 未着手 |
| 7 | Timeline、interaction、keymap、keyframe、easing、motion、text lineage | 未着手 |
| 8 | 3D、camera、depth、generative、simulation lineage | 未着手 |
| 9A | Vism package／Kit／実装計画29版とhostless配布系譜の責任接続 | DISPOSITIONED（29 blobs） |
| 9B | historical-only plugin ecosystem全12版（Unit 1の1版 + 本Unitの11版）、community politics、旧schema | DISPOSITIONED（11 new blobs、path全版完了） |
| 9C | Vism-ready反対側レビュー、A0D/A0S contract catalog、A2 legacy adapter、A7 BPM spikeの基礎契約lineage | DISPOSITIONED（10 blobs） |
| 9D | A3 external expression／Radial Repeater／LayerSource lowering lineage | DISPOSITIONED（16 blobs） |
| 9E | 専用plugin path残余: INF-7g LLM Opacity実演とcreator-author境界 | DISPOSITIONED（1 blob）。catalog／distribution専用pathは9A/9Bで完了、横断記述は各root/spec単位で処分 |
| 10 | specs/index/backlog/implementation ledger、spikes、mocks、運用文書の残余lineage | 未着手 |
| 11 | 分岐版・rename・topic横断の取りこぼし監査 | 未着手 |
| 12 | cutoff後delta、全receipt照合、完了宣言 | 未着手 |

Unit 2以降で一つのlineageが大きすぎる場合は`4A/4B`等へ割る。Unit番号は順序の目安であり、現行の実装milestone IDではない。

## 6. 既存監査の扱い

- PR #268 / commit `eb281303`: cutoff当時の現行docs 184件とhistorical-only 21 pathを棚卸しした。歴史側は旧`plugin-ecosystem.md`以外を全文処分していないため、DISPOSITIONEDには数えない。
- PR #269 / commit `3c02e039`: current corpus A 35件をT01〜T20について読了した。topic限定のREAD証跡として再利用するが、各文書の全歴史主張を処分済みとはしない。
- PR #270 / commit `c6fc3ee9`: 初期2 blobと旧`plugin-ecosystem.md` blobの価値を処分した。最初のDISPOSITIONED receiptとして数える。

過去監査を無効化して読み直すのではなく、その監査が実際に証明した範囲だけを信用する。

## 7. 完了条件

次の全てを満たした時だけ「全歴史回収完了」とする。

1. `./scripts/check-historical-docs-recovery.sh --complete`が成功し、cutoff 1,797 blobの未処分が0。
2. disposition receiptの重複とcorpus外blobが0。
3. 回収された決定・撤回・未統一・停止線が対象正本と`decision-index.md`から逆引ける。
4. 歴史文書の仮field、旧runtime、旧UI技術、旧実装順を現行仕様として誤復活させていない。
5. cutoff後deltaを別manifestで処分し、監査中に増えたdocsを無視していない。
6. 最終PRで`git diff --check`、`scripts/check-docs.sh`、coverage完全検査を通す。

途中の各PRは部分完了であり、残数を明記する。難しい主張を`archiveのみ`へ送るだけで残数を減らさず、archive判定にも「現在の判断を拘束しない理由」を要求する。
