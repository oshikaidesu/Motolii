# G0-9 native Easing popup spike

React/WebViewのGraph iconからHost IPCを通し、direct-wgpuのnative popupを開く隔離spike。
製品Document、D2、User settings codec、plugin UI契約、egui shellは変更しない。

## 自動試験

```bash
cargo fmt --manifest-path spikes/g0-9-easing-popup/Cargo.toml -- --check
cargo clippy --manifest-path spikes/g0-9-easing-popup/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/g0-9-easing-popup/Cargo.toml
```

## 実機起動

```bash
G0_9_EASING_REPORT=/tmp/motolii-g0-9-easing-popup-report.json \
G0_9_EASING_STORE=/tmp/motolii-g0-9-easing-popup-presets.json \
cargo run --manifest-path spikes/g0-9-easing-popup/Cargo.toml
```

- WebViewの`Graph`でnative popupを開く
- 4つのpresetをclickしてcurveを変更する
- graph handleをdragする。drag中はTransient、release時だけcommit counterが増える
- `Esc`またはfocus lossでdragを戻す
- `S`で現在curveをspike専用preset storeへ保存、`F`で最新user presetをFavoriteにする
- 矢印keyでfocus中handleを動かす

popupを閉じて再度開いても、またプロセスを再起動してもspike storeからuser presetを復元する。
保存するのは型付きcurve値であり、thumbnail画像、SVG path、GPU texture、px値は保存しない。

z-order、実focus、外click、異DPI monitor、VoiceOver/NVDAは実機審判であり、unit testのPASSから外挿しない。

実施結果と未証明範囲は
[`docs/spikes/g0-9-native-easing-popup.md`](../../docs/spikes/g0-9-native-easing-popup.md)を参照する。
