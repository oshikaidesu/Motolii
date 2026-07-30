# G0-6H-R reference authority役割再照合

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-R: **DONE**

## 1. 目的

二つの固定commitのauthority役割を、現行Motolii authorityと現行コード事実だけから
docs上で一意に分類し、**互いに競合しない別役割**として固定する。

- `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は、旧U0e-2 reference generation
  `u0e2-08f96cbd7754-85c0fc529ab1` **だけ**に対する不変の再現source authorityである。
- `56c318edcddab7cf95d263cc2f7dd2b4e6791134` は、現行product-owned React source asset
  であり、`G0-6H-E`が取り込んだ承認済み`#plugin-browser-candidate` normal色5画面のprovenanceである。

同時に、**Git ancestryと`check-reference`成功はvisual parityや人間承認をroute横断で証明しない**
ことを明記し、次の一粒 `G0-6H-S`（docs-only route裁定粒）へhandoffする。

## 2. 確認した事実

- F1. [`docs/mocks-ui/reference-handoff.md`](../mocks-ui/reference-handoff.md):9-16 は
  「React source authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`」「capture generation
  `u0e2-08f96cbd7754-85c0fc529ab1`」「source manifest SHA-256
  `08f96cbd77545e1734cc285970137ba20e1b9f31f3fac8f4e3704c467daa64a4`」
  「`reference-output/CURRENT`が指すgeneration内の5画面×6 variant、計30 PNG」を固定している。
- F2. [`docs/reviews/2026-07-21-m3-u0e-2-reference-fixture-contract.md`](2026-07-21-m3-u0e-2-reference-fixture-contract.md):35-36 は
  「U0e-2の比較元は`origin/codex/m3-mock-components`の固定commit
  `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`」と定めている。
- F3. [`ui/motolii-web/source-provenance.json`](../../ui/motolii-web/source-provenance.json) の
  `fixedSourceCommit` は `56c318edcddab7cf95d263cc2f7dd2b4e6791134`、`authority` は
  `docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md`、
  `sourceOwnership.product` は `@motolii/motolii-web`、owner は
  `R1-browser / R2B-easing-trigger / R3B-key-tools / R4C-inspector`。
- F4. [`docs/ui-reference-map.md`](../ui-reference-map.md):25,68 と
  [`docs/decision-index.md`](../decision-index.md):59,62 は現行React source assetを固定commit
  `56c318ed` としている。
- F5. [`docs/reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md`](2026-07-28-g0-6h-e-candidate-approval-observation.md):18 は
  承認済み5画面の固定React source authorityを
  `56c318edcddab7cf95d263cc2f7dd2b4e6791134` と記録している。同:21 は
  `npm run check-reference` が
  `reference generation OK: u0e2-08f96cbd7754-85c0fc529ab1 (30 PNGs)` を返したが
  **read-onlyの再現証拠に過ぎない**と明記する。同:22 は旧`#reference/*`と派生25枚が
  未承認であることを記録する。
- F6. [`docs/mocks-ui/README.md`](../mocks-ui/README.md):14,44,48 により、現行React候補routeは
  `#plugin-browser-candidate`、旧U0e-2固定5画面routeは`#reference/*`で、
  `reference-provenance.json`がsource/fixture/capture正本である。
- F7. `git merge-base --is-ancestor eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0
  56c318edcddab7cf95d263cc2f7dd2b4e6791134` は作業worktreeで exit 0（ancestor成立）。
  両objectは `git cat-file -t` で `commit` である。

## 3. authority役割の分類（本粒の決定）

- **R-1**: `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は generation
  `u0e2-08f96cbd7754-85c0fc529ab1` **に限った**不変の再現source authorityである。
  このcommitを現行product surfaceの所有authorityとして扱わない。
- **R-2**: `56c318edcddab7cf95d263cc2f7dd2b4e6791134` は現行product-owned React
  source assetであり、`G0-6H-E`が取り込んだ`#plugin-browser-candidate`
  normal色5画面のprovenanceである。このcommitを旧generationのsource authorityへ
  遡及記載しない。
- **R-3**: 二役割は**非競合**で、片方が他方を無効化・置換・上書きしない。
- **R-4**: `git merge-base --is-ancestor` の成立（F7）は系譜事実に留まり、
  route横断のvisual parityも人間承認も証明しない。
