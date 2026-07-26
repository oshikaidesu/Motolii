# CU-0A08IS — Inspector read-model inventory

- 日付: 2026-07-26
- 状態: **決定**
- 粒: **CU-0A08IS DONE** → 後続 **CU-0A08IP READY-RECHECK**

## §1 FACTS

- `ui/motolii-web/src/candidates/InspectorCandidate.jsx`（791行、SHA256 `1e0bdd3eebd665e517600af4db090f74d50951aef12fdd476e97a828de91a3e4`）は5つの相互排他branchを描画する: `installed && effectFocused`（L353）、`installed`（L411）、`discover`（L630）、`blocked`（L681）、default missing（L730）。
- `panelHead` の `Inspector` literal は `return (` 外の **L351** にあり、§9 走査対象に含める。
- `TrackItem` は内部タグ `kind` で `clip` / `group` を区別する（`schema.rs:193-196`）。`DocParam` だけが snake_case 外部タグ（`param.rs:25-44`）。
- `BlendMode` は `normal` / `add` / `multiply` のみ（`schema.rs:442-447`）。mock の `Screen` は現行Documentで表現不能。
- `docs/mocks-ui/fixtures/reference-document.json` は現行Documentの参照fixture。dangling 判定の layer 集合は `layers.entries`。
- fail-closed 先例は `docs/mocks-ui/src/reference/loadReferenceFixtures.js` の `requireObject` / `requireFinite` / dangling 拒否。

## §2 分類語彙

| 記号 | 意味 |
|---|---|
| `A` | 直接field読み出し |
| `D` | 決定的導出（既決fieldから追加意味なしに一意計算） |
| `U` | 表示専用chrome（decoder入力を要さない） |
| `S` | 未決（CU-0A08IPでは表示しない） |

明示的是正:

- 可視の数値 `Const` 表示は `D` であり `A` ではない。
- mock単一scalarの Depth Z / Scale 表示は現行Document意味で一致しないため `S`。
- mock-local automation mark / AUTO 文言は Document 由来binding未選択のため `S`（dial / chrome class は `U`）。
- mock単一scalarの Depth Z / Scale 表示は現行Document意味で一致しないため `S`。
- mock-local automation mark / AUTO 文言は Document 由来binding未選択のため `S`（dial / chrome class は `U`）。
- `A` / `S` の複合セルを作らない。未決評価を要する要素は §3 で `S` 一文字とする。

§3 anchor規則: `JSX位置` は `InspectorCandidate.jsx:<line>#<occ>`（`#` 省略時は `1`）。可視表現は AST 抽出レコードの `text` と一致させ、検証は canonical key `kind|line|text#occ` の extraction index 所属のみ（生行・regex・includes 禁止）。lifecycle chrome ラベルは `U`、lifecycle / availability 状態値（`NONE` 各 occurrence、`AVAILABLE AFTER ADD`、`NOT STARTED`、`REFUSED`、`RETAINED`、`SUCCEEDED`、`AVAILABLE`）は `S`。

## §3 可視要素インベントリ

### §3.1 installed-effect-focused

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| installed-effect-focused.intensity-dial | InspectorCandidate.jsx:98 | scrub-dial | U |
| installed-effect-focused.spread-dial | InspectorCandidate.jsx:98 | scrub-dial | U |
| installed-effect-focused.intensity-output | InspectorCandidate.jsx:99 | ${value}% | D |
| installed-effect-focused.spread-output | InspectorCandidate.jsx:99 | ${value}% | D |
| installed-effect-focused.intensity-automation | InspectorCandidate.jsx:119 | automation-mark | S |
| installed-effect-focused.spread-automation | InspectorCandidate.jsx:119 | automation-mark | S |
| installed-effect-focused.scrub-control-intensity | InspectorCandidate.jsx:130 | ScrubControl | U |
| installed-effect-focused.scrub-control-spread | InspectorCandidate.jsx:130 | ScrubControl | U |
| installed-effect-focused.intensity-hint | InspectorCandidate.jsx:132 | AUTO ON / AUTO OFF | S |
| installed-effect-focused.spread-hint | InspectorCandidate.jsx:132 | AUTO ON / AUTO OFF | S |
| installed-effect-focused.devinfo-summary | InspectorCandidate.jsx:163 | Developer info | U |
| installed-effect-focused.devinfo-package-label | InspectorCandidate.jsx:165 | Package | U |
| installed-effect-focused.devinfo-package-value | InspectorCandidate.jsx:166 | Vism (.vism) | S |
| installed-effect-focused.devinfo-package-third | InspectorCandidate.jsx:167 | (empty span) | U |
| installed-effect-focused.devinfo-identity-label | InspectorCandidate.jsx:170 | Identity | U |
| installed-effect-focused.devinfo-identity-value | InspectorCandidate.jsx:171 | demo.echo-bloom | A |
| installed-effect-focused.devinfo-identity-third | InspectorCandidate.jsx:172 | (empty span) | U |
| installed-effect-focused.chrome-panel-head | InspectorCandidate.jsx:351 | Inspector | U |
| installed-effect-focused.chrome-aside | InspectorCandidate.jsx:355 | aside.inspector#inspector | U |
| installed-effect-focused.section-editing-title | InspectorCandidate.jsx:359 | EDITING EFFECT | S |
| installed-effect-focused.section-on-object | InspectorCandidate.jsx:359 | ON OBJECT | U |
| installed-effect-focused.identity-icon | InspectorCandidate.jsx:362 | ◎ | U |
| installed-effect-focused.identity-name | InspectorCandidate.jsx:364 | Echo Bloom | D |
| installed-effect-focused.identity-subtitle | InspectorCandidate.jsx:365 | Pulse rings · Effect | S |
| installed-effect-focused.host-panel-span | InspectorCandidate.jsx:371 | HOST PANEL | U |
| installed-effect-focused.host-title | InspectorCandidate.jsx:371 | ECHO BLOOM | U |
| installed-effect-focused.effect-description | InspectorCandidate.jsx:374 | Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage. | S |
| installed-effect-focused.effect-scrub-intensity-label | InspectorCandidate.jsx:384 | Intensity | U |
| installed-effect-focused.effect-scrub-spread-label | InspectorCandidate.jsx:393 | Spread | U |
| installed-effect-focused.input-label | InspectorCandidate.jsx:378 | Input | U |
| installed-effect-focused.input-value | InspectorCandidate.jsx:379 | Pulse rings composite | S |
| installed-effect-focused.input-tag | InspectorCandidate.jsx:380 | TEXTURE | S |
| installed-effect-focused.effect-scrub-intensity | InspectorCandidate.jsx:382 | EffectScrubRow | U |
| installed-effect-focused.effect-scrub-spread | InspectorCandidate.jsx:391 | EffectScrubRow | U |
| installed-effect-focused.blend-label | InspectorCandidate.jsx:401 | Blend | U |
| installed-effect-focused.blend-value | InspectorCandidate.jsx:402 | Screen | S |
| installed-effect-focused.blend-third-cell | InspectorCandidate.jsx:403 | (empty span) | U |
| installed-effect-focused.devinfo | InspectorCandidate.jsx:406 | DevInfoEffectFocused | U |

