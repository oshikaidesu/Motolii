# CU-0B04R Browser island focus / geometry epoch実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B03`、`CU-0B04N`

## 1. 完了した境界

通常product HostのBrowser一島へ、Host所有のprivate lifecycle stateを追加した。

- Browser child WebViewのboundsは`NativeHostLayout`の単調増加`layout_epoch`と一緒に
  適用する。同じepochまたは古いepochはWebViewへ再適用せず、Document、D2、CSS px、
  DOM identityへ流さない。
- initialization scriptが注入した既存exact snapshotを初回projectionとし、
  `motolii-browser://product/host.html`のload完了または同instanceからの有効IPCを
  initial projection readyとして一度だけ受理する。別instance epoch、二度目の
  load完了は無視し、snapshotを再配送しない。
- focusはOSの実focusを推測するstateにせず、Hostが要求したownerだけを記録する。
  initial projection ready後にBrowserへ一度だけ要求し、Place intentを取り出す時に
  `wry::WebView::focus_parent`でnative parentへ明示移譲する。以後Browserへ自動再取得しない。
- raw winit `Focused` / pointer / key、transparent overlay、別WebView、reload / crash
  retry、evaluate script、React semantic state、公開APIを追加していない。

React source assetのcomponent、DOM、class、stable ID、ARIA、CSS、interaction、
generated product bundleは変更していない。変更したのはproduct-owned native Host
adapterと既存Browser codec/sessionのlifecycle接続だけである。

## 2. `CU-0B05`へ残すもの

reload / content process crash後のWebView再生成、snapshot再配送、同じrevision /
selectionの再投影、old instance epoch拒否、bounded retry、focus復元は実装していない。
初回ready callbackとHost所有instance stateだけを再投影入口として残し、二回目の配送を
本粒へ取り込まない。

Document、journal、selection、Undo、Timeline / Inspector projection、Place terminal
commitも非目標である。`CU-0B04R`単体をMotolii Studio Preview完成とは扱わない。

## 3. 検証

```text
cargo clippy -p motolii-ui --lib --bins -- -D warnings
passed

cargo test -p motolii-ui browser_host_runtime --lib
5 passed

cargo test -p motolii-ui --test raw_input_boundary
5 passed

cargo test -p motolii-ui
全test / doc-test passed

cargo test --workspace
全workspace test / doc-test passed

./scripts/check-docs.sh
OK: docs整合チェック全項目通過

MacBook実機 / MotoliiNativeProduct.app
opaque Browser + native Stage / Timelineの同時表示、
window zoom後の同一layout追従、Browser Create表示、Stage方向drag後に
AX focusがWeb contentからparent windowへ移ることを確認
```

Claude Opus 5へ完全model ID `claude-opus-5`、`--effort low`でread-only相談し、
focusをobserved truthでなくrequested ownerへ限定した。page-load callbackは初回
instanceのsingle-shot readyだけに閉じ、一般snapshot再配送へ広げない助言を現行
authorityとwry 0.55.1 APIへ再照合して採用した。

次PRODUCT-ASSET `DO`は`CU-0B05`。reload / crash / focus / resize後もHost snapshotから
同じrevision / selectionを再投影し、old instance epochを拒否するE2Eへ進む。
