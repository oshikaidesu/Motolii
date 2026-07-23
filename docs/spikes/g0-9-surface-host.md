# G0-9 wgpu 29 surface host実機spike（2026-07-21）

状態: **macOS部分合格／製品統合は継続停止**。

公開API、Document、plugin契約、永続layoutへ触れないisolated harnessとして
[`spikes/g0-9-surface-host/`](../../spikes/g0-9-surface-host/)を作り、決定済みtopologyを製品と同じ
wgpu majorで確認した。これはrenderer採用やG0-9完了を意味しない。

## 構成と審判

- top-level `wgpu::Surface` 1枚、frameごとのacquire/presentは1系統
- 同一surface textureをStageとTimelineの2 viewportへ分割
- 左右にopaque child WKWebViewを2枚配置
- window titleとJSON reportでresize、layout epoch、acquire、present、readback、drag、Web入力を計数
- `SurfaceLayout`のDPI変換、無効寸法、境界hit-test、present不変条件をRust unit testで固定

実行コマンド:

```bash
cargo fmt --manifest-path spikes/g0-9-surface-host/Cargo.toml -- --check
cargo test --manifest-path spikes/g0-9-surface-host/Cargo.toml
cargo build --manifest-path spikes/g0-9-surface-host/Cargo.toml
```

unit testは4件合格した。macOS実機ではComputer UseでApp bundleを操作し、最終reportは次だった。

| 観測 | 結果 |
|---|---:|
| wgpu major | 29 |
| surface / native viewport / WebView | 1 / 2 / 2 |
| resize event / layout epoch | 104 / 106 |
| acquire / present / CPU readback | 200 / 200 / 0 |
| nativeからWebViewへ境界drag | move 2、境界通過true、release true |
| Web入力 | 4 events、左右の値をAX経由で再取得 |
| WebViewからnativeへdrag | start 1、move 0、end 1 |

Stage、Timeline、両WebViewが同時表示され、100回resize後も位置が一致した。Web入力後と
minimize/restore後とfullscreen進入後にもnative描画とWebViewのAX treeを再取得できた。したがってmacOSの通常windowで
「1 surface / 2 viewport / 非重複opaque WebView islands」が成立し、CPU pixel bridgeを必要としないことは
このfixtureで合格とする。

## 未証明と停止線

- Computer Useの一括dragはWebKitへ`pointerdown`と`pointerup`を届けたが中間`pointermove`を生成しなかった。
  WebViewからnative Stageへのdrag token handoffは人間のactual pointerか分割可能な入力fixtureで再審判する
- AX treeはBrowser/Inspectorのheading、input、buttonを露出したが、GPU描画のStage/Timelineには意味ノードがない。
  bounded AccessKit treeまたは同等proxyをhost側で持つまでVoiceOver合格にしない
- 日本語文字列の値設定はIME composition、候補窓、preedit、取消の証明ではないため、人間の日本語IME試験を残す
- fullscreen進入時の描画は成立したが、focused WebViewからの自動shortcutで退出を再現できなかった。
  WebView間focus traversalとfullscreen往復はactual keyboardで再審判する
- 異DPI monitor移動、surface/device lost、Web content process終了、sandbox、実penは未試験
- Windows WebView2、per-monitor DPI、MS-IME、NVDA、offline runtimeはWindows実機だけで判定する

未証明項目を埋めるために透明WebView、複数surface、CPU readback、raw plugin権限へfallbackしない。

## CU-0G03H 追加実装（2026-07-24、2件の独立reviewの指摘を反映して訂正）

CU-0G03の物理判定（Computer Use/VoiceOver/実IME）を1つのmacOS windowで行えるよう、
harness-localの観測能力だけを追加した。トポロジ（`surface_count == 1`、
`native_viewport_count == 2`、`webview_count == 2`、acquire/present不変条件、
`readback_count == 0`）は変更していない。

- `accesskit` + `accesskit_winit`によるhost所有の境界付きaccessibility投影
  (`AccessibilityProjection`)。Stage/Timelineは常に固定6ノードで、Documentが
  存在しないためclip/key/selection数に依存し得ない。`main`のAX tree構築は
  この同じ投影を直接消費する（テスト専用の別モデルではない）ため、Stage/
  Timelineノードは実際のlayout由来のboundsを持ち、`Action::Focus`を広告する
