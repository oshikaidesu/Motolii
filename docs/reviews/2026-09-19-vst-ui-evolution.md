# ソフト音源・エフェクト(VST/AU)の UI 言語はどう育ったか — timeline + layers + shader 棚の道具へ何が写せるか

2026-09-19 / 調査のみ(コード変更なし)/ 一次資料 = 公式マニュアル・作者の発言・開発者インタビュー。引用は原文 25 語以内、URL 付き。原文が取れず要旨で書いた所は「(要旨)」と明記。

## 要約 5 行

1. プラグイン UI の最大の発明は 2007 年 NI Massive の「モジュレータをつまんでノブへ落とす → ノブの周りに深さの弧が出る」。以後 Vital・Serum・Pigments・Massive X・Phase Plant・Bitwig・Ableton Wavetable がほぼ全員採用し、事実上の共通語になった。
2. 第 2 の発明は「実信号の可視化が装飾でなく操作面そのもの」(FabFilter Pro-Q 2009: 曲線をクリックしてバンドを作る。Serum 2014: 3D wavetable。Vital 2020: 全ノブがアニメ)。見る物と触る物が同じ面に居る。
3. 第 3 は「プリセットが共有の単位」。.fxp/.vital/.vitalskin/Rack ファイルを Gumroad・フォーラム・User Library で回す。ソフトが持つのは道具、意味(音色)は作る人の物 — Motolii の芯と同じ。
4. 失敗は「写真的スキューモーフ(Arturia V Collection・初期 u-he)の可読性」「隠しページ(Massive X のルーティング)」「フィードバック無しの複雑さ」「小さすぎるノブ → 全員が可変ベクタ UI へ移行」。
5. Motolii へ写す 3 つ = ①ドラッグ = 関係の宣言(モジュレーション弧 → Effector の強さを値の上に描く)、②カードの正面 = 実信号のサムネ(WGSL の出力を小さく動かして見せる)、③init が既に動く + 棚 = プリセット(タグ・お気に入り・User バンク)。

## 事例の表

