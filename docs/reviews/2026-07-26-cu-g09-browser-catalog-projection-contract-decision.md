# CU-G09 Browser catalog projection契約決定

- 日付: 2026-07-26
- 状態: 決定
- 粒: CU-G09 DONE

## REACT AUTHORITY
- 対象面: Browser（product-owned React所有面）
- 契約: docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md
- UI runtime境界: product owner `ui/motolii-web` / mock consumer `docs/mocks-ui`
- 対応spec ID: M3 `CU-G09`（`docs/specs/M3-ui-integration.md`）

## SOURCE ASSET
- 固定commit: `56c318edcddab7cf95d263cc2f7dd2b4e6791134`
- 旧path: `docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx`
- 現行path: `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
- SHA256: `4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8`
- export: `DiscoveryBrowserCandidate`（`ui/motolii-web/src/index.js:1`）
- 参照closure: `./discovery-browser-candidate.css`, `../patterns/DiscoveryBrowser.jsx`, `ui/motolii-web/guard-tests/browser-ownership.test.mjs`, `ui/motolii-web/source-provenance.json`, `docs/mocks-ui/source-asset-inventory.json`

## PRESERVE
- DOM / class / stable ID / ARIA / drag interaction / visual state / CSS / threshold / golden / test 既存値を0バイト変更で保持する。

## REPLACE
- 本粒は実装しない。mock-local state（`useCandidateTags`、`selectedItem`）とfixture literal の交換範囲を契約化し、CU-0A08B で `DiscoveryBrowserCandidate` と Host read-model 接続を行う。

## STATE OWNER
- Catalog projection: Document外 Host read model
- selection/hover/result-view: Transient or local presentation
- thumbnail size 等: 既定どおり User settings / Workspace
- tag assignment owner: 未決（`S`）

## DIAGNOSTIC ROUTE
- 既存の製品route（Browser）を保持し、`#diagnostics/*` と分離。診断画面追加はこの粒に含めない。

## NEGATIVE ORACLE
- `N1`〜`N14` を停止線とし、二重copy、legacy import、opaque ID分岐、二重state、threshold/golden変更を拒否する。

## STOP
- 仕様未決埋めや既決契約違反が出た場合は停止し、STOPラインを維持して報告する。未決解決には新しい authority 追加を要するためこの粒を再定義しない。

## §1 FACTS
- `DiscoveryBrowserCandidate.jsx` は 1130 行（上記SHA256）。
- Catalog本体の `PluginCard` 定義は `DiscoveryBrowserCandidate.jsx:191#1`。
- `PluginCard` のinstanceは以下。`data-browser-item`、`data-preview`、`application/x-motolii-browser-item`、`motion`、`data-plugin-kind` を持つ。
  - echo-bloom: `DiscoveryBrowserCandidate.jsx:383#1`
  - type-pulse: `DiscoveryBrowserCandidate.jsx:401#1`
  - fold-field: `DiscoveryBrowserCandidate.jsx:421#1`
- `ElementCard` 定義は `DiscoveryBrowserCandidate.jsx:457#1`。
- `ElementCard` のinstanceは以下。`data-element` / `data-element-provider` / `data-preview` / `application/x-motolii-browser-item` / `motion` を持つ。
  - rectangle: `DiscoveryBrowserCandidate.jsx:664#1`
  - ellipse: `DiscoveryBrowserCandidate.jsx:665#1`
  - text: `DiscoveryBrowserCandidate.jsx:666#1`
  - solid: `DiscoveryBrowserCandidate.jsx:667#1`
  - glyph-current: `DiscoveryBrowserCandidate.jsx:668#1`
  - type-pulse: `DiscoveryBrowserCandidate.jsx:669#1`
  - ribbon-array: `DiscoveryBrowserCandidate.jsx:670#1`
  - particle-field: `DiscoveryBrowserCandidate.jsx:671#1`
- Effects catalog fixture の `data-info` は `Effects Browser|Apply visual results to an existing object or Timeline bar`（`DiscoveryBrowserCandidate.jsx:288#1`）。
- Create catalog fixture の `data-info` は `Create Browser|Browse every registered item that creates a Stage or Timeline object`（`DiscoveryBrowserCandidate.jsx:556#1`）。
- Media Browser（`AssetTile`）は `data-info` 文言 `Media Browser|Search project assets and registered folders from one surface`（`DiscoveryBrowserCandidate.jsx:862#1`）で、catalog範囲外。
- `EFFECT_TAGS` と `CREATE_TAGS` は `DiscoveryBrowserCandidate.jsx:24#1`, `30#1`。割当は `useCandidateTags` のローカルstate。
- `type-pulse` は PluginCard と ElementCard の両方で同一 `itemId` を持つ衝突がある。

