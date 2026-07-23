# G0-9 native Multi-key Graph View spike

実施日: 2026-07-22

状態: **isolated fixture core合格、製品接続は未実施**。

## 問い

固定React `GraphViewCandidate`の3 channel / 9 keyを情報構造のoracleとし、Blender Graph Editorで既知の
channel list、header、graph canvas、status、F-Curve、key、tangent handle、playheadという操作語彙を、
Motolii独自のdirect wgpu実装で再現できるか。

BlenderのGPL source、shader、icon、定数表、関数構造はcopy、翻訳、port、vendorしていない。製品Document、
D2、公開API、plugin契約、WebView/native製品統合も変更していない。

## 結果

Apple M4 / Metalで120 frameを実描画し、3 channel、9 key、203 GPU primitive、48 text runを同一surfaceへ
表示できた。[実機終了report](g0-9-native-graph-view-evidence/report.json)、
[自動run report](g0-9-native-graph-view-evidence/auto-report.json)と
[実マウスinteraction log](g0-9-native-graph-view-evidence/interaction.json)を証跡とする。

| 指標 | 結果 |
|---|---:|
| React情報oracle | fixed commit `56c318ed` |
| channel / key | 3 / 9 |
| GPU adapter / backend | Apple M4 / Metal |
| primitive / text run（Fit All） | 203 / 48 |
| presents（headless操作の実機確認終了時） | 5109 |
| readback | 0 |
| hot drag resource生成 | 0 |
| drag中semantic commit | 0 |
| release / duplicate release | 1 / 0 |
| Esc cancel後commit | 0 |
| wheel zoom / Fit All / Fit Selection | 実機合格 |
| marquee選択 | 4 key、Document変更0 |

実マウスではselected keyを`53.24 / 82.0`から`53.42 / 65.4`へdragし、key、左右handle、curveが一緒に
更新され、releaseでcommitが1だけ増えた。続けてoutgoing tangentをdragし、stemとcurveの更新および2件目の
release commitを確認した。GPU内容は現時点のmacOS AX treeへ出ておらず、これは未証明ではなく明確な後続課題である。

key dragは左右の隣接keyを越えず、値をfixture範囲へclampする。selected keyを動かすと左右handleも同じ差分で
移動する。handleは独立してdragできる。描画更新は既存bufferへのwriteだけで、drag hot pathにpipeline、buffer、
texture生成を置いていない。

座標変換、pan、zoom、fitは`understory_view2d 0.1.0`を二つの1D viewportとして再利用した。stable ID、
single/additive/marquee選択、4 logical px drag threshold、0.1 frame snap test double、release/CancelはMotolii側の
headless state machineが所有する。macOS実機でwheel zoom、Fit All、marquee、Fit Selectionを順に確認し、
navigation change 3、selection change 2、semantic commit 0だった。

## 自動試験

独立workspaceの10試験が固定React fixture一致、release exactly-once、Cancel完全復元、時間順clamp、画面の主要領域、
cursor-anchor zoom、fit、stable ID selection、marquee、drag threshold、snap、non-finite拒否を固定する。

```bash
cargo fmt --manifest-path spikes/g0-9-graph-view/Cargo.toml -- --check
cargo clippy --manifest-path spikes/g0-9-graph-view/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/g0-9-graph-view/Cargo.toml
cargo run --manifest-path spikes/g0-9-graph-view/Cargo.toml -- --auto
```

## 未証明

- middle panの実マウス操作、channel row操作、edge scroll、pointer capture lost
- 正式D2 command / 1 Undo、Timeline / Easing / Inspectorとのselection同期
- bounded semantic modelのOS AX tree接続とkeyboard-only編集
- 異DPI、第二monitor、Windows、surface lost、dock/detach
- Blenderとのpixel一致。これはlicense上も合格条件にしない

Computer Use APIはmiddle-button dragを表現できなかったため、middle panはheadless試験とwinit adapter実装までである。

したがって本結果は、**Blender-likeな情報配置と基本navigation/selection/key/handle操作のnative描画fixtureが成立する**ことを示す。
製品Graph Viewの完成、egui撤去、公開契約変更の根拠にはしない。
