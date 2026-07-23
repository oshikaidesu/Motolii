# M2再締結gate反対側レビュー証拠の価値回収（Unit 4P、2026-07-23）

状態: **歴史証拠**（cutoff 1 historical blobの処分完了、当時の発効可否だけを証明）

対象: [M2基盤再締結ゲート反対側レビュー](2026-07-15-m2-foundation-reclosure-counter-review.md)のcutoff全1版。

関連: [再締結gate全版回収](2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md)、[独立追補レビュー回収](2026-07-23-historical-m2-supplementary-review-lineage-recovery.md)、[D1l検収証拠回収](2026-07-23-historical-d1l-counter-review-evidence-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このレビューはM2再締結のコード完成を判定した文書ではない。停止gateの発効宣言が、退出条件を自己充足せず、M3への迂回口を残さず、未merge案をmain既決と誤認しない形になったかを検収した証拠である。最終P0/P1=0はPR #192の発効可否だけに有効で、後のA〜C解除、コード到達、M3入場を代替しない。

Unit 4Gはgate本文14版、Unit 4Hは解除時の独立コードレビューを回収済みである。本blobの固有価値は、その二つの間にある「gate契約自体も発効前に反対側から壊す」という運用である。

## 2. 採択した検収規律

### 2.1 authorityを先に固定する

reviewerの提案を数で採らず、固定時点の`origin/main`、対象PR差分、未merge branchを別集合として照合する。CompCamera、Param Pipeline、U0a等を未merge案からmain既決へ昇格した指摘は、もっともらしさではなくrepository事実で棄却された。

この規律は現在も有効だが、当時の`origin/main`内容やPR番号を現行仕様へ外挿しない。現在のauthorityは現行spec、decision index、実コードである。

### 2.2 activation／release／landing／entryを分ける

| 証拠 | 証明するもの | 証明しないもの |
|---|---|---|
| gate発効レビュー | 停止範囲、退出条件、証跡形式、迂回口の閉鎖 | 修復コード完成、A〜C解除 |
| A〜Cコード／試験 | 指定閉集合の実装到達と意味審判 | remote landing、解除宣言 |
| 独立追補レビュー | 固定SHAにP0/P1が残らないこと | main到達、後続task入場 |
| code landing／remote CI | 対象SHAがmainとremoteで成立 | gate解除の意味宣言 |
| release declaration | 指定gateから退出したこと | 次milestone全taskの無条件解禁 |
| M3 entry／個別task | 最新依存表でそのtaskへ入れること | 他taskの解禁 |

一つのPRやgreen workspace testへ潰すと、gateが自分自身を満たす循環になる。

### 2.3 failed roundを証拠として残す

初回反対、再review P1、横断reviewの片側P1、最終timeoutを消さず、各修正と再判定を対応付ける。timeout、無出力、誤ったbranch前提はPASS票にしない。一方の最終PASSだけで他方のtimeoutを成功扱いにせず、直前の合格後に何が変わったかを主担当が限定して判断する。

外部reviewerのmodel名や世代は歴史メタデータであり、将来も同じbackendを使う要件ではない。再利用するのはread-only独立性、固定SHA、反例、修復、再審査の構造である。

## 3. 当時のP1から残す一般則

- gate文書に停止対象をtask IDと入口単位で列挙し、backlogや既存Issueからの迂回も閉じる。
- 退出証拠はSHA、PR、test名、remote runを持ち、散文の「再宣言」で代用しない。
- 恒久面候補は一括採用せず、採択／延期／棄却を個別decisionへ分ける。
- 実装作者とread-only検収者を分け、主担当がrepository事実と仕様へ戻して裁定する。
- reviewerが存在しないpath、未merge文書、古いtask表を根拠にしたら、指摘内容を補完せず不採用理由を記録する。

## 4. 現行処分

M2基盤再締結gateは後にPR #217のコード到達とPR #218の解除宣言を経て解除済みである。本blobを再発効させず、現行M3を停止しない。D1n、D5統合、GAP-23〜25等の後発残件は、それぞれの現在の範囲で扱い、当時のgateへ遡及追加しない。

将来のmilestone gateでは本レビューの構造を再利用できるが、当時のP0/P1=0を新しい差分の合格に使わない。新しい閉集合、authority、負例、実SHAで新規reviewを行う。

## 5. 復活させないもの

- PR #192のP0/P1=0をM2コード完成または現行M3入場許可と読むこと。
- timeout／無出力を多数決のPASSへ算入すること。
- 未merge branchをmainの仕様正本としてreviewすること。
- model名やreviewer人数だけを独立性・品質の証明にすること。
- 発効PRへ解除証拠を同梱して自己充足させること。
- 当時のM3停止対象や進捗表を現在へ再発効すること。
- 後発残件を過去gateの未達へ遡及追加し、成立履歴を改変すること。

## 6. 固定歴史出典とcoverage

対象blobは現行レビュー文書とbyte一致する。全文を読み、Unit 4G／4H／4Kの既回収証拠と重複・固有部分を照合した。処分した1 unique blob（4,140 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04p-reclosure-counter-review.tsv`を正本とする。cutoff総数1,797のうち処分済みは335、未処分は1,462である。
