# G0-9 native Easing popup spike

実施日: 2026-07-22

状態: **core縦切り合格、platform受入は継続**。製品U4b接続の停止線は解除しない。

## 問い

React所有面に相当するWebViewがGraph triggerとobject・channel・pressed/disabledのaccessible stateだけを持ち、Hostが別native windowを生成し、そのwindow内の
preset shelf、user preset、Bezier curve、handle、数値表示をdirect wgpuで一体描画できるか。さらにcurveと
presetをReact/nativeへ二重所有せず、drag releaseとUser settings writeを別のtyped intentとして数えられるか。

製品Document、D2、正式User Settings codec、公開API、plugin契約、製品egui shellは変更しない。

## 結果

Apple M4 / Metal上で、WebView内のGraph buttonからnative popupを開き、handle drag、preset保存、favorite、Esc、
再表示、プロセス再起動を実行した。[raw report](g0-9-native-easing-popup-evidence/report.json)と
[interaction log](g0-9-native-easing-popup-evidence/interaction.json)を証跡とする。

さらに固定React oracle `56c318ed`を実行してnative版と比較し、popupを論理`510 x 284`、左の3列Bezier/
Advanced shelf、中央の縦長graph、右の2 handle value cardという同じ情報階層へ修正した。背景、raised面、
border、muted text、active goldはReact tokenを基準にし、通常状態、Smooth選択、handle drag、user preset保存、
favorite表示を実機で比較した。Advanced cardsはこのspikeでは**外観のみ**であり、高度補間の意味論を実装したとは扱わない。

| 指標 | 結果 |
|---|---:|
| popup content owner | native-wgpu |
| React owner | trigger-and-accessible-state-only |
| GPU adapter / backend | Apple M4 / Metal |
| handle drag release | semantic commit 1 |
| preset save + favorite | settings write 2 |
| readback | 0 |
| hot drag resource生成 | 0 |
| bounded semantic node model | 7 |
| restart後のuser preset / favorite復元 | PASS |
| Esc後のrevision / commit / settings write増分 | 0 / 0 / 0 |

初回実機試験では、popup生成直後のfocus-lossと、surface表示前のredraw要求を生命周期上の実イベントとして
回収した。初回focusを受ける前の`Focused(false)`はdismissしないようにし、surfaceがoccludedの間は
`about_to_wait`からredrawを再要求する。修正後はnative frame、preset thumbnail、curve、grid、handle、textを
同一surfaceへ表示できた。

user presetに保存したのはBezier 4値、名前、順序、favoriteだけで、thumbnail画像、SVG path、GPU texture、
px/DPI値は保存していない。再起動後のthumbnailも保存curveから再生成した。

## 自動試験

独立workspaceの7試験が次を固定する。

- work area内へのclampと上下flip
- drag中のTransient、release 1回、重複release 0回
- Escとstale revision/layout epochのcommit 0
- presetのrestart復元と同じcurve projectionからのthumbnail生成
- theme/DPI変更で保存curve不変
- bounded semantic model、readback 0、hot drag resource生成0
- 固定React oracleの見出し、preset名、3列shelf、graph/value card寸法

```bash
cargo fmt --manifest-path spikes/g0-9-easing-popup/Cargo.toml -- --check
cargo clippy --manifest-path spikes/g0-9-easing-popup/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/g0-9-easing-popup/Cargo.toml
```

## 未証明

- Hostの実AX treeへ7 nodeを接続したVoiceOver/NVDA操作
- popup外click、pointer capture loss、window外releaseの全経路
- light theme、100回open/close、resize、異DPI monitor、第二monitor
- Windows WebView2のz-order、focus、per-monitor DPI、MS-IME
- 固定React oracleとのpixel goldenおよびhover/overshoot/cancelを含む全visual matrix
- 固定commitの実React componentをproduct packageとして接続すること
- 正式User Settings codecとD2 command/Undoへの製品接続

したがって合格したのは、**React所有面相当のWebView trigger + Host lifecycle + native popup content + typed test double**のcore縦切りである。
製品U4b、egui撤去、plugin公開契約、正式永続形式の合格には数えない。
