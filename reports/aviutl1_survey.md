# AviUtl（無印 1.10 ＋ 拡張編集 0.92）のスクリプトを Motolii でそのまま動かす — 下調べ

2026-10-26。Motolii のコードは書いていない。落とした物はすべて scratchpad に置いた。どれも実行していない（読んで数えただけ）。置き場は一番下。

## 要約（5 行）

1. **有名どころ 117 本を 89 包に対応づけ、83 包・287 本の中身を読んだ。** 元にしたのは「おすすめスクリプト」の記事・動画 24 本、再生数、ダウンロード数。DLL なしで動く天井は **287 本中 237 本（83%）**。包で数えると 83 包中 56 包（67%）。
2. **許諾**：再配布可 25、使うだけ 7（rikky 6、テステス 1）、不明 57。不明のうち 38 はティムの物で、readme にも配布ページにも利用条件が書いていない。だから **同梱できるのは 25 包だけ**。59 包は「配布元から取得」で入れられる。5 包は取得不可（ページが消えた、ログインが要る）。
3. **apm の目録がそのまま見本になる。** 285 件（うちスクリプト 141 件、ティム 91 件）について、作者の配布 URL と sha384 を持っている。ティムの 91 件は、落とした zip の 89 件がハッシュまで一致した。ただし **データの許諾は CC BY-NC-SA 4.0（非営利）**。アプリ本体は MIT。
4. **今の AviUtl2 変換器との差は大きい。** 古い拡張子・Shift-JIS・`--dialog`（82/83 包が使う）を読めない。`obj.rand`・`drawpoly`・3D とカメラ・本体の効果 39 種・文字や画像の読み込み・画素の読み書き（BGRA 8bit）も無い。小さい順に足すと、動く本数は 4% → 14% → 36% → 52% → 68% → 83% と増える。
5. **無理なのは、元ソースの無い 32bit Windows DLL を要る物（39 本、14%）。** rikky_module、ティムの T_*_Module 15 個、さつき一式の一部が使う extbuffer などがそれに当たる。ただし **ティムの全 226 効果は ported_tim（MIT、ティム本人が配布を許可）が AviUtl2 向けに作り直している**。その道を使えば天井は 87% に上がる。

---

## 1. スクリプトの置き場と人気の測り方

### 置き場の種類（有名 89 包の取り先）

| 取り先 | 包の数 | 自動で取れるか |
|---|---|---|
| fc2 の直リンク（ティム tim3.web.fc2.com、テツ） | 39 | 取れる。2011 年から同じ URL |
| GitHub の releases | 15 | 取れる |
| Dropbox（93、UndoFish、kaisatsu、微熱） | 10 | 古い `/s/` リンクは `?dl=1` で取れる。kaisatsu のフォルダは「No Access」 |
| Google Drive（gometh、ちはユキ、Nagomiku、テステス） | 9 | `uc?export=download&id=` で取れる。大きい物は確認画面が出る |
| hazumurhythm.com（rikky） | 5 | サイトからの Referer が要る（普通にクリックすれば付く） |
| bowlroll（さつき一式） | 1 | ログインなしのボタン。520,300 DL で一番多い |
| axfc（おばけ大根、金の髭、3D 音声波形） | 1（＋有名でない物 3） | 生きている。ただしボタンに Cloudflare の確認がある。利用者が自分で押すしかない |
| 個人サイト（takeshima、renhagu、shummg、cumulo） | 4 | 取れる |
| 取得不可 | 5 | ePi の scrapbox はログインが要るようになった（斜、koma、ease）。白水（hksy）のサイトは 404 で、purinka.work は広告の駐車ページ。駒込ブーツ・ゆうきの物は見つからない |

その他：
- **getuploader（uploader.jp）は 2026-11-25 に終わり、ファイルは消される**（2026-10-23 のお知らせ）。野生の木屋P の花火などがここにある。
- MaverickTse の AviUtlSvg（OneDrive）は Microsoft のログインに飛ぶ。

### まとめ・目録

| 何 | どこ | 中身 |
|---|---|---|
| **apm（AviUtl Package Manager）の目録** | https://github.com/team-apm/apm-data （`src/packages/*/*/package.yaml`、配る形は v3 の json） | 285 件、スクリプト 141 件。ティム 91、rikky 10、oov 6、SEED264 5 など。1 件ごとに次を持つ：`downloadURLs`（作者のページか zip）、`pageURL`、`nicommons`、`files[].archivePath`（zip の中のどこを、どこに置くか）、`releases[].integrity`（zip と中のファイルの sha384）、`dependencies`。目録に無い作者向けに `scripts.yaml`（配布ページの URL の形 → 置くフォルダと依存）もある（13 サイト） |
| apm のアプリ | https://github.com/team-apm/apm （MIT） | 作者のサイトを中のブラウザで開く → 利用者がダウンロードを押す → アプリが受け取り、ハッシュを確かめ、`files` の通りに置く。直リンクがある物（`directURL`）は「一括インストール」で自動で取る |
| AviUtl2 のカタログ | https://github.com/Neosku/aviutl2-catalog-data | AviUtl1 の物の移植が載る：ported_tim（MIT、226 効果）、ported_satsuki（許諾は未定。「さつき氏と連絡が取れていない」）、UndoFish の Easing2020（許諾は不明）、無印ノイズ・旧発光の再現（MIT） |
| 有名さの元 | 記事 17 本＋動画の概要欄 7 本（2015〜2026）。mina さんのアンケート（2021-12、回答 27） | 一覧：scratchpad の `records/popularity_sources.json` |
| 数 | ニコ動の配布動画の再生数（getthumbinfo で 2026-10-26 に読んだ）、bowlroll の DL 数、GitHub の星 | — |

