# CU-0A08SSCI-P1 Browser post-promotion provenance chain guard 整合

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-P1**

## 1. 目的

[CU-0A08SSCI-P改訂](2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md)および[G0-6H-V1ETB-H Browser post-promotion authority再締結](2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md)で全文置換済みの §H-1 Guard 1（`PC-1`〜`PC-9` / `R-1`〜`R-8` / `K-1`〜`K-3`）と、`ui/motolii-web/guard-tests/browser-ownership.test.mjs` の `validatePostPromotionChanges` および post-promotion 専用正負試験の未統一を解消する。React source byte・`source-provenance.json` 実データは変更しない。

## 2. 事実（着手前 code fact）

1. `validatePostPromotionChanges` は `changes.length !== 1` で reject し、2件以上の正当 chain を受理できなかった。
2. entry key / literal 検査は index 0 のみ。改訂 authority の全 entry 適用（`PC-2`）および append-only hash chain（`PC-7` / `PC-8`）は未実装だった。
3. 負例配列に撤回条項「entry 2件以上を一律 reject」が残存し、`PC-1` と衝突していた。
4. 実データ `source-provenance.json` の `postPromotionChanges` は1 entry のまま、`PC-3` index 0 固定値と現行 `.jsx` byte hash は一致していた。
5. `CU-0A08SSCI-P1` が ledger「現在の並列レーン」で唯一の完全一致 `` `DO` ``（1件）だった。

## 3. 実装対応

| 条項 | 実装 |
|---|---|
| `PC-9` | `postPromotionChanges` 不在時の固定 byte 判定を1文字も変えず維持 |
| `PC-1` | 配列・`length >= 1`、上限なし |
| `PC-2` | 全 entry の5 key 厳密 |
| `PC-3` | index 0 五値固定（`POST_PROMOTION_INDEX0_CURRENT_SHA256` 追加） |
| `PC-4`〜`PC-8` | file 一致、index>=1 の task/reason、task 一意、hash chain、末尾のみ live 一致 |
| `R-1`〜`R-8` | 独立 label 付き負例（撤回条項削除） |
| `K-1` / `K-2` | 既存 hash・exports・migrations assertion を逐語維持 |
| `K-3` | 正例 P-A（実データ）/ P-B（2-entry）/ P-C（3-entry・中間 live 非一致受理）を追加 |
| `PC-9` 負例 | 「no postPromotionChanges with mismatched component bytes」を逐語維持 |

変更 file: `ui/motolii-web/guard-tests/browser-ownership.test.mjs` の post-promotion 領域のみ。

## 4. 変わらないもの

- `ui/` 配下 React/CSS/pattern byte、`source-provenance.json` 実データ、Guard 2 / Guard 3、H-2 / H-3 / H-4。
- `CU-0A08SSCI` は `` `WAIT` ``。(I) / (T) への ID 付与なし。
- authority 改訂2文書の意味。

## 5. 次 DO の裁定

- `CU-0A08SSCI-P1` は `` `DONE` ``。
- `(P)` は authority と guard 実装の両面で閉じた。
- `(I)` / `(T)` は未採番前提のまま。次の唯一の `DO` は本粒で選定せず未選定（完全一致 `` `DO` `` は **0件**）。
- `(I)` の採番と選定は後続 docs-only 粒が行う。

## 6. 非目標

- `(I)` / `(T)` への採番・lane 行追加、`CU-0A08SSCI` 本体実装、allowlist 外 mirror 更新、新規依存 install。

## 7. 検証（実測）

```text
export MOTOLII_NODE_PATH=/Users/member_ottoto/rust_ae/Motolii/spikes/g0-9-web-ui/node_modules
NODE_PATH=$MOTOLII_NODE_PATH node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs
# tests 3 / pass 3 / fail 0

NODE_PATH=$MOTOLII_NODE_PATH node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs
# tests 118 / pass 118 / fail 0

NODE_PATH=$MOTOLII_NODE_PATH node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs
# tests 39 / pass 39 / fail 0

./scripts/check-docs.sh
# OK: docs整合チェック全項目通過
```

既存のローカル `@babel/parser@7.29.7` を `NODE_PATH` から一時解決して
`npm run test:reference-guard` を実行し、275 tests / 265 pass / 10 fail。
失敗した10 suiteは base commit `8292e92d` の clean worktreeでも同一であり、
本粒による増減は0件。repo内への依存install、symlink、package/lockfile変更は行っていない。

## 8. 同期した current mirror

1. `docs/implementation-ledger.md` 43行（M3行）
2. `docs/implementation-ledger.md` 476行（運用判断散文）
3. `docs/decision-index.md` 31行 / 49行 / 211行
4. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` 107行 / 110行

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-P1` | **DONE** | guard と改訂 authority の整合完了 |
| `CU-0A08SSCI` | **WAIT** | (I)/(T) 未採番 |
| 次 `` `DO` `` | **未選定** | 0件（(I) 採番は後続 docs-only 粒） |
