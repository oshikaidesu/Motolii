# native surface renderer反対側レビュー（Fable回答・2026-07-21）

状態: **比較中**。[再選定](2026-07-21-native-surface-renderer-reselection.md)§6の6問と
[拡張サーチ](2026-07-21-native-surface-renderer-extended-search.md)§9の5問へのFable回答。
結論: **第一候補（direct wgpu primitive batch + Vello局所 + React複合）を覆す反証は構成できなかった**。
ただし調査側に弱点3点があり、spike設計へ反映しない限り「比較した」とは言えない（§3）。
本回答は助言であり、採否はMotolii fixtureと実機証拠で行う。Document/API/plugin契約を発明しない。
伸ばす方向の対レビューは[伸長レビュー](2026-07-21-native-surface-renderer-growth-review.md)を参照。

## 1. 再選定§6への回答

1. **UIフレームワーク再発明にならないか**: リスクは実在するが、範囲は監視可能である。Timeline/Stageに
   必要なのはretained projection（layout/cull/hit-test）、instance batch描画、gesture state machineの
   3つで、Motolii固有はdomain semantics（clip/key/lane/snap/D2 commit）だけ。非固有部分—text
   shaping/layout、focus管理、a11y—は既決stack（fontique/harfrust、AccessKit）とparley候補で外部化
   できる。**検出条件を置く**: 新規コードがdomain語彙（clip/key/snap）でなくUI語彙（widget/layout/
   style/theme）を公開し始めたら再発明が始まっている。Zed/Warpの「少数primitive + domain専用」先例は
   この範囲で成立を示す。処分: 命題維持、検出条件を停止線運用へ追加。
2. **2 surfaceの最小安全構成**: wgpu 29では単一Instance/Adapter/Device/Queueへ複数Surfaceが公式
   パターンで、Graphite desktopも同構成である。最小構成は (a) device/queue単一・surface別の
   swapchain config、(b) 復旧をsurface単位（`Outdated`→reconfigure、`Lost`→surface再生成）、device
   lost診断のみ一元、(c) Stage/Timelineのencoderを独立submitとし**同一frame内の相互texture依存を
   作らない**、(d) 片surfaceのacquire失敗が他方のpresentをブロックしない独立frame pacing。macOSは
   同一window内の複数CAMetalLayer sublayerで先例十分。Windowsのchild HWND vs compositionと、DPI
   混在monitor間dragが実測必要。処分: spike測定項目として具体化。
3. **Vello局所境界の一元化**: 条件付きで可能である。Velloはtextureへ描きsurfaceを所有しないため、
   surface lost対応はMotolii surface層へ一元化でき、Vello Rendererはdevice lost時のみ再生成——境界は
   保てる。Glifo atlasはRenderer内部保有でcache寿命=Renderer寿命に一致する。**破れ検出条件**: 局所
   passの起動が毎frameに達したら「局所」の定義が破れている（Timeline通常textの流入が典型経路）。
   Vello 0.9のGPU memory allocation未完はscene複雑度スパイクで予算超過し得るため、roto/curve編集の
   最悪ケースfixtureをspikeへ含める。処分: 命題維持、破れ検出条件と最悪ケースfixtureを追加。
4. **typed bridgeに不足する最小event**: bounds/intent/semantic projectionに加え、(1) focus移譲の
   双方向handshake（要求→承認→完了。奪取禁止）、(2) pointer capture境界通過の明示event（native
   drag中のWebView hover抑制）、(3) DPI/scale変更のepoch同期（boundsとscaleの更新原子性）、(4) a11y
   focus/announce転送、(5) IME開始/終了。ただし**より小さい対策が先**: nativeがtext入力とform focusを
   持たない現行分割を契約化すれば(5)はWebView側へ閉じ、(1)も単純化する。eventを増やす前に設計で
   減らす。処分: 最小5種を上限とし、設計側削減を優先。
5. **eguiがdirect wgpuへ勝つfixture**: 現証拠には存在しない。ただし調査側の公平性欠陥を指摘する:
   timeline-benchはheadless・text無しで、eguiの歴史証拠はwindowed・text込み——**同条件比較が一度も
   行われていない**。spike Aでwindowed・text込みの同一fixtureをegui枝でも1回測り、baseline数字を
   持ってから不合格を語るべきである。採否を戻す条件: egui枝がp95で勝ち、かつ停止線（loop内resource
   生成禁止・poll(Wait)禁止）を同時に満たす場合のみ。処分: egui同条件再測定をspikeへ追加。
