# 数値を「別の概念」へ変換する GUI — 出荷済みの実例と Motolii への転用

目的: 面白い GUI は slider の皮替えではなく、NUMBER を PLACE / SHAPE / 物理現象の縮図 / CHARACTER に変換する。実例を集め、規則を抜き、Motolii の 40 parameter に当てる。

## 証拠の注意(先に読む)
- この調査では live fetch を行っていない。したがって `[seen]` は 0 件。全項目は `[snippet]`(公式 manual・製品ページの記述を記憶から要約。語句は未再確認)か `[inferred]`(UI の見た目・挙動を知識から推定)。
- URL は製品/manual の入口。深い anchor は未確認。採用前に 1 度画面で確認すること(特に Ableton 12 系の新機能、Blender 4.3+ の gizmo)。
- 「gesture」「数値は見えるか」は一般的な操作の記述。version 差あり。

凡例: 数値 → 何になる → gesture → 数値が見えるか → 証拠

---

## A. 点(1 つの値が「場所にある取っ手」になる)

1. **Houdini viewport handles** — 数値: node の translate/rotate/scale/radius 等。→ 3D 空間の矢印・リング・球の handle。gesture: viewport で handle を drag(軸/平面/回転環)。数値: parameter pane に併存し、drag と同期。[snippet] https://www.sidefx.com/docs/houdini/ref/handles.html
2. **Blender Geometry Nodes Gizmos(Linear / Dial / Transform Gizmo, 4.3 以降)** — 数値: node group の入力 float / 角度 / transform。→ 作ったモデルの上に居る矢印・ダイヤル・箱 handle。gesture: モデル上で drag。数値: modifier panel に残り、drag で値が動く。要点: 値が「Inspector」ではなく「物のそば」に住む。[snippet] https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/ (Gizmo nodes の節)
3. **Figma corner radius handle / padding handle** — 数値: 角丸 px、auto layout の padding/gap。→ 角の内側の点、余白の帯。gesture: 角の点を内へ drag、余白の帯を drag。数値: drag 中に tooltip、panel にも併存。[snippet] https://help.figma.com/hc/en-us/articles/360041051493 (corner radius)

## B. 線(値が曲線の形・位置になる)

4. **Ableton Auto Filter** — 数値: Frequency / Resonance / Filter type。→ 周波数応答の曲線そのもの。gesture: 表示面を drag(横=cutoff、縦=resonance)。数値: knob も併存、drag 中は数値表示。[snippet] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Auto Filter)
5. **FabFilter Pro-Q** — 数値: band の freq / gain / Q / shape。→ スペクトラム上の「点」と、そこから生える山形。gesture: 点を drag(上下=gain、左右=freq)、wheel/scroll=Q、点の周りの曲線をなぞる。数値: ホバーで tooltip・下の欄に残る。背景に実音のスペクトラムが流れる(数値が「今の音」の上に置かれる)。[snippet] https://www.fabfilter.com/products/pro-q-4-equalizer-plug-in
6. **FabFilter Pro-C / Ableton Compressor** — 数値: Threshold / Ratio / Knee。→ 入力 dB 対 出力 dB の折れ線(transfer curve)+ 今の入力レベルが走るメーター。gesture: 折れ点を drag、線の傾きを drag。数値: 併存。要点: 「Ratio 4:1」が「折れた後の傾き」という形になる。[snippet] https://www.fabfilter.com/products/pro-c-2-compressor-plug-in , https://www.ableton.com/en/manual/live-audio-effect-reference/ (Compressor)
7. **FabFilter Saturn / Ableton Roar の shaping curve** — 数値: drive / 歪みの種類 / 帯域毎の量。→ 波形の入出力曲線、帯域ごとの山。gesture: 曲線の点を drag、band の境界を drag。数値: 併存。[snippet] https://www.fabfilter.com/products/saturn-2-multiband-distortion-saturation-plug-in , https://www.ableton.com/en/manual/live-audio-effect-reference/ (Roar)
8. **Procreate Brush Studio — pressure / tilt / velocity curve** — 数値: 筆圧→太さ/不透明度の対応。→ 入力(筆圧)対 出力の曲線。gesture: 曲線を直接 drag、その場で試し描き。数値: 小さい。要点: 試し描きの領域と曲線が隣り合う。[snippet] https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings
9. **Blender Float Curve / Curve widget(node・modifier 内)** — 数値: 0..1 入力→出力の remap。→ 編集できる曲線 widget。gesture: 制御点を drag、handle の種別切替。数値: 点を選ぶと x/y 数値欄。[snippet] https://docs.blender.org/manual/en/latest/interface/controls/templates/curve.html

