# M3: UI統合

ステータス: **ACTIVE / React Native + Rust/rust-skia + wgpuへ再基線化、製品移行開始前**（2026-08-07）

M3は、M0〜M2で成立したDocument、D2 single writer、Undo／Redo、journal、projection、render、playbackを、制作に耐える一つの製品UIへ接続する段階である。UI技術の比較を続ける段階ではない。

runtime責任の正本は[UI runtime責任境界](../ui-runtime-architecture.md)、採択根拠と旧routeの処分は[2026-08-07 runtime再基線決定](../reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md)を読む。R0〜R4を実在target、owner、oracle、依存へ分解する現在のdispatch authorityは[M3 RN runtime実行地図](../m3-rn-runtime-execution-map.md)とする。

## 1. 製品完成線

M3の完成は、個別fixture、isolated probe、renderer benchmark、特定gestureのgreenではない。通常製品routeで利用者が次を連続して行えることを問う。

```text
project open/new
  → media / object discovery
  → StageまたはTimelineへ配置
  → Stage / Timeline / Inspectorが同じrevisionを表示
  → selection / parameter edit / key / move / trim / snap / curve
  → seek / playback / pause
  → Undo / Redo / reopen
  → save / export / cancel / recovery
```

最初の移行oracleは既に意味が閉じているVS-1とする。

```text
Browser Rectangle
  → Place
  → D2 single writer
  → RN Inspector + rust-skia Timeline + Stage が同じrevision / LayerIdを表示
  → Undo
  → Redo
```

旧winit/WebView/direct-wgpu/Vello routeのVS-1成功は意味oracleであり、新RN routeの成功を代用しない。

## 2. 標準runtime

| 領域 | owner |
|---|---|
| shell、dock、tabs、Browser、Inspector、settings、forms、text、dialogs | React Native |
| Timeline | Rust headless interaction + rust-skia |
| Curve Editor | Rust headless interaction + rust-skia |
| Stage preview | wgpu |
| Stage grid／path／gizmo overlay | rust-skia、wgpu final composite |
| Document／D2／Undo／journal／projection／playback／media／render | Rust core |
| native view／surface／capture／focus／DPI／lifecycle | platform adapter |

React Nativeは既存React mockのconceptと通常panel資産を引き継ぐが、DOM/CSS/WebView bridgeを製品runtimeへ残すことを要求しない。TimelineとStageをRN component treeの大量objectへ分解しない。

## 3. M3の不変条件

1. Document mutationはD2 single writerだけが行う。
2. 全surfaceは同じrevision／generation付きsnapshotをread-only投影する。
3. selection、playhead、gesture、focus、panel layoutをDocumentへ混ぜない。
4. drag中write 0、release時commit高々1、cancel／stale／invalidはwrite 0。
5. RN/native間へper-frame、per-object、per-pointer-move message backlogを作らない。
6. renderer、toolkit、platform型をDocument、persistence、plugin public contractへ漏らさない。
7. OS window、native component、GPU device、pointer owner、Undo ownerを暗黙に増やさない。
8. macOS先行実装でもlogical／physical、lifecycle、semantic ABIをWindows移植可能に保つ。
9. fixed mock、probe、benchmark、isolated testを製品route completionと呼ばない。
10. 旧presentation routeは新route合格前に削除せず、新route合格後に恒久二重所有を残さない。

## 4. interaction backbone

```text
OS/RN input
  → platform adapter
  → normalized event
  → headless Rust gesture
  → transient projection / dirty canvas
  → terminal typed intent
  → existing D2 prepare/apply
  → journal / Undo
  → published snapshot
  → RN / Timeline / Curve / Stage reprojection
```

Timeline、Curve、Stageはpointer lifecycle、capture、cancel、viewport transform、marquee、multi-selectionのheadless語彙を再利用できる。ただしdomain meaningを一つの巨大gesture frameworkへ抽象化しない。clip trim、curve tangent、Stage gizmoはそれぞれ既存command／semantic ownerへ接続する。

Inspectorの数値・text編集も同じterminal intentとsingle writerへ接続する。canvas操作だけを特別扱いしない。

## 5. Stage

Stage native componentは次の一つのpresentation chainを持つ。

```text
render/media result
  → wgpu preview texture
  → rust-skia transparent overlay (dirty時だけ更新)
  → wgpu composite
  → native surface present
```

overlay対象:

- output frame、safe area、grid、guide、snap補助
- selected／visible object bounds
- active gizmo、group root gizmo
- path、control point、camera／depth補助

