# 革の絵の部品の名前

theme.xml に `<Parts Value="フォルダ"/>` を書くと、そのフォルダにある `名前.svg` が、窓のその絵の代わりに描かれる（フォルダは theme.xml から見た場所でも、~/ からでも、絶対の場所でもよい）。
色は、窓がその場所で使っている色に塗り直される（SVG の形だけが使われる）。無い名前は、窓のいつもの絵のまま。

- 枠は `viewBox="0 0 24 24"`。窓ではおよそ 12〜22 画素に縮めて描かれる（`Size` で変わる）ので、線の太さは 24 の枠で 1.5〜2 がちょうどいい。
- 塗りの形も、線だけの形（`fill="none"` と `stroke`）も、どちらも塗り直される。塗り直す色はその場所の色：ふだんは文字の灰色、押している時や再生中は差し色（`ChosenInk`）。
- 印は枠いっぱいに描かれる。いつもの印と大きさをそろえたいときは、24 の枠の中で 2〜3 の余白を残す。
- ここに載っているのは、窓に今出ている印だけ（窓が描かない名前は `test/skin_names_test.dart` が落とす）。

## 道具（glyph.）
`glyph.select` `glyph.move` `glyph.rect` `glyph.ellipse` `glyph.pen` `glyph.text` `glyph.image` `glyph.camera` `glyph.grid` `glyph.circle` `glyph.shape`

## タイムライン（tp.）
`tp.eye` `tp.eyeOff` `tp.solo` `tp.lock` `tp.unlock` `tp.play` `tp.pause` `tp.toStart` `tp.stepBack` `tp.stepFwd` `tp.loop` `tp.fit` `tp.plus` `tp.minus` `tp.keyFilled`

## ドック（dock.）
`dock.media` `dock.effects` `dock.fonts` `dock.colors` `dock.create` `dock.search` `dock.close` `dock.chevron` `dock.grid` `dock.list` `dock.plus` `dock.light` `dock.stylize` `dock.copies` `dock.text` `dock.time` `dock.shape` `dock.camera` `dock.nullObj` `dock.solid` `dock.rect` `dock.ellipse` `dock.polygon` `dock.star` `dock.line` `dock.radial` `dock.folder` `dock.comp` `dock.image` `dock.audio` `dock.video` `dock.font` `dock.link` `dock.stage` `dock.inspector` `dock.timeline` `dock.desk`

## 机の道具（desk.）
`desk.tools` `desk.blend` `desk.align`

## そのほか
`undo`（やり直しは同じ絵を裏返して使う）、`shade`（明暗の切り替え）、`stage.safe`、`stage.grid`