## C. 面(値が絵・板になる)

10. **Ableton Wavetable / Serum / Vital の wavetable 表示** — 数値: Position(0..100%)、warp 量。→ 波形の 3D 積層、その中の「今の 1 枚」。gesture: 位置を drag すると波形が連続変形する様子が見える。Vital/Serum は波形を直接描く(draw)。数値: 併存。要点: 位置という 1 つの数値が「束の中の 1 枚」という場所になる。[snippet] https://www.ableton.com/en/manual/live-instrument-reference/ (Wavetable) , https://vital.audio , https://www.xferrecords.com/products/serum
11. **Ableton Spectral Resonator / Spectral Time** — 数値: Freeze 位置・Decay・Stretch。→ スペクトルの断面、凍った瞬間。gesture: 周波数分割や Freeze の面を操作。[inferred(見た目の詳細)] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Spectral Resonator / Spectral Time)

## D. 場所(値が空間内の領域になる)

12. **Cavalry Falloff** — 数値: Falloff の amount / spread / shape。→ シーン上の図形(円・直線・箱)が領域として置かれ、領域内で効きが変わる。gesture: viewport で Falloff の図形を移動/拡縮、プロファイル(減衰形)を Graph で編集。数値: Attribute Editor に併存。要点: 「効き」という抽象が「図形の中/外」になる。[snippet] https://docs.cavalry.scenegroup.co/ (Falloffs)
13. **Ableton Beat Repeat** — 数値: Interval / Grid / Gate / Chance。→ 時間上の「どこを繰り返し切り出すか」(grid の格子、gate の長さ)。gesture: 格子選択、gate の長さ drag。[snippet・見た目は inferred] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Beat Repeat)
14. **Blender node socket / Viewer の preview** — 数値: socket を流れる値。→ 値の中身が 3D 空間上の色/数字/点の列として見える(Spreadsheet、Viewer node)。gesture: hover / viewer を繋ぐ。数値: ある(spreadsheet)。要点: 値の「出所を示す場所」で見える。[snippet] https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/output/viewer.html

## E. 距離(値が「どこまで届くか」になる)

15. **Blender camera focus gizmo / light gizmo** — 数値: Focus Distance、light の radius/spot size/blend。→ 視界の中の「ピント面」の平面、光の円錐の開き。gesture: viewport で面/円錐の縁を drag。数値: Properties に併存。[snippet] https://docs.blender.org/manual/en/latest/editors/3dview/display/gizmo.html
16. **Ableton Hybrid Reverb / Echo** — 数値: Pre-delay / Decay / Size / Time / Feedback。→ 部屋の減衰形(時間軸の下り坂)、Echo は残響の反射が並ぶ時間軸。gesture: 時間軸上の形を drag、Echo の Time は格子へ吸着。数値: 併存。[snippet・見た目は inferred] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Hybrid Reverb / Echo)

## F. 形(値が「形」の輪郭になる)

