# AviUtl と After Effects の「面白い物」の整理（10/26）

選び方：作者の自己紹介ではなく、他人がまとめた一覧に何回出たかで数える（キュレーター基準）。道具（イージング、整理、書き出し、リグ）は外し、絵を作る物だけを残す。「Motolii で」の欄は、手元で確かめた事だけを書く。確かめていない借り元の候補は「候補（未確認）」と書く。

## 要約
- **AE のプラグイン**：一覧 7 本から、絵を作る物 30 種。3 本以上に出たのは Element 3D・Trapcode Particular・Optical Flares・Saber・Plexus・Newton の 6 つ。
- **AviUtl 無印**：有名 89 包（一覧 24 本、前の下調べ `aviutl1_survey.md`）から、絵を作る物 40 包。Motolii では無印のスクリプトはまだ動かない（読み込み自体が未実装）。
- **AviUtl2 の変わり種**：同梱の 193 包から、一覧には出ないが面白い物 30 包。今そのまま動くのは 11 包。
- **流行りたての物が漏れた**：Tracery などの追跡の HUD（ブロブ追跡）は、定番の一覧に出ないので拾えていなかった（4 章）。
- **共通の穴**：粒子（Particular・Stardust・拡張パーティクル・GParticle）、点と線の網（Plexus）、光る線（Saber）、レンズフレア（Optical Flares・カスタムフレア）。どれも AE でも AviUtl でも人気だが、Motolii には仕組みが無い。
- **AviUtl2 で止まる理由の上位**（下の 30 包の中）：`obj.module`（Windows の DLL）8 包、`obj.computeshader` 4 包、`obj.rand` 3 包、本体の効果を名前で 4 包、図形の読み込み 2 包。

## 1. After Effects のプラグイン（一覧 7 本）

一覧：A＝School of Motion 2026、B＝Richard Rosenman 2025、C＝Black Mixture 2020（aescripts が再掲）、D＝PremiumBeat のグリッチ 10 選、E＝Miracamp 2025、F＝SpotlightFX、G＝flashbackj の無料プラグイン。

