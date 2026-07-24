# UI runtime責任境界

状態: **責任境界・surface topology・egui製品不採用を決定**（2026-07-21、2026-07-24追補）。platform受入とdirect wgpu／Vello局所利用のrenderer採否はG0-9実機spike待ち。

2026-07-22追補: 本書のnative／Reactは**presentation runtime**の分担であり、Core、bundled first-party Host module、first-party plugin、third-party pluginの分類ではない。OS window、surface実装、architectural role、provenance / trustは[軸分離決定](reviews/2026-07-22-m3-surface-extension-axis-separation.md)に従って独立に判定する。

Motoliiの製品UIは、Reactとnativeのどちらか一方へ全面統一しない。ReactはDOMが強い領域、
native Rust/wgpuは高頻度GPU workspaceを所有する。この分割は採択済みであり、G0-9が今後比較するのは
責任境界そのものではなく、WebViewとnative surfaceを安全に同居させる実装方式である。

2026-07-24追補: eguiは標準製品runtimeの候補から外す。既存のegui shell、native texture preview、
layout投影、render worker、IME/lifecycle証拠は比較・回帰・診断baselineとして保持するが、新しい製品panel、
Timeline、Stage、theme、componentをeguiへ実装しない。React所有面をeguiへ再実装せず、native所有面を
egui widget/callbackで包まない。既存baselineの物理撤去はG0-9のplatform受入と代替診断経路が成立した後の
独立作業とし、direct wgpu枝の不合格だけでeguiを自動的に製品候補へ戻さない。再採用には本決定の明示改訂を要する。

## 1. 所有境界

以下は標準製品面の**surface所有**を示す。React面とnative面はいずれもbundled first-party Host moduleであり、公開pluginまたは第三者差替え点を意味しない。一つのOS window内へnative viewportとchild WebViewを同居させるため、`native window`／`React window`という呼称で責任を決めない。

```text
Native coordinator
├─ React / WebView chrome
│  ├─ Asset Browser
│  ├─ Inspector / parameters / forms
│  ├─ Easing trigger / accessible object-channel and pressed-disabled state
│  ├─ panel / toolbar / dialog / search / settings
│  └─ product-owned versioned UI kit
├─ native wgpu Stage
│  ├─ canonical display texture
│  └─ handle / gizmo / roto presentation overlay
└─ native wgpu Timeline
   ├─ time ruler / Z(depth) rail / row-synchronous controls
   ├─ lanes / clips / keys / playhead
   └─ selection / marquee / graph / transient preview

Native popup surface
└─ Easing frame / preset library / form / curve / handles / drag preview
```

React採択の理由は、CSS layout、form、text input、IME、a11y、component資産、hot reload、
Storybook/Playwright、LLM生成容易性、community作者の入口である。Canvasやbrowser WebGPUを
React componentへ包めること自体はReact所有の理由にしない。

この作者入口は[Creator / Developer連続体](reviews/2026-07-22-creator-developer-continuum-decision.md)の一部である。製品作者だけが理解できる専用UI言語を増やさず、creatorが既存componentをinspectし、fixture上で変え、testし、将来の公開境界が定まった後にcommunity成果へ進める余地を保つ。ただしproduct packageへの到達可能性と、untrusted pluginへ同じorigin／process／権限を与えることは同義ではない。

StageとTimelineは、一つのzoom/scroll/focus/gestureへ高頻度同期する要素を領域内で分割しない。
特にtrack headerだけをReact、key surfaceだけをnativeにする構成は採らない。Reactは外側のtoolbar、
menu、popover、parameter編集を所有する。Timeline dock左の`KEYS / LAYERS`切替とAlign、Stagger、
Stretch等のtool panelはtrack headerではなく、mode/formを選ぶReact chromeとする。一方、各rowと同じ
scroll/zoom/selectionへ同期するS/M rail、time ruler、bar、key、playhead、およびZ軸Timeline / depth railは
nativeが一体で所有する。

Stage panel自体は複合panelとする。Preview canvas、Output Frame、object bounds、path、handle、gizmoはnative wgpu
viewportが所有し、panel header、Fit / magnification / view mode、transport button、timecode、quality/status表示は
React chromeが所有する。React帯とnative viewportは同じStage panelの子として一緒にdock / detach / resizeし、互いに
重ならないopaque rectangleへ配置する。透明WebViewをPreview上へ被せず、pointer captureが必要なscrubや直接操作は
nativeへ残す。React controlはtyped command intentだけをHostへ送り、playback、playhead、selectionの正本を持たない。
表示更新はHostの最新snapshotを一方向に投影し、frameごとのmessage backlogを作らない。

