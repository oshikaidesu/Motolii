# 革で使える名前（theme.xml の役割）

スキンのフォルダの `theme.xml` に `<名前 Value="..."/>` を並べる。書かない名前は窓のいつもの値のまま。窓は theme.xml が書き換わった瞬間に読み直す。見本は [paper](paper/theme.xml)（短い）と [labs](labs/theme.xml)（作り込み）。
この一覧と窓が読む名前が食い違うと `test/skin_names_test.dart` が落ちる。

## 色（`#rrggbb` か `#rrggbbaa`）

### 地と文字：まずこの2つ
| 名前 | 窓のどこ |
|---|---|
| `Background` | パネルの地と上の帯。書いた色がそのまま地になる |
| `Foreground` | 文字。書いた色がそのまま文字になる |

この2つを書くと、下の灰色の段が全部この2色の間に作られる（明るい地なら明るい革になり、`Shade` は要らない）。明るさは2色の間を順に、色味は地に近い段（Grey00〜Grey44）が Background の、文字に近い段（Grey56〜）が Foreground のものになるので、紫の地に黄の文字のような強い2色でも濁らない。段を1つずつ書けば、その段だけ上書き。

### 灰色の段（暗い順。明るい革では明るい順）
| 名前 | 窓のどこ |
|---|---|
| `Grey00` | パネルとパネルのすき間（一番奥） |
| `Grey07` | パネルの地、上の帯（= Background） |
| `Grey10` | 一段沈んだ地（舞台のまわり、タイムラインとグラフの地） |
| `Grey13` | 札（インスペクタの値の札、カード）、定規の地 |
| `Grey15` | 入力欄、押せる物の地、定規の目盛りの帯 |
| `Grey20` | 区切りの線、持ち上がった物、手を乗せた時の地 |
| `Grey26` | 強めの線、枠 |
| `Grey38` | 目盛り、使えない時の印 |
| `Grey44` | ごく薄い文字 |
| `Grey56` | 薄い文字（見出しの小さい字、単位、説明） |
| `Grey63` `Grey76` | 少し薄い文字、印 |
| `Grey91` | 文字（少し控えめ） |
| `Grey95` | 文字（= Foreground） |
| `Grey100` | 一番強い文字 |

### タイムライン
| 名前 | 窓のどこ |
|---|---|
| `BandA` `BandB` | 行の縞（交互） |
| `RowLine` | 行の間の線 |
| `Clip1` `Clip2` `Clip3` `Clip4` `Clip5` `Clip6` `Clip7` | 層の種類の色：1 形、2 文字、3 画像・動画、4 カメラ、5 グループ、6 粒子、7 音。タイムラインの帯のほか、その種類の札や印（Camera パネルの「Camera」の札は 4、作る棚の印）にも使う。選んでいない帯はこの色を少し地の色へ寄せて描く。帯の上の文字は `ControlOnForeground` |
| `Keyframe` | キーの印 |

### 差し色
| 名前 | 窓のどこ |
|---|---|
| `ChosenDefault` | 差し色：選んだ物、押している物、Render や Edit の塗り |
| `ControlOnForeground` | 差し色の塗りの上の文字と印（Render、Edit の字） |
| `ChosenInk` | 差し色を地の上で線や文字に使う時の色（書かなければ、明るい革では読める濃さに自動で沈める） |

### そのほか
| 名前 | 窓のどこ |
|---|---|
| `AxisX` `AxisY` `AxisZ` | 軸の色（X・Y・Z の印、ハンドル） |
| `HandleMove` `HandleScale` `HandleRotate` `HandlePivot` `HandleLift` | インスペクタの中で手で動かす図の色（移動・拡大・回転・基準点・持ち上げ）：変形の図、Anchor の立方体と札、光や複製のつまみ。舞台の上の枠と持ち手は差し色（`ChosenDefault`） |
| `Alert` | 知らせの赤い点（上の帯） |
| `Shadow` | 浮いている物（開いた選びの一覧、つかんだ札）の影 |
| `Glaze9` `Glaze15` | タイムラインのコマの縦線（薄く重ねる線：細かい方・太い方） |
| `ToneTransform` `ToneLayer` `ToneText` `ToneFill` `ToneBlend` `TonePlace` `TonePath` `ToneSolid` `ToneTextAnim` `ToneMask` `ToneTime` `ToneLayout` `ToneParticles` `ToneCamera` `ToneOutput` `ToneLink` | インスペクタの札の種類ごとの印の色（変形、層、文字、塗り、合成、配置、パス、立体、文字の動き、マスク、時間、並べ方、粒子、カメラ、書き出し、つながり） |

## 数
`Corner` `Size` `Spacing` `TextScale`
（どれも「いつもの何倍か」。Corner＝角の丸み、Size＝部品（行・帯・列・札）の大きさ、Spacing＝余白とすき間、TextScale＝文字の大きさ）

## 言葉
`FontMono` `FontSans` `Parts` `Shade`

- Shade: `light` か `dark`。Background を書けば要らない
- FontSans / FontMono: 書体の名前。`FontSansFile` / `FontMonoFile` に書体ファイルの場所を書くと、その名前で読み込む
- Parts: 絵の部品のフォルダ（名前は PARTS.md）。場所は theme.xml から見た相対・`~/`・絶対のどれでもよい

## 革では変わらない所
- 舞台の中（枠の外の暗い所も）：作品の絵そのもの。枠の外は作品の世界の続きを暗くして見せている
- 書類の中身の色（物の色、パレット、合成の見本の絵）
