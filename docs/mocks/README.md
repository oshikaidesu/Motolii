# M3 UIモック

## 基準: 高密度メインUI v1

[インタラクティブHTML](m3-main-ui-v1.html)をM3の視覚構成基盤とする。設定画面からのライト/ダーク切替、preview canvas、波形、driver scopeをブラウザ内で実際に描画する。

このHTMLは2026-07-11にClaude Desktopが一時scratchpadへ生成したモックを、2026-07-14に回収し、テーマ設定要件・role名token・Dark既定・contrast修正を反映したもの。静止画はHTML改訂のたびにheadless Chrome(`#dark` / `#light` hashで固定入場)から再生成し、HTMLと乖離させない。

![M3 main UI dark](m3-main-ui-v1-dark.png)

- [ライト静止画](m3-main-ui-v1-light.png)
- [ダーク静止画](m3-main-ui-v1-dark.png)
- ステータス: **視覚構成の基準モック**。Document意味論や未決機能を確定するものではない
- 色: Ableton Liveを先例にしたflat surface、細罫線、色=機能。装飾gradientなし
- 密度: asset browser、preview、property、effect stack、driver、波形、階層timeline、easing popupを同一画面へ常設
- 説明: 画面内badgeと下部対応表で、確定事項・方向確定・未決の出所を分離

## 比較案: グリッド基調 v2

[m3-main-ui-v2.html](m3-main-ui-v2.html)は、v1と同一fixture・同一token roleのまま「余白は分離の手段にしない」規約(2026-07-14追記)で組み直した比較案。G0-6の構成審判はv1とv2を同じviewport・同じ情報量で比較して行う。

![M3 main UI v2 dark](m3-main-ui-v2-dark.png)

- [v2ダーク静止画](m3-main-ui-v2-dark.png) / [v2ライト静止画](m3-main-ui-v2-light.png)
- 全spacing/行高/radiusを`--sp-*`/`--row-*`/`--radius` tokenから取得し、raw値の場当たり指定を排除(色と同じ機械検査に載る)
- 領域分離は罫線+明度のみ。カード・影・空白分離を廃止(asset browserはカード型→22px行型)
- timeline 10行常視。text / image / meshの項目種別色を追加し、item roleの全種を1画面へ収載
- 下端にBlender式文脈ヘルプのstatus bar(hover対象・操作・shortcut・実行時stat)を常設

## 基盤として固定するもの

- 3-pane + 高密度timelineの大区画
- 波形とBPM gridを含むtimeline overview
- property / effect stack / driverを一覧できる右panel
- 選択、keyframe、data mapping、bakeを別の意味色で示すこと
- context説明を右下/status領域へ追加できる構造。Blenderは文脈ヘルプだけの参考で、全体UIは模倣しない
- ライト/ダーク/custom themeとも同じsemantic token schemaを参照すること
- 設定画面で組み込みDark/Lightを選択でき、初回既定はDarkであること(土台dark neutralの規約)
- tokenはrole名のみとし、文字用途の意味色はcontrast 4.5:1以上を保つこと(具体hex値は固定しない)

## 固定しないもの

- HTML内の具体色値、panel寸法、icon、font(現在のglyphはemoji/文字のplaceholderで、icon仕様の先取りではない)
- 既知のhue近接(solo黄とwarning琥珀、domain-path緑とstate-active緑、domain-pixel紫とitem-mesh紫、accentと選択青)。roleは分離済みで、hueの再配置はG0-6のCVD測定で決める
- 組み込み2テーマ以外の配布テーマ内容。custom themeを追加できる契約だけを固定する
- 未決と表示された音楽同期emission等の機能意味論
- plugin custom UI、3D gizmo、任意track色の永続化
- HTML/CSS/Canvasという実装方式。製品UIはM3仕様どおりSlint + wgpuを使う

## 次の改訂

1. ~~右下/status領域へ短いcontext説明を追加する~~ → v2のstatus barで収載済み（Blenderはこの機能だけの参考）
2. ~~timelineをさらに高くした比較案を同じfixtureで作る~~ → v2で作成済み。G0-6でv1/v2を同一viewport比較する
3. light/dark、grayscale、CVD、125/150/200% scaleで所在認知を比較する
4. hover/focus/drag/trim/easingを操作できるprototypeへ進める