「かんな」は動画でおすすめを紹介した人で、作者ではない（yt_kanna）。89HAL は作者として見つからなかった。

作者の層（24 本のうち何本が 1 本以上名前を挙げたか）：ティム 19、さつき 17、93（Respectrum93）13、UndoFish 12、rikky 11、Aodaruma 7、SEED264 7、kaisatsu・ePi・gometh 5。

## 2. 有名どころ 89 包

- **人気**：「n 回」は 24 本の一覧のうち何本に名前が出たか。その後ろはニコ動の再生数（さつきは bowlroll の DL 数、GitHub の物は星）。
- **入れ方**：同梱 ＝ 再配布可の許諾がある。配布元から取得 ＝ 公開の URL から利用者の Mac が取る。取得不可 ＝ 消えた、ログインが要る、URL が定まらない。
- **DLL**：「閉じた」＝ 元ソースが無い（Mac では無理）。「元あり」＝ 元ソースがある（作り直せば動く）。
- **要る段**：6 章の 1〜7。
- パック（さつき一式、93 の old_script_etc、Aodaruma の置き場、sigma の効果集）は、中の全部が要る物を足して書いた。

| # | 名前 | 作者 | 人気 | 許諾 | 入れ方 | 取り先 | DLL | 要る段 |
|---|---|---|---|---|---|---|---|---|
| 1 | AviUtlスクリプト一式 (ANM1 ほか) | さつき | 15 回 / 520,300 DL | 不明 | 配布元から取得 | https://bowlroll.net/file/3777 | 閉じた: T_Color_Module, extbuffer（一部の効果だけ） | 1,2,4,5,6,7 |
| 2 | イージング トラックバー対応版 | UndoFish | 12 回 / 275,627 | 不明 | 配布元から取得 | https://www.dropbox.com/s/clvaltohynpzmm9/easing_tra_2020.zip?dl=1 | - | 1,2,3,4,5,6 |
| 3 | 色調調整セット(ver6) | ティム | 9 回 / 59,250 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/ColAdj6.zip | 閉じた: T_Color_Module | 1,2,4,5,6,7 |
| 4 | 斜めブラインド(改) | ティム | 8 回 / 40,331 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/DiaSha.zip | - | 1,5 |
| 5 | MultiSlicer_P | 93 | 8 回 | 再配布可「改変後の再配布は自由」 | 同梱 | https://www.dropbox.com/sh/u73uud29hcxlply/AABH9ZhzL1P1kX-bWrL4asdDa?dl=1 | - | 1,2,4 |
| 6 | 縁取りT | ティム | 6 回 / 40,184 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Framing.zip | 閉じた: T_Framing_Module | 1,2,5,7 |
| 7 | 扇クリッピングR | rikky | 5 回 / 47,895 | 使うだけ「そのまま配布や二次配布のような形はやめていただきたい」 | 配布元から取得 | https://cumulo.works/s/fan_clipping_(ver3).zip | - | 1,3,5 |
| 8 | カメレオン効果 | ティム | 5 回 / 36,403 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Famili.zip | 閉じた: T_Familiar_Module | 1,2,5,7 |
| 9 | なめらか縁取り | kaisatsu | 5 回 / 11,884 | 不明 | 配布元から取得 | Dropbox の単体 .anm（フォルダは No Access） | - | 1,5 |
| 10 | PixelSorter_s | 星野なたね（SEED264） | 5 回 / ★15 | 再配布可「MIT」 | 同梱 | https://github.com/SEED264/PixelSorter_s/releases/latest | 元あり: PixelSorter_s | 1 |
| 11 | 斜 | ePi | 5 回 | 不明 | 取得不可 | https://scrapbox.io/ePi5131/tiltclipping （ログインが要る） | - | 未取得 |
| 12 | 風揺れT | ティム | 4 回 / 87,327 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/WindShk.zip | - | 1,2,4,5 |
| 13 | 拡張パーティクル(R) | rikky | 4 回 / 50,359 | 使うだけ（同上） | 配布元から取得 | https://hazumurhythm.com/a/RBa6/ | 閉じた: particle_set3 | 1,2,3,6 |
| 14 | モーションタイルT | ティム | 4 回 / 48,957 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/MTile.zip | - | 1 |
| 15 | 色収差 | ティム | 4 回 / 32,167 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/ChromaV2.zip | - | 1,5,7 |
| 16 | 回転ブラー（ブラースクリプト） | ティム | 4 回 / 30,824 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/RotBlurV3.zip （apm に無い） | 閉じた: T_RotBlur_Module | 1,5,7 |
| 17 | グラデーションマップ(Re) | Aodaruma | 4 回 / 24,473 | 再配布可（MIT と LGPL-2.1 の両方の書き方がある） | 同梱（スクリプトだけ） | https://github.com/Aodaruma/Aodaruma-AviUtl-Script/releases | 閉じた: gradationmapping（rikky 製）, rikky_module | 1,3,5,6,7 |
| 18 | ParallelCamera | Aodaruma | 4 回 / ★52 | 再配布可「MIT」 | 同梱 | https://github.com/Aodaruma/ParallelCamera/releases/latest | - | 1,2,4,6 |
| 19 | FlatShadow | Aodaruma | 4 回 | 再配布可「LGPL-2.1」 | 同梱（スクリプトだけ） | Aodaruma-AviUtl-Script の releases | 閉じた: AviutlLongShadow（rikky 製）, rikky_module | 1,2,3,5,6,7 |
| 20 | DelayMove（old_script_etc） | 93 | 4 回 | 再配布可（同上） | 同梱 | 93 の Dropbox | 閉じた: extbuffer, rikky_module（パックの一部） | 1〜7 |
| 21 | ベジエ軌道T | ティム | 3 回 / 117,307 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/BezSpe.zip | - | 1,2 |
| 22 | 角付き丸吹き出し | カメ/ウサギ | 3 回 / 28,659 | 不明 | 配布元から取得 | https://takeshima.halfmoon.jp/room/aviutl/speech_balloon_anm_001.zip | - | 1,2,4,5 |
| 23 | 自動呼吸 | ちはユキ | 3 回 / 26,712 | 不明 | 配布元から取得 | https://drive.google.com/file/d/1dlS4DgCvJWRN-YrW3NWm2gM7UNA2ENm1/view | - | 1,4,5 |
| 24 | グリッドワイプ | ティム | 3 回 / 24,136 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/k2-GrdWipe.zip | - | 1,2,4,5 |
| 25 | 長方形角取り | 連結歯車 | 3 回 / 23,950 | 再配布可「CC0」 | 同梱 | renhagu.net の zip | - | 1,2,4,5 |
| 26 | パーツ分割 | rikky | 3 回 / 20,176 | 使うだけ | 配布元から取得 | https://hazumurhythm.com/a/pA2/ | 閉じた: rikky_module, rikky_separate | 1,2,5,6,7 |
| 27 | 漫画化 | gometh | 3 回 / 16,015 | 不明 | 配布元から取得 | https://drive.google.com/uc?export=download&id=13AZYerMDciBtK7-jvTttbMo0RKw-H8bV | - | 1,2,3,4,5,7 |
| 28 | （もうちょっと）ノリノリ | おばけ大根 | 3 回 / 11,075 | 不明 | 配布元から取得（axfc、利用者が押す） | https://www.axfc.net/u/3907922.dat | - | 未取得 |
| 29 | OpticsCompensation_s | SEED264 | 3 回 / ★8 | 再配布可「MIT」 | 同梱 | https://github.com/SEED264/OpticsCompensation_s/releases/latest | 元あり | 1 |
| 30 | TA-Track | 93 | 3 回 | 再配布可 | 同梱 | 93 の Dropbox | - | 1,2,3,4,5,7 |
| 31 | カスタムフレア | ティム | 2 回 / 58,497 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/CstFlaV3.zip | - | 1,2,4,5,6 |
| 32 | 曲面変形 | ティム | 2 回 / 48,777 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/k5-CurTra.zip | - | 1,2,4 |
| 33 | 文字輪郭追跡 | UndoFish | 2 回 / 43,305 | 不明 | 配布元から取得 | Dropbox | 閉じた: uf_opencv | 1,2,3,4,7 |
| 34 | 輪郭追跡 | ティム | 2 回 / 37,963 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/OutLPur.zip | - | 1,4,7 |
| 35 | ライントーン＆ハーフトーン | ティム | 2 回 / 24,666 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/LinHal.zip | - | 1,2,4,5,7 |
| 36 | koma | ePi | 2 回 / 6,675 | 不明 | 取得不可 | scrapbox（ログインが要る） | - | 未取得 |
| 37 | ディザリング / レンズディストーション | rikky | 2 回 / 6,116 | 使うだけ | 配布元から取得 | https://hazumurhythm.com/a/F3bj/ | 閉じた: dithering, LensDistortion | 1,2,3 |
| 38 | ランダム配置・角丸四角形・直線 ほか（@hksy） | 白水 | 2 回 | 不明 | 取得不可 | hksy.web.fc2.com（404）、purinka.work（駐車ページ） | - | 未取得 |
| 39 | aulut | ePi | 2 回 | 再配布可「MIT」 | 同梱 | https://github.com/ePi5131/aulut/releases/latest | 元あり | 1,5,6,7 |
| 40 | DisplacementMap-B | 93 | 2 回 | 再配布可 | 同梱 | 93 の Dropbox | - | 1,2,5,6 |
| 41 | 領域枠 | ティム | 2 回 / 46,112 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Frame2.zip | - | 1,2,4,5 |
| 42 | RadialWipe_s | SEED264 | 2 回 | 再配布可「MIT」 | 同梱 | https://github.com/SEED264/RadialWipe_s/releases/latest | 元あり | 1 |
| 43 | オートターゲット / 位置補正 | ティム | 1 回 / 102,073 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/AutoTgt.zip | - | 1,2,4 |
| 44 | パチンコ風文字 | gometh | 1 回 / 79,267 | 不明 | 配布元から取得 | Google Drive | - | 1,2,4,5,6,7 |
| 45 | 虹色グラデーション | ティム | 1 回 / 49,872 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/RbwGra.zip | 閉じた: T_R_gradation | 1,2,7 |
| 46 | 簡易スライドIO | じつかいせき | 1 回 / 47,931 | 再配布可「再配布等はご自由に」 | 同梱 | Google Drive | - | 1 |
| 47 | 手ぶれ(R) | rikky | 1 回 / 47,895 | 使うだけ | 配布元から取得 | https://hazumurhythm.com/a/2Nr/ | - | 1,2,4 |
| 48 | フィルターセット | ティム | 1 回 / 45,641 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/FltSet.zip | 閉じた: T_Filter_Module | 1,2,4,5,7 |
| 49 | 簡易モーフィング | ティム | 1 回 / 42,454 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/SplMrp.zip | 閉じた: extbuffer | 1,2,4,6,7 |
| 50 | バニシングポイント2 | ティム | 1 回 / 32,572 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/VanishP2_V2.zip | - | 1,2,4 |
| 51 | ゆらめき | ティム | 1 回 / 31,524 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Shake.zip | - | 1,2,4 |
| 52 | モザイクマスク(R) | rikky | 1 回 / 25,037 | 使うだけ | 配布元から取得 | https://hazumurhythm.com/a/Av9/ | 閉じた: rikky_module | 1〜7 |
| 53 | ラフエッジ | ティム | 1 回 / 23,016 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/RoughE.zip | - | 1,2,5,6 |
| 54 | ひび割れガラスT | ティム | 1 回 / 18,076 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/CrkGls.zip | 閉じた: T_CrackedGlass_Module | 1,5,6,7 |
| 55 | 色抽出 | ティム | 1 回 / 15,516 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/ColExt.zip | - | 1,2,5 |
| 56 | スプリット | ティム | 1 回 / 15,481 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Split.zip | - | 1,2,3,4,5,6 |
| 57 | 色付きエッジ抽出 | ティム | 1 回 / 12,223 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/ColEdge.zip | - | 1,2,5 |
| 58 | 曲げKR（アンカーで曲げ） | karoterra | 1 回 / 11,503 | 再配布可「MIT」 | 同梱 | https://github.com/karoterra/aviutl-Bend/releases/latest | - | 1,2,3,4 |
| 59 | ハーフトーン(g) | gometh | 1 回 / 10,512 | 不明 | 配布元から取得 | Google Drive | - | 1,3,7 |
| 60 | 立体化(g) | gometh | 1 回 / 10,038 | 不明 | 配布元から取得 | Google Drive | - | 1,3,4,5,7 |
| 61 | 模様 | gometh | 1 回 / 6,500 | 不明 | 配布元から取得 | Google Drive | - | 1,2,3,5,7 |
| 62 | MotionBlur_K | korarei | 1 回 / 5,681 | 再配布可「MIT」 | 同梱 | https://github.com/korarei/AviUtl_MotionBlur_K_Script/releases/latest | 元あり | 1 |
| 63 | 縁取りσ（効果集） | sigma-axis | 1 回 / 4,470 | 再配布可「MIT」 | 同梱 | https://github.com/sigma-axis/sigma_aviutl_scripts/releases/latest | - | 1〜7 |
| 64 | 反復移動プラス | テステス | 1 回 / 4,375 | 使うだけ「このスクリプトの2次配布は禁止です。」 | 配布元から取得 | Google Drive | - | 1,2,6 |
| 65 | InlineScene_S | sigma-axis | 1 回 / 2,947 | 再配布可「MIT」 | 同梱 | GitHub releases | patch.aul が要り、aviutl.exe の中身を ffi で直に読む（無理） | 1,2,3,4,5,6 |
| 66 | TrackBoundary_S | sigma-axis | 1 回 / 2,054 | 再配布可「MIT」 | 同梱 | GitHub releases | - | 1,2,3,5,6,7 |
| 67 | キャッシュRM ほか（置き場一式） | Aodaruma | 1 回 | 再配布可「基本的に LGPL-2.1」 | 同梱（スクリプトだけ） | https://github.com/Aodaruma/Aodaruma-AviUtl-Script/releases/latest | 閉じた: rikky_module ほか | 1,2,4,5,6,7 |
| 68 | Modulator | Aodaruma | 1 回 | 再配布可「MIT」 | 同梱 | Aodaruma-AviUtl-Script の releases | 元あり | 1,2,6,7 |
| 69 | マルチベジェ軌道 | 93 | 1 回 | 再配布可 | 同梱 | 93 の Dropbox | - | 1,2,4,6,7 |
| 70 | GetColor V2r | 93 | 1 回 | 再配布可 | 同梱 | 93 の Dropbox | 閉じた: rikky_module | 1〜7 |
| 71 | B2_RoundAcorner | 駒込ブーツ | 1 回 | 不明 | 取得不可（配布停止） | — | - | 未取得 |
| 72 | 値で図形 | Nagomiku | 1 回 | 不明 | 配布元から取得 | Google Drive | - | 1,2,4 |
| 73 | 角丸十字 | しゅう | 1 回 | 再配布可「MIT」 | 同梱 | https://github.com/shummg/RoundedCross/releases | - | 1,2,4 |
| 74 | Z軸ループ・粒子化 ほか（一式） | テツ | 1 回 / 59,382 | 不明 | 配布元から取得 | http://niconicotetu.web.fc2.com/script/tetu.zip | - | 1〜7 |
| 75 | 等間隔に配置 | しゅう | 1 回 | 再配布可「LGPL…再配布、改変などが（可能）」 | 同梱 | https://shummg.work/downloads/ | - | 1 |
| 76 | CircleSlicer | 93 | 1 回 | 再配布可 | 同梱 | 93 の Dropbox | - | 1〜7 |
| 77 | ディスプレイスメントマップ3D | ゆうき | 1 回 | 不明 | 取得不可（動画が消えた） | — | - | 未取得 |
| 78 | 注ぎT | ティム | - / 89,394 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Pour.zip | - | 1,2,4,5,7 |
| 79 | 燃焼 | ティム | - / 53,007 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Combst2.zip | 閉じた: T_burning_Module, extbuffer | 1,2,5,6,7 |
| 80 | シーンチェンジセット | ティム | - / 52,029 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/SceChg.zip | - | 1,2,4,5 |
| 81 | ストロークT | ティム | - / 38,922 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Stroke.zip | - | 1,2,4,5 |
| 82 | 砕けたガラス | ティム | - / 37,528 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/GlassSd.zip | - | 1,2,3,4,5,6 |
| 83 | twitchっぽいものT | ティム | - / 35,250 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/Twitch.zip | - | 1,2,4,5 |
| 84 | TrackingラインEasy | ティム | - / 34,885 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/TraEasy.zip | - | 1,2,5 |
| 85 | スカイドーム | ティム | - / 32,804 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/SkyDome.zip | - | 1,2,4 |
| 86 | 多色グラデーション | ティム | - / 32,682 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/PolCGra.zip | - | 1,2,4,5 |
| 87 | インク（＋ひょうたん） | ティム | - / 32,391 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/InkV2.zip | - | 1,2,4,5 |
| 88 | 稲妻 | ティム | - / 31,973 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/LightNg.zip | - | 1,2,4,5 |
| 89 | ライン抽出T | ティム | - / 31,511 | 不明 | 配布元から取得 | https://tim3.web.fc2.com/script/LineEx.zip | 閉じた: T_LineExtra_Module | 1,5,7 |