17. **Ableton Operator** — 数値: 4 operator の level/ratio/feedback、各 envelope。→ algorithm の接続図(誰が誰を変調するか)、envelope の折れ線。gesture: 図を選ぶ/折れ点を drag。数値: 併存。要点: 「FM 量」が「繋がり図」になる。[snippet] https://www.ableton.com/en/manual/live-instrument-reference/ (Operator)
18. **Ableton envelope displays(ADSR 一般)** — 数値: Attack / Decay / Sustain / Release 4 つ。→ 折れ線 1 本(山の形)。gesture: 折れ点 3 つを drag(横=時間、縦=レベル)。数値: 併存。要点: 4 つの数値が 1 つの「山」になり、1 gesture で複数値が動く。[snippet] https://www.ableton.com/en/manual/live-instrument-reference/ (Simpler/Operator envelope)
19. **Ableton Drift / Cycling Envelope(Drift, Meld)** — 数値: Cycle の rate/tilt/hold。→ 繰り返す山(LFO 兼 envelope)の形。gesture: 形を drag。[snippet・見た目は inferred] https://www.ableton.com/en/manual/live-instrument-reference/ (Drift)

## G. 状態(値の束が「名前のある状態」になる)

20. **Ableton Macro Controls + Macro Variations** — 数値: 8 knob が最大 16+ の parameter を範囲付きで束ねる。→ 1 knob = 「意味」(例: 「明るさ」)。Variations は knob の状態を保存/呼び出し、randomize で別の状態を得る。gesture: knob を回す、Variation を叩く、Randomize。数値: knob の下に 0–127。要点: 多数が 1 つの意味に畳まれ、状態が名前を持つ。[snippet] https://www.ableton.com/en/manual/instrument-drum-and-effect-racks/ (Macro Controls) ※Variations の導入版は未確認
21. **Figma variables / modes** — 数値: color/number/string の変数、mode(Light/Dark 等)。→ 「mode」という名前の状態切替で全体の値がまとめて入れ替わる。gesture: frame の mode を選ぶ。[snippet] https://help.figma.com/hc/en-us/articles/15339657135383

## H. 関係(数値が「これに反応する」になる)

22. **Vital / Serum / Pigments の modulation(source を knob へ drag)** — 数値: 変調量 amount。→ 「LFO/envelope を knob へ drop」し、knob の周りに範囲リングが出る。gesture: source を target へ drag、ring を drag して量を決める。数値: 併存(hover)。要点: 関係を「繋ぐ」行為で作る。量は knob の周りの弧。[snippet] https://vital.audio , https://www.arturia.com/products/pigments
23. **Max for Live Expression Control / MPE Control(Max Essentials)** — 数値: 入力(velocity, pressure 等)→ 対象 parameter の range と curve。→ 「この入力に反応する」という接続+範囲の窓+曲線。gesture: min/max を drag、curve を drag。[snippet] https://www.ableton.com/en/live/max-for-live/ (Max Essentials)
24. **Cavalry connect / Graph(attribute を繋ぐ・曲線で写像)** — 数値: attribute 間の接続、Value を graph で remap。→ 「繋がり」と「写像曲線」。gesture: 接続を drag、曲線を編集。[snippet] https://docs.cavalry.scenegroup.co/

## I. 物理現象(数値が縮図の現象になる)

25. **Ableton Grain Delay** — 数値: Spray / Frequency / Pitch / Time。→ 粒が撒かれる XY(横=遅れ、縦=ピッチ等)の 2D 面で、点を動かすと粒の散りが変わる。gesture: 2D 面で点を drag。数値: 併存。[snippet・見た目は inferred] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Grain Delay)
26. **Ableton Corpus** — 数値: Resonance type(Beam, Marimba, String, Membrane, Plate, Pipe, Tube)、Decay、Material、Tune。→ 叩かれる「物体」を選び、material/材質で音が変わる。数値が「物の種類と材質」という物理になる。gesture: type 選択、材質 knob。[snippet] https://www.ableton.com/en/manual/live-audio-effect-reference/ (Corpus)
27. **Ableton Meld / Shifter** — Meld: 2 oscillator の macro 的操作(Shape、Tilt、Bias、XY pad で 2 つの shape を連続変形)。Shifter: Pitch/Frequency Shifter, Ring Mod の切替。→ 2D 面の 1 点が「2 つの波形の混ざり」を決める。[snippet・見た目は inferred] https://www.ableton.com/en/manual/live-instrument-reference/ (Meld) , https://www.ableton.com/en/manual/live-audio-effect-reference/ (Shifter)
28. **Notch / TouchDesigner のノード内 preview(Ramp TOP の gradient、Noise の 3D 平面)** — 数値: gradient stop 位置・色、noise の period/amplitude。→ stop が帯の上の点、noise はその場の動く絵。gesture: stop を drag、値を触ると絵がその場で変わる。[inferred] https://derivative.ca/UserGuide/Ramp_TOP , https://www.notch.one

