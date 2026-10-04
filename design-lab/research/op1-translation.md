# OP-1 の「概念ごとの小さな世界」を Motolii へ翻訳する

2026-10-02。調査+設計翻訳のみ(コードなし)。`research/ideas-phenomenon.md` の #30(OP-1)を掘り下げた物。

## 0. 証拠の注意(先に読む)
- タグ: **[seen]** = 今回 URL を開いて本文を確認 / **[snippet]** = 検索結果の要約だけ / **[inferred]** = 上記から私が推論。
- 開けなかった物: 公式 guide の `.../original/synthesizers` `.../effects`(404。実在の章名は `synthesizer-mode`)。OP-1 field の PDF user guide(`https://teenage.engineering/_img/6275254dfb267f0004b9e832_original.pdf`)は本文抽出に失敗(バイナリ。poppler が無く画像化も不可)。OP-1 modules ページ(`https://teenage.engineering/products/op-1/original/modules`)は名前の一覧のみで画面の記述なし。OP-101 の Digital / Dr. Wave / Pulse / Phase の個別 description 全文と、effect 個別の画面の記述は未取得。
- **「Mother」reverb は公式 reference の effect 一覧(Delay, Grid, Nitro, Phone, Punch, Spring, CWO)に無い。** 利用者が言う Mother は今回の資料では確認できなかった(別名・後継機・field 限定・記憶違いのいずれか不明)。以下では未確認として扱い、Spring / Grid を代わりの実例にした。
- 4 encoder の色割り当て(blue→green→white→orange の並びで parameter 一覧の順)は **[inferred]**。公式 reference 表は parameter を並べるだけで色を書かない。色が確定しているのは下に [seen] と書いた物だけ。

## 1. OP-1 の事実