| 道具(年) | UI の発明 | 作者の言葉(≤25 語)+ URL | 失敗・限界 | 共同体の層 |
|---|---|---|---|---|
| NI Massive (2007) | ドラッグ&ドロップでモジュレータをスロットへ、ノブに色の弧で範囲と向き。「土星の輪」 | "A coloured ring is drawn around the knob, starting at the current position and showing the exact range and direction" — SOS 2007 https://www.soundonsound.com/reviews/native-instruments-massive | スロット式は「どこに何を挿したか」が散る | .nmsv プリセット、KVR・Splice で流通 |
| NI Massive X (2019) | 輪を継承、色分けの輪/線が「源の種類と量」を同時に示す。ルーティングは別ページ | "color-coded rings or lines next to controls" — NI マニュアル https://docs.native-instruments.com/ni-tech-manuals/massive-x-manual/en/modulation | "I can't work out why they made it so visually complicated without offering feedback" — Attack https://www.attackmagazine.com/reviews/gear-software/massive-x-the-tale-of-the-blockbuster/ 。発売時に可変 UI もブラウザも弱かった | NKS プリセット |
| NI Reaktor 6 / Blocks (2015, 6.3 で 2019) | 前面パネルでケーブル配線、色 8 種、Rack 形式で配線ごと保存 | 前面パネル配線・8 色のワイヤは Reaktor 6.3 リリース記事 https://synthanatomy.com/2019/04/native-instruments-reaktor-6-3.html | ケーブルが増えると読めない、User Library の Block は Rack 非対応(当時) | User Library(3000 超の無料 ensemble)https://userlibrary.native-instruments.com/reaktor |
| NI Kontakt | Instrument ごとに作者が UI を組む(スクリプト)。Kontakt は器で、見た目はライブラリ側 | — | 器と中身で操作語彙がばらばら | .nki ライブラリが商品 |
| FabFilter Pro-Q (2009 →4) | 表示そのものが操作面。曲線をクリックしてバンドを作る、hover でプレビュー、浮遊コントロール、リサイズ/全画面、A/B、undo | "When you hover anywhere in the display, a subtle curve preview will appear" https://www.fabfilter.com/help/pro-q/using/eqdisplay ; "they expose the features you need, when you need them" https://www.fabfilter.com/help/pro-q/support/about ; "we've never been afraid of reinventing the wheel"(同) | sketch モードは「既存の作り方より速いと感じることは稀」— SOS https://www.soundonsound.com/reviews/fabfilter-pro-q-4 | プリセット主体でない(道具型) |
| Xfer Serum (2014) / Serum 2 (2025) | 2D/3D wavetable 表示が正面、LFO 形をノブへ drag、可視の即時応答。Serum 2 でタブ化・試聴付きブラウザ・タグ・評価・お気に入り・UI 拡縮 | Serum 2 の browser: フィルタは tag/type/author/modulation type、フォルダとラベル https://unison.audio/xfer-serum-2/ (Duda の言葉は Mr. Bill Podcast #167 動画内 https://sonicstate.com/news/2025/03/25/steve-duda-on-serum-2-/ ) | 1 の UI は固定解像度で小さかった → 2 で拡縮。2 はページが増えた(「wider and tabbed」) | .fxp と wavetable の巨大市場(Splice 等)。GUI は Lance Thackerey |
| Vital (2020, Matt Tytel) | drag-drop 変調 + 落とす前の「プレビュー」、全ノブがアニメ、oscilloscope/spectrogram、全ベクタ、skin ファイル、無料枠、GPL 公開 | "Vital gives you a preview of the modulation before committing" https://vital.audio/ ; "You want really fast graphics and really tight responsive audio to visual" https://juce.com/made-with-juce/matt-tytel-from-vital-audio/ ; "It's all shader based, so it uses signed distance fields"(同) | OpenGL 必須で GUI が真っ白になる環境 https://forum.vital.audio/t/gui-for-vital-opens-but-is-blank/1940 ; 描画コスト → 自作 Visage で "not redrawing all the time" | .vital(1 ファイル)、.vitalskin(1 ファイル)、フォーラムの skin スレ https://www.vitalsynth.com/profiles/skinsharingthread/ |
| Arturia Pigments (2018) | 色分けの変調源、ノブの周りに色の輪、変調源の動きをスコープで生放送、単一画面、4 Macro | "a coloured ring will appear around that control to show the amount of modulation" — SOS 2019 https://www.soundonsound.com/reviews/arturia-pigments ; "color-coded modulation sources make it easy to see where you are and what's connected" https://www.arturia.com/products/software-instruments/pigments/overview | — | 1700 超の工場プリセット、Sound Store |
| Arturia V Collection (2004→) | 写真的スキューモーフ → 世代ごとに平面化(Analog Lab V、Augmented 2.0) | 批評:「限られた領域を装飾に食われ、可読性と機能を犠牲」(要旨)https://gearspace.com/board/reviews/1338753-arturia-v-collection-5-a.html | 楽器ごとに設計者が違い、語彙がばらばら | — |
| u-he Diva (2011) / Zebra (2004) / Hive | パネルを混ぜて組む(Diva)、1 つのモジュレータを複数の役で使う(Zebra)、70〜200% 可変 UI、skin | "Mix-and-match panels for custom layouts" / "Resizable UI from 70% to 200%" https://u-he.com/products/diva/ ; "Everything is set up to avoid redundancy. You can use the same modulator in different roles for different targets" — Urs https://www.attackmagazine.com/features/interview/we-do-it-because-we-want-to-not-because-we-see-commercial-opportunities-urs-heckmann/ | Diva の Modifier 段が「混乱」との声 https://gearspace.com/board/electronic-music-instruments-and-electronic-music-production/667617-u-he-diva-28.html ; 高解像度で小さい https://www.kvraudio.com/forum/viewtopic.php?t=543167 | 第三者 skin(Plugmon MONA・Volta)https://plugmon.jp/product/mona/ 。skin は 10px 格子で作る |
| Kilohearts Phase Plant (2019) | snapin(小さな効果)を並べる 3 レーン、変調は「小さな輪を引いて深さ」、被変調ノブは橙に、モジュレータに常時の小さな波形 | "Click and drag on the modulation knob to connect the modulation source to the target parameter and set the modulation level" https://kilohearts.com/docs/modulation ; "All modulator modules have a small animated display… that shows the current value"(同) | "complex patches can feel crowded, particularly in the modulation lane" — MusicRadar https://www.musicradar.com/reviews/kilohearts-phase-plant | snapin が別売りの単位(効果 = 商品) |
| Bitwig (2014→, Grid 2019) | モジュレータが装置の中の「スロット」で第一級、VST にも掛かる、Grid で内部を開く | "each device was given slots that can house true modulators" https://www.bitwig.com/stories/behind-the-scenes-modularity-in-bitwig-studio-21/ ; "you can open it up in The Grid, see its internals and how it was built" — Claes https://www.musicradar.com/news/10-years-bitwig | Grid はケーブルの絵が増えると読めない | プリセット + Grid パッチの共有 |
| Ableton Wavetable (2018) | 「宛先から」の変調(このパラメータを何で動かすか)、行列を見やすくする、全画面の展開ビュー | "I want this parameter to be modulated by this, rather than I have this modulator" — Matt Jackson https://www.ableton.com/en/blog/new-wave-depth-look-wavetable/ ; "Displaying the modulation information on the parameters themselves didn't seem satisfying" — Ian Hobson(同) | 行列は「最良で最難」。可視化は「音作りには役に立たない」https://sebas.design/wt.html | Live の .adv、Pack |
| Ableton Drift (2023) | 少ない操作、init で既に性格がある音、ほぼ全部が前面 | "Devices with a relatively small amount of controls, but that have a personality built in" — Marc Resibois https://cdm.link/inside-ableton-drift/ ; "Nearly all controls are directly available in the front panel"(同) | — | — |
| Ableton Operator (2004) | 折り畳み式の 1 画面 FM、値の直接入力 | — | 小さい行、Live の枠に合わせた密度 | — |
| Soundtoys (2005→) | ハードの見た目そのまま(ノブ・スイッチ)、Little/大版の 2 段構え | インタビューは音の話が中心で UI の言葉は取れず https://tapeop.com/interviews/62/ken-bogdanowicz | 固定サイズ、隠し「tweak」段 | プリセット少なめ |
| Valhalla (2011→) | 平面・少数のノブ・1 画面。道具は手に取ってすぐ使える | 「プラグインは仕事に要る物だけを持つべき、余計は事を複雑にする」(要旨、原文は 403)https://valhalladsp.com/2017/05/25/minimalism-in-plugin-guis/ ; 兄弟記事 https://valhalladsp.com/2017/06/05/modularity-plugin-design/ | 可視化が無い(意図的) | プリセット共有は小さい |
| Spectrasonics Omnisphere 2 (2015) | 常駐 Mini Browser、Sound Match(似た音を探す)、ハード連携 | "You've got the Mini Browser that's always right there, and you can see the interface at the same time" — Persing https://www.musicradar.com/news/tech/spectrasonics-eric-persing-talks-omnisphere-2-sound-design-and-more-616236 ; "as we started adding more sounds… the experience of using the instrument was getting worse"(同) | 音が増えるほど探せなくなる → ブラウザ再設計 | 巨大な工場ライブラリが主役 |
| Output Arcade/Portal | ブラウザ型の入口、4 Macro が主操作 | — | 深い編集は隠れる | 定期配信のキット |
| Cableguys ShaperBox (2017→3) | 曲線エディタが正面、ペンで描く、磁石、ランダム化、1 窓、2.0.1 で可変 UI | "Advanced features… are there when you need them – but you don't have to learn them" https://www.musicradar.com/music-tech/fx/dg-wip-our-aim-with-shaperbox-is-to-make-it-easy-for-any-producer-to-get-complex-sounding-results-all-in-one-plugin-cableguys-talk-modulation-mad-multi-effects-plugins-and-more ; 可変 UI は 2.0.1 https://www.kvraudio.com/news/cableguys-update-shaperbox-to-v2-0-1-with-resizable-ui-and-new-features-46886 | — | 曲線プリセット |
| VCV Rack (2017) | 見た目までモジュラー、ケーブル、出力に複数ケーブル、右クリックで Module Browser | "I wanted people to experience what it looked like, which I think is just as important as what it feels like" — Belt https://www.synthtopia.com/content/2018/01/22/open-source-synthesis-behind-the-scenes-with-vcv-rack-creator-andrew-belt/ ; "Stack multiple cables on outputs by holding Ctrl" https://vcvrack.com/manual/GettingStarted | ケーブルの絵が増えると読めない、パネルの小ささ | 第三者モジュール(無料・有料)、パッチ共有。"easy enough for someone with only DSP knowledge to get in and make graphics" |
| Max (1988→) / Reaktor | 極点:全部ケーブル。速いが学習の壁 | "You can make changes in a second that would take half an hour in a text-based language" — Zicarelli https://www.kvraudio.com/interviews/max-innovation-an-interview-with-david-zicarelli-48978 | "requires some dedication to learning it" | パッチ共有(Max for Live) |
| JUCE (2004→, Jules Storer) | 共通基盤:同一ソースで Win/mac/Linux、VST/AU/AAX を 1 コードから、Component/Slider/LookAndFeel が「ノブ・スライダ・skin」の下地 | 業界の事実上の標準 https://en.wikipedia.org/wiki/JUCE ; Tytel: "If they had that back when I started, I might just have stuck with JUCE" https://juce.com/made-with-juce/matt-tytel-from-vital-audio/ | CPU 描画は重い → Vital は OpenGL/SDF、後に Visage へ | JUCE 自体が「作者の共同体」の共通語 |