## J. 人格(数値が「キャラ」になる)

29. **キャラクタークリエーター / sculpt(Spore Creature Creator, Elden Ring/ Black Desert 等の顔 mod)** — 数値: 骨の長さ・太さ・位置の多数。→ 体そのもの。gesture: 体の部位を掴んで引く/押す、顔の点を動かす、slider は補助。数値: 隠れることが多い。要点: 数値群が「見た目の人格」に畳まれる。[inferred] https://www.ea.com/games/spore (Creature Creator)
30. **Teenage Engineering OP-1 の encoder 毎の画面 graphics** — 数値: 4 つの encoder の値。→ engine ごとに専用の小さな絵(synth の各 engine で異なる絵、drum の波形、tape の絵)が動く。gesture: encoder を回すと絵の特定の部位が変わる。数値: 無い/ほぼ見えない。要点: 数値が小さな絵の「具体的な部位」に対応し、絵で覚える。[snippet] https://teenage.engineering/products/op-1

(Vital/Serum の「波形を描く」、Procreate の「試し描き領域」、Figma の「mode」は上で再掲しない。)

---

## 変換の規則 — slider の皮替えでなく、現象の縮図になる条件

1. **1 つの数値は 1 つの「場所」を持つ。** 数値が Inspector の行ではなく、対象のそば/内/上に住む(Blender gizmo, Cavalry falloff, FabFilter の点)。取っ手の座標が意味を持つ。
2. **複数の数値は 1 つの形にまとまる。** ADSR → 山、Ratio+Knee+Threshold → 折れ線、Pro-Q の freq+gain+Q → 点と山。1 gesture で複数値が連動する。
3. **現象の「今の状態」が同じ面に流れる。** Pro-Q のスペクトラム、Compressor の入力レベル、Wavetable の今の 1 枚、OP-1 の動く絵。値だけ静止していない。
4. **gesture は現象の名前を持つ。** 摘む(pinch)、扇ぐ、引っ張る、こする、曲げる。drag の向きが物理の向きに一致する(上=増える、でなく、外へ開く=散る)。
5. **数値は消えないが 4 番目。** 併存はする(hover/選択/drag 中に出す)。ただし主役は絵。数値入力(欄)は最後の手段。
6. **関係は繋ぐ行為で作り、量は弧/幅で見せる。** Vital/Serum の modulation、Expression Control。「効く」は線・リングで示す。
7. (補足)**状態は名前を持ち保存できる。** Macro Variations、Figma modes。

## silhouette test(screenshot に当てる 1 句)

> **「その画面を黒く塗り潰して輪郭だけ見て、対象の現象(群れ、扇、光線、星、山)に見えるか。**
> 輪郭が slider / knob / XY pad / bar / stepper に読めたら不合格。」

補助: (a) 数値欄と目盛を消しても何の parameter か当てられるか。(b) 現象の「今の状態」が動いているか。

---

## Motolii で変換する 40 parameter

