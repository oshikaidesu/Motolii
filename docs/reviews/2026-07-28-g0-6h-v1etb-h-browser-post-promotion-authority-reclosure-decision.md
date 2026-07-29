# G0-6H-V1ETB-H Browser post-promotion authority再締結の裁定

日付: 2026-07-28

対象grain: `G0-6H-V1ETB-H`

状態: 決定

依存: `G0-6H-V1ETA`、`G0-6H-V1ETC`

## H-1 post-promotion authority guard の閉集合

`G0-6H-V1ETA` `§A-4` の許可は、P-1時点で「実装非目標」とされていたBrowser component変更に対する後発の実装非目標解除として扱い、両者が衝突した場合は`A-4`を優先する。

`G0-6H-V1ETB` は `source-provenance.json` を用いる3つの guard のみを再締結対象とし、他のguardの pin は変更しない。

1. `ui/motolii-web/guard-tests/browser-ownership.test.mjs`
2. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
3. `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`

#### 改訂注記（2026-07-29 / CU-0A08SSCI-P）

- 改訂日 `2026-07-29`、改訂粒 `CU-0A08SSCI-P`。
- 一問は (A) 1-entry固定で背骨を停止 / (B) 同一Browser componentのappend-only hash chainへ改訂 の二択であり、**VS-1 Rectangle に限り (B) を採択** した。
- 旧 §H-1 Guard 1 の **閉集合1〜8項** と **負例10件** は部分修正せず、下記 `PC-1`〜`PC-9` と `R-1`〜`R-8` へ **全文置換** した。期待値は変わる。
- 旧 **9 / 10 / 11 項**（CSS / pattern 固定byte一致と他3 migrationのbyte一致、`sourceOwnership.exports` topology 維持、hash期待値だけの書換え禁止と正負検査同一変更必須）は `postPromotionChanges` の shape を規定しない Guard 1 継続不変項であり、**期待値変更なしで存続する**。§4-1(c) の「Guard 1 継続不変項」節へ番号のみ `K-1`/`K-2`/`K-3` として逐語のまま移し、内容を書き換えない。
- 本改訂は authority 文面のみで、`ui/` の byte、`browser-ownership.test.mjs`、`source-provenance.json` 実データを変えない。したがって本commit時点で **authority と guard実装は未統一** である。未統一の解消責任者は次の唯一の `DO` である `CU-0A08SSCI-P1` 1件のみとし、解消範囲は `CU-0A08SSCI-P1` 自身の発注で決める。他の粒へ解消責任を分散させない。

### Guard 1（所有者pin固定の差分許容）

`browser-ownership.test.mjs` は固定commit（`56c318edcddab7cf95d263cc2f7dd2b4e6791134`）と現在worktreeの差を許容せず固定する責任を外し、代わりに `postPromotionChanges` を正負両方で検査する。検査は以下の閉集合のみに限定する。

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

### Guard 2（Auth Hash配列）

`browser-catalog-decoder.test.mjs` の `AUTHORITY_SHA256` のうち
`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` と
`ui/motolii-web/source-provenance.json` の2 pin を新しい実byte hashへ更新するだけとする。
新旧 `source byte` の post-promotion 判定を追加しない。

### Guard 3（Inspector read-model pin）

`inspector-read-model-decoder.test.mjs` の `AUTHORITY_SHA256` に存在する
`ui/motolii-web/source-provenance.json` の1 pin を新しい実byte hashへ更新するだけとする。
Guard 3 は `DiscoveryBrowserCandidate.jsx` を含むBrowser component pin を新設しない。
Guard 3 は `fixedSourceSha256` / `currentSha256` の照合、`postPromotionChanges` のschema / literal / 件数検査、その他post-promotion意味の正負検査を一切持たない。

post-promotion意味の成立責任は Guard 1 のみが持ち、Guard 2 / 3 に複製しない。

## H-2 presentation-only 全域写像

Starter Media projection の `mediaType` から Browser tile の `preview` class への全域写像を次で確定する。

1. `video/mp4` → `video`
2. `image/svg+xml` → `logo`
3. `image/png` → `texture`
4. `audio/wav` → `audio`

根拠は `ui/motolii-web/src/read-model/starterMediaProjectionDecoder.js` の `ALLOWED_MEDIA_TYPES` が上記4要素の閉集合であることのみとする。
`folder` / `startsWith("image/")` などの fallback・拡張規則は導入しない。

この写像は presentation-only であり、Document、公開API、schema、plugin契約、永続形式へ焼かない。

## H-3 inventory の非目標宣言

`docs/mocks-ui/source-asset-inventory.json` と
`docs/mocks-ui/guard-tests/source-asset-inventory.test.mjs` は固定commit mock closureの担当であり、`ui/motolii-web` 側のpost-promotion差分を扱わない。
そのためこの2 fileは `G0-6H-V1ETB` の非目標であり、V1ETBのallowlistへ入れない。
V1ETBが両fileの変更を必要と感じた時点で `ORDER: STOP` とし、Codexへ戻す。

## H-4 V1ETB への引き継ぎ

この節は `V1ETB implementation allowlist candidate` の選定文言を出すもので、本節の候補を本粒で確定しない。

V1ETB implementation allowlist candidate:

- `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
- `ui/motolii-web/source-provenance.json`
- H-1の3 guard
- `docs/implementation-ledger.md`

`G0-6H-V1ETB-H` 自身を指す語は本粒で統一する。`本粒` 以外の表記（この粒/この本粒等）を使わない。
`V1ETB` は本粒を引き継ぐ実装粒である。

通常routeは `developmentProjection` prop の無い既存routeでDOM / class / stable ID / ARIA / interaction を不変とし、V1ETBはこれを正例として明示検査する。

固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134` の旧byteは `git` blob 参照で継続参照し、旧snapshotのコピーを本粒で作らない。

`G0-6H-V1ETT` / `G0-6H-V1ETE` / `G0-6H-V1G` は `V1ETB` の非目標であり、同一変更へ混ぜない。
