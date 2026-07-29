# CU-0A08IS — Inspector read-model inventory

- 日付: 2026-07-26
- 状態: **決定**
- 粒: **CU-0A08IS DONE** → 後続 **CU-0A08IP DO**

## §1 FACTS

- `ui/motolii-web/src/candidates/InspectorCandidate.jsx`（804行、SHA256 `a01e7431361fdbaf1bdc1f1836045835f501d72e9d62aba1bf833ba3d53cc140`）は5つの相互排他branchを描画する: `installed && effectFocused`（L354）、`installed`（L412）、`discover`（L643）、`blocked`（L694）、default missing（L743）。
- `panelHead` の `Inspector` literal は `return (` 外の **L352** にあり、§9 走査対象に含める。
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
| installed-effect-focused.chrome-panel-head | InspectorCandidate.jsx:352 | Inspector | U |
| installed-effect-focused.chrome-aside | InspectorCandidate.jsx:356 | aside.inspector#inspector | U |
| installed-effect-focused.section-editing-title | InspectorCandidate.jsx:360 | EDITING EFFECT | S |
| installed-effect-focused.section-on-object | InspectorCandidate.jsx:360 | ON OBJECT | U |
| installed-effect-focused.identity-icon | InspectorCandidate.jsx:363 | ◎ | U |
| installed-effect-focused.identity-name | InspectorCandidate.jsx:365 | Echo Bloom | D |
| installed-effect-focused.identity-subtitle | InspectorCandidate.jsx:366 | Pulse rings · Effect | S |
| installed-effect-focused.host-panel-span | InspectorCandidate.jsx:372 | HOST PANEL | U |
| installed-effect-focused.host-title | InspectorCandidate.jsx:372 | ECHO BLOOM | U |
| installed-effect-focused.effect-description | InspectorCandidate.jsx:375 | Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage. | S |
| installed-effect-focused.effect-scrub-intensity-label | InspectorCandidate.jsx:385 | Intensity | U |
| installed-effect-focused.effect-scrub-spread-label | InspectorCandidate.jsx:394 | Spread | U |
| installed-effect-focused.input-label | InspectorCandidate.jsx:379 | Input | U |
| installed-effect-focused.input-value | InspectorCandidate.jsx:380 | Pulse rings composite | S |
| installed-effect-focused.input-tag | InspectorCandidate.jsx:381 | TEXTURE | S |
| installed-effect-focused.effect-scrub-intensity | InspectorCandidate.jsx:383 | EffectScrubRow | U |
| installed-effect-focused.effect-scrub-spread | InspectorCandidate.jsx:392 | EffectScrubRow | U |
| installed-effect-focused.blend-label | InspectorCandidate.jsx:402 | Blend | U |
| installed-effect-focused.blend-value | InspectorCandidate.jsx:403 | Screen | S |
| installed-effect-focused.blend-third-cell | InspectorCandidate.jsx:404 | (empty span) | U |
| installed-effect-focused.devinfo | InspectorCandidate.jsx:407 | DevInfoEffectFocused | U |

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
| installed.chrome-panel-head | InspectorCandidate.jsx:352 | Inspector | U |
| installed.depth-automation-mark | InspectorCandidate.jsx:432 | automation-mark | S |
| installed.opacity-automation-mark | InspectorCandidate.jsx:432 | automation-mark | S |
| installed.position-automation | InspectorCandidate.jsx:432 | automation-mark | S |
| installed.rotation-automation-mark | InspectorCandidate.jsx:432 | automation-mark | S |
| installed.scale-automation-mark | InspectorCandidate.jsx:432 | automation-mark | S |
| installed.depth-object-hint | InspectorCandidate.jsx:444 | ObjectAutoHint | U |
| installed.opacity-object-hint | InspectorCandidate.jsx:444 | ObjectAutoHint | U |
| installed.position-object-hint | InspectorCandidate.jsx:444 | ObjectAutoHint | U |
| installed.rotation-object-hint | InspectorCandidate.jsx:444 | ObjectAutoHint | U |
| installed.scale-object-hint | InspectorCandidate.jsx:444 | ObjectAutoHint | U |
| installed.chrome-aside | InspectorCandidate.jsx:450 | aside.inspector#inspector | U |
| installed.section-selected-subtitle | InspectorCandidate.jsx:454 | (empty span) | U |
| installed.section-selected-title | InspectorCandidate.jsx:454 | SELECTED OBJECT | S |
| installed.identity-icon | InspectorCandidate.jsx:457 | G | U |
| installed.identity-name | InspectorCandidate.jsx:459 | {selectedObjectName} | A |
| installed.identity-kind-child | InspectorCandidate.jsx:460 | {selectedObjectKind} | D |
| installed.transform-object-span | InspectorCandidate.jsx:466 | OBJECT | U |
| installed.transform-title | InspectorCandidate.jsx:466 | TRANSFORM | U |
| installed.position-label | InspectorCandidate.jsx:470 | Position | U |
| installed.position-x-label | InspectorCandidate.jsx:473 | X | U |
| installed.position-x-value | InspectorCandidate.jsx:473 | 0.124 | D |
| installed.position-y-label | InspectorCandidate.jsx:476 | Y | U |
| installed.position-y-value | InspectorCandidate.jsx:476 | −0.082 | D |
| installed.position-keys | InspectorCandidate.jsx:479 | 2 KEYS | D |
| installed.depth-label | InspectorCandidate.jsx:483 | Depth Z | U |
| installed.depth-value | InspectorCandidate.jsx:484 | 0.180 | S |
| installed.depth-keys | InspectorCandidate.jsx:485 | 1 KEY | D |
| installed.depth-at-key | InspectorCandidate.jsx:486 | at-key | U |
| installed.scale-label | InspectorCandidate.jsx:490 | Scale | U |
| installed.scale-value | InspectorCandidate.jsx:491 | 1.000 | S |
| installed.scale-keys-closure | InspectorCandidate.jsx:492 | (no KEYS literal) | U |
| installed.rotation-label | InspectorCandidate.jsx:496 | Rotation Z | U |
| installed.rotation-value | InspectorCandidate.jsx:497 | 0.000 rad | D |
| installed.rotation-keys-closure | InspectorCandidate.jsx:498 | (no KEYS literal) | U |
| installed.opacity-label | InspectorCandidate.jsx:502 | Opacity | U |
| installed.opacity-value | InspectorCandidate.jsx:503 | 100% | D |
| installed.opacity-keys | InspectorCandidate.jsx:504 | 2 KEYS | D |
| installed.appearance-object-span | InspectorCandidate.jsx:509 | OBJECT | U |
| installed.appearance-title | InspectorCandidate.jsx:509 | APPEARANCE | U |
| installed.fill-label | InspectorCandidate.jsx:512 | Fill | U |
| installed.fill-chip | InspectorCandidate.jsx:514 | color-chip | S |
| installed.fill-tag | InspectorCandidate.jsx:527 | COLOR | U |
| installed.stroke-label | InspectorCandidate.jsx:530 | Stroke | U |
| installed.stroke-chip | InspectorCandidate.jsx:532 | color-chip | S |
| installed.stroke-tag | InspectorCandidate.jsx:545 | COLOR | U |
| installed.group-edit-space | InspectorCandidate.jsx:550 | EDIT SPACE | U |
| installed.group-title | InspectorCandidate.jsx:550 | GROUP COMPOSITION | U |
| installed.z-label | InspectorCandidate.jsx:553 | Z Occlusion | S |
| installed.z-off | InspectorCandidate.jsx:555 | OFF / Stack | S |
| installed.z-on | InspectorCandidate.jsx:556 | ON / Group Z | S |
| installed.z-tag | InspectorCandidate.jsx:558 | Z | U |
| installed.composite-label | InspectorCandidate.jsx:561 | Composite | U |
| installed.composite-value | InspectorCandidate.jsx:562 | Child → Group bake point | U |
| installed.composite-third | InspectorCandidate.jsx:563 | (empty span) | U |
| installed.link-label | InspectorCandidate.jsx:566 | Link | U |
| installed.link-value | InspectorCandidate.jsx:573 | Position → target | S |
| installed.link-tag | InspectorCandidate.jsx:575 | TYPED | U |
| installed.driver-routes-span | InspectorCandidate.jsx:580 | 2 ROUTES | U |
| installed.driver-title | InspectorCandidate.jsx:580 | DRIVER | U |
| installed.driver-label | InspectorCandidate.jsx:583 | Audio Low | S |
| installed.driver-svg | InspectorCandidate.jsx:584 | driver-mini | U |
| installed.driver-tag | InspectorCandidate.jsx:589 | LIVE | S |
| installed.plugins-add | InspectorCandidate.jsx:594 | ＋ | S |
| installed.plugins-title | InspectorCandidate.jsx:594 | APPLIED PLUGINS | U |
| installed.plugin-grip | InspectorCandidate.jsx:597 | :: | U |
| installed.plugin-mini | InspectorCandidate.jsx:598 | ◎ | U |
| installed.plugin-name | InspectorCandidate.jsx:600 | Echo Bloom | D |
| installed.plugin-sub | InspectorCandidate.jsx:601 | IN → Effect → OUT · selected | S |
| installed.echo-title | InspectorCandidate.jsx:607 | ECHO BLOOM | U |
| installed.echo-host-panel-span | InspectorCandidate.jsx:607 | HOST PANEL | U |
| installed.echo-input-label | InspectorCandidate.jsx:610 | Input | U |
| installed.echo-input-value | InspectorCandidate.jsx:611 | Pulse rings composite | S |
| installed.echo-input-tag | InspectorCandidate.jsx:612 | TEXTURE | S |
| installed.echo-scrub-intensity | InspectorCandidate.jsx:614 | EffectScrubRow | U |
| installed.echo-scrub-intensity-label | InspectorCandidate.jsx:616 | Intensity | U |
| installed.echo-scrub-spread | InspectorCandidate.jsx:623 | EffectScrubRow | U |
| installed.echo-scrub-spread-label | InspectorCandidate.jsx:625 | Spread | U |
| installed.echo-blend-label | InspectorCandidate.jsx:633 | Blend | U |
| installed.echo-blend-value | InspectorCandidate.jsx:634 | Screen | S |
| installed.echo-blend-third | InspectorCandidate.jsx:635 | (empty span) | U |
| installed.devinfo | InspectorCandidate.jsx:638 | DevInfoInstalled | U |

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
| discover.chrome-panel-head | InspectorCandidate.jsx:352 | Inspector | U |
| discover.chrome-aside | InspectorCandidate.jsx:645 | aside.inspector#inspector | U |
| discover.not-in-project-span | InspectorCandidate.jsx:649 | NOT IN PROJECT | U |
| discover.title | InspectorCandidate.jsx:649 | DISCOVERY | U |
| discover.icon | InspectorCandidate.jsx:652 | 字 | U |
| discover.name | InspectorCandidate.jsx:654 | Glyph Current | A |
| discover.sub | InspectorCandidate.jsx:655 | Generator plugin · flowing type | S |
| discover.preview-btn | InspectorCandidate.jsx:666 | Preview | S |
| discover.add-btn | InspectorCandidate.jsx:685 | Add to selected object | S |
| discover.devinfo | InspectorCandidate.jsx:689 | DevInfoDiscover | U |

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
| blocked.chrome-panel-head | InspectorCandidate.jsx:352 | Inspector | U |
| blocked.chrome-aside | InspectorCandidate.jsx:696 | aside.inspector#inspector | U |
| blocked.title | InspectorCandidate.jsx:700 | DISCOVERY | U |
| blocked.subtitle | InspectorCandidate.jsx:701 | UNAVAILABLE | S |
| blocked.icon | InspectorCandidate.jsx:704 | ◇ | U |
| blocked.name | InspectorCandidate.jsx:706 | Fold Field | A |
| blocked.sub | InspectorCandidate.jsx:707 | Effect plugin · local file | S |
| blocked.notice-title | InspectorCandidate.jsx:711 | このHostでは評価できません | S |
| blocked.notice-body | InspectorCandidate.jsx:713 | 要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。 | S |
| blocked.lifecycle-install | InspectorCandidate.jsx:716 | Install | U |
| blocked.lifecycle-none | InspectorCandidate.jsx:716 | NONE | S |
| blocked.lifecycle-not-started | InspectorCandidate.jsx:716 | NOT STARTED | S |
| blocked.lifecycle-project | InspectorCandidate.jsx:716 | Project change | U |
| blocked.lifecycle-fallback | InspectorCandidate.jsx:717 | Fallback | U |
| blocked.lifecycle-refused | InspectorCandidate.jsx:717 | REFUSED | S |
| blocked.inspect-btn | InspectorCandidate.jsx:732 | Inspect reason | S |
| blocked.add-btn | InspectorCandidate.jsx:735 | Add | S |
| blocked.devinfo | InspectorCandidate.jsx:738 | DevInfoBlocked | U |

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
| missing.chrome-panel-head | InspectorCandidate.jsx:352 | Inspector | U |
| missing.chrome-aside | InspectorCandidate.jsx:744 | aside.inspector#inspector | U |
| missing.title | InspectorCandidate.jsx:748 | PROJECT INSTANCE | S |
| missing.subtitle | InspectorCandidate.jsx:749 | MISSING | S |
| missing.icon | InspectorCandidate.jsx:756 | ? | U |
| missing.name | InspectorCandidate.jsx:759 | Ribbon Array | A |
| missing.sub | InspectorCandidate.jsx:760 | Plugin unavailable · Project instance retained | S |
| missing.notice-title | InspectorCandidate.jsx:764 | 必要なプラグインを評価できません | S |
| missing.notice-body | InspectorCandidate.jsx:766 | identity、version要求、instance payloadを保持しています。欠落中はpayloadを解釈して似た設定へ変換しません。 | S |
| missing.lifecycle-available | InspectorCandidate.jsx:769 | AVAILABLE | S |
| missing.lifecycle-open | InspectorCandidate.jsx:769 | Project open | U |
| missing.lifecycle-succeeded | InspectorCandidate.jsx:769 | SUCCEEDED | S |
| missing.lifecycle-unrelated | InspectorCandidate.jsx:769 | Unrelated edit | U |
| missing.lifecycle-export | InspectorCandidate.jsx:770 | Required export | U |
| missing.lifecycle-export-refused | InspectorCandidate.jsx:770 | REFUSED | S |
| missing.lifecycle-payload | InspectorCandidate.jsx:770 | Payload | U |
| missing.lifecycle-retained | InspectorCandidate.jsx:770 | RETAINED | S |
| missing.review-btn | InspectorCandidate.jsx:782 | Review recovery | S |
| missing.std-editable-span | InspectorCandidate.jsx:788 | EDITABLE | U |
| missing.std-title | InspectorCandidate.jsx:788 | STANDARD TRANSFORM | U |
| missing.std-position-label | InspectorCandidate.jsx:791 | Position | U |
| missing.std-position-value | InspectorCandidate.jsx:792 | X 0.00 · Y 0.00 | S |
| missing.std-position-third | InspectorCandidate.jsx:793 | (empty span) | U |
| missing.std-scale-label | InspectorCandidate.jsx:796 | Scale | U |
| missing.std-scale-value | InspectorCandidate.jsx:797 | 100% | S |
| missing.std-scale-third | InspectorCandidate.jsx:798 | (empty span) | U |
| missing.devinfo | InspectorCandidate.jsx:801 | DevInfoMissing | U |

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

