# G0-6H-V1ETB-P Browser projection consumerとcapsule境界の裁定

日付: 2026-07-28

対象grain: `G0-6H-V1ETB-P`

状態: 決定

依存: `G0-6H-V1ETA`、`G0-6H-V1ETC`、`G0-6H-V1EB`、`G0-6H-V1ETB-H`

## 現行code事実

1. `docs/mocks-ui/src/main.jsx:16` が `const currentRouteCapture = import.meta.env.MODE === "current-route-capture";` を持つ。`import.meta.env.MODE` を読むfileは本fileだけ。
2. 同file `:94`〜`:108` のroute `plugin-browser-candidate` は `Component: LegacyHostBoundaryScreen` で、propsに `BrowserComponent: DiscoveryBrowserCandidate`（`:99`）と `developmentEmptyProjection: currentRouteCapture`（`:105`）を渡す。
3. `docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx:191` は `return <BrowserComponent {...props} />;` であり、渡されたcomponentへ全propsを転送する。
4. `docs/mocks-ui/source-asset-inventory.json` は `docs/mocks-ui/src/main.jsx` をpinしていない。
5. `docs/mocks-ui/guard-tests/starter-media-capsule.test.mjs:232`〜`:248` は `ui/motolii-web` と `docs/mocks-ui/src` の再帰walkで全fileのtextに対し `"starter-media"` / `"starterMedia"` / `"Starter Media"` / `"starter-media-generation"` の4 tokenを一つでも含めばfailさせる。例外・allowlistは現在存在しない。
6. `ui/motolii-web` 配下の実在fileは16件で、Starter Media責任を持つのは `src/read-model/starterMediaProjectionDecoder.js` だけ。`src/index.js` はpublic export topologyを固定byte一致維持対象、`src/patterns/DiscoveryBrowser.jsx` と各CSSは既存guardで固定byte一致維持対象。
7. `ui/motolii-web/src/read-model/starterMediaProjectionDecoder.js` は `ALLOWED_MEDIA_TYPES = {image/png, image/svg+xml, video/mp4, audio/wav}`、root key `["media"]`、entry key `["path","mediaType"]`、`media.length === 4` 固定、`path` のbackslash / 空 / `.` / `..` segment拒否、`path`と`mediaType`の重複拒否、失敗時 `TypeError`（`SMP1`〜`SMP7`）を持ち、fallbackしない。exportは `decodeStarterMediaProjection` のみ。
8. `docs/mocks-ui/starter-media/starter-media-provenance.json` の `media` は順に `starter-media/media/starter-clip.mp4`=`video/mp4`、`starter-media/media/starter-mark.svg`=`image/svg+xml`、`starter-media/media/starter-still.png`=`image/png`、`starter-media/media/starter-tone.wav`=`audio/wav`。

## P-1 starter-media-capsule guardのtoken閉集合

`starter-media-capsule` guardの目的は、製品コードがcapsuleのbyteおよびgeneratorへruntime依存しないための境界であり、名称言及の禁止はその境界の手段であって目的ではない。

`G0-6H-V1ETB` は4 token scanに対し ui側の exact path allowlist を次の4件だけ導入する。照合はworktree相対pathの完全一致で行い、glob / prefix / basename一致 / dirname一致を使わない。

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
2. `ui/motolii-web/src/read-model/starterMediaProjectionDecoder.js`
3. `ui/motolii-web/source-provenance.json`
4. `ui/motolii-web/guard-tests/browser-ownership.test.mjs`

導出根拠は4 tokenと同時に次を満たすためとする。

1. 1は `starterMedia` を含むdecoder import path のみ許可する `A-4` 要求。
2. 2は `§2(6)` により ui側で唯一Starter Media責任を持つfileであり、1のimport対象であるため。
3. 3は `postPromotionChanges[0].reason` の値 `development-only Starter Media projection` を許すための `A-4` 要求。
4. 4は `G0-6H-V1ETB-H` H-1 Guard 1が同literalを正負検査する責任を担うため。

path allowlist内の許可構文は次の閉集合のみ許容し、他は拒否する。

1. `DiscoveryBrowserCandidate.jsx`: `../read-model/starterMediaProjectionDecoder.js` のprivate decoder import specifierだけ許可。
2. `starterMediaProjectionDecoder.js`: 現行どおり4 tokenの出現0件を維持。
3. `source-provenance.json`: `postPromotionChanges[0].reason` の文字列値 `development-only Starter Media projection` だけ許可。
4. `browser-ownership.test.mjs`: 上記reason literalのstring literalの構造検査だけ許可。別path、コメント、識別子、fallback分岐の言及は許可しない。

allowlist内でも禁止は維持する。`"starter-media-generation"` の言及と `starter-media/media/` を含むpath文字列は禁止のまま。

allowlist外の `ui/motolii-web` 全fileと `docs/mocks-ui/src` 配下の全fileは、従来どおり4 token全面禁止を維持する。`docs/mocks-ui/src/main.jsx` は免除対象ではない。

緩和は同一変更内で allowlistの正例・負例test を追加する形のみ許可し、scan自体の削除、`forbiddenRoots`縮小、token配列削除、`skip`分岐追加、条件分岐による無効化を禁止する。

## P-2 mock consumer配線の所在

`docs/mocks-ui/src/main.jsx` 単独で配線を所有する。

`currentRouteCapture === true` の時だけ、main.jsx内のmodule-scope wrapper component を `BrowserComponent` として登録する。wrapperは受け取ったpropsを全て `DiscoveryBrowserCandidate` へ転送し、`developmentProjection` をP-3のenvelopeで追加する。