## §2 分類語彙
- `A / D / U / S` は CU-0A08IS `docs/reviews/2026-07-26-cu-0a08is-inspector-read-model-inventory.md` と同一で再利用する。
- `A`: item固有の読み取り可能意味を持つ（carrier field 含む）。
- `D`: 既知規則で1対1閉写像する。
- `U`: その要素の役割固定、装飾・構造のみ。
- `S`: 既存authority不足により未決（fixture上の可視値を含む）。

分類の強制規則（本粒）:
- `display_name`（`name` prop の可視ラベル）は `A`（CU-101 の配置意味は参照のみで再定義しない）。
- `taxonomy_refs` / `provider_ref` / `pack_ref` / `install_state_ref` / `impact` / `tag_refs` は carrier として `A`。語彙内容・発行者・cross-snapshot 寿命は `S`。
- `preview_kind` は pinned JSX の `motion` boolean 全域写像のため `D`（`data-preview` = `motion` | `poster`）。
- `folder` / `labels` / `data-search` は投影済み field からの結合であり `D`。decoder 出力へ同名 key を重複持たせない。
- `mode` / `state` / `category` / `subtype` / `kind` / `type` / `provider` 可視値 / `identity` / `thumbnail` token / tag 割当の永続 owner は `S`（availability 系の振る舞いは `install_state_ref` carrier と混同しない）。

## §3 catalog範囲
- 対象は `PluginCard` + `ElementCard` の11 item。
- `AssetTile` / `CandidateProjectBrowser` の `AssetTile` 群は除外（Media Browser）。
- 除外根拠: Media 側 fixture の `data-info` 固定文言 `Media Browser|Search project assets and registered folders from one surface` は catalog 意図ではなく project asset 探索を示す（`DiscoveryBrowserCandidate.jsx:862#1`）。Effects / Create は上記 §1 の `data-info` で catalog 面と区別する。

## §4 可視要素インベントリ

anchor規則: `JSX位置` は `DiscoveryBrowserCandidate.jsx:<line>#<occ>`（`#` 省略時は `1`）。`要素ID` は `<card族>.<instanceまたはdef>.<部位>`。

### §4.1 `PluginCard` 定義（component共通）

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| plugin-card.def.root-shell | DiscoveryBrowserCandidate.jsx:213#1 | `div.candidate-plugin-card` + `data-browser-item` | `U` |
| plugin-card.def.folder-attr | DiscoveryBrowserCandidate.jsx:219#1 | `data-folder={folder}` | `D` |
| plugin-card.def.labels-attr | DiscoveryBrowserCandidate.jsx:220#1 | `data-labels={labels}` | `D` |
| plugin-card.def.search-attr | DiscoveryBrowserCandidate.jsx:221#1 | `data-search={search}` | `D` |
| plugin-card.def.preview-kind-attr | DiscoveryBrowserCandidate.jsx:226#1 | `data-preview={motion ? "motion" : "poster"}` | `D` |
| plugin-card.def.drag-payload | DiscoveryBrowserCandidate.jsx:228#1 | `application/x-motolii-browser-item` = bare `itemId` | `S` |
| plugin-card.def.main-button | DiscoveryBrowserCandidate.jsx:235#1 | `button.candidate-plugin-card-main` | `U` |
| plugin-card.def.thumb-frame | DiscoveryBrowserCandidate.jsx:241#1 | `span.plugin-thumb` + thumbnail class | `U` |
| plugin-card.def.kind-glyph | DiscoveryBrowserCandidate.jsx:242#1 | `span.candidate-kind` = `{kind}` | `U` |
| plugin-card.def.state-overlay | DiscoveryBrowserCandidate.jsx:243#1 | `span.thumb-state` = `{state}` when set | `S` |
| plugin-card.def.impact-overlay | DiscoveryBrowserCandidate.jsx:246#1 | `span.candidate-impact` = `{impact}` when set | `A` |
| plugin-card.def.motion-mark | DiscoveryBrowserCandidate.jsx:247#1 | `span.candidate-motion-mark` ▶ | `U` |
| plugin-card.def.display-name | DiscoveryBrowserCandidate.jsx:250#1 | `<b>{name}</b>` | `A` |
| plugin-card.def.tag-chips | DiscoveryBrowserCandidate.jsx:252#1 | `#tag` chips from `tags[]` | `A` |
| plugin-card.def.taxonomy-nav | DiscoveryBrowserCandidate.jsx:258#1 | `nav.candidate-card-taxonomy` | `U` |
| plugin-card.def.category-button | DiscoveryBrowserCandidate.jsx:262#1 | `{category.label}` / `data-plugin-type` | `S` |
| plugin-card.def.subtype-button | DiscoveryBrowserCandidate.jsx:264#1 | `{subtype.label}` / `data-plugin-type` | `S` |