CU-0A08ITPでinstalled分岐の `selectedObjectName` / `selectedObjectKind` はdecode済みtargetへ接続済み。discover / blocked / missing の可視名へ `CAP-LAYER-NAME` を接続するmappingは、runtime binding前に再照合する。CU-0A08ISの採用判定を他branchの配線決定として継承しない。

## §5 CU-0A08IP 受理入力契約

### §5.1 受理wrapper

top-level object は **ちょうど4キー**: `fixtureRevision`（整数、decoder定数 **1** と等値。`fixtureRevision = 1`）、`document`（現行Document JSON）、`nodes`（NodeDesc 由来配列、id は plugin-id 語彙）、`target`（`{ "layer_id": <u64> }`）。`screen` / `mode` / `scenes` / `tokens` は受理しない。

各 `nodes` 要素は `{ "id": <plugin-id>, "params": [ { "id", "value_type", "default", "f64_domain"? } ] }` に固定する。`default` は既存 `Value` の external-tag JSON をそのまま用いる。`NodeDesc` / `ParamDef` へ serde derive を追加しない。

受理fixtureの `nodes` は **1件のみ** とし、`plugins/motolii-plugin-opacity` の `opacity_filter_desc()`（`core.filter.opacity`、param `amount` / `F64` / default `{"F64":1.0}` / `f64_domain` は unit domain）に固定する。`effect_definitions[].params` の object key と `ParamDef.id` の一致照合は本契約の範囲外とする。

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