### §3.2 installed

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| installed.object-hint-auto | InspectorCandidate.jsx:39 | AUTO ON / AUTO OFF | S |
| installed.echo-intensity-dial | InspectorCandidate.jsx:98 | scrub-dial | U |
| installed.echo-spread-dial | InspectorCandidate.jsx:98 | scrub-dial | U |
| installed.echo-intensity-output | InspectorCandidate.jsx:99 | ${value}% | D |
| installed.echo-spread-output | InspectorCandidate.jsx:99 | ${value}% | D |
| installed.echo-intensity-automation | InspectorCandidate.jsx:119 | automation-mark | S |
| installed.echo-spread-automation | InspectorCandidate.jsx:119 | automation-mark | S |
| installed.echo-scrub-control-intensity | InspectorCandidate.jsx:130 | ScrubControl | U |
| installed.echo-scrub-control-spread | InspectorCandidate.jsx:130 | ScrubControl | U |
| installed.echo-intensity-hint | InspectorCandidate.jsx:132 | AUTO ON / AUTO OFF | S |
| installed.echo-spread-hint | InspectorCandidate.jsx:132 | AUTO ON / AUTO OFF | S |
| installed.devinfo-summary | InspectorCandidate.jsx:141 | Developer info | U |
| installed.devinfo-package-label | InspectorCandidate.jsx:143 | Package | U |
| installed.devinfo-package-value | InspectorCandidate.jsx:144 | Vism (.vism) | S |
| installed.devinfo-package-third | InspectorCandidate.jsx:145 | (empty span) | U |
| installed.devinfo-identity-label | InspectorCandidate.jsx:148 | Identity | U |
| installed.devinfo-identity-value | InspectorCandidate.jsx:149 | demo.echo-bloom | A |
| installed.devinfo-identity-third | InspectorCandidate.jsx:150 | (empty span) | U |
| installed.devinfo-lifecycle-project | InspectorCandidate.jsx:153 | PROJECT | U |
| installed.devinfo-lifecycle-same | InspectorCandidate.jsx:153 | SAME EVALUATION | U |
| installed.devinfo-lifecycle-undo | InspectorCandidate.jsx:153 | Undo / Save | U |
| installed.lifecycle-preview | InspectorCandidate.jsx:153 | Preview / Export | U |
| installed.devinfo-lifecycle-host | InspectorCandidate.jsx:154 | HOST | U |
| installed.lifecycle-cache | InspectorCandidate.jsx:154 | Cache / Resource | U |
| installed.chrome-panel-head | InspectorCandidate.jsx:351 | Inspector | U |
| installed.depth-automation-mark | InspectorCandidate.jsx:419 | automation-mark | S |
| installed.opacity-automation-mark | InspectorCandidate.jsx:419 | automation-mark | S |
| installed.position-automation | InspectorCandidate.jsx:419 | automation-mark | S |
| installed.rotation-automation-mark | InspectorCandidate.jsx:419 | automation-mark | S |
| installed.scale-automation-mark | InspectorCandidate.jsx:419 | automation-mark | S |
| installed.depth-object-hint | InspectorCandidate.jsx:431 | ObjectAutoHint | U |
| installed.opacity-object-hint | InspectorCandidate.jsx:431 | ObjectAutoHint | U |
| installed.position-object-hint | InspectorCandidate.jsx:431 | ObjectAutoHint | U |
| installed.rotation-object-hint | InspectorCandidate.jsx:431 | ObjectAutoHint | U |
| installed.scale-object-hint | InspectorCandidate.jsx:431 | ObjectAutoHint | U |
| installed.chrome-aside | InspectorCandidate.jsx:437 | aside.inspector#inspector | U |
| installed.section-selected-subtitle | InspectorCandidate.jsx:441 | (empty span) | U |
| installed.section-selected-title | InspectorCandidate.jsx:441 | SELECTED OBJECT | S |
| installed.identity-icon | InspectorCandidate.jsx:444 | G | U |
| installed.identity-name | InspectorCandidate.jsx:446 | Pulse rings | A |
| installed.identity-kind-child | InspectorCandidate.jsx:447 | Group · 1 child | D |
| installed.transform-object-span | InspectorCandidate.jsx:453 | OBJECT | U |
| installed.transform-title | InspectorCandidate.jsx:453 | TRANSFORM | U |
| installed.position-label | InspectorCandidate.jsx:457 | Position | U |
| installed.position-x-label | InspectorCandidate.jsx:460 | X | U |
| installed.position-x-value | InspectorCandidate.jsx:460 | 0.124 | D |
| installed.position-y-label | InspectorCandidate.jsx:463 | Y | U |
| installed.position-y-value | InspectorCandidate.jsx:463 | −0.082 | D |
| installed.position-keys | InspectorCandidate.jsx:466 | 2 KEYS | D |
| installed.depth-label | InspectorCandidate.jsx:470 | Depth Z | U |
| installed.depth-value | InspectorCandidate.jsx:471 | 0.180 | S |
| installed.depth-keys | InspectorCandidate.jsx:472 | 1 KEY | D |
| installed.depth-at-key | InspectorCandidate.jsx:473 | at-key | U |
| installed.scale-label | InspectorCandidate.jsx:477 | Scale | U |
| installed.scale-value | InspectorCandidate.jsx:478 | 1.000 | S |
| installed.scale-keys-closure | InspectorCandidate.jsx:479 | (no KEYS literal) | U |
| installed.rotation-label | InspectorCandidate.jsx:483 | Rotation Z | U |
| installed.rotation-value | InspectorCandidate.jsx:484 | 0.000 rad | D |
| installed.rotation-keys-closure | InspectorCandidate.jsx:485 | (no KEYS literal) | U |
| installed.opacity-label | InspectorCandidate.jsx:489 | Opacity | U |
| installed.opacity-value | InspectorCandidate.jsx:490 | 100% | D |
| installed.opacity-keys | InspectorCandidate.jsx:491 | 2 KEYS | D |
| installed.appearance-object-span | InspectorCandidate.jsx:496 | OBJECT | U |
| installed.appearance-title | InspectorCandidate.jsx:496 | APPEARANCE | U |
| installed.fill-label | InspectorCandidate.jsx:499 | Fill | U |
| installed.fill-chip | InspectorCandidate.jsx:501 | color-chip | S |
| installed.fill-tag | InspectorCandidate.jsx:514 | COLOR | U |
| installed.stroke-label | InspectorCandidate.jsx:517 | Stroke | U |
| installed.stroke-chip | InspectorCandidate.jsx:519 | color-chip | S |
| installed.stroke-tag | InspectorCandidate.jsx:532 | COLOR | U |
| installed.group-edit-space | InspectorCandidate.jsx:537 | EDIT SPACE | U |
| installed.group-title | InspectorCandidate.jsx:537 | GROUP COMPOSITION | U |
| installed.z-label | InspectorCandidate.jsx:540 | Z Occlusion | S |
| installed.z-off | InspectorCandidate.jsx:542 | OFF / Stack | S |
| installed.z-on | InspectorCandidate.jsx:543 | ON / Group Z | S |
| installed.z-tag | InspectorCandidate.jsx:545 | Z | U |
| installed.composite-label | InspectorCandidate.jsx:548 | Composite | U |
| installed.composite-value | InspectorCandidate.jsx:549 | Child → Group bake point | U |
| installed.composite-third | InspectorCandidate.jsx:550 | (empty span) | U |
| installed.link-label | InspectorCandidate.jsx:553 | Link | U |
| installed.link-value | InspectorCandidate.jsx:560 | Position → target | S |
| installed.link-tag | InspectorCandidate.jsx:562 | TYPED | U |
| installed.driver-routes-span | InspectorCandidate.jsx:567 | 2 ROUTES | U |
| installed.driver-title | InspectorCandidate.jsx:567 | DRIVER | U |
| installed.driver-label | InspectorCandidate.jsx:570 | Audio Low | S |
| installed.driver-svg | InspectorCandidate.jsx:571 | driver-mini | U |
| installed.driver-tag | InspectorCandidate.jsx:576 | LIVE | S |
| installed.plugins-add | InspectorCandidate.jsx:581 | ＋ | S |
| installed.plugins-title | InspectorCandidate.jsx:581 | APPLIED PLUGINS | U |
| installed.plugin-grip | InspectorCandidate.jsx:584 | :: | U |
| installed.plugin-mini | InspectorCandidate.jsx:585 | ◎ | U |
| installed.plugin-name | InspectorCandidate.jsx:587 | Echo Bloom | D |
| installed.plugin-sub | InspectorCandidate.jsx:588 | IN → Effect → OUT · selected | S |
| installed.echo-title | InspectorCandidate.jsx:594 | ECHO BLOOM | U |
| installed.echo-host-panel-span | InspectorCandidate.jsx:594 | HOST PANEL | U |
| installed.echo-input-label | InspectorCandidate.jsx:597 | Input | U |
| installed.echo-input-value | InspectorCandidate.jsx:598 | Pulse rings composite | S |
| installed.echo-input-tag | InspectorCandidate.jsx:599 | TEXTURE | S |
| installed.echo-scrub-intensity | InspectorCandidate.jsx:601 | EffectScrubRow | U |
| installed.echo-scrub-intensity-label | InspectorCandidate.jsx:603 | Intensity | U |
| installed.echo-scrub-spread | InspectorCandidate.jsx:610 | EffectScrubRow | U |
| installed.echo-scrub-spread-label | InspectorCandidate.jsx:612 | Spread | U |
| installed.echo-blend-label | InspectorCandidate.jsx:620 | Blend | U |
| installed.echo-blend-value | InspectorCandidate.jsx:621 | Screen | S |
| installed.echo-blend-third | InspectorCandidate.jsx:622 | (empty span) | U |
| installed.devinfo | InspectorCandidate.jsx:625 | DevInfoInstalled | U |