### §4.2 `PluginCard` instance — `echo-bloom`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| plugin-card.echo-bloom.item_id | DiscoveryBrowserCandidate.jsx:384#1 | `itemId="echo-bloom"` | `A` |
| plugin-card.echo-bloom.mode | DiscoveryBrowserCandidate.jsx:385#1 | `mode="installed"` → `data-mode` | `S` |
| plugin-card.echo-bloom.folder | DiscoveryBrowserCandidate.jsx:386#1 | `folder="motion project"` | `D` |
| plugin-card.echo-bloom.labels | DiscoveryBrowserCandidate.jsx:387#1 | `labels="goto effect glow"` | `D` |
| plugin-card.echo-bloom.search | DiscoveryBrowserCandidate.jsx:388#1 | `search="echo bloom light pulse glow effect installed"` | `D` |
| plugin-card.echo-bloom.thumbnail | DiscoveryBrowserCandidate.jsx:389#1 | `thumbnail="bloom"` | `S` |
| plugin-card.echo-bloom.kind | DiscoveryBrowserCandidate.jsx:390#1 | `kind="FX"` | `S` |
| plugin-card.echo-bloom.display_name | DiscoveryBrowserCandidate.jsx:391#1 | `name="Echo Bloom"` | `A` |
| plugin-card.echo-bloom.category | DiscoveryBrowserCandidate.jsx:392#1 | Effect / effect | `S` |
| plugin-card.echo-bloom.subtype | DiscoveryBrowserCandidate.jsx:393#1 | Light / glow | `S` |
| plugin-card.echo-bloom.pack_ref | DiscoveryBrowserCandidate.jsx:394#1 | `pack="motion-kit-alpha"` | `A` |
| plugin-card.echo-bloom.preview_kind | DiscoveryBrowserCandidate.jsx:395#1 | `motion` → `data-preview="motion"` | `D` |
| plugin-card.echo-bloom.tag_refs | DiscoveryBrowserCandidate.jsx:397#1 | go-to, atmosphere | `A` |

### §4.3 `PluginCard` instance — `type-pulse`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| plugin-card.type-pulse.item_id | DiscoveryBrowserCandidate.jsx:402#1 | `itemId="type-pulse"` | `A` |
| plugin-card.type-pulse.mode | DiscoveryBrowserCandidate.jsx:403#1 | `mode="installed"` | `S` |
| plugin-card.type-pulse.folder | DiscoveryBrowserCandidate.jsx:404#1 | `folder="type"` | `D` |
| plugin-card.type-pulse.labels | DiscoveryBrowserCandidate.jsx:405#1 | `labels="effect text motion"` | `D` |
| plugin-card.type-pulse.search | DiscoveryBrowserCandidate.jsx:406#1 | `search="type pulse kinetic text motion effect"` | `D` |
| plugin-card.type-pulse.thumbnail | DiscoveryBrowserCandidate.jsx:407#1 | `thumbnail="glyph"` | `S` |
| plugin-card.type-pulse.kind | DiscoveryBrowserCandidate.jsx:408#1 | `kind="FX"` | `S` |
| plugin-card.type-pulse.display_name | DiscoveryBrowserCandidate.jsx:409#1 | `name="Type Pulse"` | `A` |
| plugin-card.type-pulse.category | DiscoveryBrowserCandidate.jsx:410#1 | Effect / effect | `S` |
| plugin-card.type-pulse.subtype | DiscoveryBrowserCandidate.jsx:411#1 | Typography / text | `S` |
| plugin-card.type-pulse.pack_ref | DiscoveryBrowserCandidate.jsx:412#1 | `pack="motion-kit-alpha"` | `A` |
| plugin-card.type-pulse.identity | DiscoveryBrowserCandidate.jsx:413#1 | `identity="motion-kit.type-pulse"` | `S` |
| plugin-card.type-pulse.impact | DiscoveryBrowserCandidate.jsx:414#1 | `impact="◆ 12 KEYS"` | `A` |
| plugin-card.type-pulse.preview_kind | DiscoveryBrowserCandidate.jsx:415#1 | `motion` → `data-preview="motion"` | `D` |
| plugin-card.type-pulse.tag_refs | DiscoveryBrowserCandidate.jsx:417#1 | kinetic | `A` |

