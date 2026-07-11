# M2E-2 ruleset 有効化ログ

- **日時**: 2026-07-12 (Cursor agent / `gh api`)
- **ruleset**: `M2E-2 require code owner review` id=`18817145`
  - URL: https://github.com/oshikaidesu/Motolii/rules/18817145
  - `require_code_owner_review=true`
  - `required_approving_review_count=1`（単独Ownerすり抜け防止。0だと著者=OwnerでCLEANになった）
  - Admin bypass: `RepositoryRole` id=5 / `bypass_mode=pull_request`
- **実地確認**: PR #42 — `reviewDecision=REVIEW_REQUIRED`, `mergeStateStatus=BLOCKED` → クローズ
- **証跡PR**: #43（ゲート[x]）, #44（CODEOWNERS履歴）
- **方針**: 人間の自己クリック承認ループは使わない。agentがPR作成・履歴残し、必要時のみ `gh pr merge --admin`