### §3.3 discover

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| discover.devinfo-summary | InspectorCandidate.jsx:181 | Developer info | U |
| discover.devinfo-package-label | InspectorCandidate.jsx:183 | Package | U |
| discover.devinfo-package-value | InspectorCandidate.jsx:184 | Vism (.vism) | S |
| discover.devinfo-package-third | InspectorCandidate.jsx:185 | (empty span) | U |
| discover.devinfo-identity-label | InspectorCandidate.jsx:188 | Identity | U |
| discover.devinfo-identity-value | InspectorCandidate.jsx:189 | demo.glyph-current | A |
| discover.devinfo-identity-third | InspectorCandidate.jsx:190 | (empty span) | U |
| discover.lifecycle-project | InspectorCandidate.jsx:193 | Project change | U |
| discover.lifecycle-project-none | InspectorCandidate.jsx:193#1 | NONE | S |
| discover.lifecycle-code | InspectorCandidate.jsx:193 | Code execution | U |
| discover.lifecycle-code-none | InspectorCandidate.jsx:193#2 | NONE | S |
| discover.lifecycle-standard-panel | InspectorCandidate.jsx:193 | Standard panel | U |
| discover.lifecycle-standard-after | InspectorCandidate.jsx:194 | AVAILABLE AFTER ADD | S |
| discover.chrome-panel-head | InspectorCandidate.jsx:351 | Inspector | U |
| discover.chrome-aside | InspectorCandidate.jsx:632 | aside.inspector#inspector | U |
| discover.not-in-project-span | InspectorCandidate.jsx:636 | NOT IN PROJECT | U |
| discover.title | InspectorCandidate.jsx:636 | DISCOVERY | U |
| discover.icon | InspectorCandidate.jsx:639 | 字 | U |
| discover.name | InspectorCandidate.jsx:641 | Glyph Current | A |
| discover.sub | InspectorCandidate.jsx:642 | Generator plugin · flowing type | S |
| discover.preview-btn | InspectorCandidate.jsx:653 | Preview | S |
| discover.add-btn | InspectorCandidate.jsx:672 | Add to selected object | S |
| discover.devinfo | InspectorCandidate.jsx:676 | DevInfoDiscover | U |