| 名前（作者） | 作る絵 | 出た一覧 | Motolii で |
|---|---|---|---|
| Element 3D（Video Copilot） | 3D の物を AE の中で描く | A B C E（4） | 有る：Filament で glTF などの 3D の物を置ける |
| Trapcode Particular／Suite（Maxon） | 粒子（塵・火・煙・光の線） | A B E ＋検索の要約（3〜4） | **無い**。粒子の仕組みが無い |
| Optical Flares（Video Copilot） | レンズフレア | A B E（3） | 一部：ISF の `Lens Flare.fs`。AE ほど作り込めない |
| Saber（Video Copilot、無料） | 光る線・ネオンの縁・レーザー | A E G（3） | 一部：ルック Neon Edges（Open Glow → エッジ → Open Glow）。線を道に沿って走らせる所は無い |
| Plexus（Rowbyte） | 点と線でつながる網 | A B F（3） | **無い** |
| Newton（Motion Boutique） | 2D の物理（重力・ぶつかり） | A B E（3） | 有る：Jolt（物理エンジン） |
| Deep Glow（aescripts） | 自然な減り方の光 | E F ＋検索の要約（2〜3） | 有る：AviUtl2 の Open Glow（同じく光が距離の二乗で減る）と Filament のブルーム |
| AE Pixel Sorter（aescripts） | 画素を並べ替えた筋 | D F（2） | 止まる：同梱の GPU ピクセルソートは `obj.computeshader` で、PixelSorter_K は `obj.module` で止まる |
| Datamosh 2（aescripts） | 動画が溶けて流れる | D F（2） | 有る：ffmpeg でキーフレームを落とす Datamosh（ASSETS.md） |
| Bad TV（Rowbyte） | ブラウン管の歪み・走査線 | D F（2） | 有る：ISF の `Bad TV.fs`、同梱の CRT ディスプレイ風（動く） |
| Displacer Pro（Plugin Everything、無料） | 置き換えの歪み・色ずれ | D G（2） | 有る：ISF の `Displace.fs` |
| Signal（aescripts） | 90 年代の VHS・電波のノイズ | D（1） | 止まる：同梱の ntsc-rs.anm2 は `obj.module` で止まる |
| Slitscan（aescripts） | 時間をずらして引き伸ばす | D（1） | 有る：ISF の `Slit Scan.fs` |
| Modulation（aescripts） | 走査線状の波形 | D（1） | 候補（未確認） |
| Twitch（Video Copilot） | ブレ・拡大・ずれ・色分けのグリッチ | D（1） | 一部：ISF の `Time Glitch RGB.fs`、同梱の Glitch_T・SomethingLikeGlitch（どちらも動く） |
| Data Glitch（Rowbyte） | デジタルの壊れ | D（1） | 有る：ISF の `JPEG Block Corruption.fs` |
| Quick Chromatic Aberration（Plugin Everything、無料） | 色収差 | D（1） | 有る：同梱の ChromaticAberration（動く） |
| Stardust（Superluminal） | ノードで組む 3D の粒子 | A（1） | **無い** |
| Sapphire（Boris FX） | グロー・歪み・切り替えの詰め合わせ | B（1） | 一部：ISF と同梱の効果で個別に |
| Red Giant Universe（Maxon） | レトロな見た目・切り替え・文字の動き | B（1） | 一部：同上 |
| DOF PRO（Rosenman） | 被写界深度とボケの形 | B（1） | 一部：Filament の被写界深度（`doc_moment.h`）。ボケの形は選べない |
| Motion Mosaic（Rosenman） | 時間のずれたタイルのモザイク | B（1） | 候補（未確認） |
| Particle Projection（Rosenman） | 明るさで粒子を 3D に置く | B（1） | **無い** |
| Gaussian Splatting（irrealix） | 3D スキャンを合成 | F（1） | 有る：PlayCanvas の点群（ASSETS.md） |
| BAO Boa（BAO） | 道に沿って層を曲げる | F（1） | 止まる：同梱の曲げKR は `obj.drawpoly` で止まる |
| TypeMonkey（aescripts） | 音に合わせて文字が並んで動く | F（1） | 一部：同梱の LyricMotioN（動く）、AutoLyricAnimation_K（`obj.rand` で止まる） |
| VC Reflect（Video Copilot、無料） | 物の映り込み | G（1） | 候補（未確認） |
| ORB（Video Copilot、無料） | 惑星などの球 | G（1） | 有る：立体の形 Ellipsoid（`filament/solids/Ellipsoid.js`）に絵を貼る |
| Thicc Stroke（無料） | マスクに沿った太い線 | G（1） | 有る：形の trim（線を描き進める） |
| Mocha Pro（Boris FX） | 平面の追跡 | C E（2） | 道具に近いので外した |

## 2. AviUtl 無印：有名 89 包のうち、絵を作る物

数は「24 本の一覧のうち何本に出たか ／ ニコ動の再生数かダウンロード数」（前の下調べ）。無印は、Motolii ではまだ読み込めない（`aviutl1_survey.md` 6 章の 1 段目から未実装）。AviUtl2 への移植（sevenc-nanashi の ported_tim、ティム氏の効果 226 個）は同梱してあり、28 本のうち 2 本が動く。