## 発明の一覧(採用数順)

| # | 発明 | 採用(上表から) | 数 |
|---|---|---|---|
| 1 | プリセットブラウザ(タグ・お気に入い・User バンク・試聴) | Massive/X, Serum 2, Vital, Pigments, Diva, Phase Plant, Omnisphere, Output, Bitwig, Ableton | 10+ |
| 2 | 可変ベクタ UI(拡縮・全画面) | FabFilter, Vital, Serum 2, Diva/u-he, Pigments, ShaperBox 2.0.1, Phase Plant, Bitwig | 8 |
| 3 | モジュレータをノブへ drag → 弧/輪で深さ | Massive, Massive X, Vital, Serum, Pigments, Phase Plant, Bitwig | 7 |
| 4 | 実信号の可視化が操作面(3D wavetable、scope、曲線を直接掴む) | FabFilter, Serum, Vital, Pigments, ShaperBox, Phase Plant(小窓), Wavetable | 7 |
| 5 | 1 画面「全部見える」(ページ無し) | Phase Plant, Pigments, ShaperBox, Valhalla, Drift, Operator, Diva | 7 |
| 6 | Macro(数個のつまみに多数を束ねる) | Pigments, Serum, Massive X, Output, Bitwig, Omnisphere, Vital | 7 |
| 7 | 右クリック割当・MIDI learn | ほぼ全員(JUCE の慣習) | 7+ |
| 8 | A/B・undo | FabFilter, Serum, Pigments, Phase Plant, Omnisphere | 5 |
| 9 | skin/テーマ(ファイル 1 個) | Vital, u-he(第三者), Serum 2, Bitwig | 4 |
| 10 | ランダム化 | ShaperBox, Vital(mod), Pigments, Serum 2 | 4 |
| 11 | ケーブル(関係を線で描く) | VCV, Max, Reaktor Blocks, Bitwig Grid | 4 |
| 12 | 落とす前のプレビュー(drag 中に結果を聴く/見る) | Vital, FabFilter(hover の曲線) | 2 |
| 13 | サンプル/wavetable をパネルへ drop | Serum, Vital, Phase Plant | 3 |
| 14 | init が既に音を出す/性格がある | Drift, Diva, Serum(init saw) | 3 |
| 15 | 「宛先から」の変調(パラメータ側に源を付ける) | Wavetable, Phase Plant, Vital(両方) | 3 |

