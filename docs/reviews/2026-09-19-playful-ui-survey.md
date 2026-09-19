# 遊びの道具の UI 調査 — 「映像制作は楽しくあるべき」の出典集(2026-09-19)

問い(利用者): 遊びの創作道具は UI の何で「作ることを楽しく」しているか。その原理のうち、タイムライン + 層 + シェーダー棚の道具(Motolii)に写せる物はどれか。AE / Cavalry の顔にならないために。

範囲: web 調査のみ(repo のコードは触っていない)。引用は 1 主張 25 語以内、出典 URL は末尾 Sources。一次資料(公式マニュアル・開発者インタビュー・社長が訊く・公式製品頁)を優先し、二次(Wikipedia・ブログ)は「二次」と明記。取れなかった一次(証明書エラー等)は「未到達」と書いた。

## 要約 5 行

1. 出典が共通して名指す芯は 4 つ — **即応(触った瞬間に音か絵が返る)・直接操作(数値でなく物を掴む)・制約が遊び(狭い箱を「楽しいように選ぶ」)・道具がキャラクター(Undo 犬・Oh no 男・imp・カエル)**。
2. ガジェット系(Ableton Session/Push/Note・volca・TR-8S・OP-1)の芯は「**演奏する感じ**」= 格子を叩く・拍に量子化されて必ず気持ちよく鳴る・手元の光と音で画面を見ない。Takahashi「Toyish… is quite a compliment」。
3. 「失敗が楽しい」は Kid Pix の設計原則に明文(消すことは作ることと同じくらい楽しく)。Undo をキャラにするのは Mario Paint / Kid Pix の 2 系統で独立に到達。
4. 写せる物: 棚(シェーダー)を「格子のパッド」にして押した瞬間に鳴らす、Undo・消去を 1 体のキャラに、数値欄の代わりに掴んで擦る、拍で量子化された配置、1 画面(層・時刻・棚が同時に見える)、作った物をその場で「演奏」する Session 型の出口。
5. 写せない物: 幼児向けの色と音(Kid Pix の全ツール効果音は「大人が疲れる」線に近い)、保存を捨てる(Electroplankton)、キャラが仕事を遮る演出。境界は「**手応えは大人向け(TE/Ableton)、意味は使う人の物**」— 音・キャラは OFF にできる小さな層として置き、時刻の正確さ(タイムラインの本分)を絶対に犠牲にしない。

## 道具ごとの表

