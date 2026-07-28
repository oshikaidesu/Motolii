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

### Guard 1（所有者pin固定の差分許容）

`browser-ownership.test.mjs` は固定commit（`56c318edcddab7cf95d263cc2f7dd2b4e6791134`）と現在worktreeの差を許容せず固定する責任を外し、代わりに `postPromotionChanges` を正負両方で検査する。検査は以下の閉集合のみに限定する。

1. `schema`: トップレベル配列
2. `entry` 数: 1
3. `entry` key: `task` / `file` / `reason` / `fixedSourceSha256` / `currentSha256` のちょうど5個。過不足はrejectする
4. `task` literal: `G0-6H-V1ETB`
5. `file` literal: `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
6. `reason` literal: `development-only Starter Media projection`
7. `fixedSourceSha256`: 固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134` の旧Browser component `blob` hash
8. `currentSha256`: V1ETB完了時の該当コンポーネント実byte hash
9. CSS / pattern の固定byte一致と、Browser以外の他3 migration のbyte一致を維持する
10. `sourceOwnership.exports` の public export topology を維持する
11. hash期待値だけを書き換えて緑にすることを禁止し、Guard 1 の正負検査追加を同一変更内で必須とする

Guard 1 の負例は次を **10件** すべて独立列挙する。すべてfailを要求する。

1. `postPromotionChanges` のentryが 0件
2. `postPromotionChanges` のentryが 2件以上
3. entry key不足（5 keyのいずれか欠落）
4. entry key余剰（5 key以外が存在）
5. `task` のliteralが `G0-6H-V1ETB` と不一致
6. `file` のliteralが `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` と不一致
7. `reason` のliteralが `development-only Starter Media projection` と不一致
8. `fixedSourceSha256` が固定commit blob hash不一致
9. `currentSha256` がV1ETB完了時実byte hash不一致
10. `postPromotionChanges` が無いのにcomponent byteが固定commitと相違

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