将来のKBar型command stripや追加transport controlもこのReact chrome seamへ置ける。ただし特定library、JS component、
command配置、第三者拡張APIを現時点の製品契約へ焼かず、既存`CommandId` / typed intent境界を再利用する。

ただし高頻度curve操作を伴うEasing popupは一般popoverの例外である。
[native Easing popup受入契約](reviews/2026-07-22-m3-native-easing-popup-acceptance.md)に従い、Reactは入口と
object・channel・pressed/disabledのaccessible stateだけを所有し、visible summary chromeは別のUI判断まで
実装しない。native wgpuはpopup frame、preset/user library、数値form、curve、grid、handle、
drag previewを一体で所有する。Host coordinatorはnative popup windowのanchor、z-order、focus、dismiss、
DPI/layout epochとUser settings codecを所有する。Reactモックから幅、枠、余白、情報階層を借りるが、
React/nativeへcurve、preset thumbnail、Undoを二重所有させない。

Web所有panel内の小さなvisualizationは、DOMのform/a11y/component資産が主体で、native側へ
semantic stateやinteraction stateを複製しない場合に限ってCanvasを使える。Stage、Timeline、roto、
大量object/keyの直接操作面をこの例外へ入れない。

### 1.1 React実装資産の所有

React所有面は固定モックを外観だけの参考にして再実装せず、
[React製品資産の直接移管契約](reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)に従って
component、CSS、stable ID、ARIA、Storybook、Playwrightをproduct packageへ直接所有移管する。
モックはproduct exportをfixtureで組み立てるconsumerへ反転し、mock/productへ同じcomponentの独立copyを残さない。

交換するのはmock固有state、legacy HTML/script bridge、fixture adapterであり、Hostのrevision付きprojectionと
typed intentへ一方向接続する。正しい独立React sourceが無いlegacy領域は、固定モック内で同形React化とparityを
先に完了する。縮約component、skeleton、CSS後追い修理を製品面の代替にしない。

このsource ownershipは内部実装の決定であり、DOM/CSSをDocument、永続形式、公開API、community互換契約へ
昇格するものではない。またproduct-owned React packageの成立だけでWebView Host、sandbox、platform受入を
合格にしない。

## 2. native surfaceは汎用UI toolkitを再実装しない

native側で自作するのはMotolii固有のdomain surfaceであり、flex、form、text editor、dialog、theme、
community runtimeを備えた汎用widget frameworkではない。

```text
platform input adapter
        ↓
NormalizedInput
        ↓
headless interaction kernel
├─ pointer lifecycle / capture / drag threshold
├─ pan / zoom / marquee / multi-selection
├─ snap候補照会 / edge scroll / cancel / focus loss
└─ deterministic gesture state machine
        ↓
toolkit非依存projection / transient preview
        ↓ release
D2 command / single writer / Undo 1回
        ↓
direct wgpu primitive batch + 必要箇所だけVello
```

headless libraryや既存実装は、新規自作より先に検索して使う。ただし採用単位は次を全て満たすものに限る。

- window、event loop、renderer、scene graphを所有しない
- 独自Document、selection正本、history、Undoを持ち込まない
- pointer cancel、focus loss、capture喪失を入力として扱える
- 固定入力列をwindow/GPUなしでdeterministicに再生できる
- library固有型をDocument、domain公開API、plugin契約へ出さない
- adapterで交換可能であり、React/nativeの状態二重所有を要求しない

外部へ委ねられるのはpointer lifecycle、drag開始距離、viewport操作、geometry、text shaping/layout、
path rasterization、a11y基盤である。Motoliiが所有するのはclip/key/objectの意味、RationalTime、snap優先度、
Edit Space、selection、transient preview、D2 commit、Undo/Cancelである。

新規nativeコードがclip/key/snap等のdomain語彙ではなく、汎用widget/layout/style/theme語彙を公開し始めたら
UI framework再発明として停止する。

## 3. rendererの現在位置