78〜89 は、一覧には出なかったが再生数が 3 万を超えるティムの物。

一覧に 1〜3 回だけ名前が出て、包を突き止めていない物が 28 ある：s_shakeもどき（まいん）、本当のブロックノイズ（プリンカ、サイトが消えた）、Turbulent@Displace（pon）、黒背景透過・塗りつぶしクロマキー（微熱）、スケールワイプ（kaisatsu）、簡易3次元LUT、反射型立体・目標拡大（なごみく）、二値化、アスキーアート化、五線譜（gometh）、合成モード追加・ランダムフォント・少々ぼかし・テキスト一枚化（rikky）、雑縁取り、直線（ちくぼん）、会話時立ち絵制御、オートブラー+（93）、ランダム8方向フェード、破れクリッピング、Simple_LUT、RadialBlind_s（SEED264）、レンズ補正もどき、角丸四角形plus、sinwave、BPM同期効果処理。

調べた包の全体（有名でない物を含む 234 包）での数：再配布可 68、使うだけ（再配布禁止）26、不明 140。入れ方は、同梱 68、配布元から取得 153、取得不可 13。

## 3. 許諾と入れ方

### 数（有名 89 包）

| 分け方 | 数 | 中身 |
|---|---|---|
| **再配布可** | **25** | MIT 13（SEED264、karoterra、korarei、sigma-axis、ePi の aulut、Aodaruma の一部、しゅう）、LGPL-2.1 3（Aodaruma、しゅう）、CC0 1（連結歯車）、明文の「再配布可」8（93 の「改変後の再配布は自由」7、じつかいせき「再配布等はご自由に」1） |
| **使うだけ** | **7** | rikky 6（サイトの説明に「そのまま配布や二次配布のような形はやめていただきたい」。改変して名前を変えれば口出ししない、とも書いてある）、テステス 1（「2次配布は禁止」） |
| **不明** | **57** | ティム 38（zip に同梱の txt 11 本にも、配布ページ・トップページにも利用条件の記載なし）、gometh 6、UndoFish 2、さつき（readme は免責だけ）、テツ、カメ ほか |