### §3.4 blocked

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| blocked.devinfo-summary | InspectorCandidate.jsx:203 | Developer info | U |
| blocked.devinfo-package-label | InspectorCandidate.jsx:205 | Package | U |
| blocked.devinfo-package-value | InspectorCandidate.jsx:206 | Vism (.vism) | S |
| blocked.devinfo-package-third | InspectorCandidate.jsx:207 | (empty span) | U |
| blocked.devinfo-identity-label | InspectorCandidate.jsx:210 | Identity | U |
| blocked.devinfo-identity-value | InspectorCandidate.jsx:211 | demo.fold-field | A |
| blocked.devinfo-identity-third | InspectorCandidate.jsx:212 | (empty span) | U |
| blocked.chrome-panel-head | InspectorCandidate.jsx:351 | Inspector | U |
| blocked.chrome-aside | InspectorCandidate.jsx:683 | aside.inspector#inspector | U |
| blocked.title | InspectorCandidate.jsx:687 | DISCOVERY | U |
| blocked.subtitle | InspectorCandidate.jsx:688 | UNAVAILABLE | S |
| blocked.icon | InspectorCandidate.jsx:691 | ◇ | U |
| blocked.name | InspectorCandidate.jsx:693 | Fold Field | A |
| blocked.sub | InspectorCandidate.jsx:694 | Effect plugin · local file | S |
| blocked.notice-title | InspectorCandidate.jsx:698 | このHostでは評価できません | S |
| blocked.notice-body | InspectorCandidate.jsx:700 | 要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。 | S |
| blocked.lifecycle-install | InspectorCandidate.jsx:703 | Install | U |
| blocked.lifecycle-none | InspectorCandidate.jsx:703 | NONE | S |
| blocked.lifecycle-not-started | InspectorCandidate.jsx:703 | NOT STARTED | S |
| blocked.lifecycle-project | InspectorCandidate.jsx:703 | Project change | U |
| blocked.lifecycle-fallback | InspectorCandidate.jsx:704 | Fallback | U |
| blocked.lifecycle-refused | InspectorCandidate.jsx:704 | REFUSED | S |
| blocked.inspect-btn | InspectorCandidate.jsx:719 | Inspect reason | S |
| blocked.add-btn | InspectorCandidate.jsx:722 | Add | S |
| blocked.devinfo | InspectorCandidate.jsx:725 | DevInfoBlocked | U |

### §3.5 missing