### §4.4 `PluginCard` instance — `fold-field`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| plugin-card.fold-field.item_id | DiscoveryBrowserCandidate.jsx:422#1 | `itemId="fold-field"` | `A` |
| plugin-card.fold-field.mode | DiscoveryBrowserCandidate.jsx:423#1 | `mode="blocked"` | `S` |
| plugin-card.fold-field.folder | DiscoveryBrowserCandidate.jsx:424#1 | `folder="experimental"` | `D` |
| plugin-card.fold-field.labels | DiscoveryBrowserCandidate.jsx:425#1 | `labels="effect space"` | `D` |
| plugin-card.fold-field.search | DiscoveryBrowserCandidate.jsx:426#1 | `search="fold field space geometry effect incompatible"` | `D` |
| plugin-card.fold-field.thumbnail | DiscoveryBrowserCandidate.jsx:427#1 | `thumbnail="fold"` | `S` |
| plugin-card.fold-field.kind | DiscoveryBrowserCandidate.jsx:428#1 | `kind="FX"` | `S` |
| plugin-card.fold-field.display_name | DiscoveryBrowserCandidate.jsx:429#1 | `name="Fold Field"` | `A` |
| plugin-card.fold-field.category | DiscoveryBrowserCandidate.jsx:430#1 | Effect / effect | `S` |
| plugin-card.fold-field.subtype | DiscoveryBrowserCandidate.jsx:431#1 | Spatial / space | `S` |
| plugin-card.fold-field.install_state | DiscoveryBrowserCandidate.jsx:432#1 | `state="Unavailable"` + blocked mode overlay | `S` |
| plugin-card.fold-field.preview_kind | DiscoveryBrowserCandidate.jsx:421#1 | instance 全体に `motion` prop なし → 定義の `motion ? "motion" : "poster"` により `data-preview="poster"` | `D` |
| plugin-card.fold-field.tag_refs | DiscoveryBrowserCandidate.jsx:434#1 | review | `A` |

### §4.5 `ElementCard` 定義（component共通）

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.def.root-button | DiscoveryBrowserCandidate.jsx:476#1 | `button.candidate-element-card` | `U` |
| element-card.def.preview-kind-attr | DiscoveryBrowserCandidate.jsx:488#1 | `data-preview={motion ? "motion" : "poster"}` | `D` |
| element-card.def.search-attr | DiscoveryBrowserCandidate.jsx:489#1 | `data-search` = name+type+provider 結合 | `D` |
| element-card.def.drag-payload | DiscoveryBrowserCandidate.jsx:493#1 | bare `itemId` payload | `S` |
| element-card.def.preview-frame | DiscoveryBrowserCandidate.jsx:500#1 | `span.candidate-element-preview` | `U` |
| element-card.def.glyph | DiscoveryBrowserCandidate.jsx:501#1 | `<i>{glyph}</i>` | `U` |
| element-card.def.provider-caption | DiscoveryBrowserCandidate.jsx:502#1 | `<small>{provider}</small>` | `S` |
| element-card.def.state-overlay | DiscoveryBrowserCandidate.jsx:503#1 | `em.thumb-state` = `{state}` | `S` |
| element-card.def.impact-overlay | DiscoveryBrowserCandidate.jsx:504#1 | `em.candidate-impact` = `{impact}` | `A` |
| element-card.def.motion-mark | DiscoveryBrowserCandidate.jsx:505#1 | motion ▶ mark | `U` |
| element-card.def.display-name | DiscoveryBrowserCandidate.jsx:508#1 | `<b>{name}</b>` | `A` |
| element-card.def.type-line | DiscoveryBrowserCandidate.jsx:509#1 | `<small>{type}</small>` | `S` |
| element-card.def.tag-chips | DiscoveryBrowserCandidate.jsx:511#1 | `#tag` chips | `A` |