毎frame CPU readback、全object full gizmo、第二GPU device、第二event loopを採らない。100〜500 gizmoはstress上限であり、通常はsemantic importanceとvisibilityで間引く。

直接操作はcanonical display transformの同一epochでdrawとhit-testを行う。resize、scale change、occlusion、unmount/remount、surface/device lostをlifecycleとして扱う。

3D gizmoは汎用UI部品ではなく、active camera、world/object transform、depth／pickingと同じStage viewport境界に属する。したがってRN/Skiaのpanel側へ操作意味論を移さず、Rust headless interactionが同一camera／display transform epochで軸・平面のhit-test、capture、drag previewを所有する。表示は通常どおりcanonical output外のrust-skia overlayでよいが、3D handleの描画だけをnative GPU presentation overlayへ差し替える余地を残す。この差し替えはrendererの選択であり、selection、preview、D2 commitの所有者を増やさない。

## 6. Timeline

TimelineはAbleton ArrangementやNLEに近い高密度canvasである。track headerは左に置き、time directionとtrack directionのscrollを分離する。

必須操作:

- continuous scroll、cursor-centered zoom、fit/visible-range navigation
- primary／multi／marquee selection
- playhead seek、drag scrub
- clip move、trim in/out、lane move
- snap、guide、modifier override
- group layer展開／折畳みとgroup selection
- edge scroll、cancel、outside release、focus loss
- semantic zoomによるlabel、thumbnail、waveform、handleの段階省略

visible projectionをboundedにし、offscreen clip/keyを描画・AX node・RN componentとして生成しない。1000 rich clipは安全余裕のprobeであり常時フル情報表示要件ではない。

既存`TimelineProjection`、typed identity、move／trim command、snap contract、Undo oracleを再利用する。旧direct-wgpu/Vello rendererの見た目やcode structureを新rendererへ強制しない。

## 7. Curve Editor

Curve EditorはTimelineと同じrust-skia canvas familyとして実装する。

- multiple curves、active interval、key／tangent selection
- add/remove key、marquee、pan、zoom、fit
- Bezier handle、linked/broken tangent、preset
- drag中transient preview、release時Interp／key command高々1回
- same revision、same channel identity、stale rejection
- selected／visible中心のbounded accessibility projection

Easing popupを別native windowへ固定しない。RN popover／panel chrome内のnative Curve component、dockable Curve panel、inline editorのどれでも同じsemantic contractを使える。window topologyはpresentation choiceであり、curve state ownerを増やさない。

## 8. React Native shellとReact資産移行

React mockから直接再利用するもの:

- information architecture、panel roles、labels、icon meaning
- component boundaries、empty/loading/error/disabled states
- Browser／Inspectorのread modelとtyped intent概念
- fixture、visual oracle、interaction test scenario

変換するもの:

- HTML element → RN primitive／product component
- CSS layout → RN StyleSheet／layout component
- DOM event → RN callback／native component typed event
- WebView bridge → Host-owned RN native module／component contract
- browser Canvas → native rust-skia component

mock data、legacy fixture adapter、DOM stable ID、localhost、HMRはrelease sourceにしない。product componentを正本とし、必要ならmock側がproduct componentをfixtureとして消費する。

## 9. product custom panelとplugin UI

bundled first-party panelはRN componentとして実装できる。これは既存React資産と将来のproduct custom panelを保つための内部拡張軸である。

third-party plugin UIは別gateとする。v1既定はHost-generated parameter panelと宣言的hint／gizmoであり、任意JS bundle、同process native code、network、eval、raw GPU textureをRN採択から自動的に許可しない。sandbox、permission、version、crash isolation、distributionはG0-3 / GAP-13で閉じる。

### UI配置保留と未配置control staging surface

M3／M4／M5の接続で、user-facingな操作意味、read projection、typed intentまたはD2 Command、owner、Undo／failureが閉じている一方、最終surfaceの配置だけが未決の場合は、[UI配置保留決定](../reviews/2026-08-09-ui-placement-deferral-staging-surface-decision.md)に従いHost-owned staging surfaceへ一時配置してよい。これはSettings、debug panel、万能Inspector、plugin UI frameworkではなく、既存`PanelLayout`／`LayoutAuthority`と既存control routeを再利用する任意表示panelである。panelの開閉、dock／detach、寸法はWorkspace profileまたはProject sessionに置き、Document、journal、render recipeへ入れない。

stagingは値やCommandのownerにならず、一つのbindingにactive placementを一つだけ持つ。final surfaceがacceptedになったcutでstaging配置を除去し、同じread projectionとtyped routeのままpresentationだけを移す。操作意味、owner、command、consumer、Undo、failureのいずれかが未決ならstagingへ仮置きせず、そのedgeを`RESEARCH_RETURN`する。