| 要素ID | JSX位置 | 可視表現 | 分類 |
| --- | --- | --- | --- |
| missing.devinfo-summary | InspectorCandidate.jsx:221 | Developer info | U |
| missing.devinfo-package-label | InspectorCandidate.jsx:223 | Package | U |
| missing.devinfo-package-value | InspectorCandidate.jsx:224 | Vism (.vism) | S |
| missing.devinfo-package-third | InspectorCandidate.jsx:225 | (empty span) | U |
| missing.devinfo-identity-label | InspectorCandidate.jsx:228 | Identity | U |
| missing.devinfo-identity-value | InspectorCandidate.jsx:229 | demo.ribbon-array | A |
| missing.devinfo-identity-third | InspectorCandidate.jsx:230 | (empty span) | U |
| missing.chrome-panel-head | InspectorCandidate.jsx:351 | Inspector | U |
| missing.chrome-aside | InspectorCandidate.jsx:731 | aside.inspector#inspector | U |
| missing.title | InspectorCandidate.jsx:735 | PROJECT INSTANCE | S |
| missing.subtitle | InspectorCandidate.jsx:736 | MISSING | S |
| missing.icon | InspectorCandidate.jsx:743 | ? | U |
| missing.name | InspectorCandidate.jsx:746 | Ribbon Array | A |
| missing.sub | InspectorCandidate.jsx:747 | Plugin unavailable · Project instance retained | S |
| missing.notice-title | InspectorCandidate.jsx:751 | 必要なプラグインを評価できません | S |
| missing.notice-body | InspectorCandidate.jsx:753 | identity、version要求、instance payloadを保持しています。欠落中はpayloadを解釈して似た設定へ変換しません。 | S |
| missing.lifecycle-available | InspectorCandidate.jsx:756 | AVAILABLE | S |
| missing.lifecycle-open | InspectorCandidate.jsx:756 | Project open | U |
| missing.lifecycle-succeeded | InspectorCandidate.jsx:756 | SUCCEEDED | S |
| missing.lifecycle-unrelated | InspectorCandidate.jsx:756 | Unrelated edit | U |
| missing.lifecycle-export | InspectorCandidate.jsx:757 | Required export | U |
| missing.lifecycle-export-refused | InspectorCandidate.jsx:757 | REFUSED | S |
| missing.lifecycle-payload | InspectorCandidate.jsx:757 | Payload | U |
| missing.lifecycle-retained | InspectorCandidate.jsx:757 | RETAINED | S |
| missing.review-btn | InspectorCandidate.jsx:769 | Review recovery | S |
| missing.std-editable-span | InspectorCandidate.jsx:775 | EDITABLE | U |
| missing.std-title | InspectorCandidate.jsx:775 | STANDARD TRANSFORM | U |
| missing.std-position-label | InspectorCandidate.jsx:778 | Position | U |
| missing.std-position-value | InspectorCandidate.jsx:779 | X 0.00 · Y 0.00 | S |
| missing.std-position-third | InspectorCandidate.jsx:780 | (empty span) | U |
| missing.std-scale-label | InspectorCandidate.jsx:783 | Scale | U |
| missing.std-scale-value | InspectorCandidate.jsx:784 | 100% | S |
| missing.std-scale-third | InspectorCandidate.jsx:785 | (empty span) | U |
| missing.devinfo | InspectorCandidate.jsx:788 | DevInfoMissing | U |

## §4 source capability候補

§4 冒頭: `採用` と書いた候補は、対応 §3 要素IDが実在し分類が `A` または `D` でなければならない。

| 候補ID | 現行source型・path・行 | 到達可能な値 | 対応する §3 要素ID | 採否 |
|---|---|---|---|---|
| CAP-LAYER-NAME | LayerIdTable.entries · crates/motolii-doc/src/ids.rs | 表示名文字列 | installed.identity-name, missing.name, discover.name, blocked.name | 採用 |
| CAP-GROUP-CHILD | TrackItem.kind · crates/motolii-doc/src/schema.rs:193-196; Group.children · schema.rs:782-785 | Group/Clip 種別語 + children.len() | installed.identity-kind-child | 採用 |
| CAP-PLUGIN-ID | EffectDefinition.plugin_id · schema.rs | plugin-id文字列 | installed.devinfo-identity-value, installed-effect-focused.devinfo-identity-value | 採用 |
| CAP-PARAM-DEF | ParamDef · motolii-plugin/src/lib.rs | id, value_type, default, f64_domain | installed-effect-focused.intensity-output, installed.echo-intensity-output | 採用 |
| CAP-BLEND-MODE | ItemEnvelope.blend · schema.rs:442 | normal/add/multiply | installed.echo-blend-value | 未採用 |
| CAP-FILL-STROKE | なし | — | installed.fill-chip | 未採用 |

## §5 CU-0A08IP 受理入力契約

### §5.1 受理wrapper

top-level object は **ちょうど4キー**: `fixtureRevision`（整数、decoder定数と等値）、`document`（現行Document JSON）、`nodes`（NodeDesc 由来配列、id は plugin-id 語彙）、`target`（`{ "layer_id": <u64> }`）。`screen` / `mode` / `scenes` / `tokens` は受理しない。

### §5.2 wrapperとDocument serdeの層分離

wrapper層は厳密4キー。document層は現行serdeに従い、flatten 位置では未知fieldを保持し得る。CU-0A08IPは汎用Document deserializerを名乗らない。

### §5.3 decoderが読んでよい範囲

§3 で `A` / `D` の要素に紐づき §4 で `採用` の候補のみ。成果名は `fixture由来read-only projection decoder`。

## §6 拒否規則 R1〜R14