- **R-5**: `check-reference` 成功（F5）は固定generationのread-only再現証拠に留まり、
  現行候補5画面との同一性、Decision template充足、G0-6H完了の代替にしない。

## 4. この決定が確定しないこと

- 旧30 PNG / 派生25枚の人間採否
- 具体token値、製品theme、閾値、goldenの選定
- route裁定（`#reference/*`と`#plugin-browser-candidate`のどちらをG0-6H人間審判の入力とするか）
- `G0-6H` / `CU-0B01` / `U0e-3` の状態変更・完了・解禁
- 現行候補と旧referenceのvisual parityそのもの

## 5. 非目標

- `docs/mocks-ui/reference-handoff.md` のDecision template / 5秒課題checklistを埋める。
- route裁定、route変更、`#reference/*` / `#plugin-browser-candidate` の入場条件変更。
- 画像・variant・generation・`CURRENT`・`reference-provenance.json` の生成/変更/再生成。
- 具体token値、製品theme、閾値、golden、期待値の選定・変更。
- React / CSS / Rust / fixture / test / guard / public API / Document / plugin契約 /
  永続形式 / serde defaults の変更。
- `docs/implementation-ledger.md` の変更（本粒では触らない。ledger反映はCodexが所有する）。
- `G0-6H` / `CU-0B01` / `U0e-3` の状態変更・完了・解禁。
- 隣接チケット（`CU-107*`、`CU-110*`、`U3a-*`、`U2h-*`）への波及。
- `G0-6H-S` の中身の先取り裁定、または複数の後続粒のhandoff。

## 6. 必須負例

- N1. `docs/mocks-ui/reference-handoff.md:9` の
  `React source authority: \`eb16d06f...\`` が変更・削除・置換されている。
- N2. 追記が `56c318ed...` を旧generation `u0e2-08f96cbd7754-85c0fc529ab1` の
  source authorityとして記載している。
- N3. Decision template / checklistの `未記入` または `[ ]` が1つでも埋まっている。
- N4. Git ancestry または `check-reference` 成功を、visual parity・人間承認・
  route同一性の根拠として書いている。
- N5. 現行候補normal色5画面の承認を旧派生25枚・旧30 PNGへ拡張して書いている。
- N6. `G0-6H` / `CU-0B01` / `U0e-3` の状態語を変更している。
- N7. `docs/implementation-ledger.md` が差分に含まれている。
- N8. 許可外file（React / CSS / Rust / fixture / test / guard / json / スクリプト）が
  差分に含まれている。
- N9. `docs/decision-index.md` に `決定/縮小採用/延期/棄却/撤回/未統一/観察/比較中/停止線`
  以外の状態語が入っている。
- N10. `docs/reviews/README.md` の既存行が並べ替え・重複・削除されている。
- N11. 追加行が `` `CU-0A08BP` ``（`DO`）形式のstale prose行を新設している。
- N12. `G0-6H-S` 以外の後続粒を新設、または `G0-6H-S` の裁定結論を先取りしている。
- N13. TODOスタブ、部分適用（4 fileのうち一部だけ変更）、lint/test抑制の追加。

## 7. STOP条件

1. 役割分類にroute変更、公開契約、Document、plugin契約、永続形式、token、threshold、
   golden、画像、source変更のいずれかが必要になる。
2. 未監査source、新しい画像生成、新しい計測を根拠にしないと分類できない。
3. 旧generationの期待値・manifest・PNGを変更する必要が生じる。
4. 現行候補と旧referenceのvisual同一性を証拠なしに仮定しないと書けない。
5. `docs/implementation-ledger.md` を変えないと整合が取れないと判断した。
6. AUTHORITY行のSHA-256と作業時のfile hashが一致しない。

## 8. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-R0` | **DONE** | 本authority再照合粒を選定 |
| `G0-6H-R` | **DONE** | 旧generation authorityと現行product source authorityを非競合の別役割として分類 |
| `G0-6H-S` | **DO** | docs-onlyのroute裁定粒。`#reference/*`と`#plugin-browser-candidate`のどちらをG0-6H人間審判の入力routeとするかだけを裁定する |
| `G0-6H` | **DO / HUMAN** | 据え置き（未完了） |
| `U0e-3` | **WAIT** | 据え置き |