| 入れ方 | 数 |
|---|---|
| 同梱 | 25（ただし Aodaruma の 3 包は中の DLL が rikky 製。同梱できるのはスクリプトだけ） |
| 配布元から取得 | 59（自動で取れる 52、利用者が押す要る 7：hazumurhythm 5、bowlroll 1、axfc 1） |
| 取得不可 | 5（ePi 2、白水、駒込ブーツ、ゆうき） |

気をつける所：
- 93 の文言は「改変後の再配布は自由」で、改変しないままの再配布は書いていない。.lua の部品も名前に入っていない。安全をとるなら 93 も「配布元から取得」にする。Dropbox は安定している。
- 同梱できる 25 包の中にも、閉じた DLL を要る物がある（Aodaruma の FlatShadow など）。

### apm の目録を借りる道

- **形**：YAML（`src/`）から v3 の json を作って配っている（jsDelivr）。1 件の例（ティムのレンズフレア）：
  - `downloadURLs: [https://tim3.web.fc2.com/script/LensFla.zip, https://tim3.web.fc2.com/sidx.htm#LensFla]`
  - `files: [{filename: script/tim/Lensflare.obj, archivePath: LF}]`
  - `releases[].integrity.archive: sha384-…`
- **合うか**：ティムの 91 包の zip を落としてハッシュを比べると、89 包が一致した。違ったのは色調調整セットとツイスターで、作者が後から差し替えていた。→ 入れるボタンは「ハッシュが違ったら、作者の新しい版かもしれないと言って止める」作りにする。
- **許諾**：データは **CC BY-NC-SA 4.0**（README に明記）。
  - Motolii を売るなら、目録を丸ごと写すことはできない。
  - 無料の場合でも、SA があるので Motolii 側の目録も同じ許諾になる。
  - URL やハッシュの一つ一つはただの事実で、作品とまでは言えないかもしれない。それでも、まとめて写すなら Team apm に聞くのが筋。
  - **今回自分で集めた 234 包の記録（URL・許諾の文言・DLL）は、apm から写した物ではない。**