- 大量rect/line/key/gizmo: core-owned device上のdirect wgpu primitive batchが第一候補
- 複雑path、curve、roto、採択済みglyph描画: Velloを局所rendererとして再利用
- font discovery/shaping: 採択済みfontique + harfrust。単純layoutで不足する実例が出た時だけParleyを比較
- accessibility: AccessKitを基盤にし、全keyをnode化しないbounded semantic projectionを作る
- egui: 製品runtimeには不採用。成立済みbaseline/debug・回帰比較として、G0-9のplatform受入と代替診断経路が閉じるまで削除しない

direct wgpuは採択候補であって、headless benchmarkだけで製品renderer確定とはしない。Velloの「局所」は
呼出頻度でなく、所有語彙、描画面積、primitive数、allocation、GPU時間で判定する。毎frame呼ばれても
scene/input/Documentを所有せず予算内なら境界違反ではない。

## 4. surface topologyとcoordinator境界

React/native間を流せるのは、typed bounds、domain intent、read-only semantic projection、focus移譲、
pointer capture境界、DPI epoch、bounded a11y projectionである。DOM event、CSS px、Canvas scene、
toolkit object、raw GPU handleをDocumentやplugin契約へ流さない。

通常windowはcore-owned device/queueへ接続した**トップレベル`wgpu::Surface`を1枚だけ**持つ。StageとTimelineは
同じsurface texture、同じframe submission内のviewport/scissor rectangleとして描く。React chromeはdock/tab
stackごとのopaque child WebView rectangleとしてOS compositorが上へ合成する。1 panelごとにWebViewを増やさず、
同じversioned React bundleをrole付きで起動する。native rectangleをまたぐDOM popupだけ、必要寸法の一時opaque
child WebViewを使える。

coordinatorは単調増加`layout_epoch`ごとにchild WebViewのlogical bounds、native viewportのphysical bounds、
hit-test transform、a11y rectangleを一括反映する。古いepochを部分反映せず、CSS px、AppKit point、Win32 pixel、
DPIをDocument、D2、plugin契約へ流さない。

Stage/Timeline別の複数surface、全画面transparent WebView、browser/native GPU texture共有、Windows
CompositionControllerは正規経路にしない。通常のwindowed child WebViewがWindows実機で修正不能な失敗を再現した
時だけCompositionController、別window、最後にCEF OSRの順で再審判する。証拠と停止線は
[surface topology決定](reviews/2026-07-21-ui-surface-topology-decision.md)を正とする。

### 4.1 built-in WebView Hostの再入場条件

[historical React / WebView lineage回収](reviews/2026-07-23-historical-react-webview-lineage-recovery.md)で、
現行treeから失われていたbuilt-in Host foundationの不変条件を再採択した。これはproduct-owned React面を載せる
bundled first-party Host moduleだけの条件であり、community custom UIや公開plugin runtimeを許可しない。

- R0〜R6の固定source直接移管とmock consumer化を先行し、docs、mock、dev server、diagnostic routeをrelease sourceにしない
- productionはnetwork 0の決定的offline bundleと閉じたasset manifestから起動し、CDN、localhost、HMR、file URL、未掲載assetを拒否する
- Host所有のclosed typed schemaからWeb側型とconformance vectorを生成し、role、direction、instance epoch、sequence、size/depthをfail closedで検査する
- IPC callbackはdecode、session gate、bounded event-loop inboxへのenqueueだけを行い、Document/D2へreentrant mutationしない
- Host coordinatorがlayout/instance epoch、focus、reload、process lost、bounded retryを所有し、再生成時は最新Host snapshotから再投影する
- navigation、new window、download、form、外部network、任意evalは既定denyとし、必要能力は後続のtyped capabilityで個別追加する

旧H1 exact contractのpackage、dependency version、4 role、wire field、byte/depth/queue上限、custom schemeは現在の
実装契約ではない。current source assetとprojection / intent、platform adapterが固まった後にclosed contractを
再作成し、現行依存と一次資料で値を再固定する。React package成立だけでWebView/native統合、platform受入、
community sandboxを合格にしない。

## 5. 決定済みと未決の境界

### 決定済み