- `FocusCoordinator`: native Stage → left Web → native Timeline → right Webの
  順で循環するfocus ring。native側のTab/Shift+Tabだけでなく、各WKWebViewの
  DOM側でも`keydown`をcaptureしてTab既定動作を`preventDefault`し、
  `tab-forward`/`tab-backward`を型付きIPCでhostへ中継する。native focusへ
  遷移する際はwry `WebView::focus_parent`で現在first responderのWKWebViewを
  明示的にresignし、windowを前面化するだけでは済ませない。WebView側で
  マウスクリック等により`focusin`が発火した場合も型付きIPCでcoordinatorへ
  同期する。AccessKit `Action::Focus`とWeb IPC `focus-request:<role>:<epoch>`
  （右WebViewへのfocus要求を含め、すべての明示的focus遷移はこの1本のgrammar
  を通る）はすべて同じ状態機械を通り、現行epochと既知roleだけを受理する。
  focus epochはhostのlayout epochへ直結しており、各WebViewが型付き`ready`
  messageを送るまでその値を配信しない。stale epochまたは未知roleは型付き
  errorで拒否し、focus/counter/report状態を変更しない
- 左右WebViewの`keydown`captureはEnter/Escape/Spaceも同じ型付きIPCでhostの
  `ShortcutSink`へ中継する（native側のwinitキー処理だけに依存しない）。左右
  WebViewのeditable inputには実DOM `compositionstart`/`compositionupdate`
  /`compositionend`のobserverがあり、`compositionupdate`は`ShortcutSink`の
  独立counterへ観測記録される。composing中はEnter/Escape/Spaceを一切計上
  しないことをunit testで固定した。これは実IME PASSの宣言ではない
- fullscreenとminimize/restoreを別々の`LifecycleRecorder`で観測する。F11は
  winit `Window::set_fullscreen`を呼び、実際の遷移完了は`Resized`イベントで
  `Window::fullscreen()`を再確認して確定する。F9は`Window::set_minimized`を
  呼び、遷移完了は`WindowEvent::Occluded`（winit自身がminimize/coverの実信号
  として文書化している）で確定する。`Occluded`をfullscreenの代用として扱わ
  ない。どちらもCU-0G04的な注入ではなく実際のwinit APIと型付きharness
  操作のみを使う。自動テストはこの記録の構造だけを固定し、実際の
  fullscreen/minimize遷移やVoiceOver/実IME合否は依然として人間のCU-0G03
  実機確認が必要

実行コマンドと必須negative/positive testはunit test（`cargo test --manifest-path
spikes/g0-9-surface-host/Cargo.toml`）に追加済みで、drag/resize/present/readback
既存testは変更なく緑のまま。CU-0G03 DONEやG0-9L PASSはこの実装のどのテストも
宣言しない。

## CU-0G03H2 機械focus E2E追補（2026-07-24）

CU-0G03の人間審判で、左WebViewからnative Timelineへの最初のTabでは
`FocusCoordinator`が遷移する一方、`wry::focus_parent`後の親NSViewが次のTabを
winitへ渡さず、実focus ringが停止する事実を観測した。内部coordinatorの値と
実first-responderを同一視していたCU-0G03Hの機械審判不足であり、既知制約には
格下げしない。

CU-0G03H2は実NSWindowへTab / Shift+Tabを入力し、次の三点を1つのmanifestへ記録する。

- native Stage → 左WebView → native Timeline → 右WebView → native Stageの正逆順
- 各遷移後の`NSWindow.firstResponder` classと期待するnative/Web種別の一致
- native NSEvent monitorまたはWebView DOM `keydown` relayの実到達元

Ctrl / Option / Cmd等を伴うTabをnative focus relayとDOM relayが奪わないことは、
OSのapp切替等をE2Eから起動しないよう純粋判定とrelay sourceのunit負例で固定する。

これはfocus配送だけの機械E2Eであり、synthetic composition、文字列の直接設定、
scripted VoiceOverを追加しない。実IME候補窓、VoiceOverの読み上げとfocus追従、
fullscreen/minimize後のfocus/IME復帰はCU-0G03の人間審判に残す。製品window、
Document、公開UI/keyboard契約、topology、CU-0G04のfailure injectionへ広げる
必要が生じた場合は停止する。

機械E2Eは次で実行し、process exit 0とreportの
`automated_focus_pass == true`を両方要求する。

```bash
G0_9_RESIZE_TARGET=0 \
G0_9_AUTOMATE_FOCUS=1 \
G0_9_REPORT=/tmp/motolii-cu-0g03h2-focus-e2e.json \
cargo run --manifest-path spikes/g0-9-surface-host/Cargo.toml
jq -e '.automated_focus_pass == true' \
  /tmp/motolii-cu-0g03h2-focus-e2e.json
```

固定Macで正逆8遷移すべてのrole、実first-responder種別、到達元が一致し、
process exit 0と`automated_focus_pass == true`を確認した。20 lib + 7 main
tests、workspace、docsが全緑で、Grok/FableともP0/P1=0、`VERDICT: ACCEPT`。
これはCU-0G03 DONEまたはG0-9L PASSを宣言しない。