- **アプリ**：MIT。中のブラウザで作者のページを開き、落ちてきたファイルを受け取って確かめる流れは、そのまま見本になる。apm は依存（例：`rikky/rikkyModule2`、`ePi/LuaJIT`）も持っていて、「この包は閉じた DLL が要る」とインストール前に言える。
- 目録の偏り：スクリプト 141 件のうち 91 件がティム。さつき・UndoFish・93・gometh は「配布サイト」として登録されているだけ（ハッシュなし）。

## 4. 技術の差（今の AviUtl2 変換器と比べて）

今の変換器（filament/aviutl.h, aviutl.cpp, aviutl_fx.h）は .anm2 / .obj2 だけを読む。Lua は LuaJIT 2.1 で、os・io・debug・dofile・loadfile を外してある。obj は 13 個の関数を持ち、そのほかの名前は「not yet」で止まる。

### ファイルと文字

| 項目 | AviUtl1 | 今の変換器 | 有名 83 包で使う数 |
|---|---|---|---|
| 拡張子 | .anm .obj .scn .tra .cam | .anm2 .obj2 だけ（`aviutl_is`） | .scn/.tra/.cam は 16 包 |
| @ファイル（1 本に何本も入れる） | あり | あり（同じ書き方） | 36 包 |
| 文字コード | **Shift-JIS**（531/581 本。UTF-8 は 3 本） | UTF-8 として読む | 82 包 |
| SJIS の 2 バイト目が 0x5C の字（表・ソ・能など） | 本体が実行の前にエスケープする（lua.txt 0.92「0x5cをエスケープ処理してから実行」） | なし | 48 包 |
| 大きさの上限 | 約 32KB（超える物は dofile で分ける） | なし | 32KB 超は 8 包 |
| 頭書き | `--track0:`〜`--track3:`、`--check0:`、**`--dialog:名前/種類,変数=値;…`**、`--color:`、`--file:`、`--param:`、.tra の `--twopoint` `--speed` | `--track0:` と `--check0:` だけ読める。`--dialog` `--color:` `--file:` `--param` は読めない | `--dialog` 82、`--check0` 56、`--color:`/`--file:`/`--param` 13 |