6. **child surface不成立時にCPU bridge/透明WebViewへ逃げない構成**: 拡張サーチで実在回答（Graphite
   のCEF OSR + 共有GPU texture）が出たが、これは最終段であり中間段を飛ばさない。撤退順序:
   (1) 不成立原因がz-order/clippingなら**window分離**（Timeline別window化は再選定が既に許容）、
   (2) 片OSのみ不成立なら**platform非対称**（macOS=WKWebView sibling、Windows=WebView2 composition
   等）、(3) 両OSでsibling不成立が確定した時のみCEF比較spikeを起票。処分: 撤退順序を序列化。

## 2. 拡張サーチ§9への回答

1. **GraphiteのTauri放棄理由の転移**: 具体理由は一次資料で未特定（LWN snippetは「insurmountable
   technical incompatibility」のみ）。**事実欠損のまま自構成へ転移させない**。状況証拠上も転移条件が
   異なる: Graphiteはwindow全面web chromeの中へnative viewportを合成する（重複合成）のに対し、
   Motoliiは非重複sibling矩形で合成要求が緩い。spike前判定は不能だが、Graphiteのissue/PR履歴から
   放棄理由を特定する小コスト先行調査（1時間規模）には価値がある。処分: 転移未証明と記録、先行調査を
   任意タスク化。
2. **CEF予備枝との分岐判定**: 分岐点はwindowed spike実機合格条件のplatform固有項目に紐づける——
   macOS非重複合成、Windows child surface、focus traversal、resize/DPIのいずれかが「修正不能なOS
   挙動」型で落ちた時のみCEF比較spikeを起票する。それまでのCEF事前投資は0（調査記録のみ）。工数上限は
   各platform 1週間の合成spikeで判定可能。処分: 判定条件を合格条件へ紐づけ、事前投資0を明記。
3. **CustomPaintSourceから写す最小界面**: lifecycle 3操作のみ——(1) device/queue handle受領
   （resume）、(2) 出力textureの登録/解除、(3) suspend（device lost/背景化）。**写してはいけない**の
   はBlitzのdocument合成モデル（Vello scene内へ埋込む方向）で、Motoliiは合成方向が逆（native surface
   が主、Velloが局所pass）。借りるのはlifecycle契約の形であってAPI形状ではない。処分: 3操作のみ採用。
4. **focus traversalの所有**: window単位のfocus ringは**native coordinator一択**である。AccessKit
   graft treeもWebViewも自tree内focusしか知らず、tree間巡回はwindow全体のsemanticsだから、どちらかの
   treeへ埋めると他方が不可視になる。coordinatorがring（WebView A→native Timeline→WebView B）を
   所有し、各treeへenter/leaveを通知する。未検証はWebView2/WKWebViewがhost主導のfocus譲渡をどこまで
   許すか（tao #208の初回click問題がここへ刺さる）。処分: coordinator所有を勧告、譲渡APIをspike項目へ。
5. **wgpu 29固定の期限**: 日付でなくイベントで置く——(1) 採択済みVelloの保守版がwgpu 30必須になった
   時、(2) wgpu 29系へのsecurity fixが止まった時、(3) windowed spike完了時（spikeを跨ぐbumpは測定を
   無効化するため禁止）。「30が出たから上げる」は理由にならない。処分: イベント駆動の期限設定、
   vello次版のwgpu要求をwatchlist再確認トリガーへ追加。

## 3. 調査側への反証（弱点3点）

1. **egui baselineの同条件比較が未実施**: §1-5の通り。headless数値とwindowed歴史証拠を並べたまま
   spikeへ進むと、比較でなく確認作業になる。spike Aへegui同一fixture測定を1回入れることで解消できる。
2. **CEF予備枝の過大評価リスク**: Graphite放棄理由が未特定のまま「実在回答」と書くと、child WebView
   spikeの失敗時にCEFへ倒す圧力が根拠より先に立つ。撤退順序（§1-6）を先に固定したのはこのためである。
3. **「同型出荷例ゼロ」の過大解釈リスク**: 非重複sibling合成はWKWebView/WebView2/CAMetalLayerという
   OS標準部品の組合せであり、新規性はcoordinator層（focus/bounds/a11y同期）に限定される。「誰も出荷
   していない＝不可能に近い」ではなく「合成部品は枯れているが統合の証拠が無い」と読む。悲観でなく
   spike優先度の根拠として使う。

## 4. 最終境界

- 第一候補（direct wgpu + Vello局所 + React複合）: 維持。反証は構成できなかった
- 追加した検出条件: UI語彙輸出の監視（§1-1）、局所pass毎frame化の検出（§1-3）、撤退順序（§1-6）
- spikeへ追加する測定: egui同条件fixture、roto/curve最悪ケース、focus譲渡API、DPI混在drag
- 採否の正本は本回答でなく、[再選定](2026-07-21-native-surface-renderer-reselection.md)の自動/実機
  合格条件を満たすspike証拠とする
