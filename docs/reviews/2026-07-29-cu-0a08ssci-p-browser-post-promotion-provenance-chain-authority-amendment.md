# CU-0A08SSCI-P Browser post-promotion provenance chain authority 改訂

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-P**

## 1. 目的

`G0-6H-V1ETB-H` §H-1 Guard 1 の post-promotion provenance authority は entry を厳密に1件へ固定しており、同一Browser componentへの2件目以降の正当な変更を authority レベルで表現できない。本粒は **docs のみ** で、その authority を append-only hash chain へ全文置換し、次の唯一の `DO` を `CU-0A08SSCI-P1` へ送る。コード・test・provenance実データは1 byteも変えない。

## 2. 事実

1. `ui/motolii-web/guard-tests/browser-ownership.test.mjs:105` の `validatePostPromotionChanges` は、`provenance.postPromotionChanges` が `undefined` の場合のみ component byte と固定commit hash の一致を要求し、存在する場合は `changes.length !== 1` で throw する（同file 117行）。
2. 同guardは entry key を `POST_PROMOTION_ENTRY_KEYS`（`task` / `file` / `reason` / `fixedSourceSha256` / `currentSha256`、同file 31行）の厳密5個に固定し、過不足を throw する（124–136行）。
3. 同guardは `task` を `"G0-6H-V1ETB"`、`file` を `"ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx"`、`reason` を `"development-only Starter Media projection"` に literal 固定し（28–30行、137–145行）、`fixedSourceSha256` を `4edb3dfc…d5b8`、`currentSha256` を当該fileの実SHA-256と照合する（146–151行）。
4. 同guardの負例は `postPromotionChanges` について10件を列挙する（同file 322–441行）。
5. `ui/motolii-web/source-provenance.json:81-89` の `postPromotionChanges` は上記条件を満たす1 entry のみ。`currentSha256` = `866124a6…c42f4` は `DiscoveryBrowserCandidate.jsx` の現行実SHA-256と一致する。
6. 固定commit blob `7bfcb32a255cfd647bea593a4d2fc71d4dfeba19` の content SHA-256 は `4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8` であり、`fixedSourceSha256` と一致する。
7. `docs/implementation-ledger.md` の「現在の並列レーン」で `CU-0A08SSCI-P` 行の状態は `` `DO` ``（1件）、`CU-0A08SSCI` は `` `WAIT` ``。「発注依存証跡」で `CU-0A08SSCSD` / `G0-6H-V1ETB-H` / `G0-6H-V1ETB` はいずれも `` `DONE` ``。
8. 文字列 ``ORACLE-GUARD `CU-0A08SSCI-P``` を含む current mirror は repo 内に **7箇所**。内訳: `docs/implementation-ledger.md` 2箇所（43行 M3行 / 474行 運用判断散文）、`docs/decision-index.md` 3箇所（31行 / 49行 / 211行）、`docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` 2箇所（107行 / 110行）。
9. `docs/reviews/2026-07-29-cu-0a08ssci-browser-place-source-seam-prerequisite-order-decision.md` にも同文字列が2箇所あるが、これは当時の状態を記録した過去形の裁定本文であり **allowlist外・意図的に不変** とする。
10. 本worktreeに `docs/mocks-ui/node_modules` は無く、`@babel/parser` を要する gate（`browser-ownership.test.mjs`、`inspector-read-model-inventory.test.mjs`、`npm run test:reference-guard`）は実行できない。`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` は実行でき、着手前実測で **118 pass / 0 fail**。
11. worktree は clean。前回 Grok `REJECT` の差分は本worktreeに存在せず、継承しない。

## 3. 一問と裁定

- **(A)** `postPromotionChanges` を1 entry固定のまま据え置き、同一componentへの後続変更の背骨を停止させる。
- **(B)** 同一Browser component に対する **append-only hash chain** へ改訂する。

**VS-1 Rectangle に限り (B) を採択** する。