文字の扱いについて：Lua の中は SJIS のバイトのままにして、obj に渡す所（効果名、図形名、`load("text")`、フォント名、ファイル名）でだけ UTF-8 に直すのがよい。AviUtl2 自身も「lua内部はSJIS扱い」と書いている（lua.txt の require の項）。前もって UTF-8 に直すと、`#s` や `string.sub` で文字を切るスクリプトの結果が変わる（今回は 1 包で見た）。

### Lua

- AviUtl1 本体の Lua は lua51.dll（2010 年、Lua 5.1.4）。**スクリプトファイルでは io・os もそのまま使える**（AviUtl1 は何も外していない）。
- ePi の「LuaJIT 化」は lua51.dll を LuaJIT と入れ替える物。ffi を使う 7 包、bit を使う 4 包はこれが前提。今の変換器の LuaJIT 2.1 で足りる。元の配布ページ（ePi の scrapbox）は閉じた。今は Per-Terra/LuaJIT-Auto-Builds が引き継いでいる。
- 有名 83 包で使う数：
  - `io.*` 9（ほとんどが io.open で設定やデータを読む）
  - `os.*` 3（clock・time・date。os.execute と os.remove も各 1 本ある）
  - dofile / loadstring 1
  - 横に置いた .lua を require する物 29（easing.lua、extbuffer.lua、noise など）
  - package.loadlib 1（curve_editor.auf を直に読む）
- **ffi で Windows の中身を触る物は無理**。例：InlineScene_S は `ffi.C.GetModuleHandleA` で aviutl.exe のメモリを直に読み、版を確かめている。

### obj の違い（0.92 の lua.txt と AviUtl2 の lua.txt を並べた）

**AviUtl1 にだけある物、または振る舞いが違う物**

| 物 | AviUtl1（0.92） | AviUtl2 | 有名 83 包 |
|---|---|---|---|
| `obj.getpixeldata([opt])` | 引数なしで、今のオブジェクトを **BGRA 8bit（α は乗算しない）** で返す。`"work"` `"alloc"` もある | `getpixeldata(target[,format])`。target が要る | 23 包（work/alloc は 4） |
| `obj.putpixeldata(data)` | 引数は 1 つ | `putpixeldata(target,data,w,h[,format])` | 同上 |
| `obj.pixeloption("get"/"put", "obj"/"frm")` | 名前は obj・frm。frm には α が無い | "object"・"framebuffer" | 8 包 |
| `obj.getpixel(x,y,"yc")` と pixeloption "yc" | 中の形式 YC48 のまま | 「YCbCr旧内部形式」として残る | 5 包 |
| `obj.setoption("antialias")`、`("shadow")` | あり | 無い（ported_tim も「削除」と書く） | 17 包 |
| `obj.filter(name,…)` | あり | 無い | 0 包 |
| `obj.effect("名前","項目",値…)` | 項目名は 0.92 の名前（"color"、"type" など） | effect.conf の `[OldScript.*]` で読み替える | 54 包・39 種 |
| 合成モード | 0〜9 と alpha_add / alpha_max / alpha_sub / alpha_add2 | 16 種（数での指定も読み替える） | 0/1 以外を使う物 46 包 |
| `obj.zoom` / `obj.aspect` | — | 「旧スクリプトファイルの利用かで算出反映処理が異なります」 | — |
| 中の画 | YC48（YCbCr 12bit 相当）＋α。フレームバッファには α が無い | RGBA16F・乗算済み | — |
| 本体の Lua | Lua 5.1.4 | 旧ファイルでは Lua 5.1、新しいファイルでは LuaJIT | — |

**今の変換器に無い物（AviUtl1 で使われる順）**