| 名前（作者） | 人気 | 作る絵 |
|---|---|---|
| AviUtlスクリプト一式（さつき） | 15 回 ／ 520,300 DL | 文字の分解・配置・図形の詰め合わせ |
| 斜めブラインド（改）（ティム） | 8 回 | 斜めの帯で出入り |
| MultiSlicer_P（93） | 8 回 | 何本にも切ってずらす |
| 縁取りT（ティム） | 6 回 | きれいな縁取り |
| 扇クリッピングR（rikky） | 5 回 | 扇形に切り抜く |
| カメレオン効果（ティム） | 5 回 | 背景の色になじむ |
| PixelSorter_s（SEED264） | 5 回 | 画素の並べ替え |
| 斜（ePi） | 5 回 | 斜めに切ってずらす |
| 風揺れT（ティム） | 4 回 ／ 87,327 | 布のように揺れる |
| 拡張パーティクル（R）（rikky） | 4 回 ／ 50,359 | 粒子 |
| モーションタイルT（ティム） | 4 回 | 並べて敷き詰める |
| 色収差（ティム） | 4 回 | 色ずれ |
| 回転ブラー（ティム） | 4 回 | 回るぼかし |
| グラデーションマップ（Re）（Aodaruma） | 4 回 | 明るさで色を付け替え |
| ParallelCamera（Aodaruma） | 4 回 | 平行投影のカメラ |
| FlatShadow（Aodaruma） | 4 回 | 長い影 |
| DelayMove（93） | 4 回 | 遅れて付いてくる動き |
| ベジエ軌道T（ティム） | 3 回 ／ 117,307 | 曲線に沿って動く |
| パーツ分割（rikky） | 3 回 | 絵を部品に分ける |
| 漫画化（gometh） | 3 回 | 漫画の線と影 |
| カスタムフレア（ティム） | 2 回 ／ 58,497 | レンズフレア |
| 曲面変形（ティム） | 2 回 | 曲面に貼る |
| 文字輪郭追跡（UndoFish） | 2 回 | 文字の縁を光が走る |
| 輪郭追跡（ティム） | 2 回 | 絵の縁を光が走る |
| ライントーン＆ハーフトーン（ティム） | 2 回 | 網点・線の網 |
| DisplacementMap-B（93） | 2 回 | 置き換えの歪み |
| パチンコ風文字（gometh） | 1 回 ／ 79,267 | 派手な立体文字 |
| 虹色グラデーション（ティム） | 1 回 | 虹色 |
| 簡易モーフィング（ティム） | 1 回 | 2 枚の絵を変形でつなぐ |
| ひび割れガラスT（ティム） | 1 回 | ひび割れ |
| 立体化（g）（gometh） | 1 回 | 厚みを付ける |
| ディスプレイスメントマップ3D（ゆうき） | 1 回 | 明るさで 3D に押し出す |
| Z軸ループ・粒子化（テツ） | 1 回 ／ 59,382 | 奥へ繰り返す・粒子にほどく |
| 注ぎT（ティム） | ― ／ 89,394 | 液体が注がれる |
| 燃焼（ティム） | ― ／ 53,007 | 燃えて消える |
| 砕けたガラス（ティム） | ― ／ 37,528 | 割れて飛び散る |
| twitchっぽいものT（ティム） | ― ／ 35,250 | Twitch 風グリッチ |
| インク（＋ひょうたん）（ティム） | ― ／ 32,391 | インクがにじむ |
| 稲妻（ティム） | ― ／ 31,973 | 稲妻 |
| ストロークT（ティム） | ― ／ 38,922 | 線を描き進める |

## 3. AviUtl2 の変わり種（同梱の 193 包から、一覧の外）

「動く」は、同梱の全部を写真に 1 本ずつ掛けた結果（10/26）。止まる物は、止まった所の理由。