採択理由: [CU-0A08SSCI 前提順序裁定](2026-07-29-cu-0a08ssci-browser-place-source-seam-prerequisite-order-decision.md) §3 は、現行guard下で `.jsx` の byte 変更が取り得る表現が「(i) 2件目entry追加 → throw」「(ii) 1 entryのまま `currentSha256` だけ更新 → provenance改竄」の2つしか無く、(P) が (I)(T) の厳密な先行条件であると裁定済み。(A) は (i)/(ii) の袋小路を固定するため背骨を停止させる。

## 4. 改訂内容

[G0-6H-V1ETB-H Browser post-promotion authority再締結](2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md) §H-1 Guard 1 を次で全文置換した。

1. **PC-1**: `postPromotionChanges` はトップレベル配列。entry数 `N >= 1`。上限は設けない。
2. **PC-2**: 各 entry は `task` / `file` / `reason` / `fixedSourceSha256` / `currentSha256` のちょうど5 keyとする。過不足はrejectする。
3. **PC-3**: `index 0` の5値を既存証拠へ固定する。`task` = `G0-6H-V1ETB`、`file` = `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`、`reason` = `development-only Starter Media projection`、`fixedSourceSha256` = `4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8`、`currentSha256` = `866124a69caaa168fa19c67e6c723db97fec67a61071bdbe66973576266c42f4`。
4. **PC-4**: 全 entry の `file` は `index 0` の `file` と同一とする。
5. **PC-5**: `index >= 1` の `task` と `reason` は非空文字列とする。
6. **PC-6**: `task` は全 entry で一意とする。
7. **PC-7**: `index >= 1` について `fixedSourceSha256[i] === currentSha256[i-1]` とする。
8. **PC-8**: 末尾 entry の `currentSha256` だけが現行component実byte hashと一致する。中間 entry の `currentSha256` に現行一致を要求しない。
9. **PC-9**: `postPromotionChanges` が不在なら、component byte は固定commit blob hashと一致する。

#### Guard 1 継続不変項（期待値変更なし）

1. **K-1**: CSS / pattern の固定byte一致と、Browser以外の他3 migration のbyte一致を維持する
2. **K-2**: `sourceOwnership.exports` の public export topology を維持する
3. **K-3**: hash期待値だけを書き換えて緑にすることを禁止し、Guard 1 の正負検査追加を同一変更内で必須とする

Guard 1 の負例は次を **8件** すべて独立列挙する。すべてfailを要求する。

1. **R-1**: entry が 0件。
2. **R-2**: いずれかの entry で5 keyのいずれかが欠落。
3. **R-3**: いずれかの entry に5 key以外の余分keyが存在。
4. **R-4**: `index 0` の PC-3 五値のいずれかが不一致。
5. **R-5**: 末尾 entry の `currentSha256` が現行component実byte hashと不一致。
6. **R-6**: 鎖切れ（ある `i >= 1` で `fixedSourceSha256[i] !== currentSha256[i-1]`）。
7. **R-7**: 正しい chain を成す entry 列の並べ替え。
8. **R-8**: `index >= 1` の `task` または `reason` が空、`task` が重複、`file` が `index 0` と不一致、のいずれか。

#### 旧→新 対応表

| 旧 §H-1 Guard 1 | 新 | 期待値の変化 |
|---|---|---|
| 旧負例 1（entry 0件） | `R-1` | 変化なし |
| 旧負例 2（entry 2件以上を一律reject） | **撤回**。`R-6` / `R-7` / `R-8` へ置換 | 変化あり。正しい chain を成す2件以上を受理し、鎖切れ・並べ替え・entry不整合だけをrejectする |
| 旧負例 3（5 keyのいずれか欠落） | `R-2` | 適用範囲が `index 0` から全 entry へ拡大 |
| 旧負例 4（5 key以外が存在） | `R-3` | 適用範囲が `index 0` から全 entry へ拡大 |
| 旧負例 5 / 6 / 7（`task` / `file` / `reason` literal不一致） | `R-4` の該当部分。`file` は `R-8` の `file` 不一致部分でも継続 | `index 0` は固定のまま。`index >= 1` は literal 固定から `PC-5` / `PC-6` / `PC-4` の条件へ変化 |
| 旧負例 8 / 9（`fixedSourceSha256` / `currentSha256` 不一致） | `R-4` の該当部分と `R-5` | `index 0` の両値は固定のまま。`index >= 1` は `PC-7` の chain 条件と `PC-8` の末尾のみ現行一致へ変化 |
| 旧負例 10（`postPromotionChanges` 不在でcomponent byte相違） | `PC-9` | 変化なし。負例から閉集合項へ位置のみ移動 |
| 旧閉集合 9 / 10 / 11 | `K-1` / `K-2` / `K-3` | 変化なし（逐語存続） |

