# 革（スキン）の作り方

革は、窓の見た目だけを着せ替えるもの。中身（書類・動き・絵）は何も変わらない。
革は1つのフォルダで、中身は次の3つ。

```
my_skin/
  theme.xml      ← 色・形・書体（これだけでも革になる）
  parts/*.svg    ← 印の絵（なくてもよい）
  *.otf / *.ttf  ← 書体ファイル（なくてもよい）
```

## 1. まず着てみる

見本の `labs.xml`（Google Labs 風の白い革）を、自分の場所に写す。

```bash
cp skins/labs.xml ~/.motolii/theme.xml && cp -R skins/labs ~/.motolii/labs
```

窓は `theme.xml` が書き換わった瞬間に読み直すので、開いたままでよい。
元に戻すときは `~/.motolii/theme.xml` を消す。

## 2. ひな形から始める

- 暗い革なら `themes/Default Dark.xml`、明るい革なら `themes/Default Light.xml` を写して直す。どちらにも色が全部並んでいる。
- 書かなかった名前は、窓のいつもの値のまま。だから、変えたい所だけ書いてもよい。

## 3. 色

`<名前 Value="#rrggbb"/>` で書く（透明度つきは `#rrggbbaa`）。名前の全部は [ROLES.md](ROLES.md) にある。

- **Grey00〜Grey100**：窓の灰色の段。暗い革では Grey00 が一番奥の地、数が大きいほど明るくなり、Grey95 が文字。明るい革にするときは段をひっくり返して（Grey00 側を白、Grey95 側を黒）、`<Shade Value="light"/>` を書く。`labs.xml` がその例。
- **ChosenDefault**：選んだ物・押している物の色（窓の差し色は1つ）。
- **Clip1〜7**：タイムラインの帯の色。**Keyframe**：キーの印。**AxisX/Y/Z**：軸の色。
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

- **並べ方**：パネルの位置や大きさは、ドックで各自が決める。
- **窓全体の倍率**：見る人の好み。上の帯の「100%」をこすって 1% 刻みで変える（`~/.motolii/window.xml` に残る）。
- **動き方**：革は見た目だけ。

## 8. 配る

フォルダをそのまま zip にして渡す。受け取った人は、`theme.xml` を `~/.motolii/theme.xml` に、残りをその隣に置く。

## まだできないこと

- 部品の大きさのうち、約300か所はまだ決め打ち。`Size` で変わらない所がある。
- タイムラインの目盛りや曲線などの手描きの線は、太さを変えられない。
- 窓の中で革を選んだり作ったりする道具はまだない。ファイルを置くだけ。

## 開発者向け：撮って確かめる

```bash
flutter test integration_test/skin_test.dart -d macos
```

いつもの革・labs・labs の125%を撮って `reports/skins/` に置く。