`currentRouteCapture === false` の時、`BrowserComponent` は現行どおり `DiscoveryBrowserCandidate` 自身（同一identity）を使用し、wrapperを経由させない。`developmentProjection` propは一切渡さず、`undefined` として渡さない。通常routeのDOM / class / stable ID / ARIA / interaction は不変とする。

wrapperはmodule-scope定数で定義し、render中に生成しない。`LegacyHostBoundaryScreen` 側のmemo依存によるremountを起こさないため。

次の4 fileは不変とする。

1. `docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx`
2. `docs/mocks-ui/src/legacy/LegacyRegions.jsx`
3. `docs/mocks-ui/source-asset-inventory.json`
4. `docs/mocks-ui/guard-tests/source-asset-inventory.test.mjs`

`main.jsx` は `docs/mocks-ui/source-asset-inventory.json` にpinされていないため、inventory pin更新は発生しない。

新route、hash、query、global、別served entry、Vite config、package script、`playwright.config.js` の追加・変更は行わない。

## P-3 envelopeの形

`main.jsx` のmodule-scope literal 1箇所に閉じる。`main.jsx`内で2箇所以上置かない。

root key は `media` のみ、entry key は `path` と `mediaType` のみ。entryは4件、順序は clip → mark → still → tone。

1. `{ path: "starter-clip.mp4", mediaType: "video/mp4" }`
2. `{ path: "starter-mark.svg", mediaType: "image/svg+xml" }`
3. `{ path: "starter-still.png", mediaType: "image/png" }`
4. `{ path: "starter-tone.wav", mediaType: "audio/wav" }`

`path` はbasename-onlyとし、`/`、`\`、`.`、`..`、`starter-media/media/` prefixを含めない。

`H-2` 写像により4 mediaTypeは各1回だけ現れ、Browser tile の `preview` class は順に `video` / `logo` / `texture` / `audio` とする。`folder` 規則や `startsWith("image/")` fallback 追加はしない。

capsuleのbyte、`starter-media-provenance.json`、generator（`docs/mocks-ui/scripts/starter-media-generation.mjs`）をruntimeで読まない。`fs`、`fetch`、`import`、Vite glob importでcapsuleへ到達しない。

envelopeの状態所有は `Transient / development presentation` とし、Document、User settings、Workspace、Project session、公開API、schema、serde、plugin契約、永続形式へ焼かない。

## 維持する既決（本粒で変更しない）

`G0-6H-V1ETA` A-4（`developmentProjection` prop、通常route不変、tab / rail / results / tileの閉集合、provenanceの `postPromotionChanges` 5 keyと `reason` literal）、`G0-6H-V1ETB-H` H-1（Guard 1の11条件と10負例、Guard 2 / 3のhash pin更新限定）、H-2（4 mediaType全域写像）、H-3（inventory 2 fileの非目標とSTOP）、および§2(7)のdecoder契約（key閉集合、4件固定、重複拒否、`TypeError`、fallbackなし）を、そのまま有効なものとして参照する。文面の再定義・言い換え・数値変更をしない。

## 引き継ぎ: V1ETB implementation allowlist（最終8点）

`G0-6H-V1ETB-H` H-4候補6点へ2点を追加し、次の8点で確定する。

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
2. `ui/motolii-web/source-provenance.json`
3. `ui/motolii-web/guard-tests/browser-ownership.test.mjs`
4. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
5. `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
6. `docs/mocks-ui/src/main.jsx`
7. `docs/mocks-ui/guard-tests/starter-media-capsule.test.mjs`
8. `docs/implementation-ledger.md`

## 負例

1. capsule guardの `forbiddenRoots` から `ui/motolii-web` または `docs/mocks-ui/src` を外す。
2. 4 token配列から token を削る、または正規表現へ緩める。
3. allowlistをglob / prefix / basename一致で書く。
4. allowlistを5件以上、または3件以下にする。
5. `docs/mocks-ui/src/main.jsx` をallowlistへ入れる。
6. allowlist内fileへ `starter-media-generation` または `starter-media/media/` を書く。
7. `LegacyHostBoundaryScreen.jsx` / `LegacyRegions.jsx` を編集して配線する。
8. `source-asset-inventory.json` またはその guard を編集する。
9. 通常modeで `developmentProjection`（`undefined` を含む）を渡す。
10. wrapperをrender中に生成する、または通常modeでも`BrowserComponent`をwrapperにする。
11. envelopeを `main.jsx` 外file、JSON fixture、公開exportへ置く。
12. envelope `path` へ `starter-media/media/` prefixやdirectory区切りを含める。
13. envelopeのentry数を4以外にする、mediaTypeを重複させる、key を足す。
14. capsule byte / provenance json / generator をruntimeで読む。
15. H-2 へ `folder` や `startsWith("image/")` のfallbackを足す。
16. decoder失敗時にfallback描画する、または `try/catch` で握り潰す。
17. guard・test期待値・hash期待値・thresholdの書き換えだけで緑にする。
18. 本粒でcode / fixture / guard / provenance / 画像を1 byteでも変更する。

## 非目標・停止線

`G0-6H-V1ETB` / `V1ETT` / `V1ETE` / `V1G` の実装、R-9実描画、Playwright追加、reference generationは非目標。

公開API、Document意味、plugin契約、永続形式、serde、schema、Undo、selectionへ触れる必要が見えた時点で `ORDER: STOP`。

allowlistが5件目を必要とする、4件のいずれかが不要と判明する、`main.jsx` 単独で配線が成立しない、envelopeがcapsule読取なしで成立しない、のいずれかが判明した時点で `ORDER: STOP`。

## 次の一粒

`G0-6H-V1ETB`
