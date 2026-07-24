# M3 surface実装と拡張所有の軸分離

作成日: 2026-07-22

状態: **決定**。製品UIのwindow／surface実装と、Core・first-party・third-party拡張の責任分類を独立に判定する。コード、公開plugin API、Document、永続形式は変更しない。

2026-07-25限定改訂: [制御されたMicrokernelとHost capability module並列化決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)により、Core／Hostのarchitectural roleは「authority ownership」と「具体実装provider」をさらに分離する。本書のsurface runtime、公開plugin、provenance／trustの軸分離は維持するが、§2／§6の「Coreに残す」は具体実装を一枚岩のCoreへ固定する根拠にしない。

## 1. 決定

これまで「native window対React window」と「Core対first-party／third-party plugin」を同じ選択として扱う記述があった。以後は少なくとも次の4軸を分離する。

| 軸 | 選択肢 | 判定するもの |
|---|---|---|
| OS topology | top-level window / child view / popup / detached window | window、focus、z-order、DPI、lifecycle |
| presentation runtime | native wgpu / React・WebView / headless | 表示、layout、hit-test、入力adapter |
| architectural role | Core kernel / bundled Host module / plugin | 作品意味、標準製品面、拡張境界のどこに属するか |
| provenance / execution trust | first-party / third-party、TCB role / untrusted isolated realm | 配布元と、権限、sandbox、障害隔離を独立判定 |

一つのOS windowにnative viewportとopaque child WebViewを同居させられるため、nativeとReactを別windowの同義語にしない。nativeで描画するfirst-party Host moduleはnative pluginではなく、Reactで描画するfirst-party Host moduleもcommunity pluginではない。first-partyという供給元だけからCore所属や公開plugin契約参加を推論しない。

2026-07-25信頼境界改訂により、公開plugin codeはfirst-party／third-partyを問わず非信頼とする。
TCBに入るのはControlled Coreと製品buildへ明示的にadmitされたHost moduleであり、first-partyという
provenanceだけでは入らない。現行static first-party pluginの同一process実行はコード事実であって
隔離完成ではない。正本は[Controlled Microkernel決定 §6](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md#6-pluginという語と信頼境界の分離)。

## 2. Motoliiの現行分類

| 能力 | architectural role | presentation runtime | 現行判定 |
|---|---|---|---|
| Document、stable ID、時刻、D2、Undo、評価順、Preview/Export同一評価、cache/resource契約 | Core kernel | headless | pluginへ委譲しない |
| 標準Timelineのlayout、hit-test、描画、gesture projection | bundled first-party Host module | native wgpu | 製品同梱。公開差替えpluginにしない |
| 標準Stage / Preview viewer、gizmo、overlay、window投影 | bundled first-party Host module | native wgpu | evaluator、Transport、resource lifecycleはCoreに残す |
| Browser、Inspector、form、toolbar | bundled first-party Host module | React / WebView | product-owned source。community pluginではない |
| Opacity、Sine、Radial Repeater等 | first-party plugin | Host生成panel | 公開plugin境界を反証する同梱実装 |
| 将来のcommunity Effect / Tool / panel | third-party plugin候補 | 未決 | G0-3 / GAP-13で別途審判する |

標準TimelineとStageをcrateやprivate interfaceで交換可能に保つことはできるが、欠落可能なplugin、動的ロード単位、第三者差替え契約にする決定ではない。二つ目の実装と欠落時意味が成立する前に公開interfaceを固定しない。

## 3. TimelineとPreviewの分界

Timelineは全体をCoreまたはpluginのどちらかへ置かない。

- Core: RationalTime、clip/key identity、projection入力、selection意味、typed intent、D2 command、Undo/Cancel。
- bundled Host module: visible range layout、semantic zoom、hit-test、native描画、gesture adapter、bounded accessibility projection。
- plugin候補: 標準意味へtyped commandを提案する専門Toolや、宣言済みparameter／domainの投影。独自Timeline正本、Undo、Document探索は持たない。

Previewも同様に分ける。

- Core: `render(t)`、Preview/Export同一評価、Transport clock、latest generation、Quality、cache、GPU resource lifecycle。
- bundled Host module: Stage window／viewport、zoom、grid、gizmo、overlay、別monitor投影。
- plugin候補: 宣言入力から評価結果を返す表現、またはHostが管理する診断／解析overlay。独自preview schedulerやcache正本は持たない。

## 4. G0-9とG0-3を分離する

G0-9はMotolii標準製品面のsurface topologyとplatform受入を判定する。対象はReact/WebView chrome、native Stage/Timeline、popup、focus、DPI、resize、z-order、capture、a11y接合、rendererである。

G0-3 / GAP-13はplugin UI公開境界を判定する。対象はfirst-party pluginとthird-party pluginの共通／差分、sandbox、権限、version互換、配布、署名、障害隔離、Host生成fallbackである。G0-9の測定証拠を入力にできるが、次を同義にしない。

- G0-9合格 ≠ custom plugin UI公開許可。
- product-owned React package成立 ≠ community runtime成立。
- 同じcomponent/test kitを再利用する長期原則 ≠ 同じorigin、process、権限、window topology。
- plugin sandbox未決 ≠ first-party製品surface実装の全面停止。Host surfaceはpluginではなくTCB roleとして別審判する。

plugin所有UI codeの公開契約は引き続き停止する。停止理由と解除審判はG0-3 / GAP-13へ置き、G0-9完了だけで解除しない。

## 5. 用語ガード

- `native window` / `React window`だけで責任を表さず、`OS window`、`native surface`、`WebView island`を分けて書く。
- `core plugin`は使わない。必要なら`Core kernel`、`bundled first-party Host module`、`first-party plugin`を明記する。
- `native plugin`は実行形式だけを表し得るため、UI surface、供給元、権限を別記する。
- `Host/community同一kit`はcomponentとtest語彙の再利用を指す。公開runtime、権限、配布の同一化を意味しない。

## 6. 非目標とSTOP

- crate再編、dynamic loader、module ABI、plugin manifest、custom UI APIを実装しない。
- Timeline／Previewを欠落可能なpluginへ変更しない。
- Core責任を減らすためにDocument、Undo、selection、cache、Transportを製品moduleへ複製しない。
- native/Reactの採否からfirst/third-partyの信頼境界を逆算しない。
- first-party実装で成功したsurface APIを、そのままthird-party公開契約へ昇格しようとしたら停止する。

歴史処分追補(2026-07-23): A1の「外部crate」が証明したsource-level capabilityと、未成立のinstall/load/trust runtimeの分界は[公開capability／provenance回収](2026-07-23-historical-public-capability-provenance-lineage-recovery.md)を正とする。
