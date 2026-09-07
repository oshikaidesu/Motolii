# Inspector の先例調査 — 公式 docs で「表以外の答え」を探す(2026-09-08)

状態: **観察**。[Inspector 批評](2026-09-08-inspector-critique.md)の問い「プロパティ名と値を縦に並べた表であるべきか」に対し、
Blender / Houdini / Cinema 4D / Figma / Cavalry / Rive の公式文書で確認した。実機は撮っていない(利用者出先、公式 docs 指定)。

## 結論

- **表を捨てた製品は 1 つも無い。** 6 製品全てが名前と値の縦の表(Rive は選択に応じて中身が変わるが形は表)。
- 答えが出ているのは**表の中の 1 マス** — 値の触り方・精度・単位・連動・戻し方。批評の 9〜13・22〜24 に相当。
- 批評の 1〜8(面積配分・まとまり・対象との対応)、16(Anchor の結果予告)、17〜19(モード・toggle・Parent が結果を語る)は**どこにも答えが無い**。ここが Motolii の余地で、同時に「先例なし = 自作」なので利用者裁定が要る。

## 1 マスの答え(借りる)

| 要件 | 先例 | 何をしているか |
|---|---|---|
| 量感が図で読める(13) | Blender、Cavalry | 上下限が決まっている値は**塗りつきスライダー**として描く。Cavalry は hard limit を持つ単値を自動でスライダーに |
| 精度を手で選ぶ(10・24) | Houdini ladder、Figma、Blender | Houdini: MMB 長押しで段(0.01 / 0.1 / 1 / 10)が縦に出て、上下で段・左右で量。Figma: 上下の位置で 2x / 1x / ½ / ¼。Blender: Ctrl で刻み、Shift で細かく。**どれも「ドラッグ中に速度を変える」で、表の外に出ない** |
| 触り方が見える(11) | Figma、Blender | Figma はラベルに hover で scrub カーソル。Blender は数値の左右に ◂ ▸。**井戸(枠)そのものが「触れる」の印**(Lumit も同じ) |
| 単位(9) | Figma、Cavalry | 値の横に `°` `px` `%`。Figma は角度を −180〜180 に正規化 |
| 式(10) | Figma、Cavalry、C4D | `12*3`、`+`、`/`、`sqrt`。井戸に式を打てる |
| 形を保つ(12) | Figma、Cavalry | W / H の間に**比率 lock**。Cavalry は int2 に緑の toggle、Figma は Ctrl で一時解除 |
| 軸をまとめて触る(12) | Cavalry、C4D、Blender | Cavalry: Alt でドラッグすると全 field。C4D: Ctrl で複数 parameter 同時、Shift+Ctrl で相対。Blender: 縦ドラッグで隣接 field をまとめて |
| 変更済みの印(22) | C4D、Cavalry | C4D は名前の色でキー選択、現在 frame にキーがあると赤丸。Cavalry は ◇ / ◆ / ◆(現在 frame)の 3 状態 |
| 試して戻す(23) | C4D、Cavalry | C4D: Esc で確定前に戻す、矢印を右クリックで既定に。Cavalry: 行の右クリックに reset to default / delete animation / disconnect |
| 隣に効かせる(12・19) | Cavalry | 行の右クリックから「null で制御」「expression」「behaviour」を生やす。**Parent 相当を行から直接結ぶ** |
| 基準点(14〜16) | Rive | origin は % で、Stage 上の Freeze で動かすか Inspector で数値。3×3 の絵は無し。**結果の予告は無い** |

## 答えが無い所(Motolii の余地)

- **1〜4 面積とまとまり**: 全製品が同じ高さの行。Blender の panel 折り畳みと C4D の tab が唯一の「まとまり」で、面積配分はしていない
- **5〜6 対象との対応**: Rive は選択で中身が変わるだけ。thumbnail・Stage との線は無い
- **16 Anchor の結果予告**: 無し
- **17〜18 モードと toggle が結果を語る**: 無し(全部 checkbox か文字)
- **19 Parent を関係として**: Cavalry の「行から結ぶ」が一番近いが、見えるのは文字

## 出典

- Houdini value ladder: https://www.sidefx.com/docs/houdini/basics/ladder.html
- Cinema 4D Attribute Manager: https://help.maxon.net/c4d/r25/en-us/Content/html/5822.html
- Figma Adjust alignment, rotation, position, and dimensions: https://help.figma.com/hc/en-us/articles/360039956914
- Cavalry Control Rows – Interaction: https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-interaction/
- Rive Inspector / Freeze and Origin: https://rive.app/docs/editor/interface-overview/inspector 、https://rive.app/docs/editor/fundamentals/freeze-and-origin
- Blender number fields(manual は 403 で本文未取得、検索抜粋で Ctrl 刻み / Shift 精密 / 縦ドラッグ複数編集を確認): https://docs.blender.org/manual/en/latest/interface/controls/buttons/fields.html
- Nuke Properties panel は URL 失効で未確認
