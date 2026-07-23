# M3 G0-9 platform gate段階化

作成日: 2026-07-23
状態: **決定 / Grok反対側レビューP0/P1=0 / 製品粒は独立停止**

## 1. 目的

G0-9の責任境界とsurface topologyは変更せず、platform受入の証拠を次の二段階へ分ける。

- 現在所有する主開発Macだけに限定したplatform prerequisite evidenceを集めるlocal gate
- Windowsと追加hardwareを含むDistribution Readyを判定するdistribution gate

現在のG0-9は両者を一つの完了条件へ束ねているため、Windows実機と追加monitorを入手するまで
主開発Macのplatform prerequisite evidenceも閉じられない。一方、Macの部分証拠だけでWindows、異DPI、追加hardwareを
合格にするとplatform品質を免除してしまう。本改訂は親gateを完了扱いにせず、検証済み構成だけへ
限定確定する。これは製品粒を解禁しない。

## 2. gate IDと状態

| ID | 役割 | 完了時に許可すること | 完了時にも許可しないこと |
|---|---|---|---|
| `G0-9` | React chrome + native Stage/Timeline製品surfaceの親gate | `G0-9L`と`G0-9D`の証拠を束ね、対応platform全体の合否を記録する | 子gate未完了での全platform合格 |
| `G0-9L` | 固定した主開発Mac構成のlocal platform prerequisite evidence gate | 固定構成でplatform prerequisite evidenceを限定確定する | W0b、H1b、Motolii Studio Preview、通常製品window、Windows/追加hardwareへの外挿、Distribution Ready、egui baseline削除 |
| `G0-9D` | Windows・追加hardware・配布対象Macのdistribution gate | 対応platform matrixを閉じ、Distribution Readyを判定する | 未所有hardwareのsynthetic PASS、G0-3 plugin UI公開契約の解禁 |

`G0-9L`が合格しても親`G0-9`は**platform受入継続**、`G0-9D`は**WAIT / HARDWARE**のまま残す。
G0-6H、G0-3 / GAP-13、Document、plugin/community公開契約、永続layout形式は本改訂の対象外であり、`G0-9L`のPASSで省略・迂回しない。

## 3. G0-9Lの固定構成

初回のlocal gateは次の一構成だけを対象にする。OS、runtime、GPU、displayのいずれかが変わった場合は、
既存結果を自動継承せずmanifestを更新して再審判する。

| 項目 | 初回対象 |
|---|---|
| hardware | MacBook Air `Mac16,12`、Apple M4、16 GB unified memory |
| GPU | Apple M4 8-core GPU、Metal 3 |
| OS | macOS 15.5 build 24F74 |
| display | 内蔵2560×1664 Retina、scale factor 2.0、単一display |
| WebView | WKWebView / wry。opaque child WebViewだけ |
| native GPU | wgpu 29、1 top-level Surface、Stage/Timeline 2 viewport |
| product topology | native top-level window + non-overlapping opaque WebView islands |

serial number、hardware UUID、user path等の個体識別情報を証拠manifestへ保存しない。

## 4. local acceptance harness

W0b完成後の製品そのものをG0-9Lの入力にすると循環するため、G0-9Lは
**local platform prerequisite evidence harness**でplatform能力を判定する。

このharnessは固定Mac構成のplatform条件だけを使う。

- 1 top-level wgpu Surface、同一device/queue、Stage/Timeline 2 viewport
- non-overlapping opaque child WKWebView
- harness専用のdeterministic offline fixture bundle、custom protocol、CSP、Ready/Pingだけの閉じたtyped codec
- Host所有の`layout_epoch`、WebView instance epoch、focus移譲、bounded a11y projection
- read-onlyなrevision付きsentinel snapshotとTransient selection

harnessが持ってよいsemantic inputは、reload/crash/lost前後の不変性を確認する固定sentinelだけである。
Place、Document command、Undo、Browser catalog、Inspector編集、Timeline編集を持たない。
diagnostic harnessをMotolii Studio PreviewまたはW0b完成と呼ばない。

fixture bundleはproduct React package、`docs/mocks-ui`、H1b成果物をimportせず、製品asset所有や
H1b完了をG0-9Lの前提にしない。逆にfixture用component、role、codec wireを製品packageへcopyしない。
G0-9L合格後もW0b、H1b、製品window結合はそれぞれ別の停止中製品粒であり、G0-9L harnessを
製品画面として再利用しない。harnessで確認したcustom protocol、CSP、epoch/lifecycleのplatform事実は
後続の検討入力にとどめ、製品codec、bundle、通常routeの合格を与えない。

## 5. G0-9Lの必須審判

### L1. renderer同条件比較

direct wgpu、direct wgpu + Vello局所pass、現行egui baselineを次の同一条件で比較する。

- 同じMac、OS、display、window寸法、present mode
- 同じ1,000 clip / 100,000 key fixture、visible range、selection、text/icon量
- 同じopaque WKWebView枚数、resize/input操作列、warm-up、測定時間
- frame/input latency、CPU、GPU、memory、resource生成回数、readback回数のraw結果を保存

絶対閾値を測定後に追加しない。採択は正しさ、resource hot-loop生成0、readback 0、
既存G0-4手順のp50/p95と外れ値を根拠に記録する。egui baselineのsourceと試験を削除しない。

### L2. IME、VoiceOver、focus、keyboard

人間が実windowで次を確認する。

- 日本語IMEのpreedit、候補位置、確定、取消
- composition中のEnter / Esc / Spaceがアプリshortcutへ漏れない
- native → WebView → nativeのTab / Shift+Tabと明示focus移譲
- bounded VoiceOver treeでBrowser/Inspector相当のheading/controlとStage/Timeline proxyを読める
- fullscreen、minimize/restore後にfocusとIMEが復帰する

