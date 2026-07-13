# M3 UIモック

## 基準: 高密度メインUI v1

[インタラクティブHTML](m3-main-ui-v1.html)をM3の視覚構成基盤とする。ライト/ダーク切替、preview canvas、波形、driver scopeをブラウザ内で実際に描画する。

このHTMLは2026-07-11にClaude Desktopが一時scratchpadへ生成したモックを、2026-07-14に回収してそのまま保全したもの。静止画は回収後のHTMLからheadless Chromeで再生成した。

![M3 main UI dark](m3-main-ui-v1-dark.png)

- [ライト静止画](m3-main-ui-v1-light.png)
- [ダーク静止画](m3-main-ui-v1-dark.png)
- ステータス: **視覚構成の基準モック**。Document意味論や未決機能を確定するものではない
- 色: Ableton Liveを先例にしたflat surface、細罫線、色=機能。装飾gradientなし
- 密度: asset browser、preview、property、effect stack、driver、波形、階層timeline、easing popupを同一画面へ常設
- 説明: 画面内badgeと下部対応表で、確定事項・方向確定・未決の出所を分離

## 基盤として固定するもの

- 3-pane + 高密度timelineの大区画
- 波形とBPM gridを含むtimeline overview
- property / effect stack / driverを一覧できる右panel
- 選択、keyframe、data mapping、bakeを別の意味色で示すこと
- context説明を右下/status領域へ追加できる構造
- ライト/ダークとも同じsemantic tokenを参照すること

## 固定しないもの

- HTML内の具体色値、panel寸法、icon、font
- 未決と表示された音楽同期emission等の機能意味論
- plugin custom UI、3D gizmo、任意track色の永続化
- HTML/CSS/Canvasという実装方式。製品UIはM3仕様どおりSlint + wgpuを使う

## 次の改訂

1. 右下/status領域へBlender型のcontext説明を追加する
2. timelineをさらに高くした比較案を同じfixtureで作る
3. light/dark、grayscale、CVD、125/150/200% scaleで所在認知を比較する
4. hover/focus/drag/trim/easingを操作できるprototypeへ進める