### §4.6 `ElementCard` instance — `rectangle`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.rectangle.item_id | DiscoveryBrowserCandidate.jsx:664#1 | `element="rectangle"` / itemId rectangle | `A` |
| element-card.rectangle.display_name | DiscoveryBrowserCandidate.jsx:664#1 | `name="Rectangle"` | `A` |
| element-card.rectangle.type | DiscoveryBrowserCandidate.jsx:664#1 | `type="Shape"` | `S` |
| element-card.rectangle.provider_ref | DiscoveryBrowserCandidate.jsx:664#1 | `provider="Built-in"` | `A` |
| element-card.rectangle.glyph | DiscoveryBrowserCandidate.jsx:664#1 | `glyph="□"` | `U` |
| element-card.rectangle.thumbnail | DiscoveryBrowserCandidate.jsx:664#1 | `thumbnail="rectangle"` | `S` |
| element-card.rectangle.preview_kind | DiscoveryBrowserCandidate.jsx:664#1 | `motion` prop なし → `data-preview="poster"` | `D` |
| element-card.rectangle.tag_refs | DiscoveryBrowserCandidate.jsx:664#1 | layout | `A` |

### §4.7 `ElementCard` instance — `ellipse`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.ellipse.item_id | DiscoveryBrowserCandidate.jsx:665#1 | `element="ellipse"` / `itemId` ellipse | `A` |
| element-card.ellipse.display_name | DiscoveryBrowserCandidate.jsx:665#1 | `name="Ellipse"` | `A` |
| element-card.ellipse.type | DiscoveryBrowserCandidate.jsx:665#1 | `type="Shape"` | `S` |
| element-card.ellipse.provider_ref | DiscoveryBrowserCandidate.jsx:665#1 | `provider="Built-in"` | `A` |
| element-card.ellipse.glyph | DiscoveryBrowserCandidate.jsx:665#1 | `glyph="○"` | `U` |
| element-card.ellipse.thumbnail | DiscoveryBrowserCandidate.jsx:665#1 | `thumbnail="ellipse"` | `S` |
| element-card.ellipse.preview_kind | DiscoveryBrowserCandidate.jsx:665#1 | `motion` prop なし → `data-preview="poster"` | `D` |
| element-card.ellipse.tag_refs | DiscoveryBrowserCandidate.jsx:665#1 | `[]`（初期割当なし） | `A` |

### §4.8 `ElementCard` instance — `text`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.text.item_id | DiscoveryBrowserCandidate.jsx:666#1 | `element="text"` / `itemId` text | `A` |
| element-card.text.display_name | DiscoveryBrowserCandidate.jsx:666#1 | `name="Text"` | `A` |
| element-card.text.type | DiscoveryBrowserCandidate.jsx:666#1 | `type="Layer"` | `S` |
| element-card.text.provider_ref | DiscoveryBrowserCandidate.jsx:666#1 | `provider="Built-in"` | `A` |
| element-card.text.glyph | DiscoveryBrowserCandidate.jsx:666#1 | `glyph="T"` | `U` |
| element-card.text.thumbnail | DiscoveryBrowserCandidate.jsx:666#1 | `thumbnail="text"` | `S` |
| element-card.text.preview_kind | DiscoveryBrowserCandidate.jsx:666#1 | `motion` prop なし → `data-preview="poster"` | `D` |
| element-card.text.tag_refs | DiscoveryBrowserCandidate.jsx:666#1 | brand-kit | `A` |

### §4.9 `ElementCard` instance — `solid`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.solid.item_id | DiscoveryBrowserCandidate.jsx:667#1 | `element="solid"` / `itemId` solid | `A` |
| element-card.solid.display_name | DiscoveryBrowserCandidate.jsx:667#1 | `name="Solid"` | `A` |
| element-card.solid.type | DiscoveryBrowserCandidate.jsx:667#1 | `type="Layer"` | `S` |
| element-card.solid.provider_ref | DiscoveryBrowserCandidate.jsx:667#1 | `provider="Built-in"` | `A` |
| element-card.solid.glyph | DiscoveryBrowserCandidate.jsx:667#1 | `glyph="■"` | `U` |
| element-card.solid.thumbnail | DiscoveryBrowserCandidate.jsx:667#1 | `thumbnail="solid"` | `S` |
| element-card.solid.preview_kind | DiscoveryBrowserCandidate.jsx:667#1 | `motion` prop なし → `data-preview="poster"` | `D` |
| element-card.solid.tag_refs | DiscoveryBrowserCandidate.jsx:667#1 | brand-kit | `A` |