文字列の直接設定、synthetic composition event、AX treeの取得だけでは合格にしない。
判定者、操作列、結果、既知の非対象をmanifestへ記録する。

### L3. lifecycle、geometry、failure recovery

同じ実windowで次を行う。

- 100回以上のresizeとlayout epoch更新
- minimize/restore、fullscreen往復、0×0相当の一時不可視
- native/WebView境界をまたぐpointer capture、cancel、focus loss
- stale layout/WebView epochの拒否
- injected Surface `Lost` / `Outdated`と再configure
- WKWebView content process終了、reload、bounded retry/backoff
- offline起動、navigation/new-window/download/network既定拒否

各操作後にnative/Webのbounds、revision、selection sentinelが一致し、Document/history相当の
semantic writeが0、CPU readbackが0、古いepochの反映が0であることを確認する。
synthetic resizeだけ、process reloadだけ、別harnessの個別成功を足し合わせて合格にしない。

### L4. local gate判定

L1〜L3のraw evidence、実行commit、toolchain、固定構成、未合格platformを一つのmanifestへ束ねる。
反対側レビューでP0/P1=0になった後、`G0-9L: PASS`と固定Macのplatform prerequisite evidence限定確定を別decisionへ記録する。

現在の証拠は次の部分合格だけであり、G0-9L合格ではない。

| 既存証拠 | 現在の扱い |
|---|---|
| 1 Surface / 2 viewport / 2 opaque WKWebView | macOS topology部分合格 |
| 104 resize / 106 layout epoch | L3のresize部分合格 |
| acquire/present 200/200、CPU readback 0 | L1/L3の基礎証拠 |
| minimize/restore、fullscreen進入 | L2/L3の部分合格 |
| Web controlのAX tree | L2の構造部分合格。VoiceOver読上げではない |
| synthetic IME抑止 | L2の自動負例。実IME合格ではない |
| 同一display multi-Surface spike | detachの構造証拠。G0-9L通常topologyの代替ではない |

## 6. G0-9Dの閉集合

G0-9Dは少なくとも次を別証拠として保持する。

- Windows 10/11、WebView2、per-monitor-v2 DPI、MS-IME、NVDA
- WebView2 runtime未導入、offline install/start、`ProcessFailed`復旧
- 異DPI monitor移動、第二monitor、fullscreen、detach/re-dock
- HDR/SDR差、実surface/device loss
- 対応distribution Mac構成でのlocal fixture再審判
- 対象platformごとの同じLocal Alpha制作fixture

現在所有していないhardwareは`WAIT / HARDWARE`とし、macOS結果、headless test、synthetic scale、
一次資料でPASSへ上げない。Linuxの配布方式は本改訂で推測しない。

## 7. 限定確定と停止範囲

`G0-9L: PASS`後に確定するのは、固定Mac構成でL1〜L3を満たしたplatform prerequisite evidenceだけである。
対象なしの「限定解禁」は行わず、このPASSは後続製品粒の依存充足、実装許可、通常routeの受入を意味しない。

引き続き停止する範囲:

- G0-6Hの人間審判、およびU0e-3とW0bの製品前提
- H1b、Motolii Studio Preview、WebView/nativeの通常製品window結合
- `G0-9D`未合格platformのDistribution Ready主張
- egui baselineと比較fixtureの削除
- community panel、自由plugin UI、sandbox/capability公開契約
- Linux runtime、Windows固有fallback、CompositionController採択
- transparent WebView、複数通常Surface、CPU pixel fallback
- Document、journal、plugin ABI、永続layout形式の変更
- Local Alpha後へ送った全panel detach/re-dock製品接続

## 8. 必須負例

- Macの合格をWindows、追加monitor、HDR、penへ転載する
- IME/VoiceOver/process crashをG0-9Dへ追放してlocal gateを軽くする
- R0〜R6のvisual合格だけでG0-9LをPASSにする
- individual spikeのPASSを一つのlocal manifestへ無検査で合算する
- diagnostic harnessを通常製品routeまたはMotolii Studio Previewと呼ぶ
- product-owned React source、Host snapshot、native surfaceのowner違反をplatform成功で免除する
- local gate合格と親G0-9完了、G0-6H、W0b、H1b、Motolii Studio Preview、Distribution Ready、G0-3解禁を同義にする
- gate合格のためgolden、visual threshold、期待値、固定fixtureを変更する

## 9. 反対側レビュー結果

2026-07-23のTerra実装差分をCursor Grok 4.5 Highがread-only検収し、次の観点を確認した。

1. local harnessがW0b完成を要求する循環になっていないか
2. harnessと製品windowの差が大きく、platform合格を誤外挿していないか
3. IME、VoiceOver、focus、capture、surface lost、Web process crashをlocal gateへ十分残したか
4. parent G0-9、G0-9L、G0-9D、G0-3の状態が独立しているか
5. 既存の部分証拠を新しい全体合格へ誤昇格していないか
6. 未所有hardwareとWindowsの負例をsynthetic PASSできないか
7. 限定確定がW0b、H1b、Motolii Studio Preview、window結合、公開API、Document、plugin契約、永続形式へ広がっていないか

R5でP0/P1=0、`VERDICT: ACCEPT`、runner終了コード0を確認したため本書を決定へ上げる。
この採択が確定するのはgate構造と証拠効力だけであり、`G0-9L`自体は未合格である。
W0b、H1b、Motolii Studio Preview、window結合は各固有依存と受入条件まで実装しない。
