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
| 処分済み（Unit 1 + 2A + 2B） | 31 |
| 未処分 | 1,766 |

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
| 2C | historical-only D2 / selection / headless Timeline契約5 path | 未着手 |
| 3 | Core / plugin境界、最小Core、FrameDesc、native/WASM、plugin UI lineage | 未着手 |
| 4 | Document、schema、migration、journal、Undo、permanence、sidecar lineage | 未着手 |
| 5 | render、GPU、media、cache、analysis、audio、color、export lineage | 未着手 |
| 6 | UI runtime、window/surface、Stage/Preview、Browser/Inspector、workspace lineage | 未着手 |
| 7 | Timeline、interaction、keymap、keyframe、easing、motion、text lineage | 未着手 |
| 8 | 3D、camera、depth、generative、simulation lineage | 未着手 |
| 9 | Vism、package、Kit、distribution、community、creator/author lineage | 未着手 |
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