### §4.10 `ElementCard` instance — `glyph-current`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.glyph-current.item_id | DiscoveryBrowserCandidate.jsx:668#1 | `element="glyph-current"` / `itemId` glyph-current | `A` |
| element-card.glyph-current.display_name | DiscoveryBrowserCandidate.jsx:668#1 | `name="Glyph Current"` | `A` |
| element-card.glyph-current.type | DiscoveryBrowserCandidate.jsx:668#1 | `type="Generator"` | `S` |
| element-card.glyph-current.provider_ref | DiscoveryBrowserCandidate.jsx:668#1 | `provider="Motion Kit"` | `A` |
| element-card.glyph-current.glyph | DiscoveryBrowserCandidate.jsx:668#1 | `glyph="G"` | `U` |
| element-card.glyph-current.thumbnail | DiscoveryBrowserCandidate.jsx:668#1 | `thumbnail="glyph"` | `S` |
| element-card.glyph-current.pack_ref | DiscoveryBrowserCandidate.jsx:668#1 | `pack="motion-kit-alpha"` | `A` |
| element-card.glyph-current.preview_kind | DiscoveryBrowserCandidate.jsx:668#1 | `motion` → `data-preview="motion"` | `D` |
| element-card.glyph-current.tag_refs | DiscoveryBrowserCandidate.jsx:668#1 | animated | `A` |

### §4.11 `ElementCard` instance — `type-pulse`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.type-pulse.item_id | DiscoveryBrowserCandidate.jsx:669#1 | `element="type-pulse"` / `itemId` type-pulse（PluginCard と衝突） | `A` |
| element-card.type-pulse.display_name | DiscoveryBrowserCandidate.jsx:669#1 | `name="Type Pulse"` | `A` |
| element-card.type-pulse.type | DiscoveryBrowserCandidate.jsx:669#1 | `type="Text"` | `S` |
| element-card.type-pulse.provider_ref | DiscoveryBrowserCandidate.jsx:669#1 | `provider="Motion Kit"` | `A` |
| element-card.type-pulse.glyph | DiscoveryBrowserCandidate.jsx:669#1 | `glyph="T"` | `U` |
| element-card.type-pulse.thumbnail | DiscoveryBrowserCandidate.jsx:669#1 | `thumbnail="text"` | `S` |
| element-card.type-pulse.pack_ref | DiscoveryBrowserCandidate.jsx:669#1 | `pack="motion-kit-alpha"` | `A` |
| element-card.type-pulse.identity | DiscoveryBrowserCandidate.jsx:669#1 | `identity="motion-kit.type-pulse"` | `S` |
| element-card.type-pulse.impact | DiscoveryBrowserCandidate.jsx:669#1 | `impact="▱ + ◆ 12"` | `A` |
| element-card.type-pulse.preview_kind | DiscoveryBrowserCandidate.jsx:669#1 | `motion` → `data-preview="motion"` | `D` |
| element-card.type-pulse.tag_refs | DiscoveryBrowserCandidate.jsx:669#1 | brand-kit, animated | `A` |

### §4.12 `ElementCard` instance — `ribbon-array`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.ribbon-array.item_id | DiscoveryBrowserCandidate.jsx:670#1 | `element="ribbon-array"` / `itemId` ribbon-array | `A` |
| element-card.ribbon-array.display_name | DiscoveryBrowserCandidate.jsx:670#1 | `name="Ribbon Array"` | `A` |
| element-card.ribbon-array.type | DiscoveryBrowserCandidate.jsx:670#1 | `type="Generator"` | `S` |
| element-card.ribbon-array.provider_ref | DiscoveryBrowserCandidate.jsx:670#1 | `provider="Motion Kit"` | `A` |
| element-card.ribbon-array.glyph | DiscoveryBrowserCandidate.jsx:670#1 | `glyph="≋"` | `U` |
| element-card.ribbon-array.thumbnail | DiscoveryBrowserCandidate.jsx:670#1 | `thumbnail="ribbon"` | `S` |
| element-card.ribbon-array.pack_ref | DiscoveryBrowserCandidate.jsx:670#1 | `pack="motion-kit-alpha"` | `A` |
| element-card.ribbon-array.install_state | DiscoveryBrowserCandidate.jsx:670#1 | `state="Missing"` | `S` |
| element-card.ribbon-array.preview_kind | DiscoveryBrowserCandidate.jsx:670#1 | `motion` prop なし → `data-preview="poster"` | `D` |
| element-card.ribbon-array.tag_refs | DiscoveryBrowserCandidate.jsx:670#1 | prototype | `A` |