## 5. 未統一と解消条件

本commit時点で改訂済み §H-1 Guard 1 authority と `browser-ownership.test.mjs` 実装は **未統一** である。未統一の解消責任者は `CU-0A08SSCI-P1` 1件のみとし、解消範囲は `CU-0A08SSCI-P1` の発注で決める。他の粒へ解消責任を分散させない。

## 6. 変わらないもの

- React source byte（`ui/` 配下すべて）、DOM、class、stable ID、ARIA、interaction、visual state、`sourceOwnership.exports` topology、`source-provenance.json` の実データ（`postPromotionChanges` の現行1 entryを含む）、`browser-ownership.test.mjs` の期待値・literal・hash・負例実装、Guard 2 / Guard 3、H-2 / H-3 / H-4、公開API、Document、serde、永続形式、Undo単位、plugin契約、Place owner、bare `itemId` drag payload と JSX `identity` literal の `S`。
- 改訂対象の `postPromotionChanges` が記述する対象は `ui/motolii-web/source-provenance.json` の **build-time provenance record**（Document でも User settings でも Workspace でも Project session でも Transient でもない、リポジトリ内の証跡データ）である。

## 7. 非目標

- (P) / (I) / (T) のいずれかを実装すること。
- `browser-ownership.test.mjs` / `browser-catalog-decoder.test.mjs` / `inspector-read-model-decoder.test.mjs` を変更すること。
- `ui/` 配下の byte を1つでも変えること。
- `source-provenance.json` の `postPromotionChanges` 実データを編集し、2件目 entry を追加すること。
- 型・callback・event・payload・props名・module・export・wire・transport・decoder名を決めるか命名すること。
- (I) / (T) に ID を与え lane 行を追加すること（本粒が追加する lane 行は `CU-0A08SSCI-P1` の1行のみ）。
- allowlist外の stale mirror を本粒で書き換えること。
- Rust / schema / plugin / Host transport / typed intent / JSX binding / drag payload / `S` 行に触れること。

## 8. 同期した current mirror

1. `docs/implementation-ledger.md` 43行（M3行）
2. `docs/implementation-ledger.md` 474行（運用判断散文）
3. `docs/decision-index.md` 31行
4. `docs/decision-index.md` 49行
5. `docs/decision-index.md` 211行
6. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` 107行
7. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` 110行

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-P` | **DONE** | append-only hash chain authority 改訂完了（docs-only） |
| `CU-0A08SSCI-P1` | **DO** | 改訂済み §H-1 Guard 1 と `browser-ownership.test.mjs` 実装の未統一解消 |
| `CU-0A08SSCI` | **WAIT** | 前提(P) authority 改訂は完了。(I)(T) は未採番のまま |
| (I) / (T) | 未採番前提 | IDを与えない |

## 10. STOP条件

次のいずれかに遭遇したら実装を止め `ORDER: STOP` を返す — 型・event・payload・props名・decoder名・module path を決めないと文面が閉じない／`ui/` のbyte・guard期待値・`source-provenance.json` を変えないと gate が緑にならない／公開API・Document・serde・永続形式・plugin契約・Place ownerの変更が必要になる／laneの完全一致 `` `DO` `` を1件（`CU-0A08SSCI-P1`）に保てない／allowlist外fileの変更が必要になる／(I) や (T) に ID を与える必要が生じる。
