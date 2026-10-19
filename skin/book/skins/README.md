# 革（スキン）の作り方

革は、窓の見た目だけを着せ替えるもの。中身（書類・動き・絵）は何も変わらない。
革は1つのフォルダで、`~/.motolii/skins/` に置く。

```
~/.motolii/skins/my_skin/
  theme.xml      ← 色・形・書体（これだけでも革になる）
  parts/*.svg    ← 印の絵（なくてもよい）
  *.otf / *.ttf  ← 書体ファイル（なくてもよい）
  preview.png    ← 撮った窓（shoot.sh が書く）
```

## 1. いちばん短い革

3色で窓全体が変わる。灰色の段（地・札・線・薄い文字…）は全部 `Background` と `Foreground` から作られ、明るい地なら明るい革になる。

```xml
<Motolii>
	<Theme>
		<Background Value="#f1ead9"/>
		<Foreground Value="#2b2118"/>
		<ChosenDefault Value="#e0452b"/>
		<Corner Value="0"/>
		<FontSans Value="Georgia"/>
	</Theme>
</Motolii>
```

これが見本の [paper](paper/theme.xml)。作り込んだ見本は [labs](labs/theme.xml)（灰色を1段ずつ書いて、印の絵も差し替えている）。

## 2. 着る・直す・見る

- **着る**：フォルダを `~/.motolii/skins/` に入れて、窓のメニュー **View › Skin** から選ぶ。**Show Skins Folder** でそのフォルダが開く。**Default** で窓のいつもの見た目に戻る。
- **直す**：着たまま `theme.xml` を保存すると、窓はすぐ読み直す。
- **見る（窓を開かずに）**：

```bash
skins/shoot.sh ~/.motolii/skins/my_skin
```

窓全体をその革で撮って、フォルダに `preview.png` を書く（2分ほど）。撮る書類には立体・光・形・文字・カメラ・アニメの層とキーが入っているので、層の色・縞・キーの印も写る。フォルダの外には何も書かない。いくつでも並べて渡せる。

- 書かなかった名前は、窓のいつもの値のまま。全部の色が値つきで並んだ一覧が欲しいときは [../themes/Default Dark.xml](../themes/Default%20Dark.xml)・[../themes/Default Light.xml](../themes/Default%20Light.xml)（`skin/book/themes/`）。

## 3. 色

`<名前 Value="#rrggbb"/>` で書く（透明度つきは `#rrggbbaa`）。名前の全部は [ROLES.md](ROLES.md) にある（窓が読む名前と食い違うと `test/skin_names_test.dart` が落ちる）。

- **Background / Foreground**：パネルの地の色と文字の色。書いた色がそのまま地と文字になり、ほかの灰色の段（すき間・札・入力欄・線・薄い文字…）はこの2色の間に作られる。明るい地なら明るい革になる（`Shade` は要らない）。
- **Grey00〜Grey100**：灰色の段を1つずつ決めたいとき（書いた段だけ上書き）。どの段が窓のどこかは [ROLES.md](ROLES.md) の表。[labs](labs/theme.xml) が段を1つずつ書いた例。
- **ChosenDefault**：差し色（選んだ物・押している物・Render の塗り）。その上の文字は **ControlOnForeground**。
- **Clip1〜7**：層の種類の色（1 形、2 文字、3 画像・動画、4 カメラ、5 グループ、6 粒子、7 音）。タイムラインの帯と、その種類の札や印に使う。**Keyframe**：キーの印。**AxisX/Y/Z**：軸の色。
- **Tone…**：インスペクタの札ごとの色（変形・塗り・効果…）。

## 4. 形（数）

どれも「いつもの何倍か」。1 がいつもの大きさ。

| 名前 | 変わるもの | 例 |
|---|---|---|
| `Corner` | 角の丸み | 0 で角ばる、3.5 で丸い錠剤 |
| `Spacing` | 余白とすき間 | 1.25 でゆったり |
| `Size` | 行・帯・列・札の大きさ | 1.15 で大きめ |
| `TextScale` | 文字の大きさ | 1.08 |

## 5. 書体

```xml
<FontSans Value="Avenir Next"/>
<FontMono Value="JetBrains Mono"/>
```

Mac に入っている書体なら名前だけでよい。入っていない書体は、ファイルを革のフォルダに入れて場所も書く。

```xml
<FontSans Value="Inter"/>
<FontSansFile Value="Inter.ttf"/>
```

## 6. 印の絵（SVG）

```xml
<Parts Value="parts"/>
```

と書くと、そのフォルダの `名前.svg` が窓の印の代わりに描かれる。名前の全部は [PARTS.md](PARTS.md) にある（再生ボタンは `tp.play.svg`、道具の選択は `glyph.select.svg`…）。

- 枠は `viewBox="0 0 24 24"` で描く。
- 色は窓がその場所の色で塗り直すので、形だけ描けばよい（色は何でもよい）。
- 無い名前は、窓のいつもの絵のまま。1枚だけ差し替えるのもあり。
- 場所は theme.xml から見た相対・`~/`・絶対のどれでもよい。相対にしておくと、フォルダごと配れる。

## 7. 革に入らないもの

- **舞台の中**：枠の外の暗い所も含めて、作品の絵そのもの。
- **並べ方**：パネルの位置や大きさは、ドックで各自が決める。
- **窓全体の倍率**：見る人の好み。上の帯の「100%」をこすって 1% 刻みで変える（`~/.motolii/window.xml` に残る）。
- **動き方**：革は見た目だけ。

## 8. 配る

フォルダをそのまま zip にして渡す。受け取った人は `~/.motolii/skins/` に置いて View › Skin で選ぶ。

## 9. LLM に頼むとき

そのまま渡せる頼み方：

> `skin/book/skins/README.md` と `skins/ROLES.md` を読んで、`~/.motolii/skins/<名前>/theme.xml` に「（欲しい見た目）」の革を書いて。`skin/book/skins/shoot.sh ~/.motolii/skins/<名前>` で撮って、`preview.png` を見て、気になる所を直して、また撮る。読めない文字や差し色の埋もれがなくなるまで繰り返して。

## まだできないこと

- 部品の大きさのうち、約300か所はまだ決め打ち。`Size` で変わらない所がある。
- タイムラインの目盛りや曲線などの手描きの線は、太さを変えられない。
- 窓の中で色を選んで革を作る道具はない。書くのは theme.xml。