| 名前（作者） | 作る物 | Motolii で |
|---|---|---|
| AviUtl2Browser_H（HexBrowns） | ブラウザで描く（p5.js のスケッチなど） | 動く（約束の代役。中のブラウザ表示はまだ） |
| VRM_Toolkit（nove-nove） | VRM のキャラを出して動かす | 止まる：`obj.module` |
| GameOfLife（sigma-axis） | ライフゲーム | 止まる：本体の「単色化」 |
| PenroseTile_S（sigma-axis） | ペンローズの敷き詰め | 止まる：ほかのスクリプト（ぼかしσ など）が見つからない |
| RainOnPane（nctype） | 窓に付いた雨粒 | 止まる：`obj.copybuffer`（framebuffer から） |
| ReflectionOnPane（nctype） | 窓の映り込み | 止まる：本体の「色調補正」 |
| PuniCollision（nctype） | ぷにぷにぶつかる | 止まる：`obj.getvalue` |
| LiquidGlass（tettu0530） | 液体のガラス | 止まる：`obj.pixeloption` |
| SumiDamari（saluf） | 墨だまり | 動く |
| SomethingLikeGlitch（nctype） | グリッチ | 動く |
| Glitch_T（sagami121） | グリッチ | 動く |
| SliceGlitch（暘_弥涼） | 切ってずらすグリッチ | 止まる：`obj.rand` |
| JPEGCrush（しけもち） | JPEG の崩れ | 止まる：`obj.module` |
| ntsc-rs.anm2（sevenc-nanashi） | アナログ放送・VHS | 止まる：`obj.module` |
| CRT ディスプレイ風（sevenc-nanashi） | ブラウン管 | 動く |
| ドカベン（sevenc-nanashi） | ドカベン風に文字が回る | 止まる：`obj.drawpoly` |
| disassembler（sevenc-nanashi） | 文字をばらす | 止まる：`obj.module` |
| budoux（sevenc-nanashi） | 日本語の自然な改行 | 止まる：`obj.module` |
| PolygonMorph_F（Fu-Majime） | 多角形の変形 | 動く |
| LyricMotioN（NeAc） | 歌詞の動き | 動く |
| AutoLyricAnimation_K（korarei） | 歌詞の自動アニメ | 止まる：`obj.rand` |
| FlowType_K（korarei） | 流れる文字 | 6 本中 2 本動く（ほかは `obj.module`） |
| Egasumi（NeAc） | 絵巻の霞 | 止まる：図形「円」 |
| SpeedLine（NeAc） | 集中線 | 止まる：図形「背景」 |
| rich-audio-spectrum（Gohankaiju） | 音のスペクトラム | 動く |
| NCS 風音声波形（Tettu0530） | NCS 風の波形 | 止まる：`obj.getaudio` |
| GParticle（sagami121） | GPU の粒子 | 止まる：`obj.computeshader` |
| 光学モーションブラー（zopty） | 動きから作るブラー | 止まる：`obj.pixelshader` の一部 |
| twinkle_sl（seilor0） | きらめき | 動く |
| VectorCanvas（mogesystem） | ベクター図形 | 23 本すべて動く |

## 4. 流行りたての物（定番の一覧にはまだ出ない）

定番の一覧は何年も前からある物が並ぶので、流行りたての物が漏れる。10/26 にユーザーが Tracery を挙げて気付いた。漏れた種類と、その探し方の穴をここに書く。

| 名前（作者） | 作る絵 | 出典 | Motolii で |
|---|---|---|---|
| Tracery（Pseudo Effects、aescripts） | 色で物を追い、四角・中心の印・文字・矢印を重ね、追った点を線でつなぐ（順番・星形・全部・最小全域木） | aescripts の製品ページ（403 で読めず、検索の要約から） | **無い** |
| AE Blob Tracker（Kyle Sullivan、無料） | 明るさ・色・動きでブロブを見つけ、HUD 風の四角と座標を重ねる | https://github.com/KyleSullivan321/AE-Blob-Tracker （ライセンスの書類なし、星 95） | 借りられない（ライセンスなし） |
| blob-tracey（Batkixni） | 上の物を真似た Web 版 | https://github.com/Batkixni/blob-tracey （MIT、TypeScript、星 0、中身は未読） | 候補（未確認） |
| TouchDesigner の Blob Track | 同じ見た目の定番の作り方 | https://docs.derivative.ca/Blob_Track_CHOP （中身は OpenCV） | 候補：Motolii は OpenCV を使っている（緑を抜いた動画の輪郭、ASSETS.md） |
| TrackingラインEasy（ティム、AviUtl 無印） | 追跡の線 | 有名 89 包の 84 番（34,885 回再生）。2 章の表から落としていた | 無印は未対応 |

探し方の穴：aescripts の新着、TikTok・Instagram で流行っている見た目の名前（blob tracking など）、TouchDesigner のチュートリアルの流行りを見ていない。