### §4.13 `ElementCard` instance — `particle-field`

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| element-card.particle-field.item_id | DiscoveryBrowserCandidate.jsx:671#1 | `element="particle-field"` / `itemId` particle-field | `A` |
| element-card.particle-field.display_name | DiscoveryBrowserCandidate.jsx:671#1 | `name="Particle Field"` | `A` |
| element-card.particle-field.type | DiscoveryBrowserCandidate.jsx:671#1 | `type="Generator"` | `S` |
| element-card.particle-field.provider_ref | DiscoveryBrowserCandidate.jsx:671#1 | `provider="Orbit Forge"` | `A` |
| element-card.particle-field.glyph | DiscoveryBrowserCandidate.jsx:671#1 | `glyph="✣"` | `U` |
| element-card.particle-field.thumbnail | DiscoveryBrowserCandidate.jsx:671#1 | `thumbnail="particles"` | `S` |
| element-card.particle-field.preview_kind | DiscoveryBrowserCandidate.jsx:671#1 | `motion` → `data-preview="motion"` | `D` |
| element-card.particle-field.tag_refs | DiscoveryBrowserCandidate.jsx:671#1 | animated | `A` |

## §5 catalog item identity
- 1件の識別子は `identity = (scope_ref, item_id)` とする。
- `scope_ref` は本粒で固定せず opaque に保つ（`plugin`/`element`などは未決）。
- `type-pulse` は `PluginCard`（`DiscoveryBrowserCandidate.jsx:401#1`）と `ElementCard`（`DiscoveryBrowserCandidate.jsx:669#1`）で同一 `itemId=type-pulse` を使用しているため、bare `itemId` は同一性を閉じられない。
- `scope_ref`/`item_id` 以外から（`display_name`/`thumbnail`/`labels`/配列index）意味を導くことは禁止（移管契約 §7）。
- 既存 drag payload は `itemId` のみを運ぶ既知の open defect として記録し、DOM/interaction 変更は CU-0A08B 以降で対応する。

## §6 受理入力契約
- Top-level: `catalog_revision` / `vocabularies` / `catalogs` の厳密3キー。
- `catalog_revision` は CU-0A08B decoder 定数 `CATALOG_REVISION = 1` と厳密一致する safe integer のみ受理する。
- `catalogs[]`: `{ scope_ref, items }`
- `items[]` 各要素は厳密に次の9キー: `item_id`, `display_name`, `taxonomy_refs`, `provider_ref`, `pack_ref`, `install_state_ref`, `preview_kind`, `impact`, `tag_refs`（欠落は `null`、key 省略不可）。
- `impact`: `{ measures: [{ amount, unit_ref }] }`（`measures` 空配列可。各 measure は `amount` / `unit_ref` 必須）。
- `preview_kind`: `motion` | `poster` のみ（閉enum）。
- `vocabularies` は同一 snapshot 内宣言表として次の7キー必須: `scopes`, `taxonomies`, `providers`, `packs`, `install_states`, `impact_units`, `tags`。
- 各語彙 entry は `{ id, label, scope_ref? }`（`scope_ref` は scoped 語彙のみ、非 scoped は `null`）。
- decoder は id / label 比較・解析・意味推定を行わず、typed reference 解決のみを行う。
- 件数上限（超過は拒否、切詰めなし）: scopes 16 / catalogs 16 / items 4096/scope / 各語彙表 4096 / taxonomy_refs 2/item / tag_refs 64/item / impact measures 16/item / ID 128 UTF-8 bytes / label 1024 UTF-8 bytes。

### §6a CU-0A08IS 禁止 output-key の引き継ぎ

CU-0A08IS `docs/reviews/2026-07-26-cu-0a08is-inspector-read-model-inventory.md` §6a の **decoder 出力禁止 key 閉集合**を本 Browser catalog projection の decoder 出力にもそのまま適用する。とくに literal key `availability` と `availability_lifecycle` は出力してはならない。