| ID | 拒否対象 |
|---|---|
| R1 | wrapper top-level に §5.1 の4キー以外のkeyがある |
| R2 | wrapper top-level に4キーのいずれかが欠落している |
| R3 | fixtureRevision が非整数、または decoder 定数と不等 |
| R4 | object を期待する位置が null / 配列 / 非object（requireObject 相当） |
| R5 | 数値を期待する位置が非number または非有限（NaN / ±Infinity。requireFinite 相当） |
| R6 | 現行typed variantの未知タグ: DocParam 外部タグが const/keyframes/data/vec2_axes/look_at/follow 以外、LookAtAxis が plus_y/plus_x 以外、TrackItem.kind が clip/group 以外、ParamDef.value_type が F64/Vec2/Vec3/Color/AssetRef 以外 |
| R7 | dangling参照: target.layer_id が document.layers.entries に無い／effects[].definition_id に対応する effect_definitions[].id が無い／LookAt.target・Follow.target が layers.entries に無い |
| R8 | ID重複: nodes[].id の重複、または同一node内 params[].id の重複 |
| R9 | plugin ID未解決: effect_definitions[].plugin_id と同じ文字列を持つ nodes[].id entry が無い |
| R10 | blend が normal/add/multiply 以外、opacity の Const(F64) が [0,1] 外、非F64 parameter への f64_domain 付与、f64_domain の各boundが非有限、min_inclusive > max_inclusive、default が bounds 外 |
| R11 | default発明: 欠落値のdefault補完、serde defaultの新設、単位・書式の推定補完 |
| R12 | semantic write: React側からDocument意味を書き戻すこと |
| R13 | DOM変更: R4C由来のDOM / class / stable ID / ARIA / interaction / visual threshold / golden の変更 |
| R14 | decoder出力wrapperまたはnode拡張に §6a 禁止key語彙を置くこと（入力 document / DocParam serde key には適用しない） |

### §6a R14の適用範囲

R14は **decoder出力objectのkeyとnode拡張keyのみ**に適用する。`document` / `DocParam` / 既存serde面のkeyには適用しない。

禁止key語彙（閉集合）: `fill_color`, `stroke_color`, `z_occlusion`, `occlusion_mode`, `depth_z`, `bake_point`, `composite_bake`, `driver_route`, `driver_routes`, `applied_plugin_history`, `availability`, `availability_lifecycle`, `effect_description`, `input_socket_label`, `socket_type_tag`, `link_label`, `link_target_label`, `primary_selection`, `selected_object`, `editing_effect`, `at_key`。

R14 whitelist（交差してはならない）: `scale`, `blend`, `data`, `look_at`, `follow`, `params`, `extra`, `const`, `keyframes`, `vec2_axes`, `target`, `axis`, `offset`, `track`, `fallback`, `x`, `y`。さらに `docs/mocks-ui/fixtures/reference-document.json` の全keyとも交差してはならない。

R7 / R11 / R14 の適用面は互いに素であり、R14は現行serdeの未知field保持を上書きしない。

## §7 入力例

`document` メンバの実体は `docs/mocks-ui/fixtures/reference-document.json` をpath参照する。

非JSON・模式表記（そのまま入力しない）

```text
fixtureRevision : 整数（decoder定数と等値）
document        : docs/mocks-ui/fixtures/reference-document.json の現行Document object
nodes           : NodeDesc 由来 entry の配列（id は plugin-id 文字列語彙）
target          : { layer_id: u64 }
```

`{}` を受理・合格・accepted 例として掲載しない。

## §8 非目標・STOP

decoder実装、Host transport、typed intent、React/CSS/DOM、Document/plugin公開契約の変更はSTOP。`S` の意味を決める必要が出たらSTOP。

## §9 coverage manifest

抽出規則（Babel AST oracle）:

1. `@babel/parser` + `@babel/traverse` で JSX を1回解析する。
2. `JSXText` はノード全体を1レコードにする（`text = node.value.replace(/\s+/g," ").trim()`、`line = node.loc.start.line + leading whitespace 内の改行数`）。capitalized `JSXOpeningElement`、attribute-free `<span />`、`className` 文字列/テンプレート、可視 template/conditional、`objectRow` 第2引数label、第4引数 `keys-arg`、既知 component の `label` StringLiteral（`prop-label`、`aria-label` 除外）を multiset として記録する（重複除去しない）。canonical key は `kind|line|text#occ`（`occ` は同一 `(kind,line,text)` の走査順1-based）。

§9 sentinel: `PROJECT INSTANCE`（L735）、`Inspector`（L351）、`Vism (.vism)`（L144/L166/L184/L206/L224）。