| 関数など | 有名 83 包 | 今の変換器 |
|---|---|---|
| `obj.effect` の本物（ぼかし・単色化・反転・領域拡張・クリッピング・リサイズ・斜めクリッピング・グラデーション・色調補正・マスク…39 種） | 54 | 何もしない時と、領域拡張の余白だけ |
| `drawpoly` と 3D（rx/ry/oz、`camera_param`、billboard） | 40（camera_param は 10） | 無い（`obj.draw` は平面だけ） |
| `rand` | 26 | **無い**（`obj.rand` は not yet になる） |
| 図形の「四角形」以外（円 53 本、背景、三角形、五角形、六角形、星型） | 24 | 四角形だけ |
| getpixeldata / putpixeldata | 23 | 無い |
| `load("text")` / `setfont` | 20 | setfont は何もしない |
| getpixel(x,y) / putpixel | 19 | 大きさを返すだけ |
| `load("image"/"movie"/"layer")` | 12 | 無い |
| `interpolation` | 12 | 無い |
| copybuffer の frm・image | 8 | 止まる |
| `HSV` | 一部 | error で止まる |
| `getaudio` | 4 | 無い |
| `getpoint`（.tra） | .tra の全部 | 無い |

### .scn / .tra / .cam

Motolii にはまだ受け口が無い：
- .scn はシーンチェンジ（2 つの絵の間の切り替え）。
- .tra はトラックバーの動き方（イージング）。
- .cam はカメラ効果。

どれも Motolii の「切り替え」「動き方」「カメラ」にどう当てるかを先に決める必要がある。有名 83 包のうち 16 包が持つ。

### DLL

- AviUtl1 の DLL はすべて **32bit の Windows 用**。Mac では動かない。64bit 版の Windows Motolii のプロセスにも入らない。
- **閉じた DLL（元ソースなし）**：
  - rikky 製：rikky_module、rikky_memory、particle_set3、dimensionize、irozure_ri、oekakifunc、hagikora、rikky_separate、dithering、LensDistortion。Aodaruma が同梱する AviutlLongShadow と gradationmapping も rikky 製。rikky_module の中には AGPL-3 の線検出の部品が 1 本入っているが、本体のソースは無い。
  - ティム：T_*_Module 15 個、MMD2AviUtl。
  - その他：extbuffer_core（ゆうき、配布元も消えた）、uf_opencv（UndoFish）、SharpBorder（微熱）、astroid（mimaraka）、ease.dll（ePi）、ocv_module（てつ）、xCameraAssist、NagoEffect。
- **元ソースのある DLL**：SEED264 の 4 個（DirectionalBlur_s は NVIDIA の閉じた cudart32_75.dll も要る）、GLShaderKit（karoterra、MIT）、aulut、Modulator・UVmapping・ImageFFT（Aodaruma）、MotionBlur_K、Bevel_And_Emboss_M、ImageLoader_M、unmult_core、textmodule、PSDToolKit、Extram（oov）、auls の物（zip に C++ がある）。
- **数（有名 83 包・287 本）**：閉じた DLL が要る物は 21 包・39 本（14%）、元ソースのある DLL が要る物は 6 包・11 本（4%）。
- `pcall(require,"rikky_module")` の形で、rikky_module が無くても止まらない作りの物は、これとは別にある。調べた全体で 32 か所あり、無いとダイアログなどが劣る。
- **GLShaderKit を使う物は、中に GLSL の文を持っている**（Mr-Ojii、sigma-axis、azurite の物）。GLShaderKit の API（MIT、元ソースあり）を Motolii 側で真似れば、isf.cpp の GLSL→WGSL の道に乗る見込みがある。

## 5. AviUtl2 本体は AviUtl1 のスクリプトを動かすか

**公式に「一部だけ」動かす。** KENくんの文書（AviUtl2 v2.1.15 の zip に同梱）から：

- lua.txt 冒頭：「旧スクリプトファイル(*.anm,*.obj,*.cam,*.scn,*.tra)も利用できますが、一部の機能が非対応」。
- lua.txt 注意点：「旧スクリプトファイルはSJIS」「旧スクリプトファイル形式と一部仕様が異なる場合があります。※pixel入出力系の引数等」。
- 頭書き：`--track0:` `--check0:` `--color:` `--file:` `--dialog` `--param` は「互換対応」。旧ファイルの Lua は LuaJIT ではなく Lua。
- aviutl2.txt の「互換性」：「以前の拡張編集のスクリプトに互換対応していますが、一部機能に対応していません」「以前の32bitDLLのプラグインやDLLを同梱しているスクリプトは基本的に利用できません」。
- effect.conf：`obj.effect` の古い項目名を新しい名前に読み替える表（`[OldScript.単色化] color=色` など）。合成モード・移動モード・アニメーション効果の番号の読み替えもある。

だから AviUtl2 は「AviUtl1 の物を読む時の正解の 1 つ」として使える。ただし AviUtl2 本体と effect.conf は「再配布はご遠慮ください」で、写さずに振る舞いを見るだけにする。

## 6. 見積もり：何を足せば何 % 動くか（小さい順）

有名 83 包・287 本を、足す物ごとに数えた。どの段も、その前の段がそろっている前提。閉じた DLL・元ありの DLL が要る物は、最後まで数に入らない。

