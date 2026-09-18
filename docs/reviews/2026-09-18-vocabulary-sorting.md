# 語彙の仕分け — 今日の棚で作れる / 解き手が要る / まだ口が無い(2026-09-18 夜)

元: 2026-09-18-expression-vocabulary.md(映像表現 75 ジャンル・作家 31・褒め言葉 81)、2026-09-18-generative-vocabulary.md(技法 66・作家 31・Genuary 186)。
規則(生成の側の報告が出した境目): **格子・タイル・関数の絵は純、粒子・成長・群れ・反応拡散・物理は記憶。** Motolii では純 = 棚の block(Vism、Rust 0)、記憶 = 解き手(Rapier / GPU の RopePass の型)。

## A. 今日の棚で作れる(純関数、block 1 枚 + 台本)

| 語彙(技法 / ジャンル) | 今日の札か関数 | 足りない物 |
|---|---|---|
| sine wave grid、AdditiveWave、Lissajous、harmonograph、spirograph、Oscillator 系 | Wave、cv_oscillator、Formations の circle/helix | 無し |
| Truchet、10 PRINT、hexagonal/Penrose tiling、grid of rotating lines、recursive subdivision(Mondrian) | Mazin、cv_grid、cv_random | 形の種類は台本側(rectangle/line)。タイルの「回転で繋がる」は Mazin の型 |
| phyllotaxis、circle distribution、Fibonacci、spiral、Rose | cv_circle、cv_spiral、Formations | cv_fibonacci(ゴールデン角)を棚に 3 行 |
| moiré、op-art、interference | Concentrick、Optical Art | 無し |
| noise displacement、domain warping(位置に掛ける範囲)、Perlin loop | Zero Gravity、Ring Ting、cv_noise | loop は cv_noise に周期の引数(Genuary「Perfect loop」) |
| falloff 入場、Range、Sequence、Stagger、kinetic typography の入場の型 | Falloff Reveal、Arrive、箱の Stagger、cv_stagger/cv_range | Split の単位が物になる事(語・字ごとに block が掛かる) |
| formations(table→sphere→helix→grid)、Blend Sub-Mesh Positions、three.js periodic table | Formations | 無し |
| Molnár の disorder、Nees の Schotter(格子 + 番号で増える乱れ) | cv_grid + cv_random × (row) | 札 1 枚(10 行) |
| LeWitt 系(指示の絵)、Sol LeWitt wall drawing | 台本そのものが指示 | 無し |
| kaleidoscope / symmetry / wallpaper group | 無い | 鏡映の口(motion で写像を掛ける: 反転の行列)。中 |
| Mandelbrot / Julia、SDF、raymarching(画素の絵) | 棚の fx(surface / pass)側、block ではない | 既にある(Shadertoy 型の fs) |
| dithering、halftone、pixel sort(静止の絵として)、color quantization | fx 側(pass) | 既存の fs で書ける。pixel sort は行ごとの並べ替えで pass 1 枚 |
| ASCII art、generative typography、SDF text | 文字の層 + block | Split の単位 = 物 |
| ジャンル: kinetic type、lyric video(ボカロ型の一枚絵 + 歌詞)、Swiss/International、brutalism、Corporate Memphis、Cavalry 系、Lottie 系 | 鍵と層 + 今日の札 | 無し(素材と色の決め切りの問題) |
| ジャンル: Y2K、Frutiger Aero、vaporwave、acid graphics | 素材と fx の問題(ガラス・グラデ・粒子) | 質感は fx 側(既存の Glass・Gradient・Noise・Bloom) |

## B. 解き手が要る(記憶。Rapier か GPU の ping-pong)

| 語彙 | 何が記憶か | Motolii の置き場 |
|---|---|---|
| flow field / particle trails / curl noise / sand spline / Substrate | 粒子が進む(前のコマの位置) | RopePass と同じ型(GPU の state、読み戻さない)。「粒子」の口は未 |
| boids / flocking、gravity / orbits | 群れ・軌道(速度) | Rapier か GPU。Field(場)で半分は出る |
| reaction-diffusion(Gray-Scott)、Physarum、cellular automata(Life・Rule 30/110)、WFC | 格子の前状態 | fx の pass で ping-pong texture(既存の Background Delay / Time Difference の型) |
| differential growth、space colonization、L-system の成長 | 世代 | 時刻の関数に畳めない。解き手側 |
| spring / verlet cloth、strand / rope(Ringers の紐と杭) | ばね | 紐は今日 GPU で開けた(Rope)。布は同じ型の 2D 版 |
| feedback loop(映像フィードバック)、ripple / wave equation | 前のコマの絵 | pass の ping-pong(既存の型) |
| ジャンル: datamosh、glitch(動画の壊し)、VJ の audio-reactive | 復号の副データ・音の入力 | 核の文書の「口 6」(元素材の他コマ・復号の副データ)と音の口 |
| ジャンル: Sync(Cavalry)、物理で落ちて揃う | 解き手 | 物理の 4 住所のまま |

## C. まだ口が無い(道具の穴)

1. **複製が物になる**(Duplicator)。A の半分がこれ待ち(格子・タイル・文字の単位)。for 文で層を作る限り 118 が上限感
2. **Split の単位 = 物**(字・語・行に block が掛かる)。kinetic type と lyric video の芯
3. **鏡映・対称の写像**(kaleidoscope、wallpaper group)。motion に行列を 1 つ足す
4. **粒子の口**(GPU の state を持つ物、RopePass の一般化)。flow field と trails。「記憶を持つ物は解き手側」の規則で、名前と欄は相談
5. **音の口**(音量・拍を host に)。失われた世代の時計。A の全部に掛かる係数
6. **Strength / Falloff の外枠**(全札に係数)。掛け算の裁定待ち

## 褒め言葉の側(2026-09-18-expression-vocabulary.md §4)から、測れる物

- buttery / snappy / floaty(悪)/ sluggish(悪)/ タメ・ツメ・緩急 → 補間の話。間は法が埋めるので、札の ease と settle の欄
- weight / decisive / restrained / 決め切る → 素材・色・法の数。zz_reach の「手」の数と対
- うるさい(悪)/ 綺麗すぎる(悪)/ 情報量 → 1 手あたりの波及(zz_reach の値)。多すぎても少なすぎても悪

判断は書かない。C の順は利用者の裁定。