R10は blend / domain / default / opacity の**入力検証**のみを行う。blend を decoder 出力へ昇格させない（§4 `CAP-BLEND-MODE` は**未採用**のまま）。

### §6a R14の適用範囲

R14は **decoder出力objectのkeyとnode拡張keyのみ**に適用する。`document` / `DocParam` / 既存serde面のkeyには適用しない。

禁止key語彙（閉集合）: `fill_color`, `stroke_color`, `z_occlusion`, `occlusion_mode`, `depth_z`, `bake_point`, `composite_bake`, `driver_route`, `driver_routes`, `applied_plugin_history`, `availability`, `availability_lifecycle`, `effect_description`, `input_socket_label`, `socket_type_tag`, `link_label`, `link_target_label`, `primary_selection`, `selected_object`, `editing_effect`, `at_key`。

R14 whitelist（閉集合。禁止key語彙と交差してはならない）: `scale`, `blend`, `data`, `look_at`, `follow`, `params`, `extra`, `const`, `keyframes`, `vec2_axes`, `target`, `axis`, `offset`, `track`, `fallback`, `x`, `y`。

R14 **禁止key語彙**（§6a 閉集合）は whitelist と互いに素であり、さらに `docs/mocks-ui/fixtures/reference-document.json` を再帰走査した全key集合とも**交差してはならない**（reference-document key との非交差要件は禁止setにのみ適用し、whitelist には課さない）。