- Inspector read-model の lifecycle / availability **意味は再定義しない**（CU-0A08IS 正本のまま）。
- Browser 側の install / block / missing 可視振る舞いは、禁止 key ではなく別責任の `install_state_ref`（§6 carrier）で参照する。`install_state_ref` は Inspector lifecycle フィールドの別名・写像・再解釈ではない。

## §7 拒否規則
| ID | 拒否ファミリー | 不合格例 |
|---|---|---|
| B1 | revision不一致 | `catalog_revision: "1"`（非整数）; `catalog_revision: 2`（整数だが decoder 定数 `CATALOG_REVISION = 1` と不等） |
| B2 | 3キー欠落/過剰 | top-levelに `catalogs` と `items` を混在 |
| B3 | 位置外unknown key / §6a禁止key | `items[0]` に `availability` を追加（CU-0A08IS §6a 禁止語彙） |
| B4 | preview enum違反 | `preview_kind: "static"` |
| B5 | 重複ID | catalog内で同一 `item_id` |
| B6 | dangling参照 | `provider_ref: "x"` がvocabulary未登録 |
| B7 | cross-scope参照 | `taxonomy_refs` が別scope_refを指す |
| B8 | 語彙entry不備 | `{ id: null }` / `label` 非string |
| B9 | 非finite型 | 文字列やNaN値を数値欄へ投入 |
| B10 | 上限越え | `tag_refs` が65件 |
| B11 | optional省略 | `provider_ref` を省いた |
| B12 | fallback推定 | `display_name` 空時に `"Unknown"` を補完 |

## §8 fixtureと将来decoderが共有するoracle
- `§4`/`§6`/`§7` のみが `DiscoveryBrowserCandidate.jsx` fixture投影とCU-0A08B decoderの単一oracle。
- fixture専用の緩和・decoder専用の追加解釈を禁止する。

## §9 非目標とSTOP
- 非目標: CU-0A08B実装、Host transport、typed intent、JSX binding、Rust/schema/plugin/community契約変更、Document/serde、Undo/selection、Media Browser、Browser tab分類P41、CU-109/CU-104/CU-0A08IT等。
- STOP条件は `STOP` と同内容。特に `S` の意味決定、`S`値の追加、value-setの同一snapshot外再定義、ID推定、default補完を要する場合はここで停止する。

## §10 未決（S）一覧
- `provider` / `pack` / `install_states` 語彙の内容・発行者・cross-snapshot 寿命は `S`（authority: community `protocol/schema/UI`、decision-index L24）。ただし[CU-205S](2026-07-31-cu-205s-opacity-direct-route-split-decision.md#4-cu-205b-browser-source)は、通常製品のprivate first-party snapshot一件に限り、`first-party-effects` scope内の`built-in` / `installed`を同一snapshot宣言語彙として固定した。これはcommunity語彙、外部発行者、cross-snapshot既定を解決しない。
- `taxonomy_refs` / `provider_ref` / `pack_ref` / `install_state_ref` / `impact.measures[].unit_ref` / `tag_refs` は §6 の `A` carrier のみ。語彙 ID の意味集合は snapshot 内宣言に委ね、cross-snapshot 既定は書かない。
- `mode` / `state` / `category` / `subtype` / `kind` / `type` / `provider` 可視文字列 / `identity` / `thumbnail` class token の意味は `S`（`install_state_ref` は carrier のみで振る舞いは未決）。CU-205Sの`Effect` / `Color` / `Built-in` / `Installed`はOpacity一件のprivate snapshot vocabulary labelであり、これらの一般可視意味、icon、thumbnail、availability振る舞いを解決しない。
- tag assignment の永続 owner は `S`（`useCandidateTags` mock-local）。
- Browser tab / scope 名固定 / provider 由来の primary navigation（P41 未統一）は `S`。
- drag payload open defect は `S`（`application/x-motolii-browser-item` は現状 bare `itemId` のみ）。CU-205SはOpacity正常routeで`(scope_ref, item_id)` carrierへの交換だけを決定し、drag/drop Commit targetとDocument writeは未決のまま0とする。
- `impact` の可視書式（`◆ 12 KEYS`、`▱ + ◆ 12` 等）と glyph/unit 順は `S`。
- `folder` / `labels` / `data-search` は §4 で `D` と分類済み。decoder 重複 field としての意味決定は不要（未決一覧には載せない）。
