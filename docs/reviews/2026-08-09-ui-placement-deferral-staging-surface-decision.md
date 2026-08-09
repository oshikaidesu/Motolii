# UI配置保留と未配置control staging surface決定

状態: **決定**（2026-08-09、DOCS ONLY / RUNTIME NOT STARTED）

対象: M3／M4／M5の並列接続で、操作意味とruntime routeは閉じているが、最終的な製品UI配置だけが未決のcontrol

## 1. 結論

意味、read projection、typed intentまたはD2 Command、owner、Undo／failure意味が閉じているcontrolは、最終surfaceの決定を待たず、Host所有の**未配置control staging surface**へ一時配置して製品接続を進めてよい。

これは製品上の新しい「設定画面」、万能Inspector、debug panel、plugin UI frameworkではない。最終配置が決まるまで既存controlを置く任意表示のHost panelであり、製品表示名も本決定では固定しない。会話上の`Utility Rack`は候補名に留める。

staging surfaceは値、Document意味、Command、preview lifecycle、resource lifecycleを所有しない。同じread projectionと同じtyped write routeを最終surfaceと共有し、最終配置時にはpresentationだけを差し替える。したがってUI配置の未決だけを理由に、意味と接続が閉じた並列laneを停止しない。

逆に、操作意味、owner、command、consumer、Undo、failureのいずれかが未決ならstagingへ逃がさない。そのedgeは`RESEARCH_RETURN`または`WAIT_TARGET`とし、仮controlから意味を逆算しない。

## 2. 既知実装preflight

- **MECHANISM CLASS**: Host-owned panel composition、read-model projection、typed intent dispatch、workspace/session layout、temporary presentation placement。
- **KNOWN IMPLEMENTATION SEARCH**: repoの`PanelLayout`／`LayoutAuthority`、product-owned `InspectorCandidate.jsx`、Document→surfaceのread-only projection、D2 Command／typed intent、Workspace profile／Project session／Transientの状態分離、`NodeDesc.params`自動生成panel fallbackを照合した。
- **CANDIDATES**: A) 既存Host panelへ既存controlを一時配置する、B) plugin parameterに限り既決`NodeDesc.params` fallbackを使う、C) 新しいcontrol schema／registry／declarative layoutを作る、D) 各実装者が仮UIを個別に作る。
- **ADOPTION ROUTE**: Aを`REUSE`する。配置とpanel geometryは既存`PanelLayout`／`LayoutAuthority`へ従属し、値のread／writeは対象laneの既存projectionとtyped routeをそのまま使う。plugin saved parameterだけはBを既決範囲で`REUSE`する。
- **REJECTED CANDIDATES**: CとD。新しい`ControlId`、`WidgetHint`、`ValueType`、公開plugin UI、汎用placement registry、第二layout owner、surfaceごとのcommand adapterを作らない。
- **THIN MOTOLII SEAM**: 既存control bindingを「最終配置待ち」としてHost panelへ列挙し、既存layout intentから表示rectを得るprivate presentationだけである。
- **THIN MOTOLII RESIDUAL**: staging panelの開閉、既存bindingの一時配置、最終配置時の除去、重複配置を防ぐguard。exact runtime targetとallowlistはcurrent mainから実装粒ごとにcompileする。
- **RETIREMENT**: final surfaceが製品routeでacceptedになったbindingはstaging配置を同じcutで除去する。互換alias、二重表示、二重writerを残さない。
- **BUILD JUSTIFICATION**: `NONE`。一般frameworkは不要で、既存Host layoutとtyped routeの薄い再利用だけで成立する。
- **BUILD: FORBIDDEN**: public UI API、Document schema、plugin contract、declarative UI language、generic control registry、第二command bus、第二layout engine。

## 3. 配置状態とowner

発注とledgerで次の語を使う。これは内部の進捗語彙であり、永続enum、公開型、Document fieldではない。

| 配置状態 | 意味 |
|---|---|
| `UNPLACED` | 操作意味は閉じているが、最終surfaceもstaging配置もまだない |
| `STAGED` | staging surfaceから既存routeを操作できる。最終UI合格ではない |
| `ASSIGNED` | exact final surfaceとplacement ownerが決まり、製品routeで接続済み |
| `STAGING_RETIRED` | final assignment後にstaging配置を除去し、二重配置がない |

値のownerは元のDocument、User settings、Workspace profile、Project session、Transientの分類から動かさない。staging panel自体の開閉、dock／detach、寸法、並びはWorkspace profileまたはProject sessionに属し、Document、journal、render recipe、plugin parameterへ入れない。panelを閉じる、detachする、layoutをresetする操作は作品意味を変えない。

