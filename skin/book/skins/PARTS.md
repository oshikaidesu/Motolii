# 革の絵の部品の名前

theme.xml に `<Parts Value="フォルダ"/>` を書くと、そのフォルダにある `名前.svg` が、窓のその絵の代わりに描かれる（フォルダは theme.xml から見た場所でも、~/ からでも、絶対の場所でもよい）。
色は、窓がその場所で使っている色に塗り直される（SVG の形だけが使われる）。無い名前は、窓のいつもの絵のまま。

- 枠は `viewBox="0 0 24 24"`。窓ではおよそ 12〜22 画素に縮めて描かれる（`Size` で変わる）ので、線の太さは 24 の枠で 1.5〜2 がちょうどいい。
- 塗りの形も、線だけの形（`fill="none"` と `stroke`）も、どちらも塗り直される。塗り直す色はその場所の色：ふだんは文字の灰色、押している時や再生中は差し色（`ChosenInk`）。
- 印は枠いっぱいに描かれる。いつもの印と大きさをそろえたいときは、24 の枠の中で 2〜3 の余白を残す。
- ここに載っているのは、窓に今出ている印だけ（窓が描かない名前は `test/skin_names_test.dart` が落とす）。

## 枠（frame.）：部品の形そのもの
印と違って、枠は**描いた色のまま**、部品のまわりにのばして描かれる（CSS の border-image と同じ）。四隅は描いたまま、辺と真ん中がのびる。四隅の大きさは SVG の一番外のタグに `data-slice="9"` と書く（書かなければ短い辺の3分の1）。枠を持ってきた部品は、いつもの塗りと角を描かない。

| 名前 | 部品 |
|---|---|
| `frame.render` | 上の帯の Render ボタン（`frame.render.hover`：手を乗せた時） |
| `frame.card` | インスペクタの札1枚（見出しの帯も含む：見出しは上から約26の高さ） |

見本は [streets](streets/parts)（黒い箱に白い太枠の Render、青い見出しの帯の札）。

## 道具（glyph.）：舞台の左の縦の道具の列
| 名前 | 印 |
|---|---|
| `glyph.select` | 選ぶ（矢印） |
| `glyph.move` | 動かす（十字の矢印） |
| `glyph.rect` `glyph.ellipse` `glyph.circle` `glyph.shape` | 四角・楕円・円・形を描く |
| `glyph.pen` | ペン |
| `glyph.text` | 文字 |
| `glyph.image` `glyph.camera` `glyph.grid` | 画像・カメラ・格子 |

## タイムライン（tp.）：タイムラインの上の帯と、層の行
| 名前 | 印 |
|---|---|
| `tp.play` `tp.pause` | 再生・一時停止 |
| `tp.toStart` `tp.stepBack` `tp.stepFwd` | 頭へ・1コマ戻る・1コマ進む |
| `tp.loop` | くり返し |
| `tp.fit` `tp.plus` `tp.minus` | 全体を収める・時間を広げる・縮める |
| `tp.eye` `tp.eyeOff` | 層の行の「見える・見えない」 |
| `tp.solo` | 層の行の「これだけ見る」 |
| `tp.lock` `tp.unlock` | 層の行の「触れない・触れる」 |
| `tp.keyFilled` | キーの印 |

## ドック（dock.）：パネルの見出しの印、作る棚・効果の棚の印（名前が中身のまま：`dock.media` はメディアのパネル、`dock.star` は星の形を作る）
`dock.media` `dock.effects` `dock.fonts` `dock.colors` `dock.create` `dock.search` `dock.close` `dock.chevron` `dock.grid` `dock.list` `dock.plus` `dock.light` `dock.stylize` `dock.copies` `dock.text` `dock.time` `dock.shape` `dock.camera` `dock.nullObj` `dock.solid` `dock.rect` `dock.ellipse` `dock.polygon` `dock.star` `dock.line` `dock.radial` `dock.folder` `dock.comp` `dock.image` `dock.audio` `dock.video` `dock.font` `dock.link` `dock.stage` `dock.inspector` `dock.timeline` `dock.desk`

## 机の道具（desk.）
`desk.tools` `desk.blend` `desk.align`

## そのほか
`undo`（やり直しは同じ絵を裏返して使う）、`shade`（明暗の切り替え）、`stage.safe`、`stage.grid`