| 段 | 足す物 | 大きさ | 動く本数（287 本中） | 包（83 中） |
|---|---|---|---|---|
| 1 | **読み込み**：.anm/.obj/.scn/.tra/.cam、SJIS（バイトのまま、obj に渡す所で直す）、`--dialog` `--color:` `--file:` `--param`、0x5C の扱い | 小 | 12（4%） | 3 |
| 2 | **小さな obj**：rand（同じ時刻で同じ値。seed の正負の決まりもある）、interpolation、HSV、getpoint、図形の円・三角形・五角形・六角形・星型・背景、copybuffer の全部の組み合わせ（frm・image を含む）、合成モード 0〜9 と alpha_*、antialias と shadow は受け流す | 小 | 29（10%） | 4 |
| 3 | **Lua の周り**：io.open は読むだけ（スクリプトのフォルダと利用者が選んだファイル）、os.clock・time・date、dofile と横の .lua の require、ffi・bit（LuaJIT にもうある） | 小 | 41（14%） | 4 |
| 4 | **立体**：drawpoly（4 点と uv）、draw の rx/ry/oz、AviUtl のカメラ（camera_param、billboard、culling）と Filament のカメラの対応 | 大 | 103（36%） | 14 |
| 5 | **本体の効果を名前で**：39 種。多い順に 反転 25、単色化 21、領域拡張 20、ぼかし 18、クリッピング 15、リサイズ 15、斜めクリッピング 13、グラデーション 12、色調補正 11、マスク 9 …。上位 10 種だけで足りる包は 17 | 大 | 149（52%） | 32 |
| 6 | **文字・画像・動画・音**：load("text")・setfont（字体と装飾 0〜4）、load("image"/"movie"/"layer")、getaudio | 中 | 196（68%） | 41 |
| 7 | **画素を CPU で**：getpixel(x,y)・putpixel・copypixel・pixeloption、getpixeldata／putpixeldata（BGRA 8bit・α は乗算しない・"work"/"alloc"、ffi で読める生のポインタ） | 中 | **237（83%）＝ 天井** | **56（67%）** |

- 一番効くのは 4（立体）。「立体だけが足りない」という本が 62 本ある。立体を抜いて他を全部そろえても 76 本（26%）にしかならない。
- 「動く」は「止まらずに絵が出る」こと。AviUtl1 と同じ絵になるかは別の話。中の色が YC48 から RGBA16F に変わり、効果・合成・文字の出方も変わるので、見比べながら合わせる手間がかかる。

### 無理な物

- **閉じた DLL が要る物：21 包・39 本（14%）。** 例：
  - さつき一式の一部
  - 色調調整セット・縁取りT・カメレオン・回転ブラー・フィルターセット・虹色グラデーション・簡易モーフィング・ひび割れガラス・燃焼・ライン抽出（ティム）
  - 拡張パーティクル(R)・パーツ分割・ディザリング・モザイクマスク（rikky）
  - 文字輪郭追跡（UndoFish）
  - FlatShadow・GradationMap・キャッシュRM（Aodaruma。中の DLL は rikky 製）
  - GetColor・DelayMove（93）
- **Windows の中身を触る物**：InlineScene_S（ffi で aviutl.exe を読む）、patch.aul が前提の物。
- 元ありの DLL（6 包・11 本）は「そのまま」では無理。ただし作り直せば動く：SEED264 の 3 個、aulut、Modulator、MotionBlur_K。

### 別の道：AviUtl2 向けの移植を使う

- **ported_tim**（sevenc-nanashi、MIT、2026-09-01 更新）：
  - ティムの 110 本・226 効果をすべて AviUtl2 に移している（TASKLIST で 226 個すべて動作確認済み）。DLL を使う 73 効果は Rust で作り直してある。
  - README に「配布の許可も出してくださったティム様に感謝します」とある。
  - 今の AviUtl2 変換器で動かせば、ティムの物は **同梱できる**。閉じた T_*_Module の壁も消える。必要なのは tim2.mod2 を Mac で組み直すことで、AviUtl2 の下調べ（aviutl2_converter.md の 11 章）で「作り直す」と判定済み。
  - この道を使うと、有名どころで動かない包は 27 から 17 に減る。本数の天井は 83% から **87%**（287 本中 249 本）に上がる。
- **ported_satsuki**（さつき一式 233 個）：許諾は「未定」なので、今は使えない。
- **rikky_modoki.aux2**（sevenc-nanashi、MIT）：AviUtl2 向けの rikky_module のそっくりさん。中身は見ていない。

## 7. 借りられる見本（先に読む物）

- **0.92 の lua.txt**（exedit92.zip に同梱）：AviUtl1 の obj の仕様そのもの。969 行。
- **AviUtl2 の lua.txt と effect.conf**：旧スクリプトをどう読み替えるかの正解（写さず、見るだけ）。
- **apm（MIT のアプリ）**：作者のページから取って、ハッシュを確かめて置く流れ。
- **apm-data**（CC BY-NC-SA）：形の見本。中身を写すなら Team apm に聞く。
- **ported_tim（MIT）**：ティムの DLL を Rust で作り直した物。T_*_Module が何をしていたかの答えでもある。
- **GLShaderKit（MIT）**：OpenGL の DLL の API。真似れば GLSL を持つスクリプトが動く。
- **sashimono-edit（MIT）**：AviUtl の obj を Python で真似る物。本体の効果の名前の読み替えがある（AviUtl2 の下調べで既出）。

## 8. 手元に残した物（scratchpad。実行していない）

- 本体：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/exedit/`（exedit92.zip、aviutl110.zip、UTF-8 に直した lua92.txt）
- apm の目録：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/apm-data/`
- ティムの 95 包（zip と展開、再生数 views.tsv）：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/tim/`
- ほかの作者の物：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/web/`（rikky、さつき、UndoFish ほか）、`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/cloud/`（93、gometh、GitHub の物）
- 記録：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1/records/`
  - web.json・cloud.json：包ごとの URL・許諾の文言・DLL
  - popularity.json・popularity_sources.json：人気の元
  - packages_scanned.json：234 包、中身を数えた物
  - famous_tiers.json：有名どころの要る段
- 数えた道具（Python）：`/private/tmp/claude-501/-Users-motolii-maker-motolii/b7181a2a-2e86-4cec-b8cc-9c0117b390ea/scratchpad/au1tools/`（scan1.py、merge.py、famous.py、tiers.py）