| 道具 | 年 | 何が遊びか | UI の仕掛け 3 つ | 出典 |
|---|---|---|---|---|
| Mario Paint(SNES) | 1992 | 描く・作曲・アニメ・ハエ叩きが 1 本、マウス同梱 | ① Undo がキャラ(Undodog、Art と Music 両モードの道具箱) ② 楽器がアイコン(マリオ=ピアノ、キノコ=バスドラ、犬・猫・豚の声) ③ タイトル画面の文字を押すと全部が反応(O が爆発、A が落ちてマリオを弾く、T が虹クレヨン) | Wikipedia [S1]、Super Mario Wiki [S2](二次。一次インタビューは未到達) |
| Mario Artist(64DD) | 1999–2000 | 描く→顔を貼る→3D→踊らせる、の連結 | ① 自分の顔を取り込み(Capture Cassette)キャラに貼る ② Paint Studio に Pokémon Snap 風のミニゲーム ③ Polygon Studio の「Sound Bomber」が後の WarioWare の種 | Wikipedia [S3](二次) |
| うごくメモ帳 / Flipnote Studio(DSi) | 2008 | 紙のメモの延長でパラパラ漫画、投稿して拍手 | ① 色は黒白+赤青の 2 色に絞る ② 音は 2 秒×3 本を本体マイクで ③ 共有先(Hatena)を最初から本体に | Wikipedia [S4](二次)、社長が訊く 要旨 [S5][S6]: "a place where people share their creations… and offer each other applause" |
| WarioWare D.I.Y.(DS) | 2009 | 5 秒のゲームを毎日 1 本、作るのが「仕事でも楽しかった」 | ① 制約が箱(マイクロゲーム 5 秒「so it's all very practicable」) ② 絵が苦手でもスタンプ ③ 同梱ゲームは全部中身が見える("everything about how that microgame was made is available… as reference") | 社長が訊く [S7][S8](一次) |
| Rhythm 天国(GBA) | 2006 | 見ないで叩く、ドラムを自由に叩く Studio | ① 視覚でなく音で合図(つんく♂「視覚に頼らないリズムゲーム」) ② ボタン=ドラムの各パーツを自由に叩く技術デモが原型 ③ 譜面なし | Wikipedia [S9](二次) |
| Otocky(FC ディスク) | 1987 | 撃つと拍に合わせて音階が鳴る | ① 方向=音程、発射=発音 ② 発音が拍に量子化 ③ 進むと作曲モードが解錠 | Wikipedia [S10](二次) |
| Electroplankton(DS) | 2005 | 目的なし、触ると光と音 | ① 10 種の「生き物」= 10 の触り方 ② 保存なし(「道具」になってメニューが要るのを嫌った) ③ 作者名がパッケージに | Wikipedia [S11](二次)、Iwai インタビュー [S12] "creative software played on the game platforms other than game software" |
| Tenori-on(Yamaha) | 2007 | 16×16 の光る格子が楽器 | ① 音=光(押した所が光り、再生が走査する) ② 形の美しさを楽器の条件に("an electronic instrument of beauty") ③ モード切替で同じ格子が別の楽器 | Wikipedia [S13](二次、Iwai 発言を引用) |
| Kid Pix(Mac) | 1989 | 描く過程が絵より大事、消すのも楽しい | ① Undo が男(「Oh no!」と声) ② 消しゴムがダイナマイト(画面が同心円で爆発) ③ 変な筆(垂れる絵具・葉のない木) | Hickman 原則の転記 [S14]、Wikipedia [S15](二次。Hickman 本人の essay は未到達 [S16]) |
| Dreams(PS4, Media Molecule) | 2020 | 彫る・動かす・論理が 1 空間、imp がカーソル兼分身 | ① imp が物を「憑依」して動かす ② メニューとスライダーを避けジェスチャ(両手を離すとズーム) ③ Stealth Create = 小さな課題で自信を積む | Game Developer(GDC 講演要約) [S17] "The things that make traditional tools intimidating are endless menus, and endless sliders."、Wikipedia [S18] |
| LittleBigPlanet(PS3) | 2008 | 遊ぶ世界の中で作る(Play, Create, Share) | ① Popit(ポップアップの道具箱を遊びの画面の上に) ② ステッカー・表情=自己表現がそのまま道具 ③ 作品が即遊ばれ評価される | Wikipedia [S19](二次) |
| Teenage Engineering OP-1 | 2011 | 制約が最大の機能、絵が効果を説明 | ① 効果の画面が図解でなく絵(Punch=ボクサー) ② 4 色のエンコーダ=画面の色に対応(製品ガイド) ③ 4 トラックの「テープ」の見立て | Wikipedia [S20] "limitations are OP-1's biggest feature"、SFMOMA インタビュー [S21] "A product can be entertaining and be a tool"、公式ガイド [S22] |
| KORG volca / Electribe | 2013– | ソファで落書き、スピーカー内蔵 | ① 16 ステップのボタン列 ② Motion Sequence(つまみの動きを記録) ③ 電池・スピーカーで「場所の形式」を外す | KORG 公式 [S23]、Takahashi(WBGO) [S24] "Toyish, I actually find, is quite a compliment" |
| Roland TR-8S | 2018 | 16 ボタンと色のフェーダーで演奏する | ① TR-REC の 16 ボタン ② 楽器ごとの RGB フェーダー ③ "immediate and effortless" を製品の言葉に | Roland 公式 [S25] |
| Ableton Live Session View | 2001– | 順番を決めずに鳴らす、格子を演奏する | ① クリップの格子(列=トラック、行=シーン) ② 発火が拍に量子化 ③ Arrangement(時間軸)と Session(格子)を同居 | 公式マニュアル [S26] "played at any time and in any order"、Hein [S27] "like inventing a new musical instrument, every time"(二次) |
| Ableton Push | 2013– | 画面を見ずに 64 パッドで叩く | ① Lego に釘打ちの試作から ② パッドの色が音階と形の地図 ③ 「マウスとノートPC では演奏にならない」 | 公式ブログ [S28] "Using a mouse and looking at a laptop didn't cut it"、公式 [S29] |
| Ableton Note(iOS) | 2022 | 演奏で発想、1 画面 | ① 25 パッド+16 パッド ② Capture MIDI(弾いた後に「録っておいた」) ③ "A playable iOS app for forming musical ideas" | 公式 [S30] |
| Scratch | 2007 | 積み木を嵌める、低い床・広い壁・高い天井 | ① ブロックは形が合う所にしか嵌まらない ② 「wide walls」= 多様な作品への道 ③ tinkerability(少しずつ試す) | Resnick [S31](本人 Medium 転載、原典未到達) |
| PICO-8 | 2015 | 128×128・16 色・カート、制約が「楽しいように選ばれた」 | ① コード・スプライト・マップ・SFX・音楽の編集器が 1 本の中 ② カートが PNG で中身も見える ③ "harsh limitations… carefully chosen to be fun to work with" | 公式 [S32] |
| Bitsy | 2017 | 小さな世界の小さな編集器 | ① 部屋ごと 3 色 ② 8×8 タイル ③ 「歩く・話す・そこに居る」だけ | itch.io [S33] "a little engine for little games, worlds, and stories" |
| Figma / FigJam | 2016– | 他人のカーソルが見える、/ で叫ぶ | ① Cursor chat(/ キー、5 秒で消える、52 字) ② 他人が打っている途中が見える ③ 期間限定の懐古カーソル(April Fun Day) | 公式ヘルプ [S34]、公式ブログ [S35] "cursors… also chat, emote, and high five" |
| Rive | 2020– | 状態機械を絵で繋ぐ | ① アニメ=状態、遷移を線で ② 実行結果がその場で動く ③ (編集器自体の遊び要素は出典なし) | 公式 [S36](二次的、UI の遊びは未確認) |

## 共通する原理(出典が言っている物)

- **音・光の即応** — Otocky は発射=発音、Tenori-on は押した所が光る、Rhythm 天国は「視覚に頼らない」[S9][S10][S13]。Push は "so sensitive they can actually sense your finger before you hit the pad" [S28]。Bret Victor: "Creators need an immediate connection to what they're creating." [S37]。
- **直接操作(数値でなく物)** — Dreams は "endless menus, and endless sliders" を怖さの正体と名指し、ジェスチャに寄せた [S17]。Victor は値を掴んで擦る・両手で演じてアニメを作る(キーフレームでなく人形劇)[S37][S38]。
- **1 画面** — PICO-8 は編集器 5 つを 1 本の中に [S32]、Note は 1 画面で発想 [S30]、Session View は格子と時間軸の同居 [S26]。うごメモは色 2 色・音 3 本と道具そのものを少なく [S4]。
- **道具がキャラクター** — Undodog(Mario Paint)[S1]、Undo Guy「Oh no!」と Dynamite(Kid Pix)[S15]、imp(Dreams)[S18]、Popit(LBP)[S19]。共通点: 破壊系(undo / clear)にこそキャラを置く。
- **失敗が楽しい** — Kid Pix 原則「消しゴムは気まぐれに。破壊は創造と同じくらい楽しく」[S14]。Session View は拍に量子化されるので押し間違えても崩れない [S26][S27]。
- **制約が遊び** — PICO-8 "carefully chosen to be fun to work with" [S32]、OP-1 "limitations are OP-1's biggest feature" [S20]、WarioWare 5 秒 "so it's all very practicable" [S7]、Kouthoofd "I stick to rules… then I can become free again" [S21]。
- **すぐ結果・すぐ見せる** — WarioWare は同梱作品の中身が全部見える [S8]、Flipnote は投稿と拍手が本体に [S5]、PICO-8 のカートは PNG で中身つき [S32]、Dreams の Stealth Create(小さな成功を先に)[S17]。
- **手描き感・触感** — Push は Lego 試作 [S28]、OP-1 は効果を絵で(ボクサー)[S20]、Figma の懐古カーソルは "It feels so tactile" [S35]。
- **ガジェットのケレン味(利用者の追加問)** — 出典が言う中身は 4 つ。(a) **物理の見立て**: OP-1 の「テープ」と絵の効果 [S20][S22]、TR-8S の楽器ごとの色フェーダー [S25]。(b) **光と音の即応**: Push のパッド色が地図 [S29]、Tenori-on [S13]。(c) **格子**: Session の格子、volca/TR の 16 ステップ、Push 64 パッド [S23][S25][S26]。(d) **演奏する感じ**: Behles「マウスとノートPC では演奏にならない」[S28]、Takahashi "You've got to have fun to connect with an instrument" [S24]。「Toyish は褒め言葉」は Takahashi が明言 [S24]、TE も同旨(二次 [S20])。
- **学術**: Gaver "Designing for Homo Ludens"(2002, I3 Magazine)= 課題でなく好奇心・探索・曖昧さで動く ludic 設計 [S39]。Resnick の低い床・広い壁・高い天井・tinkerability [S31]。Nicky Case の型: "Start Small, Build Big"、"Author-guided & Player-driven" [S40]。

## タイムライン+層+棚の道具に写せる物(具体案)

1. **棚 = パッド格子(Session 型)** — シェーダー棚のカードを 4×4 か 8×8 の格子で並べ、押した瞬間にプレビューが**その拍から**鳴る(Session の launch quantization)[S26]。マウスのホバーで音無しの光、クリックで音。ドラッグで層に落とす。棚は「選ぶ場所」でなく「叩く場所」。
2. **Undo と消去を 1 体のキャラに** — 層パネルの左下に犬 1 匹(Undodog/Undo Guy の系統 [S1][S15])。Undo で吠える、層の削除で犬が咥えて持っていく。声・動きは設定で OFF、既定は「小さく、1 回」。破壊系だけに置く(作る側にキャラは置かない = Kid Pix の分業)。
3. **数値欄を「掴んで擦る」に** — インスペクタの数値を Victor 式にドラッグで擦る [S37]、擦っている間はビューアが動く。キーフレームの打鍵は「時刻を掴んで置く」1 動作に。
4. **タイムラインの拍量子化** — 層の開始点・キーフレームのスナップを「拍」に(BPM 指定時)。Otocky/Session の「間違えても拍に乗る」[S10][S26]。ずらすときは Shift で自由に。
5. **1 画面** — 層・時間・棚・ビューアを同時に見せ、モーダルを持たない(PICO-8 [S32]、Note [S30])。棚は右のドロワーではなく、タイムラインの下に常駐する格子。
6. **物理の見立て(ガジェット)** — 層のミュート/ソロを TR-8S 型の**色つきフェーダー**に、時間軸の再生ヘッドを「テープ」の見立てで走らせる(OP-1 の tape [S22])。動きに音(ヘッドの走り出し、ループの折り返し)。
7. **Capture(後から録っておいた)** — ビューアで手で動かした軌跡を、後から「今のを取っておく」で層に落とす(Note の Capture MIDI [S30]、Victor の両手演奏 [S38])。キーフレームを打つ前に演じる。
8. **中身が見える例** — 同梱作品は全部開ける・全部触れる(WarioWare [S8]、PICO-8 のカート [S32])。棚の 1 枚をダブルクリックで、その式が読める。
9. **Stealth Create** — 起動直後の空舞台に「1 つ触ると鳴る」物を置いておく(Dreams [S17])。空の AE を見せない。
10. **起動画面の小さな反応** — タイトルの文字を押すと 1 つずつ違う反応(Mario Paint [S2])。効果は 1 秒以内、仕事を遮らない。

## 写せない・危険な物(子供っぽさと軽薄さの境)

- **全ツールに効果音** — Kid Pix は子供向けの前提。大人の道具では「音は破壊系と拍だけ」に絞る。既定 ON でも 1 クリックで全 OFF。
- **保存を捨てる** — Electroplankton の「道具になるのを嫌う」[S11]は玩具だから成立。Motolii は道具。写すなら「保存はあるが、保存を意識させない自動保存」。
- **キャラが仕事を遮る** — Clippy 型。キャラは反応するだけで、話しかけない・提案しない。
- **格子で時間軸を置き換える** — Session は Arrangement を消さなかった [S26]。タイムラインの正確な時刻(フレーム・秒)は AE 利用者の生命線。格子は棚と発想の側に留める。
- **幼児の色** — Kid Pix/Flipnote の原色を写すと軽薄に見える。参照は TE/Ableton/TR-8S の「黒地に色つきの光」[S25][S29]。
- **制約の押し付け** — PICO-8 の制約は「選んで入る箱」。Motolii の制約は棚(1 枚のカード)側に置き、コンポジット全体に天井を作らない。
- **自動で意味を変える** — 触った量で 2D/3D を推定する類(memory: no-auto-projection)。遊びの道具は皆「押した物が押した通りに」動く。

## Sources

- [S1] Mario Paint — Wikipedia: https://en.wikipedia.org/wiki/Mario_Paint (二次)
- [S2] Mario Paint — Super Mario Wiki: https://www.mariowiki.com/Mario_Paint (二次)
- [S3] Mario Artist — Wikipedia: https://en.wikipedia.org/wiki/Mario_Artist (二次)
- [S4] Flipnote Studio — Wikipedia: https://en.wikipedia.org/wiki/Flipnote_Studio (二次)
- [S5] Iwata Asks: Flipnote Studio(Nintendo UK 告知): https://www.nintendo.com/en-gb/News/2009/Iwata-Asks-Flipnote-Studio-251365.html (一次、本文は https://iwataasks.nintendo.com/interviews/ds/dsi/6/0/ — 証明書エラーで未到達)
- [S6] GoNintendo の要約: https://gonintendo.com/archives/87861-iwata-asks-vol-7-flipnote-studio-creation-flipnote-was-originally-created-for (二次)
- [S7] Iwata Asks WarioWare D.I.Y. 1: https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-WarioWare-D-I-Y-/Iwata-Asks-WarioWare-D-I-Y-/1-It-Started-Over-Five-Years-Ago/1-It-Started-Over-Five-Years-Ago-214996.html (一次)
- [S8] 同 4: https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-WarioWare-D-I-Y-/Iwata-Asks-WarioWare-D-I-Y-/4-Sharing-New-Microgames/4-Sharing-New-Microgames-215173.html (一次)
- [S9] Rhythm Tengoku — Wikipedia: https://en.wikipedia.org/wiki/Rhythm_Tengoku (二次)
- [S10] Otocky — Wikipedia: https://en.wikipedia.org/wiki/Otocky (二次)
- [S11] Electroplankton — Wikipedia: https://en.wikipedia.org/wiki/Electroplankton (二次)
- [S12] Toshio Iwai interview(GoNintendo archive): https://www.gonintendo.com/archives/4564-toshio-iwai-interview
- [S13] Tenori-on — Wikipedia: https://en.wikipedia.org/wiki/Tenori-on (二次)
- [S14] The Kid Pix Way(Hickman 原則の転記): https://pketh.org/kid-pix.html (二次)
- [S15] Kid Pix — Wikipedia: https://en.wikipedia.org/wiki/Kid_Pix (二次)
- [S16] Hickman, "Kid Pix – The Early Years": http://red-green-blue.com/kid-pix-the-early-years/ (一次、証明書エラーで未到達)
- [S17] How Media Molecule designed a fun and robust toolset for Dreams(GDC 講演要約): https://www.gamedeveloper.com/design/how-media-molecule-designed-a-fun-and-robust-toolset-for-i-dreams-i-
- [S18] Dreams — Wikipedia: https://en.wikipedia.org/wiki/Dreams_(video_game) (二次)
- [S19] LittleBigPlanet — Wikipedia: https://en.wikipedia.org/wiki/LittleBigPlanet (二次)
- [S20] Teenage Engineering OP-1 — Wikipedia: https://en.wikipedia.org/wiki/Teenage_Engineering_OP-1 (二次)
- [S21] SFMOMA, "Stay curious, stay naïve": https://www.sfmoma.org/read/stay-curious-stay-naive-an-interview-with-teenage-engineering-jesper-kouthoofd/ (一次)
- [S22] OP-1 field guide(公式): https://teenage.engineering/guides/op-1/
- [S23] KORG volca beats(公式): https://www.korg.com/us/products/dj/volca_beats/
- [S24] WBGO, Tatsuya Takahashi on Engineering Fun: https://www.wbgo.org/2017-12-22/analog-for-the-people-synth-master-tatsuya-takahashi-on-engineering-fun (一次インタビュー)
- [S25] Roland TR-8S(公式): https://www.roland.com/global/products/tr-8s/
- [S26] Ableton Live 12 manual, Session View: https://www.ableton.com/en/live-manual/12/session-view/ (一次)
- [S27] Ethan Hein, Ableton Session View and instrument design: https://www.ethanhein.com/wp/2014/ableton-session-view-and-instrument-design/ (二次)
- [S28] Ableton blog, The Evolution of Push: https://www.ableton.com/en/blog/the-evolution-of-push/ (一次)
- [S29] Ableton Push(公式): https://www.ableton.com/en/push/
- [S30] Ableton Note(公式): https://www.ableton.com/en/note/
- [S31] Resnick, Designing for Wide Walls: https://mres.medium.com/designing-for-wide-walls-323bdb4e7277 (403 で未到達、検索結果の要旨)
- [S32] PICO-8(公式): https://www.lexaloffle.com/pico-8.php ; zep 講演 "Cozy Design Spaces" スレッド: https://www.lexaloffle.com/bbs/?tid=31634
- [S33] Bitsy(itch.io): https://ledoux.itch.io/bitsy
- [S34] Figma Help, Use cursor chat: https://help.figma.com/hc/en-us/articles/4403130802199-Use-cursor-chat-in-Figma-Design
- [S35] Figma blog, April Fun Day cursors: https://www.figma.com/blog/april-fun-day-cursors/
- [S36] Rive, State Machines: https://rive.app/blog/how-state-machines-work-in-rive
- [S37] Bret Victor, Inventing on Principle(transcript): https://jamesclear.com/great-speeches/inventing-on-principle-by-bret-victor
- [S38] Bret Victor, Stop Drawing Dead Fish: https://vimeo.com/64895205
- [S39] Ludic interface / Gaver "Designing for Homo Ludens"(2002): https://en.wikipedia.org/wiki/Ludic_interface (二次)
- [S40] Nicky Case, Explorable Explanations(design patterns): https://blog.ncase.me/explorable-explanations/
