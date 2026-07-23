# G0-9 native Depth Rail spike

実施日: 2026-07-22

状態: **isolated fixture合格／製品D2・Auto Key・Preserve Appearanceは未接続**。

## 結果

固定React `TimelineCandidate.jsx`（commit `56c318ed`）のDepth Railをoracleに、native direct wgpuで
Edit-Space Z axis、同一Z stack、parent-local scope、Camera Depth read-only marker、Layer Order Distribute
previewを再現した。

Apple M4 / Metalで120 frameをpresentし、5 object、25 primitive、16 text run、GPU readback 0、semantic state
owner 1で合格した。[自動report](g0-9-native-depth-rail-evidence/auto-report.json)と
[実機interaction](g0-9-native-depth-rail-evidence/interaction.json)を証跡とする。

初期root scopeでは4 objectのZ=0を扇状展開せず`0 × 4`のmarker 1個で表示する。`D`でfar=-0.25、
near=+0.25のauthoring-order previewを開き、`R`で割当だけを反転し、`Enter`で1回確定できた。previewとReverseは
semantic commit 0、Applyは1、duplicate Applyは0である。`C`は`ROOT / pulse-rings`へ切り替え、childだけを投影する。

座標、pan、zoom、Fit Allは`understory_view2d 0.1.0`を使用する。stable ID、scope、stack、selection、distribution、
Cancel、commit境界はMotolii headless kernelが所有し、Depth専用Document channelや別Undoを作っていない。

## 自動試験

Timeline / Depthの既存試験に、一般panel placement、再帰split、tab、window resizeを加えた16試験が合格した。
Stage / Timeline / Graph / Browser / Inspectorの5 roleを同じloopへ通し、全roleでdetach、top-level resize、
tab再ドック、split再ドックが成立し、snapshot revision、selection、semantic commitが不変であることを固定した。
headless矩形計算は`taffy 0.12.2`へ限定し、nested horizontal / vertical split、tabの同一矩形、window bounds、
120×90 logical panel min、split ratio 0.15〜0.85 clamp、NaN / Infinity拒否を確認した。
[dock layout report](g0-9-native-depth-rail-evidence/dock-layout.json)を証跡とする。

```bash
cargo fmt --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml -- --check
cargo clippy --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml
G0_9_TIMELINE_VISUAL_REPORT=/tmp/motolii-depth.json \
  cargo run --manifest-path spikes/g0-9-timeline-visual-parity/Cargo.toml -- --depth --auto
```

## multi-windowとの関係

Timeline / GraphだけでなくStage / Browser / Inspectorもdockまたは別top-levelへ置くheadless placement modelを
同じfixtureへ追加した。全roleのdetach/re-dockとresizeでsnapshot revision、selection、semantic commitが不変である。

OS側は[G0-10 multi-Surface fixture](g0-10-multi-surface-window.md)をmain作業ツリーへ回収して再実行し、2 top-level /
2 Surface、共有device 1、片側疑似lost・close/reopen後の他方present継続、Host snapshot不変を確認した。これは
Timeline/Graphの製品描画接続ではなく、一般detach lifecycleとpanel projectionをつなぐ二つのisolated証拠である。

## 未証明

- marker直接drag、range handle、Auto Key、正式D2/Undo、Preserve Appearance
- 100 layer、camera診断、遮蔽policy、bounded AX
- Timeline/Graphの実製品rendererを同じ2 windowへ同時投影する結合試験
- 実マウスによるdock preview、divider capture、tab tear-off、OS window間drop
- 異DPI第二monitor、HDR、Windows、実surface/device lost、React panel detach

したがって本結果はDepth Railの外観・基本headless意味と一般multi-window基盤の成立を示すが、製品統合停止線は解除しない。