| # | parameter | 変える現象(1 行) |
|---|---|---|
| 1 | Scatter amount | 小さな群れ(flock)を摘んで広げる。つまむ=集まる、放す=散る |
| 2 | Scatter spread | 群れの輪郭(楕円)を引き伸ばす領域。縁を drag |
| 3 | Scatter seed | 「振り直し」= 群れを軽く揺すって別の配置へ(サイコロでなく、揺れの結果) |
| 4 | Stagger offset | カードの扇(fan)。開き幅=時間差 |
| 5 | Stagger direction | 扇の要(かなめ)を回して向きを決める |
| 6 | Stagger range | 扇の枚数範囲を示す弧、どこからどこまで効くか |
| 7 | Falloff strength | 領域の濃さ(中心の密度) |
| 8 | Falloff shape | 領域の形そのもの(円・線・箱)を図形として置く(Cavalry 型) |
| 9 | Falloff graph | 距離→効きの曲線(縁がなだらか/急) |
| 10 | Refraction IOR | 1 本の光線が曲がる(屈折)。入射角と出射角の差 |
| 11 | Roughness | 表面をこする。撫でると粗くなる、光沢の広がりが見える |
| 12 | Glow radius | 星の腕の長さ |
| 13 | Glow intensity | 星の明るさ(芯の輝度) |
| 14 | Glow threshold | 星に点火する明るさの閾値線(水位) |
| 15 | Blur radius | ピントの外れた光点(ボケ玉)の大きさ |
| 16 | Blur aperture | 絞り羽根の形(多角形)、開閉 |
| 17 | Blur direction | 動いた軌跡の矢(motion の尾) |
| 18 | Noise frequency | 布/地形の起伏の細かさ(つまんで密にする) |
| 19 | Noise amplitude | 起伏の高さ(山を押す/引く) |
| 20 | Noise evolution | 水面/流れの向きと速さ(流れを撫でる) |
| 21 | Easing(ADSR 風) | 山の折れ線(Ableton envelope 型)。折れ点を drag |
| 22 | Echo/trail feedback | 反響が並ぶ残像の列、減衰する足跡 |
| 23 | Repeat count | 並ぶコピーの列(縮んでいく行進) |
| 24 | Repeat offset | 軌道上の間隔(歩幅) |
| 25 | Rotation | 地球儀/ダイヤル(物体を掴んで回す) |
| 26 | Spin | 回転の勢い(独楽。弾いて回す) |
| 27 | Scale stretch | ゴム(伸びる板)。引くと細く長くなる |
| 28 | Colour | 光のスペクトル/絵の具の混色皿(色を選ぶのでなく混ぜる) |
| 29 | Warp/distort | 布の格子を引っ張る。つまんだ点の周りが歪む |
| 30 | Mask feather | 縁の霧(ぼかしの帯)、縁を指でぼかす |
| 31 | Shadow light direction | 光源(太陽)を回すと影が伸びる。小さな球と影の針 |
| 32 | Shadow softness | 光源の大きさ(面光源)、半影の幅 |
| 33 | Depth of field focus | ピント面の平面(Blender の focus gizmo 型)、奥行き方向に動かす |
| 34 | Camera shake | 手持ちの揺れの軌跡(線が震える量) |
| 35 | Time remap speed | テープ/針(再生ヘッドの傾き)、傾きが速度 |
| 36 | Loop | 輪(周回する軌道)、継ぎ目の位置 |
| 37 | Wiggle | 糸で吊られた点の揺れ、振幅の円 |
| 38 | Spring/damping | 実際のばね+おもり。揺れの収束が見える |
| 39 | Gravity/attractor | 質量のある点(重力井戸)。周りの粒が引かれる |
| 40 | Extrude depth | 板を押し出す(厚み)、側面が育つ |

追加の候補(上限超え、必要なら差し替え): Stroke width(筆圧の太さ曲線、Procreate 型)、Dash(糸/点線の目)、Grid cols/rows/gap(格子を引っ張る・隙間を挟む)、Opacity(曇りガラスの透け)、Hue/Saturation(色相環/皿に落とす水の濃さ)。

## 次の使い方(研究側からの提案、実装ではない)
- 40 個すべてを作らない。silhouette test に通る 3〜5 個を試作し、Widgetbook で knob 比較する(既存の UI loop に従う)。
- 先に作る候補: Glow(星)、Scatter(群れ)、Stagger(扇)、Falloff(Cavalry 型)、Easing(山)。それぞれ「数値が 4 番目で残るか」を確認する。
- 確認が要る点: 各 [snippet] の公式 manual 再確認、OP-1 と Spore の見た目(画像で確認)。