### 1.1 共通の骨格
- 4 つの色付き encoder は常に画面と対応する。「緑の図形または文字は、緑 encoder がその値や位置を変える合図」。[seen] https://teenage.engineering/guides/op-1/original/layout
- 各 synth engine の画面は T1 の下に常にあり、engine を選ぶと最初に出る。T2 = envelope、T3 = effect、T4 = LFO。[seen] https://teenage.engineering/guides/op-1/original/synthesizer-mode
- **同じ 4 本の encoder が、mode / 画面ごとに意味を入れ替える**(Envelope は Blue=Attack, Green=Decay, White=Sustain, Orange=Release [seen]、mixer は Blue/Green/White/Orange = Track 1-4 の音量、shift で pan [seen] https://teenage.engineering/guides/op-1/original/mixer、tape は Orange=録音レベル, Blue=scrub, White=tape 速度 [seen] https://teenage.engineering/guides/op-1/original/tape-mode)。この「固定 4 + 絵の差し替え」が設計の核。
- 1 engine の parameter は常に 4 つ(reference 表)。[seen] https://teenage.engineering/guides/op-1/original/reference

### 1.2 Synth engine
| engine | 画面(世界) | 4 parameter(reference 順 = blue, green, white, orange と推定 [inferred]) | 変調で動く物 / 数値のまま | 出所 |
|---|---|---|---|---|
| Cluster | 4 つの仮想ダイヤルの下に 4 本の動く波。ダイヤルは 270°(7:30-4:30) | Blue=# of waves(oscillator 数) / Green=Wave envelope(enveloped LPF。中央付近は dead zone) / White=Spread(detune の drift 深さ) / Orange=Unitor(pitch の揺れの速さ) | 波が増減・うねる。**この engine の絵はダイヤル+波=「数値の皮替え寄り」** | [seen] op-101 https://op-101.blogspot.com/2011/09/cluster-description.html(色割り当ては同記事。公式 reference は同 4 parameter 名) |
| String | 画面の大半が震える弦の束。弦の動きが音の動作を見せる | Blue=Tension(弦長が低いと垂れ、高いと張る) / Green=Impulse decay(1 本の棒グラフ) / White=Detune(2 点の位相差。2 つの白い形) / Orange=Impulse type(折れ線の棒、pick の硬さ) | 弦が震え、値で垂れ・張りが変わる。**tension だけが「物理の絵」で、残り 3 つは棒グラフ** | [seen] https://op-101.blogspot.com/2011/09/string-description.html |
| FM | 4 つの等角投影ワイヤーフレーム立方体。各立方体の色 = 対応 encoder。1 つが Base | Blue=FM intensity(立方体の青い輪郭が明るくなる) / (field は beige)=Frequency(8 値の組を巡回) / Grey=Topology(9 algorithm。立方体の並びが carrier/modulator を示す) / Orange=Detune(立方体が左右にずれる) | 輪郭の明度、位置のずれ。**関係(誰が誰を変調)が絵の構造そのもの** | [seen] Attack Magazine https://www.attackmagazine.com/technique/hardware-focus/unwrapping-an-unsung-hero-of-fm-synthesis-perhaps-exploring-the-op-1-field-op-1-og/。field では frequency を「予め決めた組から選ぶ」点が違う |
| Phase | 位相歪みの対話的な図。Blue(Phase shift)と Green(Distortion)は、同色の 2 つの正弦形が左右に滑る | Blue=Phase shift / Green=Distortion amount / White,Orange は未取得(reference 上は Phase filter, Phase tilt が Dr. Wave、Phase 自体は Filter, Amplitude, Second pulse, Mod. の表記が混在) | 形が滑る。**音の中身(再生位置の加減速)を 1 枚の絵にした好例** | [snippet] op-101 https://op-101.blogspot.com/2011/09/phase-description.html(本文は検索要約のみ) |
| Digital | 粒子と距離/棒の拘束(particle + stick constraint)のシミュレーション | wave shaper / octave / detune + ring mod / digitalness | 粒が動く。**OP-101 は「何を伝えているか分からず、ほぼ機能しない」と批判** | [snippet] https://op-101.blogspot.com/2011/09/digital-critique.html — **失敗例として規則 R7 の根拠** |
| Dr. Wave | 未取得 | Phase shift / Distortion amount / Phase filter / Phase tilt | — | 一覧のみ [seen] reference |
| Pulse | 未取得 | Filter / Amplitude / Second pulse / Mod. | — | 同上 |
| DNA, Voltage, D-Synth, D-Box, Sampler | 未取得 | reference に parameter 名のみ | — | 同上 |

### 1.3 Effect(T3)
reference の parameter 名 [seen] https://teenage.engineering/guides/op-1/original/reference。画面の中身と色割り当ては**取得できず**、下の「世界」は一般的な説明の [snippet] + 推論。
| effect | 型 | 4 parameter | 世界(取得できた分) |
|---|---|---|---|
| Delay | solid state delay | Size / Speed / Feedback / Mix | 未取得 |
| Grid | 三次元 feedback plate | X size / Y size / Z feedback / Mix | 立方体的な板(X=音程 Y=遅れ Z=feedback は [snippet] op-101 系記述)。**軸 = 物理の寸法に parameter を割る** [inferred] |
| Nitro | dual resonant turbo filter | Frequency / Filter freq follow / Resonance | 未取得 |
| Phone | hacked telephone system | Tone / Phonic / Baud | 未取得 |
| Punch | hard hitting LPF | Frequency / Punch / Rounds / Power | 未取得 |
| Spring | mathematic reverb | Tone / Turns / Damping / Send | ばね(Turns = 巻き数)[snippet] |
| CWO | pitch shifting delay | Frequency / Delay / Feedback / Sideband | 未取得 |
利用者の言う「Mother = 人と空間の距離」: **未確認**。ただ「reverb の量を『人と部屋の距離』という概念の絵にする」発想自体は採用する(下の Echo / Shadow / Camera)。

### 1.4 LFO(T4)・tape・mixer
- LFO 7 種(bend, crank, element, MIDI, random, tremolo, value)。それぞれ 4 encoder の役が違う(Element: Blue=source, Green=amount, White=destination, Orange=destination の詳細。Random: Blue=amount, Green=speed, White=destination, Orange=envelope。Tremolo: Blue=speed, Green=pitch, White=volume, Orange=envelope、shift+Orange で波形 sine/saw/exp/square/blip。Value: Blue=speed, Green=amount, White/Orange=destination。Crank: 手で回して振動。Bend: Orange が LFO 本体、shift+Orange で向き反転)。[seen] https://teenage.engineering/guides/op-1/original/lfo。「変調されると画面の中の図が動く」と TE は言う(製品説明)。
- Tape(Orange=rec level, Blue=scrub, White=speed)[seen]。Mixer: T1 トラック音量(青緑白橙 = トラック 1-4)/ T2 EQ(Blue=low, Green=mid, White=high, Orange=amount)/ T3 master effect / T4 master out(balance, drive, release)。shift+Mixer で signal flow 図。[seen]
- Sequencer は 7 種(Pattern, Finger, Endless, Tombola=回転する球, Sketch=自由描画, Arpeggio)。Tombola の parameter(Speed, Gravity, Shape, Mass)は「重力・質量」という物理語。[seen] reference。
- OP-1 field 固有の差: FM の frequency が組から選ぶ形(上)。その他の画面の差は取得できず。

### 1.5 何が数値のまま残っているか
String の Impulse decay と Impulse type は棒グラフ(世界ではなく数値の皮)。Cluster はダイヤル。Digital は絵が情報を運ばない。**OP-1 でも全 parameter が「現象」になっているわけではない。** 世界になっているのは tension(弦の垂れ)、FM の構造、Phase の位相の滑り、Grid の寸法あたり。ここが Motolii の勝ち筋: 4 つ全部を現象にする。

## 2. 翻訳の規則(10)
**R1. 1 概念 = 1 つの世界。** Inspector 行の頭に 1 つの小箱を置き、箱の中が効果全体の縮図。個別 parameter ごとの widget は作らない。
**R2. grab zone は 4 つまで、色は箱の中の要素と一致。** 色スロットは固定: **A=Fam.stagger(青) / B=Fam.along(緑) / C=N.g95(白) / D=Fam.follow(橙)**(OP-1 の blue/green/white/orange を既存 token に写す。新色を足さない = DESIGN.md の記号を増やさない)。箱の枠・主役は効果の Fam か N.g76 の灰。drag 中は対応する要素だけを残して他を g38 に落とす。
**R3. 同じ 4 つの握りが全世界で同じ位置・同じ順。** 左上から A,B,C,D。Inspector で効果を切り替えても指が覚えている(OP-1 の最大の価値)。
**R4. 待機でも生きている。** 非 hover 時に低振幅の呼吸(周期 3-6 s、振幅 ≤2 px)。再生中は実値で動く。**変調・キーフレーム・Relation で値が動くと、絵が実時間でその通りに動く**(本来の値の鏡)。
**R5. 数値は隠れた層。** 絵が主役。数値は drag 中・hover 中だけ出る静かな読み出し(N.g56 mono 10 px、値が既定と違えば Role.changed)。欄への入力は最後の手段。
**R6. 輪郭試験。** 黒塗りにして輪郭が slider / dial / XY / bar / stepper に見えたら不合格(ideas-phenomenon.md の silhouette test)。「円を位置のつまみに」「棒を量に」は禁止。
**R7. 絵は情報を運ぶ。** Digital の失敗(動くが何も語らない)を避ける: 絵の全要素は parameter のどれか、または入力から出力への写像のどれかに対応する。飾りの動きは待機の呼吸だけ。
**R8. 視覚言語は極小。** 1 px 線、点と小片、単色+アクセント 1-2 色、塗りつぶしは中心の 1 要素だけ、影・グラデーションの装飾なし。箱はおよそ正方形(Product Home の正方形基調)で 1 辺 96-128 px の「小」。
**R9. 動きは入力直後に始まり ease-out、hover で geometry は動かさない**(memory: motion-rule)。drag 以外で形が変わるのは値の変化だけ。
**R10. 世界は意味を決めない。** 相対/絶対・clamp・対象・undo の区切りは host が決める(presentation-intent-boundary)。世界は「何を握ったか」を送るだけで、値の算術を持たない。1 gesture = 1 undo。
**R11(補)。** 世界を持たない parameter はそのまま数値(Tombola 的に、物理語で名付ける:「重さ」「張り」)。全部を無理に絵にしない。

## 3. 30 の世界提案
共通の約束: 箱 112 px 角、背景 N.g07、線 N.g63、アクセント = 世界の Fam(無ければ Role.selected)。色スロット A 青 / B 緑 / C 白 / D 橙。範囲・既定・単位は **[inferred]**(実 host の parameter 名・範囲との照合が要る。Motolii の実 effect 名はパネル内の名前に従うが、range は下書き)。「待機」は idle life。読み出しは drag/hover 時のみ。pheno = Phenomenon A-E の既存 30 miniature との重なり(book/lib/sets/pheno_*.dart は作成中で、この時点で repo に存在せず読めなかった。重なりは ideas-phenomenon.md の「40 parameter」表の項目名に基づく推定で、**ファイルが出来たら 1 対 1 に照合し直すこと**)。

### 3.1 Relation(relation_set.dart の gadget 15 個から)
**1. Falloff(領域の濃さ)** — 世界: 箱の中心から広がる同心の波紋と、波紋の輪の濃さ(効きの強さ)。A: Radius(0-100 %、既定 50 %、px 換算は host) 輪を外へ押し広げる / B: Curve(0.4-4、既定 1.6)輪の間隔が詰まる・開く / C: Strength(0-100 %、既定 100)中心の点の輝度 / D: Centre(箱内位置を 2 軸で)点を滑らす。動くと: 波紋が外へ 1 周期流れ、値が keyed ならその軌跡を g38 の点列で。待機: 波紋が 4 s 周期で外へ。読み出し: `r 50 · ×1.6`。輪郭: 「波紋の的」(同心円=ダイヤルに見えやすいので輪は等間隔にせず curve に従い不揃いにして、盤面ではなく水面に見せる)。pheno: Falloff strength / shape / graph の 3 つを 1 つに統合(supersede)。
**2. Direction / Attract** — 世界: 小さな矢の群れが 1 点へ傾く風向図。A: Angle(0-360°、既定 0°) 風向 / B: Pull(0-1、既定 .55) 矢が中心点へ巻き込まれる / C: Strength(0-100 %) 矢の長さ / D: Target 点の位置(箱内)。動くと: 矢が絶え間なく流れ、target を動かせば全体がなびく。待機: 矢が微風で揺れる。読み出し: `45° pull .55`。輪郭: 「風に倒れる草」。pheno: 該当の「向き」系を superset。
**3. Scatter(群れ)** — 世界: 約 20 点の雲。A: Spread(0-200 px、既定 60)雲が膨らむ / B: Seed(1-999 整数、既定 3) 箱をクリックで「カチャッ」と別配置へ(click-clack のカード音は無く、点が 80 ms で跳ねて落ち着く) / C: Density(8-160 点、既定 60) 点数(箱は 20 点で代表) / D: Falloff(0-1、既定 .4)雲の一部が沈む(外縁の点が g38 に)。動くと: Spread の keyframe で雲が呼吸。待機: 点が ±1 px ゆらぐ。読み出し: `spread 60 · seed 3`。輪郭: 「雲」。pheno: Scatter amount / spread / seed の 3 つを supersede(群れ=flock 版と同じ主題。点数 20 と Seed の揺すりは重なる)。
**4. Stagger(階段のカード)** — 世界: 8 枚のカードが階段状に並び、ずれの幅が段差。A: Offset(0-1 s、既定 .08 s/個) 段差 / B: Range(1-100 %、既定 100) 効く枚数の範囲(弧の幅) / C: Direction(0-360°) 階段の向き / D: Ease(.5-2.5、既定 1)段差の間隔が詰まる。動くと: 再生ヘッド(白線)が階段を舐め、各カードが時間差で浮く。待機: ヘッドが 3 s で往復。読み出し: `80 ms ×8`。輪郭: 「階段」。pheno: Stagger offset / direction / range を supersede(扇 → 階段へ。扇のほうが良ければ pheno 側を残す)。
**5. Along Path** — 世界: 曲線上に並ぶ点列と、曲線の端の 2 つの旗。A: Start(0-100 %、既定 0)始点の旗 / B: End(0-100 %、既定 100) 終点の旗 / C: Spacing(0-1、既定 .5) 点の間隔 / D: Bias(.4-2.5、既定 1)点が片側に寄る。動くと: 点が曲線上を滑って詰まる。待機: 点がゆっくり前進して戻る。読み出し: `0–100 %`。輪郭: 「数珠」。pheno: 該当なし(新規)。
**6. Scale distribution** — 世界: 大きさが次第に変わる 7 つの丸の列。A: Min(0-200 %、既定 40) 列の端の大きさ / B: Max(既定 100) / C: Curve(.4-2.5) 変化の山の形 / D: Seed(乱れ 0-1、既定 0)。動くと: 丸が呼吸して列の形を保つ。待機: 全体が ±2 % 呼吸。読み出し: `40→100 %`。輪郭: 「ふくらむ列」。pheno: Scale 系を superset。
**7. Rotation distribution** — 世界: 9 本の短い針の扇。A: Start(°、既定 0) / B: Step(−90..90°、既定 20) 針ごとの回転 / C: Jitter(0-1、既定 .15)針の乱れ / D: Pivot 点(箱内)。動くと: 針が扇ぐ。待機: 微風。読み出し: `+20° ×9`。輪郭: 「扇」。pheno: Stagger direction の扇と形が競合するので片方を残す(推奨: Rotation = 針、Stagger = カード階段)。
**8. Colour gradient** — 世界: 色の帯を点が流れる(8 つの小さな玉が帯の上で色を変える)。A: From 色相(既定 Role.selected)/ B: To 色相 / C: Ease(.4-2.5、既定 1) 中間の色が寄る / D: Spread(0-100 %)。注意: 色の世界では R2 の色スロットを一時的に玉の色に譲らない — グラデは常に中身の色なので、grab の色分けは枠の 4 隅の小点だけで示す。動くと: 玉が帯上を滑る。待機: 帯が ±1 玉分流れる。読み出し: `#… → #…`。輪郭: 「流れる玉」。pheno: Fill / gradient 系を superset。
**9. Repeat(格子)** — 世界: 小さな図形が 5×3 に増殖する。A: Columns(2-8、既定 5) / B: Rows(2-6、既定 3) / C: Gap(0-100 %、既定 25) / D: Jitter(0-1、既定 .25)。動くと: 増える時に新しい図形が元から「ぽん」と分裂。待機: 図形が 1 つずつ点滅して元の位置を示す。読み出し: `5×3`。輪郭: 「増える図形」。pheno: Repeater(提案 11)と同一系統で統合推奨。
**10. Noise(relation)** — 世界: 値の波形が細かく震える 1 本の線。A: Roughness(0-1、既定 .45) 線の細かさ / B: Drift(0-2、既定 .5) 流れる速さ / C: Amount(0-100 %)線の振れ幅 / D: Seed。動くと: 線が流れ、drift 0 で止まる。待機: 流れが続く(これが生きている印)。読み出し: `rough .45`。輪郭: 「地震計の紙」。pheno: Noise frequency / amplitude を supersede。
**11. Graph / Link** — 世界: 2 つの小さな丸が線で繋がれ、片方を動かすともう一方が写像曲線に沿って動く。A: Strength(.1-1、既定 .7) / B: Type(line/curve/step 3 択) 線の形 / C: Offset(−100..100 %) / D: Pulse(on/off)。動くと: 繋がった線を光点が走る(変調=見える)。待機: 光点がたまに走る。読み出し: `×0.7 curve`。輪郭: 「2 点と糸」。pheno: Relation 系の接続は pheno が持つなら重複。
**12. Audio react** — 世界: 小さな耳(縦の 1 本の線)から出る波が、対象の点を揺する。A: Smooth(0-1、既定 .5) 波の滑らかさ / B: Gain(0-400 %、既定 100) / C: Threshold(−60..0 dB、既定 −30)水位線 / D: Band(low/mid/high)波の帯。動くと: 実際の音に合わせて波が点を揺する(R4 の模範)。待機: 音が無ければ水平線。読み出し: `−30 dB`。輪郭: 「水位と波」。pheno: Glow threshold の水位線と描画語彙が近い。

### 3.2 Effect
**13. Gaussian Blur** — 世界: ピントの外れた光の玉(ボケ玉)。A: Radius(0-200 px、既定 10)玉の大きさ / B: Aperture(3-8 枚、既定 6)玉が多角形になる / C: Direction(0-360°)玉が尾を引く(方向 blur のとき)/ D: Mix(0-100 %)。動くと: 玉が膨らむ・縮む。待機: 玉が淡く瞬く。読み出し: `10 px`。輪郭: 「にじむ光」。pheno: Blur radius / aperture / direction を supersede。
**14. Glow** — 世界: 光の点とそのハロ。A: Radius(0-300 px、既定 40) ハロの広がり / B: Intensity(0-300 %、既定 100) 点の輝度 / C: Threshold(0-100 %、既定 60)光り始める水位線 / D: Tint(色)。動くと: ハロが脈動(keyframe 通り)。待機: 点が ±3 % 瞬く。読み出し: `r 40 · 100 %`。輪郭: 「星」(腕の長さ=半径)。pheno: Glow radius / intensity / threshold を supersede。
**15. Echo** — 世界: 1 つの点から後ろへ減衰する複製の列(残像の尾)。A: Count(1-12、既定 4)複製数 / B: Delay(0-1 s、既定 .08)複製間の間隔 / C: Decay(0-100 %、既定 60) 尾の減り方 / D: Mix。動くと: 点が動けば残像が後ろに続く。待機: 点が往復し尾が追う(Echo の因果を毎コマ見せる)。読み出し: `×4 · 80 ms`。輪郭: 「彗星」。pheno: 該当なし(Echo は Time 系 pheno と近い可能性、要照合)。OP-1 の Delay / CWO と同じ主題。
**16. Wave Warp** — 世界: 格子の糸が波打つ。A: Amplitude(0-100 px、既定 20) 波の高さ / B: Wavelength(10-400 px、既定 120) / C: Phase(0-360°)波を滑らせる / D: Direction(0-360°)。動くと: Phase の keyframe で波が走る。待機: 波が自然に流れる(周期 4 s)。読み出し: `20 px / 120`。輪郭: 「旗」。pheno: Noise amplitude の「起伏」と似るが「波の進行」が主題で分ける。
**17. Fractal Noise** — 世界: 雲海(起伏の等高線)。A: Scale(1-1000 px、既定 200)起伏の細かさ / B: Contrast(0-300 %、既定 100) 山の高さ / C: Evolution(°/s、0-360、既定 0)雲が動く / D: Complexity(1-8、既定 3)等高線の細かさの層。動くと: 等高線が湧く・沈む。待機: evolution が 0 でも極微に呼吸。読み出し: `s200 c3`。輪郭: 「等高線の地図」。pheno: Noise(property)と重なる → 1 つに統合。
**18. Shadow** — 世界: 物体から落ちる影、その向きと長さと柔らかさ。A: Distance(0-100 px、既定 10)影の長さ / B: Angle(0-360°、既定 135°) 光の向き(小さな太陽の点が箱の縁を回る) / C: Softness(0-100 px、既定 12) 影の縁のにじみ / D: Opacity(0-100 %、既定 50)。動くと: 太陽が動けば影が伸縮。待機: 影がごく僅かに息をする。読み出し: `10 px @135°`。輪郭: 「日時計」。pheno: 該当なし。Mother 的な「距離」(物体と地面の距離 → 影のにじみ)を使う。
**19. Glass(屈折)** — 世界: ガラスの小片に光が入って曲がる。A: IOR(1.0-2.5、既定 1.5) 光が曲がる角度 / B: Roughness(0-1、既定 .1) 面を擦る → 出る光の束が散る / C: Thickness(0-100 px、既定 20) 小片の厚み(出射位置のずれ) / D: Dispersion(0-1、既定 0)出る光が虹に割れる。動くと: 入射の光(白い 1 本)が箱を横切り、小片で屈折して出る。待機: 入射角が ±5° ゆれる。読み出し: `ior 1.50`。輪郭: 「プリズム」。pheno: Refraction IOR / Roughness を supersede。
**20. Transform(layer の小さな舞台)** — 世界: 箱が layer の小舞台。layer の縮図の矩形が箱の中に居る。A: Position — 矩形を箱の中で運ぶ(原点は host 解釈)/ B: Scale(0-400 %、既定 100) 矩形の寸法 / C: Rotation(°)矩形の回り / D: Anchor。ここは XY パッド・ダイヤル・ハンドルの禁止に最も近い(R6)。差別化: 矩形を「掴む」のではなく、矩形に**重さと慣性の残像**を持たせ(動かすと軽い影が遅れて付く)、4 つの握りは矩形の 4 つの縁/隅ではなく、同心の 4 本の薄い「痕跡」に色を割る。**これは本提案で最も輪郭試験に危ういので、Phase 1 では採用せず検証枠。** 動くと: 矩形が keyframe の通りに動く(軌跡を点列で)。待機: 矩形が ±0.5 px。読み出し: `x 120 y −40`。pheno: 該当なし。
**21. Opacity / Blend** — 世界: 2 枚のすりガラスが重なる。A: Opacity(0-100 %、既定 100) 上の板の濃さ / B: Blend(Normal/Add/Multiply/Screen…) 重なりの色の混ざり(重なり部分の色が切り替わる)/ C: Clip to below(on/off) 上の板が下の形に切り取られる / D: Ghost。動くと: 重なりが濃淡する。待機: なし(静)でよい。読み出し: `70 % · Add`。輪郭: 「重なる 2 枚の板」。pheno: Opacity / Fill 系と重なる(要照合)。
**22. Fill / Gradient(Fill の世界)** — 8. と別に、Fill 単体: 塗り桶に色が満ちる。A: Colour(色相)/ B: Opacity(0-100 %)桶の水位 / C: Gradient angle(0-360°、既定 90°)傾き / D: Stops(2-5、既定 2)層の数。動くと: 水位が keyframe で上下。待機: 液面が微かに揺れる。読み出し: `#E974AB 80 %`。輪郭: 「水位と色の層」。pheno: Fill を supersede。
**23. Trim paths / Stroke** — 世界: 1 本の線が描かれていく(ペンが走る)。A: Start(0-100 %、既定 0) / B: End(0-100 %、既定 100)(2 つの「ペン先」)/ C: Width(0-60 px、既定 2) 線の太さ / D: Offset(0-100 %)線が回る。動くと: End を動かせば線が描かれ、Offset は線が循環。待機: 線の端が呼吸。読み出し: `0–100 %`。輪郭: 「筆の運び」。pheno: Stroke width を supersede。
**24. Mask feather** — 世界: 窓の穴とその縁のにじみ。A: Feather(0-200 px、既定 0)縁のにじみ / B: Expansion(−100..100 px)穴が広がる / C: Opacity(0-100 %) / D: Invert(on/off)穴と壁の反転。動くと: 縁が太る。待機: 縁が静かに脈打つ。読み出し: `feather 12`。輪郭: 「窓」。pheno: Falloff shape と近い領域図 → 一部重なる。

### 3.3 Property・動きの性質
**25. Easing** — 世界: 点が走り、軌跡が加減速を刻む(等間隔の足跡が詰まる・開く)。A: In(0-100 %、既定 33) 出だし / B: Out(0-100 %、既定 33)終わり / C: Overshoot(0-100 %、既定 0)行き過ぎて戻る / D: Duration(0-5 s、既定 1)。動くと: 点が走り直す(ループ 2 s)。待機: 同上。読み出し: `33/33`。輪郭: 「足跡」(曲線 editor = bar/bezier ハンドルは禁止なので、曲線を見せず足跡だけ出す)。pheno: Curve preset / Easing 系を supersede。
**26. Spring** — 世界: 点が 1 本のバネで壁に繋がれ、跳ねて落ち着く。A: Stiffness(1-500、既定 120) バネの太さ / B: Damping(0-50、既定 12) 揺れの収まり / C: Mass(.1-10、既定 1) 点の大きさ / D: Rest(位置)。動くと: 引いて放すと本物のばねで揺れる(Tombola 的)。待機: 点が時々自然に弾む。読み出し: `k120 c12`。輪郭: 「ばねの重り」。pheno: Spring/Ease の物理縮図と重なる可能性(要照合)。
**27. Wiggle** — 世界: 点が足元の円の中をうろつく。A: Frequency(0-20 Hz、既定 2)うろつく速さ / B: Amplitude(0-200 px、既定 20) うろつく円の半径 / C: Octaves(1-6、既定 1)震えの細かさ / D: Seed。動くと: 円が大きくなる。待機: 本当にうろつく(これが世界の主役)。読み出し: `2 Hz · 20`。輪郭: 「迷子」。pheno: Noise と重なる → Relation Noise(10)と区別: こちらは Property 単位。
**28. Time remap** — 世界: 2 本の帯(元の時間 / 出る時間)の間を渡る糸。A: Speed(−400..400 %、既定 100) 糸の傾き / B: Offset(−10..10 s)帯のずれ / C: Hold(0-100 %)止まる区間 / D: Smooth。動くと: 糸が傾くと再生中の点が加速・減速。待機: 点が帯を流れる。読み出し: `×1.0`。輪郭: 「織り糸」。pheno: Time 系 pheno と重なる可能性(要照合)。
**29. Loop** — 世界: 点が周回する小さな軌道。A: Count(1-∞、既定 ∞)周回数 / B: Type(cycle / ping-pong / offset) 軌道の形(円/往復/螺旋)/ C: Start(0-100 %) / D: End。動くと: 点が回る。待機: 常に回る。読み出し: `∞ cycle`。輪郭: 「軌道」。pheno: 該当なし。
**30. Camera** — 世界: 上から見た小さなカメラ・錐台・被写体。A: Zoom(10-300 mm、既定 50) 錐台の開き / B: Distance(Z) カメラと被写体の距離(Mother の「人と空間の距離」)/ C: Yaw(−180..180°) カメラの向き / D: Pitch/Roll。動くと: 錐台が keyframe の通りに動く。待機: カメラが微かに呼吸。読み出し: `50 mm`。輪郭: 「錐台」。pheno: Zoom / Yaw / Pitch / Roll(inspector_parts の項目)を supersede。stage_set.dart の「Camera frustum (top view)」を再利用する。

### 3.4 要求された残り(上でまとめたもの)
- **Repeater**(Amount / Spread / Rotation)= 9. Repeat を拡張: A: Amount(1-50、既定 5)小さな図形の数 / B: Spread(0-400 px、既定 80)広がり / C: Rotation(−180..180°、既定 15°) 1 個ごとの回り / D: Scale(0-200 %)。動くと: 図形が「分裂して」増える。待機: 全体がゆっくり回る。輪郭: 「増殖する小片」。
- **Anchor / Parent** — 世界: 子の点が親の点に糸で繋がれ、親が動くと子が遅れて付く。A: Anchor 位置(箱内)/ B: Lag(0-1 s)糸のたるみ / C: Follow(0-100 %)追従量 / D: Parent の入れ替え(離す)。pill や dropdown でなく「糸を引きちぎる」で入れ替え。動くと: 親の動きに子が付く。待機: 糸が揺れる。読み出し: `parent: Gem 04`。pheno: Follow / Attach を superset。
- **Path follow**(Along Path と別に、レイヤー 1 つが道をたどる)— 5 と統合。独立提案にするなら: 世界 = 道と 1 台の車。A: Progress / B: Auto-orient(on/off)/ C: Banking(0-100 %、既定 0) 曲がるときの傾き / D: Speed。
- **Text animator** — 世界: 6 文字の列が波として立ち上がる。A: Range(0-100 %) 効く文字範囲 / B: Offset(%)窓の滑り / C: Stagger(ms)文字ごとの時間差 / D: Amount(0-100 %)。動くと: 文字が順に跳ねる。待機: 窓がゆっくり往復。読み出し: `chars 1–6`。輪郭: 「文字の波」。Stagger(4)の兄弟で語彙を共通化。
- **Colour grade** — 世界: 3 つの玉(影・中間・光)が 1 本の傾いた軸に刺さる。A: Shadows(色相+量)/ B: Midtones / C: Highlights / D: Exposure(−3..3 EV)。動くと: 玉が軸上を動いて画面が変わる。待機: 玉が微動。読み出し: `+0.3 EV`。輪郭: 「三つ玉の串」。pheno: Fill / gradient 系と色が重なるが主題(階調の三分割)で区別。
- **Mix / Variation bank** — 世界: 4 つの小さな世界のスナップショットが 4 つの角に在り、中の点がその間を補間する(Macro Variations: ideas-phenomenon.md #20)。A: Bank 1 / B: Bank 2 / C: Bank 3 / D: Bank 4 の重み。点を動かすと 4 つの重みが 1 gesture で動く。待機: 点がゆっくり漂う。読み出し: `A .4 B .3 …`。輪郭: 「4 隅の羅針」(XY パッドに近いので、4 つの角を絵の「景色」にして点を置く)。**host が「重み」の正規化を決める(R10)。**
- **Render quality** — 世界: 1 枚の絵が粗い粒から細かい粒へ解像する。A: Samples(1-64、既定 8)粒の細かさ / B: Resolution(25-100 %、既定 100) 絵の大きさ / C: Motion blur(0-100 %) 尾 / D: Preview/Final。動くと: 値を変えると粒が揃っていく様子(本来の描画の縮図)。待機: 低 quality の時だけ粒が揺れる。読み出し: `8 spp · 100 %`。輪郭: 「ざらつき」。pheno: 該当なし。

(要求の 30 は 1-30 に、残り 6 は 3.4 に入れた。通し 34 項目 = 本来求めた全リストを網羅)

## 4. 未決・次の一手
1. **pheno_*.dart が出来たら、各提案の「pheno」欄を実ファイルと 1 対 1 で照合**(今は ideas-phenomenon.md 表からの推定)。
2. parameter の range / 既定は host の実 schema と合わせる(今は [inferred])。Glass / Repeater の parameter は実在するかも要確認。
3. **世界を 1 つだけ作って輪郭試験にかける**: 推奨は Glass(19)。難所の Transform(20)・Mix bank(§3.4)は最後。
4. 色スロット A-D を 1 つの token として足す案(`Slot.a..d` → Fam.stagger / along / g95 / follow)は新しい意味の導入に当たるので、利用者の裁定を取ってから。
