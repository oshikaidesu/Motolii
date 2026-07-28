# G0-6H-V1ETB-Q Browser route oracle allowlist補正の裁定

日付: 2026-07-28
対象grain: `G0-6H-V1ETB-Q`
状態: 決定
依存: `G0-6H-V1ETA`、`G0-6H-V1ETC`、`G0-6H-V1ETB-H`、`G0-6H-V1ETB-P`

## 現行code事実

1. `docs/implementation-ledger.md` の「現在の並列レーン」には `G0-6H-V1ETB-Q` と `G0-6H-V1ETB` の行があり、現時点の遷移先はそれぞれ `DO` / `WAIT` である。
2. `docs/mocks-ui/playwright.current-route-capture.config.js` は `testDir: "./tests"` かつ `testMatch: /current-route-capture-v1etc[.]playwright[.]js$/` を持ち、`port` は `4174`、起動コマンドは `npm run dev -- --mode current-route-capture` で固定。
3. `docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js` は `test.describe("G0-6H-V1ETC current-route-capture empty projection")` の下で既存2件のテストを持つ。
4. `docs/mocks-ui/package.json` には専用current-route-capture scriptはなく、専用実行は `npx playwright test --config playwright.current-route-capture.config.js` である。通常routeは `npm run test:visual` 所有（71件）。
5. `docs/reviews/2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md` の「引き継ぎ: V1ETB implementation allowlist（最終8点）」は8点を列挙し、`G0-6H-V1ETB-P` の最終allowlistは8点である。
6. BASE_SHA で `./scripts/check-docs.sh`、`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`、`node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs` は全pass。
7. `docs/implementation-ledger.md` と `docs/decision-index.md` は本粒で追加する決定行を受ける準備があり、4 file許可で行を1行ずつ増やす。

## Q-1 既存のtest/configを再利用するか

新規test/configを作らない。`G0-6H-V1ETA` `A-2` が `playwright.current-route-capture.config.js` を専用channelの唯一のownerとして固定し、`testMatch` を `current-route-capture-v1etc[.]playwright[.]js$/` へ（既存1件）限定しているため、新規test追加は `testMatch` 変更を伴い衝突する。

このため本粒は既存の `docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js` に追記し、`docs/mocks-ui/playwright.current-route-capture.config.js`、`playwright.config.js`、`package.json`、`docs/mocks-ui/package.json` のscript、`testMatch`、`testDir`、`port` は変更しない。

## Q-2 allowlistの補正内容

`G0-6H-V1ETB-P` の最終8点へ、既存Playwright test file `docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js` を9点目として追加し、`G0-6H-V1ETB` implementation allowlist を9点で確定する。

`G0-6H-V1ETB-P` 非目標として文書化された `Playwright追加` は、**新規Playwright file / config / channel の追加**を指す。既存専用test fileへのV1ETB oracle追記は本裁定で許可する。

## Q-3 追記されるoracleの責任分割

1. `docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js` の既存 `test.describe("G0-6H-V1ETC current-route-capture empty projection")` 内2件は、`name` / `selector` / `expect` を変更せず維持する。
2. `G0-6H-V1ETB` の正例・負例は新規の `test.describe` として追記する。正例は `G0-6H-V1ETA` `A-4` の閉集合（`tab row` `Media source rail` `results` `tile`）を検証し、負例は `A-4` が0件とする selector 群を検証する。
3. 同一modeの通常route不変テストは、既存V1ETCのテスト2件目が所有しているため、V1ETBでは重複させない。
4. 通常Playwright（`playwright.config.js` / `npm run test:visual` の71件）は別gateに残し、専用channelへ取り込まない。
5. golden画像 / visual threshold / 既存期待値 / `package.json` / `vite.config` は変更しない。

## 維持する既決（本粒で変更しない）

`G0-6H-V1ETA` `A-2` / `A-3` / `A-4` / `A-5` / `A-6`、`G0-6H-V1ETB-H` `H-1`〜`H-4`、`G0-6H-V1ETB-P` `P-1` / `P-2` / `P-3` と、同裁定の負例18点をそのまま有効にする。

## 引き継ぎ: V1ETB implementation allowlist（最終9点）

`G0-6H-V1ETB-P` の §97〜108 で示した順序と文面を保持し、1〜8点をそのまま引き継ぐ。

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
2. `ui/motolii-web/source-provenance.json`
3. `ui/motolii-web/guard-tests/browser-ownership.test.mjs`
4. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
5. `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
6. `docs/mocks-ui/src/main.jsx`
7. `docs/mocks-ui/guard-tests/starter-media-capsule.test.mjs`
8. `docs/implementation-ledger.md`
9. `docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js`

## 負例

1. 新規test fileを追加する
2. `testMatch` を変更する
3. `playwright.config.js` を変更する
4. `package.json` の script を追加する
5. 既存V1ETCの2件を削除・改名・期待値変更する
6. golden / threshold を書き換えて合格を取る
7. 通常routeの71件を専用channelへ移す
8. 本粒で code / fixture / guard / provenance / 画像を1 byte でも変更する

## 非目標・停止線

`G0-6H-V1ETB` / `V1ETT` / `V1ETE` / `V1G` の実装、R-9実描画、`component` / `fixture` / `guard` / `provenance` / `image` / `config` / `package script` / `golden` / `threshold` の変更は非目標。

公開API、Document意味、plugin契約、永続形式、serde、schema、Undo、selection へ触れる必要が見えた時点で `ORDER: STOP`。
`testMatch` 変更または新規test file が必要と判明した時点で `ORDER: STOP`。
allowlist が10点目を必要とする、または9点のいずれかが不要と判明した時点で `ORDER: STOP`。

## 次の一粒

`G0-6H-V1ETB`