R7 / R11 / R14 の適用面は互いに素であり、R14は現行serdeの未知field保持を上書きしない。

## §7 入力例

`document` メンバの実体は `docs/mocks-ui/fixtures/reference-document.json` をpath参照する。

非JSON・模式表記（そのまま入力しない）

```text
fixtureRevision : 1（decoder定数と等値）
document        : docs/mocks-ui/fixtures/reference-document.json の現行Document object
nodes           : NodeDesc 由来 entry の配列（id は plugin-id 文字列語彙）
target          : { layer_id: u64 }
```

`{}` を受理・合格・accepted 例として掲載しない。

## §8 非目標・STOP

decoder実装、Host transport、typed intent、React/CSS/DOM、Document/plugin公開契約の変更はSTOP。`S` の意味を決める必要が出たらSTOP。

### §8.1 CU-0A08IP 着手境界（DO）

CU-0A08IPの閉じた実装成果は、product-owned・非export の pure decoder module に限定する。呼び出しは fixture / test のみとし、Host transport、typed intent、JSX binding、`S` 行の意味決定、Rust / schema / plugin 契約変更は非目標のままとする。

## §9 coverage manifest

抽出規則（Babel AST oracle）:

1. `@babel/parser` + `@babel/traverse` で JSX を1回解析する。
2. `JSXText` はノード全体を1レコードにする（`text = node.value.replace(/\s+/g," ").trim()`、`line = node.loc.start.line + leading whitespace 内の改行数`）。capitalized `JSXOpeningElement`、attribute-free `<span />`、`className` 文字列/テンプレート、可視 template/conditional、installed identityの既知projection binding `{selectedObjectName}` / `{selectedObjectKind}`、`objectRow` 第2引数label、第4引数 `keys-arg`、既知 component の `label` StringLiteral（`prop-label`、`aria-label` 除外）を multiset として記録する（重複除去しない）。canonical key は `kind|line|text#occ`（`occ` は同一 `(kind,line,text)` の走査順1-based）。