## 失敗

- **スキューモーフ疲れ**:V Collection の写真パネルは「領域を装飾に食われ可読性を落とす」。Diva の Modifier 段も「混乱」。Valhalla・FabFilter は最初から平面で、hover と表示に情報を置いた。教訓 = 見た目の忠実さは操作の忠実さではない(Belt だけは「見た目が体験」と意図して選び、それが VCV の人気の芯)。
- **小さいノブ・固定解像度**:Serum 1・Soundtoys・u-he 旧版・Massive X 発売時。全員が可変ベクタ UI へ移った(u-he 70〜200%、ShaperBox 2.0.1、Serum 2)。
- **隠しページ**:Massive X のルーティング別ページ、Serum 2 のタブ化、Output の tweak 段。1 画面派(Phase Plant・Pigments)は「crowded」で払う。どちらも払う物がある。
- **フィードバック無しの複雑さ**:Massive X の envelope(Attack 評)、Wavetable の行列(「最良で最難」)。Hobson の言葉が核心 — 値の上に情報を置くだけでは「満足に足りない」、招く形を探した。
- **CPU を食う可視化**:Vital は OpenGL 必須で GUI が真っ白になる環境が出た。Tytel は「常時再描画しない」に向かった。可視化は正面だが、止まっている時は描かない。
- **ケーブルの読めなさ**:VCV・Grid・Reaktor Blocks は数十本で線が読めなくなる。Bitwig は装置の中の「スロット」で線を隠し、Zebra は「同じモジュレータを複数の役で」重複を消した。

