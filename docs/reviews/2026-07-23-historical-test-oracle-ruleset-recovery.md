# Test oracle保護ruleset履歴の価値回収（Unit 4E、2026-07-23）

状態: **観察**（cutoff 1 historical blobの処分完了）

対象: `docs/reviews/2026-07-12-M2E-2-ruleset-activation.md`のcutoff全1版。

関連: [M2E-2有効化ログ](2026-07-12-M2E-2-ruleset-activation.md)、[M2入場条件](2026-07-11-M2-entry-gate.md)、[golden policy](../../crates/motolii-testkit/golden_policy/README.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

この1版は製品仕様ではなく、test oracleの変更権限を実装者から分離したGitHub運用証跡である。CODEOWNERSだけではmergeを止めないため、main rulesetでcode owner reviewを必須にし、単独Ownerの例外もPR履歴を残すAdmin bypassに限定した点を現行運用として保持する。

2026-07-23にGitHub APIでruleset id `18817145`を再取得し、次を確認した。

- `enforcement: active`
- 対象: `refs/heads/main`
- pull request rule: `required_approving_review_count=1`
- `require_code_owner_review=true`
- RepositoryRole Adminのbypassは`pull_request`のみ
- `.github/CODEOWNERS`は自身、golden、golden policy、CPU reference、toleranceを`@oshikaidesu`所有としている

したがって歴史ログの内容は現在も設定と一致する。ただし2026-07-12のPR #42が証明するのは当時のblocked状態であり、将来のruleset driftを自動的に否定しない。

## 2. 保持する責任分離

| 層 | 責任 |
|---|---|
| CODEOWNERS | 保護pathとreview ownerを宣言し、自身の改変も保護対象にする |
| GitHub ruleset | mainへのmergeでOwner reviewを必須にする |
| protected diff CI | 保護資産と非保護実装を同じ変更へ混載する迂回を拒否する |
| golden policy | semantic oracleとprovisional artifactの変更可否を分類する |
| Admin bypass | 単独Owner環境で必要な例外を、PRと監査履歴を残して通す |

どれか一つを他の代用にしない。CODEOWNERSの存在だけ、CI緑だけ、Admin権限があることだけではoracle保護を証明しない。

## 3. 停止線

- stacked recovery branchはmain rulesetの直接対象ではない。各PRが最終的にmainへ入る時の保護を、途中branchの存在から推論しない。
- ruleset ID、owner名、bypass actorは運用設定であり、Document、plugin契約、package形式へ焼かない。
- semantic oracle変更をprovisionalへの再分類で迂回しない。
- CODEOWNERSから保護行を外す変更とoracle変更を連続PRで隠さない。
- Admin bypassを通常の自己承認代替にせず、例外理由をPRへ残す。
- 歴史PR #42の状態だけで現在もactiveと報告しない。現行保証が必要な時はAPIとCODEOWNERSを再確認する。

## 4. 固定歴史出典とcoverage

唯一のblob `c83e5cc7`を全文で読み、2026-07-23のlive rulesetと現行CODEOWNERSへ照合した。処分した1 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04e-test-oracle-ruleset.tsv`を正本とする。cutoff総数1,797のうち処分済みは254、未処分は1,543である。