- React/WebView chrome + native Stage/Timelineという責任分担
- StageはReact header/transport + native Preview canvasを一つのdockable複合panelとして扱う。透明overlayは使わない
- ReactはDOMの優位性がある領域へ使い、高密度Canvas workspaceのownerにしない
- Timelineの`KEYS / LAYERS` tool panelはReact、time/Z軸に同期するrail・bar・key・playheadはnative
- Stage/Timelineのlayout、hit-test、interaction modelはtoolkit/renderer非依存に置く
- native interactionはheadless部品を優先し、Motolii固有意味だけを自作する
- React/nativeの両側へselection、snap、Undo、semantic stateを二重所有させない
- Hostとcommunityがcomponent/test語彙を再利用できる長期原則。ただし公開runtime、origin、process、権限、window topologyの同一化ではない
- 1 top-level wgpu Surface + Stage/Timeline 2 viewport + opaque child WebView islandsという通常window topology
- Timeline / Stage / Browser / Inspectorはbundled first-party Host moduleであり、surface runtimeからplugin分類を推論しない
- eguiは製品runtimeへ採用せず、新しい製品surfaceを実装しない。既存shellは比較・診断baselineとして撤去条件成立まで保持する

二重所有の禁止は最適化方針ではなく不変条件である。Document編集はD2 single writerだけ、Transient selectionと
sessionはHost coordinatorだけが所有する。React Inspector、native Preview、native Timelineは同じrevision付き
snapshotのread-only projectionであり、独自writer、独自Undo、surface別selection正本を持たない。reload、detach、
crash復旧では最新Host snapshotから再投影し、surface間の双方向state syncを追加しない。

通常製品panelのplacement能力もrole別に分けない。Stage / Timeline / Graph / Browser / Inspectorは同じdock treeで
tab化、horizontal / vertical split、divider resize、top-level detach / re-dock、window resizeを行える。modal、popover、
toastはdock panelではない。headless layoutはlogical rectangleだけを計算し、window位置、DPI、split比をDocumentへ
書かず、panelを移してもHost snapshot、selection、Undo ownerを増やさない。isolated fixtureの合否と未証明範囲は
[detachable panel / multi-window契約](reviews/2026-07-22-m3-detachable-panel-window-contract.md)を正とする。

### G0-9L限定確定 / G0-9D未決

- macOS/Windowsのfocus、DPI、resize、z-order、pointer capture、surface/device lost、a11y tree接合の受入
- direct wgpu枝とdirect wgpu + Vello局所pass枝の製品採択
- egui baselineの撤去条件を満たすplatform証拠と代替診断経路
- Linuxでsystem WebViewを使うかCEF比較へ進むか

G0-9Lは固定した主開発Mac構成のplatform prerequisite evidenceだけを限定確定し、G0-9DはWindowsと追加hardwareを含む
Distribution Readyを判定する。G0-9L合格後もparent G0-9、G0-9D、G0-6H、G0-3は未完了のまま残し、
egui baselineを削除しない。G0-9LはW0b、H1b、Motolii Studio Preview、通常製品window結合を解禁しない。
G0-9Lは固定Macのplatform prerequisite evidenceに限って実機合格したが、これらの製品結合を実装しない。

### G0-3 / GAP-13で未決

- first-party pluginとthird-party pluginのcustom UI公開境界
- community UIのsandbox、権限、互換、配布、署名、障害隔離
- product-owned component/test語彙のうち、何を公開kitとしてversion保証するか

G0-9のplatform証拠はG0-3へ入力できるが、G0-9合格だけでplugin UI公開契約を許可しない。逆にplugin sandbox未決を理由に、依存を満たしたbundled first-party製品surfaceまで第三者pluginと同じ停止線へ置かない。

未決項目を理由に決定済み責任境界やegui製品不採用を再び「全面egui対全面React」の比較へ戻さない。逆に責任境界の決定を、
未合格のWebView/native合成やplugin公開契約の実装許可として扱わない。

## 6. 証拠と後続

- [React / WebView再選定](reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md)
- [native surface renderer再選定](reviews/2026-07-21-native-surface-renderer-reselection.md)
- [拡張サーチ](reviews/2026-07-21-native-surface-renderer-extended-search.md)
- [Fable反対側レビュー](reviews/2026-07-21-native-surface-renderer-counter-review.md)
- [Fable伸長レビュー](reviews/2026-07-21-native-surface-renderer-growth-review.md)
- [surface topology決定](reviews/2026-07-21-ui-surface-topology-decision.md)
- [G0-9部分スパイク](spikes/g0-9-ui-runtime.md)と[確認点マトリクス](spikes/g0-9-verification-matrix.md)

実装合否と停止線は[M3仕様 G0-9/U3a](specs/M3-ui-integration.md)を正本とする。本書は設計責任を固定するが、
新しい公開API、Document field、永続layout形式、plugin GPU/UI契約を許可しない。