一つのbindingは同時に一つのactive placementだけを持つ。final surfaceとstaging surfaceから同じwrite routeを同時に操作できる状態を移行完了として残さない。presentation上の一時配置を、値やcommandの第二ownerにしない。

## 4. stagingへ置けるもの

次をすべて満たすcontrolだけを対象にする。

1. user-facingな操作意味、対象、read projection、typed intentまたはD2 Commandが既に閉じている。
2. 成功、拒否、cancel、Undo／Redo、保存／再読込への影響が対象契約で決まっている。
3. scalar、`Vec2`、color、enum、toggle、または既存のone-shot typed actionとして、既存Host componentで表せる。
4. panel上の操作でも本来の意味を偽らず、Stage／Timeline上の空間文脈を必要としない。
5. final placementへの移動がpresentation差分だけで済む。

pluginの保存parameterは既決の`NodeDesc.params`自動生成fallbackへ送る。staging surfaceをplugin UIの別公開口にしない。User settings値を置く場合も保存ownerはUser settingsのままであり、staging panelをSettings正本にしない。

## 5. stagingへ置かないもの

- 意味、owner、command、consumer、Undo、failureが未決の操作
- Timelineのtrim／key drag、Stage gizmo、Depth Railの直接操作など、位置、hit-test、pointer capture、drag previewが製品意味の一部であるinteraction
- drag and drop、file chooser、modal transaction、keyboard focus chain、複数段gestureをpanel controlへ縮約すると別操作になるもの
- raw内部値、cache knob、debug actionなど、user-facing意味が閉じていないもの
- stagingを理由に新しいDocument field、plugin field、public `ControlId`、`WidgetHint`、`ValueType`、declarative layoutを要求するもの

これらは最終UIが未決だから止めるのではなく、presentationだけでは分離できない契約が未閉鎖だから該当edgeを返却する。

## 6. 並列発注へ追加するUI disposition

M3／M4／M5のclosed order capsuleは、UIを伴う場合に次を埋める。

```text
UI SEMANTICS: CLOSED | NOT_APPLICABLE | UNRESOLVED
FINAL SURFACE: <existing exact target> | PENDING
STAGING SURFACE: ALLOWED | FORBIDDEN
STAGING ROUTE: <existing read projection + typed intent/Command> | NONE
RETIREMENT: <final target acceptance removes staging placement> | NONE
UI GATES: <visual/focus/keyboard/a11y/human gates still pending>
```

- `UI SEMANTICS: UNRESOLVED`なら実装担当へ送らず、exact gapを`RESEARCH_RETURN`する。
- 意味が`CLOSED`、final surfaceが`PENDING`、stagingが`ALLOWED`なら、staging接続を一つの閉じたcutとして進めてよい。
- final surfaceが実在するなら直接そこへ接続し、不要なstagingを経由しない。
- 実装担当はfinal placement、製品表示名、操作意味、保存ownerを決めない。
- UI非該当laneへ形式的なpanelを追加しない。

## 7. acceptanceと状態の繰り上げ禁止

staging接続cutは少なくとも次を機械審判する。

1. stagingと将来のfinal targetが同じread projection、typed intent／Commandを使い、surface専用writerが0である。
2. 一つのaccepted gesture／actionが対象契約どおり一つのUndo単位になり、cancel／拒否／validation failureではDocument writeが0である。
3. panelのopen／close／detach／layout resetでDocument、journal、Preview／Export意味、cache recipeが変わらない。
4. active placementはbindingごとに最大1で、final assignment cut後のstaging entryが0である。
5. 新しいDocument field、公開API、plugin contract、generic registry、第二layout ownerが0である。

stagingから通常runtime routeを操作できれば、そのrouteについて`product-connected`候補にはできる。ただし状態は`FINAL_PLACEMENT_PENDING`であり、最終surfaceのvisual、density、focus、keyboard、a11y、direct manipulation、human judgmentは未実行のまま残す。staging成功だけで`product-integrated`、UI完成、M3完成、製品完成へ繰り上げない。

## 8. 実装再入場

本決定はruntime実装を開始しない。次の実装粒はcurrent mainで以下を再照合してから一契約境界へcompileする。

- 既存`PanelLayout`／`LayoutAuthority`のexact panel role、geometry consumer、dock／detach route
- 対象bindingのexisting read projection、typed intent／Command、owner、test oracle
- product-owned componentのexact source assetと、縮約copyを作らず直接再利用できるtarget
- bindingごとのfinal placement pending記録とretirementをprivateに閉じる最小target

これらのいずれかが存在しなければ、汎用registryで埋めず、該当bindingだけを`RESEARCH_RETURN`する。他の閉じたlaneは継続する。