§9 sentinel: `PROJECT INSTANCE`（L748）、`Inspector`（L352）、`Vism (.vism)`（L144/L166/L184/L206/L224）。

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
| Inspector | 352 | installed-effect-focused.chrome-panel-head |
| aside.inspector#inspector | 356 | installed-effect-focused.chrome-aside |
| EDITING EFFECT | 360 | installed-effect-focused.section-editing-title |
| ON OBJECT | 360 | installed-effect-focused.section-on-object |
| ◎ | 363 | installed-effect-focused.identity-icon |
| Echo Bloom | 365 | installed-effect-focused.identity-name |
| Pulse rings · Effect | 366 | installed-effect-focused.identity-subtitle |
| ECHO BLOOM | 372 | installed-effect-focused.host-title |
| HOST PANEL | 372 | installed-effect-focused.host-panel-span |
| Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage. | 375 | installed-effect-focused.effect-description |
| Input | 379 | installed-effect-focused.input-label |
| Pulse rings composite | 380 | installed-effect-focused.input-value |
| TEXTURE | 381 | installed-effect-focused.input-tag |
| EffectScrubRow | 383 | installed-effect-focused.effect-scrub-intensity |
| Intensity | 385 | installed-effect-focused.effect-scrub-intensity-label |
| EffectScrubRow | 392 | installed-effect-focused.effect-scrub-spread |
| Spread | 394 | installed-effect-focused.effect-scrub-spread-label |
| Blend | 402 | installed-effect-focused.blend-label |
| Screen | 403 | installed-effect-focused.blend-value |
| (empty span) | 404 | installed-effect-focused.blend-third-cell |
| DevInfoEffectFocused | 407 | installed-effect-focused.devinfo |
| automation-mark | 432 | installed.position-automation |
| ObjectAutoHint | 444 | installed.position-object-hint |
| aside.inspector#inspector | 450 | installed.chrome-aside |
| SELECTED OBJECT | 454 | installed.section-selected-title |
| (empty span) | 454 | installed.section-selected-subtitle |
| G | 457 | installed.identity-icon |
| {selectedObjectName} | 459 | installed.identity-name |
| {selectedObjectKind} | 460 | installed.identity-kind-child |
| TRANSFORM | 466 | installed.transform-title |
| OBJECT | 466 | installed.transform-object-span |
| X | 473 | installed.position-x-label |
| 0.124 | 473 | installed.position-x-value |
| Y | 476 | installed.position-y-label |
| −0.082 | 476 | installed.position-y-value |
| 0.180 | 484 | installed.depth-value |
| 1.000 | 491 | installed.scale-value |
| 0.000 rad | 497 | installed.rotation-value |
| 100% | 503 | installed.opacity-value |
| APPEARANCE | 509 | installed.appearance-title |
| OBJECT | 509 | installed.appearance-object-span |
| Fill | 512 | installed.fill-label |
| color-chip | 514 | installed.fill-chip |
| COLOR | 527 | installed.fill-tag |
| Stroke | 530 | installed.stroke-label |
| color-chip | 532 | installed.stroke-chip |
| COLOR | 545 | installed.stroke-tag |
| GROUP COMPOSITION | 550 | installed.group-title |
| EDIT SPACE | 550 | installed.group-edit-space |
| Z Occlusion | 553 | installed.z-label |
| OFF / Stack | 555 | installed.z-off |
| ON / Group Z | 556 | installed.z-on |
| Z | 558 | installed.z-tag |
| Composite | 561 | installed.composite-label |
| Child → Group bake point | 562 | installed.composite-value |
| (empty span) | 563 | installed.composite-third |
| Link | 566 | installed.link-label |
| Position → target | 573 | installed.link-value |
| TYPED | 575 | installed.link-tag |
| DRIVER | 580 | installed.driver-title |
| 2 ROUTES | 580 | installed.driver-routes-span |
| Audio Low | 583 | installed.driver-label |
| driver-mini | 584 | installed.driver-svg |
| LIVE | 589 | installed.driver-tag |
| APPLIED PLUGINS | 594 | installed.plugins-title |
| ＋ | 594 | installed.plugins-add |
| :: | 597 | installed.plugin-grip |
| ◎ | 598 | installed.plugin-mini |
| Echo Bloom | 600 | installed.plugin-name |
| IN → Effect → OUT · selected | 601 | installed.plugin-sub |
| ECHO BLOOM | 607 | installed.echo-title |
| HOST PANEL | 607 | installed.echo-host-panel-span |
| Input | 610 | installed.echo-input-label |
| Pulse rings composite | 611 | installed.echo-input-value |
| TEXTURE | 612 | installed.echo-input-tag |
| EffectScrubRow | 614 | installed.echo-scrub-intensity |
| Intensity | 616 | installed.echo-scrub-intensity-label |
| EffectScrubRow | 623 | installed.echo-scrub-spread |
| Spread | 625 | installed.echo-scrub-spread-label |
| Blend | 633 | installed.echo-blend-label |
| Screen | 634 | installed.echo-blend-value |
| (empty span) | 635 | installed.echo-blend-third |
| DevInfoInstalled | 638 | installed.devinfo |
| aside.inspector#inspector | 645 | discover.chrome-aside |
| DISCOVERY | 649 | discover.title |
| NOT IN PROJECT | 649 | discover.not-in-project-span |
| 字 | 652 | discover.icon |
| Glyph Current | 654 | discover.name |
| Generator plugin · flowing type | 655 | discover.sub |
| Preview | 666 | discover.preview-btn |
| Add to selected object | 685 | discover.add-btn |
| DevInfoDiscover | 689 | discover.devinfo |
| aside.inspector#inspector | 696 | blocked.chrome-aside |
| DISCOVERY | 700 | blocked.title |
| UNAVAILABLE | 701 | blocked.subtitle |
| ◇ | 704 | blocked.icon |
| Fold Field | 706 | blocked.name |
| Effect plugin · local file | 707 | blocked.sub |
| このHostでは評価できません | 711 | blocked.notice-title |
| 要求された能力が未対応です。近い既存Effectへ置換せず、非互換理由を表示します。 | 713 | blocked.notice-body |
| Project change | 716 | blocked.lifecycle-project |
| NONE | 716 | blocked.lifecycle-none |
| Install | 716 | blocked.lifecycle-install |
| NOT STARTED | 716 | blocked.lifecycle-not-started |
| Fallback | 717 | blocked.lifecycle-fallback |
| REFUSED | 717 | blocked.lifecycle-refused |
| Inspect reason | 732 | blocked.inspect-btn |
| Add | 735 | blocked.add-btn |
| DevInfoBlocked | 738 | blocked.devinfo |
| aside.inspector#inspector | 744 | missing.chrome-aside |
| PROJECT INSTANCE | 748 | missing.title |
| MISSING | 749 | missing.subtitle |
| ? | 756 | missing.icon |
| Ribbon Array | 759 | missing.name |
| Plugin unavailable · Project instance retained | 760 | missing.sub |
| 必要なプラグインを評価できません | 764 | missing.notice-title |
| identity、version要求、instance payloadを保持しています。欠落中はpayloadを解釈して似た設定へ変換しません。 | 766 | missing.notice-body |
| Project open | 769 | missing.lifecycle-open |
| SUCCEEDED | 769 | missing.lifecycle-succeeded |
| Unrelated edit | 769 | missing.lifecycle-unrelated |
| AVAILABLE | 769 | missing.lifecycle-available |
| Required export | 770 | missing.lifecycle-export |
| REFUSED | 770 | missing.lifecycle-export-refused |
| Payload | 770 | missing.lifecycle-payload |
| RETAINED | 770 | missing.lifecycle-retained |
| Review recovery | 782 | missing.review-btn |
| STANDARD TRANSFORM | 788 | missing.std-title |
| EDITABLE | 788 | missing.std-editable-span |
| Position | 791 | missing.std-position-label |
| X 0.00 · Y 0.00 | 792 | missing.std-position-value |
| (empty span) | 793 | missing.std-position-third |
| Scale | 796 | missing.std-scale-label |
| 100% | 797 | missing.std-scale-value |
| (empty span) | 798 | missing.std-scale-third |
| DevInfoMissing | 801 | missing.devinfo |
| Position | 470 | installed.position-label |
| 2 KEYS | 479 | installed.position-keys |
| Depth Z | 483 | installed.depth-label |
| 1 KEY | 485 | installed.depth-keys |
| at-key | 486 | installed.depth-at-key |
| Scale | 490 | installed.scale-label |
| (no KEYS literal) | 492 | installed.scale-keys-closure |
| Rotation Z | 496 | installed.rotation-label |
| (no KEYS literal) | 498 | installed.rotation-keys-closure |
| Opacity | 502 | installed.opacity-label |
| 2 KEYS | 504 | installed.opacity-keys |

## §10 OPPORTUNITIES / ADVICE

- `loadReferenceFixtures.js` の fail-closed 形（`requireObject` / `requireFinite`）を CU-0A08IP が独自moduleで写す。汎用helperへ昇格しない。
- `PluginDiagnosticReason`（`plugin_resolution.rs`）は blocked/missing の近接typed語彙。mock lifecycle 文言をそのまま昇格しない。
- `S` が多く残るのは [分割決定](2026-07-26-cu-0a08i-inspector-read-model-split-decision.md)どおりの責任分割である。
- CU-0A08ITへ渡す前に、discover / blocked / missing の layer 名mappingは §4 `CAP-LAYER-NAME` 注記どおり runtime binding 前に再照合する。

<!-- STATS: section3=226 manifest=199 rules=14 forbidden=21 -->
