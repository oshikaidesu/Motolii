# U4b-0P Add Position Key 通常製品接続決定

- 日付: 2026-08-02
- 状態: **実装・通常製品window E2E DONE**
- U4b-0P: **DONE**

## 1. 利用者成果

通常製品windowでprimary RectangleのInspector Position行にある既存automation markを押すと、
現在playheadへPosition keyを1件追加する。別時刻でもう一度追加すれば、Timelineで2 keyと区間Easingを選択できる。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| AUTHORITY | M3 U4b、[U4b-0 closed contract](2026-08-02-u4b0-add-position-key-closed-contract-decision.md)、React製品資産直接移管契約 |
| INTERNAL TARGET | `DocumentWriter::prepare_add_position_key`、`DocumentEditRuntime`、`ProductApp.playhead` / primary / projection generation、Inspector Host |
| OWNER | Position curve / KeyframeIdはDocument、playheadはProject session、gesture deliveryはHost Transient、Reactはread-only projection |
| WRITE ROUTE | product Position action mark → exact Host identity → Host current playhead → existing shared writer → journal-first D2 → all surface republish |
| GAP | durable commandはあるが製品trigger、Position projection、Host intent、queue actionが未接続 |
| RESOLUTION ROUTE | `REUSE`: 既存source seat、Host exact codec/inbox、document queue、single writerを接続 |
| DISPOSITION | `PASS` |

## 3. React製品資産の強制動線

1. `REACT AUTHORITY`: Inspector Position面。直接移管契約、React chrome / native Timeline境界、M3 U4b / U4b-0。
2. `SOURCE ASSET`: `InspectorCandidate.jsx` SHA-256 `3c9e0096c95ea3692105eed016a7a2ff2c0f944d84984df258175982e5aa896e` のPosition row / `automation-mark`、CSS SHA-256 `730e2861a893b2b07fa66d5acef0038a49bdcf337e8c5a037785b0a58d829cbe`。product-owned exportをそのまま使う。
3. `PRESERVE`: `aside#inspector`、section / row / `param-label` / `automation-mark`、Position / key count、button位置と既存class。mock installed branchはbyte不変とする。
4. `REPLACE`: product safe branchではmock-local toggle意味を持ち込まず、同じ外観位置をHostのtyped Position projectionと`add-position-key` actionへ接続する。`aria-pressed`、`AUTO ON/OFF`、`at-key`は使わない。
5. `STATE OWNER`: curve / keys=`Document`、playhead=`Project session`、pending intent=`Transient`、表示だけ=`local presentation`。
6. `DIAGNOSTIC ROUTE`: 通常製品Inspector safe branchだけへ接続する。mock installed branchはconsumer oracleのまま、diagnostic-only画面を作らない。
7. `NEGATIVE ORACLE`: component copy 0、mock state import 0、raw JSON scan 0、React側allocator / evaluator / Document / playhead state 0、別button 0、Auto Key 0、same-time追加0、stale identity write 0、`at_key`解禁0、seek時whole Document再送0、threshold / golden変更0。
8. `STOP`: Vec2以外のPosition source、未決の編集意味、公開API / Document / plugin /永続形式変更、既存source seat不在、owner違反が必要なら当該粒を止める。

## 4. exact product projection / intent

Hostはprimaryから、`layer_id / projection_generation / key_count`だけをexact projectionする。
対象sourceは`Const(Vec2)`または`Keyframes(Vec2)`だけ。値評価、時刻、`at_key`はReactへ渡さない。

buttonは`key_count > 0`でautomationが存在するvisualを持つが、toggleでも押下状態でもない。
clickは`Add Position Key @ Host current playhead`である。同時刻でも同じintentを送ってよいが、既存prepareの
`AlreadyPresent`によりDocument / journal / Undo / stable ID変更0とする。

Hostは受信時に`layer_id / projection_generation`が現在projectionと完全一致する場合だけ、容量8のprivate inboxへ入れる。
playheadはReactから受け取らず、queue投入時のHost current valueだけを使う。このためseekでInspectorへwhole Documentを再送しない。
成功時だけ既存Document queueの一actionとしてshared writerへ渡し、通常publish pathでStage / Timeline / Inspector / Stage chromeを更新する。

Opus 5のread-only反対側レビューは、`at_key`解禁、toggle carrier再定義、seekごとのwhole Document再送を主要riskとして指摘した。
採用した修正は上記3点を削除し、既存Position markの外観と位置だけを再利用すること。Timelineの`◆ n / ◇＋`全面展開は本粒へ追加しない。

## 5. 必須oracle

- Const Positionで1 key、別playheadで2 keyになり、Timelineは同じKeyframeIdを表示する。
- same-time click、stale generation、selection変更後の旧intent、unsupported Position sourceは変更0。
- 1成功click=1 journal edit / 1 Undo。Undo / Redo / reopenでkey countとIDが一致する。
- Position row以外のmock-only Transform / appearance / driver / plugin UIを製品safe branchへ持ち込まない。
- 2 key成立後、既存Easing triggerがactive intervalを開ける。

## 6. 実装・通常製品window証跡

- 通常製品windowでRectangleを選択すると、既存Inspector safe branchにPosition action markが現れる。
- playhead 0秒で1回、5秒で1回押し、Inspector `1 KEY → 2 KEYS` とTimeline Tools `◆ 0 → ◆ 1 → ◆ 2`を同じ操作列で確認した。
- Timeline上の左keyを選択するとStage Transportの`Rectangle · PositionのInterval Easing Editorを開く`が有効化し、通常のnative popup windowを開けた。
- current binary SHA-256: `3f8f76bc3160e986baffbf35bfe4d670af9cd28b497ee5d446790a1e5a8311d2`
- E2E project SHA-256: `8a9db85ba21dda17e38dd5f96f0088dff295a8ed9b394dfbbbaf211e6332c63f`
- journal WAL SHA-256: `62742356c2c86c1aa592809e0c3993a9ee26c5c14e442dcce1622dfc11a14d59`
- 2-key product screenshot SHA-256: `110bcc9a149036522ea21eed1c485500beada4d118d5825cc496a953f6ce0835`
