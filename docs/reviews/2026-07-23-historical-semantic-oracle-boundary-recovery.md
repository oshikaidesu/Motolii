# Semantic oracle保護境界lineageの価値回収（Unit 4O、2026-07-23）

状態: **意味契約は決定・実装済み／enforcement自己保護だけGAP-25**（cutoff 1 historical blobの処分完了）

対象: [D1i-4 semantic oracle境界訂正](2026-07-17-d1i4-semantic-oracle-boundary-decision.md)のcutoff全1版。

関連: [M2仕様](../specs/M2-document-model.md)、[M2E-2 ruleset回収](2026-07-23-historical-test-oracle-ruleset-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

この決定が訂正したのは「テストコードを永久固定する」ことではなく、「既存variantの意味を表す最小oracle artifactを永久固定する」ことである。期待入力・variant・写像・数値・bytesはimmutable、現行APIでactualを得るharnessはmutableと分離する。意味を変える時は既存oracleを更新せず、新variant＋新oracleを追加する。

現行treeはこの境界を実装している。BlendModeは独立TSVへ移行済みで、CAM-G0も独立oracleを持つ。PathOp／LookAt・Follow／Bezier／Transformはwhole-file semanticのまま残り、必要になるまで一括declassificationしない。clipping maskはprovisionalで、regenerate条件を持つ別分類である。

ただし、oracle変更を拒否するscriptとworkflow自体はCODEOWNERS対象ではない。active ruleset `18817145`はcode-owner reviewを要求するが、現行Owner pathは`.github/CODEOWNERS`、`golden/**`、`golden_policy/**`、CPU reference、toleranceに限られる。同一PRで`check-golden-update-policy.sh`やCI stepを弱めれば、分類台帳とoracleを直接触らずに後続保護を空洞化できる。D1i-4の意味契約を未完了へ戻さず、自己保護だけをGAP-25で閉じる。

## 2. 採択した境界

### 2.1 immutable oracleとmutable harness

| 資産 | 変更規則 |
|---|---|
| semantic oracle | 既存内容の変更・削除・降格は禁止。新variantは新fileで追加 |
| harness | import、fixture、runtime取得、公開API接続へ追随可。期待値を複製しない |
| migration ledger | 旧harness→新oracleの対応を保持し、削除・retargetを禁止 |
| provisional artifact | `MOTOLII_REGENERATE_WHEN`がある時だけ更新可。semanticへ昇格可 |

BlendMode移行で成立した規律を、API変更のない残りharnessへ一括適用しない。oracleの所在を移すこと自体が保護差分なので、必要になった単位ごとに同値転記、harness接続、分類置換、負例を閉じる。

### 2.2 現行分類

- 独立oracle: BlendMode、CAM-G0 planar identity。
- whole-file semantic: PathOp geometry、LookAt／Follow、Bezier、Transform composition。
- provisional: D7 clipping mask。

分類候補や古い完了表だけでoracleを増減しない。とくにD7はsRGB blend依存の暫定状態をsemanticと誤記しない。

## 3. enforcement再照合

`check-golden-update-policy.sh`はbase ref／merge-base／diff取得をfail-closedにし、classificationとmigrationの削除、semanticの変更・削除・降格を負例で拒否する。CIもPR時にこれを呼ぶ。この本体は決定に一致する。

一方、M2E-2の`check-protected-diff.sh`は保護対象を`golden/**`、CPU reference、toleranceに限定し、base ref不在時は「変更なし」として通す。CIの通常PRはbaseをfetchし、bootstrap条件も既存pathで閉じるため、これだけで現行通常PRが直ちに無検査になるとは断定しない。しかし次の自己保護は未成立である。

1. semantic gate script、protected-diff script、workflowの該当stepがCODEOWNERS対象外。
2. active rulesetはcode-owner reviewのみで、required status checkを同じrulesetに固定していない。
3. semantic oracleの実pathは分類gateで守るが、独立oracle directory自体をOwner pathにはしていない。
4. protected-diff単体のmissing-base動作が、semantic gateのfail-closed規律と不統一。

## 4. GAP-25の閉じ方

- `.github/workflows/ci.yml`の保護step、両gate script、分類／migration台帳、semantic oracle pathを自己保護集合として列挙する。
- 既存rulesetとCIのどちらが何を止めるかを負例表にし、script削除、step削除、path除外、missing base、classification削除をそれぞれ拒否する。
- live GitHub rulesetの実状態とrepository内fixtureを分け、API確認結果だけを恒久証明にしない。
- oracle値、tolerance、既存variant、分類の意味は変更しない。保護対象を広げるPRと意味資産更新PRを混ぜない。
- required status checkを採る場合はcheck名とworkflow rename／fork PRの挙動を先にfixture化し、未決のGitHub設定をdocsだけで「有効」と書かない。

## 5. 復活させないもの

- test harness全体をbyte-for-byte凍結すること。
- API移行のたびに互換aliasを追加すること。
- PR番号、branch名、管理者overrideを通常のoracle更新口にすること。
- semantic oracleへregenerate marker例外を設けること。
- whole-file semanticをAPI変更前に一括declassificationすること。
- D7 provisionalを現在の分類だけでsemanticへ昇格すること。
- script／workflowの自己保護不足を理由に既存oracle値を書き直すこと。
- live rulesetの一回のAPI応答だけをrepository内の再現可能なenforcement証明とすること。

## 6. 固定歴史出典とcoverage

対象blobは現行決定文書とbyte一致する。全文を読み、分類台帳、migration、gate script、CI、CODEOWNERS、2026-07-23時点のactive rulesetへ照合した。処分した1 unique blob（4,807 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04o-semantic-oracle.tsv`を正本とする。cutoff総数1,797のうち処分済みは334、未処分は1,463である。