## Motolii への写し

Motolii = timeline + layers + shader 棚。効果の積みはプラグインチェーン、棚の札(WGSL + パラメータ)は装置、関係の宣言は「これをあれに落とす → 線が出る」。

| プラグインの発明 | Motolii の写し | 具体 |
|---|---|---|
| モジュレータを drag → 弧 | **anchor/関係の手つき** | 札(Effector・場・留め具)を層/値の上へ drag。Vital 式に drag 中は仮の線 + 仮の値で舞台がプレビュー、落として確定。Wavetable 式に「宛先側から」も開く(値の右クリック → 源を選ぶ)。線は Bitwig 式に既定で隠し、選択中の 1 つだけ描く(VCV の読めなさを避ける) |
| 変調の輪(深さ・向き) | **Effector Strength を値の上に描く** | 値の欄の周りに弧:現在値から効き幅を色で。色 = 源の種類(Pigments)。被変調の欄は色を変える(Phase Plant の橙)。数値は hover で(FabFilter) |
| 実信号の可視化が正面 | **札の正面 = 舞台の小窓** | 棚の札は名前でなく、その WGSL を小さく動かした絵。Phase Plant の「モジュレータの小さな波形」= Effector 札に現在の出力の小さな曲線。止まっている時は描かない(Tytel) |
| プリセットブラウザ | **棚** | タグ・お気に入り・User バンク・作者・「関係の種類」でフィルタ(Serum 2 の modulation type)。hover で舞台に仮適用(試聴)。ファイル 1 個 = 札 1 枚(.vital の型) |
| init が音を出す | **既定の文書が既に動く** | Drift 式:少ない欄で「性格がある」既定。空の舞台でなく、箱 1 つが物理で落ちている状態から始める |
| Macro | **欄 → 欄** | 上位の欄 1 つに複数の欄を束ね、束ねた関係も弧で見せる。「1 つが全体に干渉する」の UI 側 |
| A/B | **法の設定 2 つを比べる** | 物理・並べ・座標の法の設定を A/B で切り替え、差分をハイライト。FabFilter の Instance List 式に同種の札を横に並べる |
| 可変ベクタ UI | **Flutter** | 既に vector。70〜200% の拡縮と全画面は理論上無料 — 舞台の小窓を同じ描画路で出す(fork の re_renderer)なら Vital と同じ「触った物と見える物が同じ面」 |
| 落とす前のプレビュー | **drag 中の舞台** | 札を層へ運ぶ間、舞台が仮の結果を動かす。「happy accident」の入口 |
| 右クリック割当・undo | 既存の慣習 | 値を右クリック → 源を選ぶ / 関係を切る。undo は関係の付け外しも 1 手 |
| skin | 後回し | Vital の .vitalskin 型(1 ファイル)は写せるが完成の定義ではない |

**写せない物(音固有)**:MIDI learn(外部つまみの学習 — 対応物は無い、必要なら OSC/MIDI は後)、audio-rate 変調(FM)、試聴 = 音(Motolii は絵の試聴で置換)、wavetable 3D(時間 × 波形の積層 — 代わりに「時刻 × 値」の小さな曲線)、ポリフォニック per-note 効果、レイテンシ・CPU メータ、ホスト DAW のオートメーションレーン(Motolii は自分がタイムライン)。

## Sources