| 可視literal / component | JSX行 | 対応要素ID |
|---|---|---|
| AUTO ON / AUTO OFF | 39 | installed.object-hint-auto |
| scrub-dial | 98 | installed-effect-focused.intensity-dial |
| ${value}% | 99 | installed-effect-focused.intensity-output |
| automation-mark | 119 | installed-effect-focused.intensity-automation |
| ScrubControl | 130 | installed-effect-focused.scrub-control-intensity |
| AUTO ON / AUTO OFF | 132 | installed-effect-focused.intensity-hint |
| Developer info | 141 | installed.devinfo-summary |
| Package | 143 | installed.devinfo-package-label |
| Vism (.vism) | 144 | installed.devinfo-package-value |
| (empty span) | 145 | installed.devinfo-package-third |
| Identity | 148 | installed.devinfo-identity-label |
| demo.echo-bloom | 149 | installed.devinfo-identity-value |
| (empty span) | 150 | installed.devinfo-identity-third |
| Preview / Export | 153 | installed.lifecycle-preview |
| SAME EVALUATION | 153 | installed.devinfo-lifecycle-same |
| Undo / Save | 153 | installed.devinfo-lifecycle-undo |
| PROJECT | 153 | installed.devinfo-lifecycle-project |
| Cache / Resource | 154 | installed.lifecycle-cache |
| HOST | 154 | installed.devinfo-lifecycle-host |
| Developer info | 163 | installed-effect-focused.devinfo-summary |
| Package | 165 | installed-effect-focused.devinfo-package-label |
| Vism (.vism) | 166 | installed-effect-focused.devinfo-package-value |
| (empty span) | 167 | installed-effect-focused.devinfo-package-third |
| Identity | 170 | installed-effect-focused.devinfo-identity-label |
| demo.echo-bloom | 171 | installed-effect-focused.devinfo-identity-value |
| (empty span) | 172 | installed-effect-focused.devinfo-identity-third |
| Developer info | 181 | discover.devinfo-summary |
| Package | 183 | discover.devinfo-package-label |
| Vism (.vism) | 184 | discover.devinfo-package-value |
| (empty span) | 185 | discover.devinfo-package-third |
| Identity | 188 | discover.devinfo-identity-label |
| demo.glyph-current | 189 | discover.devinfo-identity-value |
| (empty span) | 190 | discover.devinfo-identity-third |
| Project change | 193 | discover.lifecycle-project |
| NONE | 193#1 | discover.lifecycle-project-none |
| Code execution | 193 | discover.lifecycle-code |
| NONE | 193#2 | discover.lifecycle-code-none |
| Standard panel | 193 | discover.lifecycle-standard-panel |
| AVAILABLE AFTER ADD | 194 | discover.lifecycle-standard-after |
| Developer info | 203 | blocked.devinfo-summary |
| Package | 205 | blocked.devinfo-package-label |
| Vism (.vism) | 206 | blocked.devinfo-package-value |
| (empty span) | 207 | blocked.devinfo-package-third |
| Identity | 210 | blocked.devinfo-identity-label |
| demo.fold-field | 211 | blocked.devinfo-identity-value |
| (empty span) | 212 | blocked.devinfo-identity-third |
| Developer info | 221 | missing.devinfo-summary |
| Package | 223 | missing.devinfo-package-label |
| Vism (.vism) | 224 | missing.devinfo-package-value |
| (empty span) | 225 | missing.devinfo-package-third |
| Identity | 228 | missing.devinfo-identity-label |
| demo.ribbon-array | 229 | missing.devinfo-identity-value |
| (empty span) | 230 | missing.devinfo-identity-third |
| Inspector | 351 | installed-effect-focused.chrome-panel-head |
| aside.inspector#inspector | 355 | installed-effect-focused.chrome-aside |
| EDITING EFFECT | 359 | installed-effect-focused.section-editing-title |
| ON OBJECT | 359 | installed-effect-focused.section-on-object |
| ◎ | 362 | installed-effect-focused.identity-icon |
| Echo Bloom | 364 | installed-effect-focused.identity-name |
| Pulse rings · Effect | 365 | installed-effect-focused.identity-subtitle |
| ECHO BLOOM | 371 | installed-effect-focused.host-title |
| HOST PANEL | 371 | installed-effect-focused.host-panel-span |
| Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage. | 374 | installed-effect-focused.effect-description |
| Input | 378 | installed-effect-focused.input-label |
| Pulse rings composite | 379 | installed-effect-focused.input-value |
| TEXTURE | 380 | installed-effect-focused.input-tag |
| EffectScrubRow | 382 | installed-effect-focused.effect-scrub-intensity |
| Intensity | 384 | installed-effect-focused.effect-scrub-intensity-label |
| EffectScrubRow | 391 | installed-effect-focused.effect-scrub-spread |
| Spread | 393 | installed-effect-focused.effect-scrub-spread-label |
| Blend | 401 | installed-effect-focused.blend-label |
| Screen | 402 | installed-effect-focused.blend-value |
| (empty span) | 403 | installed-effect-focused.blend-third-cell |
| DevInfoEffectFocused | 406 | installed-effect-focused.devinfo |
| automation-mark | 419 | installed.position-automation |
| ObjectAutoHint | 431 | installed.position-object-hint |
| aside.inspector#inspector | 437 | installed.chrome-aside |
| SELECTED OBJECT | 441 | installed.section-selected-title |
| (empty span) | 441 | installed.section-selected-subtitle |
| G | 444 | installed.identity-icon |
| Pulse rings | 446 | installed.identity-name |
| Group · 1 child | 447 | installed.identity-kind-child |
| TRANSFORM | 453 | installed.transform-title |
| OBJECT | 453 | installed.transform-object-span |
| X | 460 | installed.position-x-label |
| 0.124 | 460 | installed.position-x-value |
| Y | 463 | installed.position-y-label |
| −0.082 | 463 | installed.position-y-value |
| 0.180 | 471 | installed.depth-value |
| 1.000 | 478 | installed.scale-value |
| 0.000 rad | 484 | installed.rotation-value |
| 100% | 490 | installed.opacity-value |
| APPEARANCE | 496 | installed.appearance-title |
| OBJECT | 496 | installed.appearance-object-span |
| Fill | 499 | installed.fill-label |
| color-chip | 501 | installed.fill-chip |
| COLOR | 514 | installed.fill-tag |
| Stroke | 517 | installed.stroke-label |
| color-chip | 519 | installed.stroke-chip |
| COLOR | 532 | installed.stroke-tag |
| GROUP COMPOSITION | 537 | installed.group-title |
| EDIT SPACE | 537 | installed.group-edit-space |
| Z Occlusion | 540 | installed.z-label |
| OFF / Stack | 542 | installed.z-off |
| ON / Group Z | 543 | installed.z-on |
| Z | 545 | installed.z-tag |
| Composite | 548 | installed.composite-label |
| Child → Group bake point | 549 | installed.composite-value |
| (empty span) | 550 | installed.composite-third |
| Link | 553 | installed.link-label |
| Position → target | 560 | installed.link-value |
| TYPED | 562 | installed.link-tag |
| DRIVER | 567 | installed.driver-title |
| 2 ROUTES | 567 | installed.driver-routes-span |
| Audio Low | 570 | installed.driver-label |
| driver-mini | 571 | installed.driver-svg |
| LIVE | 576 | installed.driver-tag |
| APPLIED PLUGINS | 581 | installed.plugins-title |
| ＋ | 581 | installed.plugins-add |
| :: | 584 | installed.plugin-grip |
| ◎ | 585 | installed.plugin-mini |
| Echo Bloom | 587 | installed.plugin-name |
| IN → Effect → OUT · selected | 588 | installed.plugin-sub |
| ECHO BLOOM | 594 | installed.echo-title |
| HOST PANEL | 594 | installed.echo-host-panel-span |
| Input | 597 | installed.echo-input-label |
| Pulse rings composite | 598 | installed.echo-input-value |
| TEXTURE | 599 | installed.echo-input-tag |
| EffectScrubRow | 601 | installed.echo-scrub-intensity |
| Intensity | 603 | installed.echo-scrub-intensity-label |
| EffectScrubRow | 610 | installed.echo-scrub-spread |
| Spread | 612 | installed.echo-scrub-spread-label |
| Blend | 620 | installed.echo-blend-label |
| Screen | 621 | installed.echo-blend-value |
| (empty span) | 622 | installed.echo-blend-third |
| DevInfoInstalled | 625 | installed.devinfo |
| aside.inspector#inspector | 632 | discover.chrome-aside |
| DISCOVERY | 636 | discover.title |
| NOT IN PROJECT | 636 | discover.not-in-project-span |
| 字 | 639 | discover.icon |
| Glyph Current | 641 | discover.name |
| Generator plugin · flowing type | 642 | discover.sub |
| Preview | 653 | discover.preview-btn |
| Add to selected object | 672 | discover.add-btn |
| DevInfoDiscover | 676 | discover.devinfo |
| aside.inspector#inspector | 683 | blocked.chrome-aside |
| DISCOVERY | 687 | blocked.title |
| UNAVAILABLE | 688 | blocked.subtitle |
| ◇ | 691 | blocked.icon |
| Fold Field | 693 | blocked.name |
| Effect plugin · local file | 694 | blocked.sub |
| このHostでは評価できません | 698 | blocked.notice-title |
| 要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。 | 700 | blocked.notice-body |
| Project change | 703 | blocked.lifecycle-project |
| NONE | 703 | blocked.lifecycle-none |
| Install | 703 | blocked.lifecycle-install |
| NOT STARTED | 703 | blocked.lifecycle-not-started |
| Fallback | 704 | blocked.lifecycle-fallback |
| REFUSED | 704 | blocked.lifecycle-refused |
| Inspect reason | 719 | blocked.inspect-btn |
| Add | 722 | blocked.add-btn |
| DevInfoBlocked | 725 | blocked.devinfo |
| aside.inspector#inspector | 731 | missing.chrome-aside |
| PROJECT INSTANCE | 735 | missing.title |
| MISSING | 736 | missing.subtitle |
| ? | 743 | missing.icon |
| Ribbon Array | 746 | missing.name |
| Plugin unavailable · Project instance retained | 747 | missing.sub |
| 必要なプラグインを評価できません | 751 | missing.notice-title |
| identity、version要求、instance payloadを保持しています。欠落中はpayloadを解釈して似た設定へ変換しません。 | 753 | missing.notice-body |
| Project open | 756 | missing.lifecycle-open |
| SUCCEEDED | 756 | missing.lifecycle-succeeded |
| Unrelated edit | 756 | missing.lifecycle-unrelated |
| AVAILABLE | 756 | missing.lifecycle-available |
| Required export | 757 | missing.lifecycle-export |
| REFUSED | 757 | missing.lifecycle-export-refused |
| Payload | 757 | missing.lifecycle-payload |
| RETAINED | 757 | missing.lifecycle-retained |
| Review recovery | 769 | missing.review-btn |
| STANDARD TRANSFORM | 775 | missing.std-title |
| EDITABLE | 775 | missing.std-editable-span |
| Position | 778 | missing.std-position-label |
| X 0.00 · Y 0.00 | 779 | missing.std-position-value |
| (empty span) | 780 | missing.std-position-third |
| Scale | 783 | missing.std-scale-label |
| 100% | 784 | missing.std-scale-value |
| (empty span) | 785 | missing.std-scale-third |
| DevInfoMissing | 788 | missing.devinfo |
| Position | 457 | installed.position-label |
| 2 KEYS | 466 | installed.position-keys |
| Depth Z | 470 | installed.depth-label |
| 1 KEY | 472 | installed.depth-keys |
| at-key | 473 | installed.depth-at-key |
| Scale | 477 | installed.scale-label |
| (no KEYS literal) | 479 | installed.scale-keys-closure |
| Rotation Z | 483 | installed.rotation-label |
| (no KEYS literal) | 485 | installed.rotation-keys-closure |
| Opacity | 489 | installed.opacity-label |
| 2 KEYS | 491 | installed.opacity-keys |

## §10 OPPORTUNITIES / ADVICE

- `loadReferenceFixtures.js` の fail-closed 形（`requireObject` / `requireFinite`）を CU-0A08IP が独自moduleで写す。汎用helperへ昇格しない。
- `PluginDiagnosticReason`（`plugin_resolution.rs`）は blocked/missing の近接typed語彙。mock lifecycle 文言をそのまま昇格しない。
- `S` が多く残るのは [分割決定](2026-07-26-cu-0a08i-inspector-read-model-split-decision.md)どおりの責任分割である。

<!-- STATS: section3=226 manifest=199 rules=14 forbidden=21 -->