## 5. 共通の穴と借り元の候補

どれも「候補（未確認）」。中身を読んでから決める。

| 穴 | 欲しがっている物 | 借り元の候補（未確認） |
|---|---|---|
| 粒子 | Particular・Stardust・Particle Projection・拡張パーティクル・GParticle | GParticle の computeshader を動かす（同梱）／ three.js・PlayCanvas の粒子の見本 |
| 点と線の網 | Plexus | three.js の見本 `webgl_buffergeometry_drawrange`（点を線でつなぐ網） |
| 光る線を道に沿って | Saber・輪郭追跡・文字輪郭追跡・ストロークT | 形の trim ＋ Open Glow（今ある物の組み合わせ）／ Saber の中身は非公開 |
| レンズフレア | Optical Flares・カスタムフレア | ISF の `Lens Flare.fs`（今ある）より作り込める物は未調査 |
| Windows の DLL | VRM・ntsc-rs・budoux・JPEGCrush など 8 包 | DLL の元ソースがある物（ntsc-rs は Rust で公開）は Mac 向けに作り直す道（`aviutl1_survey.md` 4 章と同じ） |
| 3D キャラ | VRM_Toolkit、YMM4 の 3D モデル読み込みプラグイン | Filament の glTF（VRM は glTF の拡張） |
| 追跡の HUD | Tracery・AE Blob Tracker・TrackingラインEasy | OpenCV（今使っている）で物を見つけ、ThorVG で四角と線を描く。TouchDesigner の Blob Track と同じ組み方 |

前に話に出た「YMM4 でマイクラを動かせる物」は **YMM4Craft**（いるかぁぁ、MIT、Java、https://github.com/Dolphin-kun/YMM4Craft 、紹介 https://ymm4-info.net/plugin/1792142745633371 ）。Minecraft Java 版の Mod（Fabric）と YMM4 のプラグインの 2 つで、YMM4 のタイムラインからマイクラの世界のカメラ・時刻・天気を動かし、プレイヤーや Mob を置き、遊んだ様子を好きな視点で再生し直す。世界を動画に畳まずに編集の中で使う形。初めの検索（米国の検索、言葉を詰め込みすぎ）では見つけられず、ユーザーが「ymm4 マイクラ」で見つけた。同じ作者の YMM43D（3D）・YMM4Physics2D（2D の物理）も MIT。

見落とし：YMM4 のプラグインの目録 ymm4-info.net を目録ごと見ていない。日本語の物は、アプリ内のブラウザで日本の検索と各目録を直接引いて探し直す。

## 出典
- School of Motion「The Best After Effects Plugins and Effect Packs You NEED in 2026」 https://www.schoolofmotion.com/blog/best-after-effects-plugins-and-effect-packs-you-need-in-2026
- Richard Rosenman「The Best After Effects Plugins of 2025」 https://richardrosenman.com/the-best-after-effects-plugins-of-2025/
- aescripts「Top 10 best plugins for After Effects 2020」（Black Mixture の一覧の再掲） https://aescripts.com/learn/top-10-best-plugins-for-after-effects-2020
- PremiumBeat「After Effects glitch plugins」 https://www.premiumbeat.com/blog/after-effects-glitch-plugins/
- Miracamp「Best After Effects Plugins in 2025」 https://www.miracamp.com/learn/after-effects/best-plugins
- SpotlightFX「Best After Effects plugins」 https://spotlightfx.com/blog/best-after-effects-plugins
- flashbackj「After Effects & Premiere Pro で使える無償プラグイン」 https://flashbackj.com/after-effects-premiere-pro-free-plugins
- YMM4 3D モデル読み込みプラグイン https://booth.pm/ja/items/8139706
- AviUtl 無印の人気の数：`reports/aviutl1_survey.md`（記事 17 本＋動画の概要欄 7 本、2015〜2026）
- AviUtl2 の動く・止まる：同梱の 594 本を写真に掛けた結果（`reports/aviutl2/catalogue.md`）