- SOS: NI Massive review (2007) https://www.soundonsound.com/reviews/native-instruments-massive
- NI Massive X manual — Modulation https://docs.native-instruments.com/ni-tech-manuals/massive-x-manual/en/modulation
- Attack: Massive X review https://www.attackmagazine.com/reviews/gear-software/massive-x-the-tale-of-the-blockbuster/
- Reaktor 6.3 front panel patching https://synthanatomy.com/2019/04/native-instruments-reaktor-6-3.html ; User Library https://userlibrary.native-instruments.com/reaktor
- FabFilter Pro-Q help: display https://www.fabfilter.com/help/pro-q/using/eqdisplay ; about https://www.fabfilter.com/help/pro-q/support/about ; SOS Pro-Q 4 https://www.soundonsound.com/reviews/fabfilter-pro-q-4
- Serum 2 features https://unison.audio/xfer-serum-2/ ; Duda on Serum 2 (video) https://sonicstate.com/news/2025/03/25/steve-duda-on-serum-2-/ ; Duda interview https://soundand.design/interview-with-steve-duda-of-xfer-records-d1b2799535f3
- Vital https://vital.audio/ ; Tytel × JUCE https://juce.com/made-with-juce/matt-tytel-from-vital-audio/ ; skin thread https://www.vitalsynth.com/profiles/skinsharingthread/ ; blank GUI https://forum.vital.audio/t/gui-for-vital-opens-but-is-blank/1940
- Arturia Pigments https://www.arturia.com/products/software-instruments/pigments/overview ; SOS Pigments (2019) https://www.soundonsound.com/reviews/arturia-pigments ; V Collection 5 review https://gearspace.com/board/reviews/1338753-arturia-v-collection-5-a.html
- u-he Diva https://u-he.com/products/diva/ ; Urs Heckmann (Attack) https://www.attackmagazine.com/features/interview/we-do-it-because-we-want-to-not-because-we-see-commercial-opportunities-urs-heckmann/ ; Plugmon MONA https://plugmon.jp/product/mona/ ; GUI size thread https://www.kvraudio.com/forum/viewtopic.php?t=543167
- Kilohearts docs: Modulation https://kilohearts.com/docs/modulation ; MusicRadar Phase Plant https://www.musicradar.com/reviews/kilohearts-phase-plant ; Kilohearts interview (403、未確認) https://www.admiralbumblebee.com/music/2019/07/13/Interview-with-Kilohearts.html
- Bitwig: Modularity https://www.bitwig.com/stories/behind-the-scenes-modularity-in-bitwig-studio-21/ ; 10 years https://www.musicradar.com/news/10-years-bitwig
- Ableton Wavetable https://www.ableton.com/en/blog/new-wave-depth-look-wavetable/ ; critique https://sebas.design/wt.html ; Drift (CDM) https://cdm.link/inside-ableton-drift/
- Soundtoys / Bogdanowicz (Tape Op) https://tapeop.com/interviews/62/ken-bogdanowicz
- Valhalla: Minimalism in plugin GUIs (2017, 403 で要旨) https://valhalladsp.com/2017/05/25/minimalism-in-plugin-guis/ ; Modularity https://valhalladsp.com/2017/06/05/modularity-plugin-design/
- Spectrasonics / Persing https://www.musicradar.com/news/tech/spectrasonics-eric-persing-talks-omnisphere-2-sound-design-and-more-616236
- Cableguys https://www.musicradar.com/music-tech/fx/dg-wip-our-aim-with-shaperbox-is-to-make-it-easy-for-any-producer-to-get-complex-sounding-results-all-in-one-plugin-cableguys-talk-modulation-mad-multi-effects-plugins-and-more ; 2.0.1 resizable https://www.kvraudio.com/news/cableguys-update-shaperbox-to-v2-0-1-with-resizable-ui-and-new-features-46886
- VCV Rack: Belt interview https://www.synthtopia.com/content/2018/01/22/open-source-synthesis-behind-the-scenes-with-vcv-rack-creator-andrew-belt/ ; manual https://vcvrack.com/manual/GettingStarted
- Max / Zicarelli https://www.kvraudio.com/interviews/max-innovation-an-interview-with-david-zicarelli-48978
- JUCE https://en.wikipedia.org/wiki/JUCE ; Storer podcast https://www.theaudioprogrammer.com/content/podcast/creating-the-juce-framework-w-jules-storer-juce-ep-5