Timeline trim／key drag、Stage gizmo、Depth Rail direct manipulation、drag and drop、pointer captureなど、位置やgesture自体が操作意味であるinteractionは対象外とする。staging routeのgreenはruntimeの`product-connected`候補に限り、final placement、visual、density、focus、keyboard、a11y、human judgmentを完了扱いにしない。

## 10. 実装wave

以下は利用者成果のwaveであって、一wave一発注を意味しない。施工境界、並列可否、現在状態は[M3 RN runtime実行地図](../m3-rn-runtime-execution-map.md)で判定する。

### Wave R0 — product runtime seat

- RN application/window lifecycle
- Rust Host native module/component contract
- revisioned snapshot、typed intent、diagnostic envelope
- one macOS Stage placeholder native component
- zero Document semantics change

出口: offline Release appが起動し、resize、focus、unmount/remount後も同じread-only snapshotを表示する。

### Wave R1 — VS-1再閉鎖

- Browser Rectangle conceptをRNへ移す
- Stage wgpu preview + rust-skia overlay
- rust-skia Timeline read projection
- RN Inspector read projection
- Place、Undo、Redoを既存D2へ接続

出口: 通常RN product routeでVS-1を一つのLayerId／revisionで完走する。

### Wave R2 — 制作操作

- Timeline selection／scrub／move／trim／snap／lane
- Stage selection／gizmo／group／grid／snap／path
- Inspector parameter edit／key
- Curve Editor

出口: 一つのfixtureで同じobjectを三面から編集し、Undo／Redo／reopenできる。

### Wave R3 — project workflow

- media import、save／save as、recovery
- playback／audio clock／degraded preview
- export settings／progress／cancel／atomic artifact
- keyboard operations、clipboard、menu、DnD

出口: new/openからexportまで通常routeで完走する。

### Wave R4 — platform/distribution

- macOS human IME／VoiceOver／DPI／device lost
- Windows RNW Component View／Composition/DX12／capture／focus／DPI／NVDA
- arm64/x64 artifacts、offline resources、license notice、crash recovery

出口: Local AlphaとDistribution Readyを別々に判定できる。

## 11. gate

| Gate | 必須証拠 | synthetic代用 |
|---|---|---|
| automated semantic | deterministic event sequence、one commit、zero-write reject、revision一致 | 可 |
| renderer | raw CPU/GPU/upload/frame値、bounded visible set、resize resource | 可 |
| macOS product | real RN app、surface present、outside release、focus、remount、recovery | 不可 |
| human | IME composition、keyboard-only、VoiceOver、drag feel、visual density | 不可 |
| Windows product | real RNW app、Composition/DX12、DPI、capture、focus、device lost、NVDA | 不可 |
| distribution | signed/package artifact、offline、notice、reopen/crash recovery | 不可 |

Windows共通coreのcompile成功はplatform根幹riskを下げるが、Windows product gateの代用ではない。Windows gateはmacOS操作体系の実装開始を止めないが、cross-platform完成、旧route全面撤去、Distribution Readyを止める。

## 12. license

rust-skiaはMIT、SkiaはBSD 3-clause系である。commercial runtime fee／copyleftはない。配布artifactへ両license noticeとdisclaimerを含め、依存更新時にnotice closureを検査する。

## 13. 非目標

- RN／Rust／Skiaのどれか一つへ全面統一
- WebView islandsの問題を別の埋め込みbrowserで再現すること
- Timeline clip／keyを大量RN componentとして表現すること
- native側でdock、form、theme、text editor、汎用widget toolkitを作ること
- direct-wgpu/Vello／eguiを即時削除すること
- mock、probe、focused greenをM3完成と呼ぶこと
- Windows実機未通過を黙ってmacOS結果へ一般化すること

## 14. 現在地

- runtime選定: `DONE`
- isolated dense canvas／real-surface／RN native component probe: `DONE / PRODUCT NOT IMPLEMENTED`
- Windows common Rust target compile: `DONE`
- RN product runtime seat: `READY-RECHECK / UNINTEGRATED CANDIDATE`
- VS-1 RN route: `WAIT_R0`
- Stage／Timeline／Curve product cutover: `WAIT_R1`
- macOS human gate: `EXTERNAL_GATE_PENDING`
- Windows product gate: `EXTERNAL_GATE_PENDING`
- third-party custom UI: `G0-3 / GAP-13 PENDING`
