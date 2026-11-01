# Blender で一番売れている簡単な拡張（11/1）

ユーザーの問い：「今、ブレンダーで一番売れてる簡単プラグインプリセット集みたいなのって何があるか調べてみて。MV でブレンダー使ってる人がいちいち最初からモデリングとかしてるわけないからな、動きとかも」。

## 結論

- 店で売れているのは、人形に動きを移す道具と、出来合いの素材の棚。MV の人に絞ると Direktor 3D のような専用の棚があり、After Effects か AviUtl で仕上げている。
- 1 位は Auto-Rig Pro で 70,700 本超。動きのプリセットそのものではなく、動きを別の人形へ移せる。
- 文字や図形の動きだけのプリセット集は桁が小さい。MoGraph Toolbox が 800 本超、CinePack が 3,900 本超。
- 売っている形はどれも同じ。絵の付いた棚から 1 手で入れて、数字で直す。Motolii の Look のダブルクリックと同じ形で、足りないのは棚の中身。
- ユーザーの見立ては半分当たり。出来合いは確かに売れる。ただし Hard Ops、MACHIN3tools、MESHmachine のようなモデリングの道具も上位にいる。Blender の買い手全体の話で、MV の人だけの数字ではない。

## 売れた本数

Superhive（旧 Blender Market）の商品ページに出ている数字。売上順の一覧は 11/1 に読んだ。

| 物 | 売れた本数 | 値段 | 何を買っているか | 手数 | 順位 |
|---|---|---|---|---|---|
| Auto-Rig Pro | 70,700+ | $25〜 | 骨を入れる。Full は BVH や FBX の動きを移す | 道具 | 1 |
| Botaniq Trees | 31,300+ | $1.99〜 | 木と草 1300 超、地面ごとの組 185 | 棚から出す | 4 |
| Flip Fluids | 18,900+ | $79〜 | 液体の計算 | 計算が要る | 7 |
| Poly Haven Asset Browser | 18,800+ | $49 | 2000 超の HDRI、材質、模型 | ドラッグ | 8 |
| Geo-Scatter | 17,800+ | $99〜 | 面に物を散らす | 1 クリック | 11 |
| Procedural Crowds | 14,300+ | $18〜 | 群衆。スキャンした人 50 超、立つ・歩く・歓声 | 種類を選ぶ | 17 |
| CinePack | 3,900+ | $30 | 動き付きカメラ 123 超、手持ちの揺れ | 1 クリック | 上位 24 の外 |
| Dynamic VFX Elemental | 2,300+（無料込み 27,000+） | $0〜 | 火、電気、天気、粒子 | ドラッグ | 上位 24 の外 |
| MoGraph Toolbox | 800+（無料込み 3,700+） | $0〜 | 複製と減衰のプリセット 20 超 | 道具 | 上位 24 の外 |

本数が読めなかった物：Quick Motion（$16、文字の動きなど 56）、Motion Animate（$45、文字 40）、Dead Simple Motion（$1〜）。Khaos（爆発）と Sanctus Library（材質）はページを開けなかった。GeoMotion は 10 本ほど。

面白いのは Poly Haven。素材は CC0 で誰でも無料なのに、棚にしただけの物が 18,800 本超売れている。売れているのは素材より「探して入れる手間が無い事」。

## MV を作る人が見せている物

ユーザーの言葉：「あいつらって自己顕示欲の塊だからさ、もっと見えやすいというか、人が多いところで公開してんだろう」。その通りで、YouTube に山ほどあった。

YouTube を 15 語で検索して、題名が MV のメイキングや解説らしい動画 49 本を集め、説明欄の道具名を機械で数えた（11/1）。

| 説明欄に出た名前 | 本数 |
|---|---|
| After Effects | 13 |
| AviUtl | 6 |
| DaVinci Resolve | 4 |
| AI の道具 | 4 |
| BOOTH | 3 |
| Superhive | 2 |
| Direktor 3D | 2 |
| Geometry Nodes | 2 |
| Mixamo、Rokoko、VRoid、Sketchfab | 各 1 |

見えた事：

- MV 専用の棚は既にある。Max Novak の **Direktor 3D**（$77）は、舞台 14、物 100 超、効果 14、文字の型 12、灯りの組 20、ダンス 21。M.V.P Music Visualizer Pro（$11.19）は、絵の型 68、配色 96、カメラ 20、音に反応、1 クリックで舞台。Motolii の棚の手本はこの 2 つ。
- Max Novak の動画は「5 EASY Ways」22.6 万回、「Animate Your Music in 5 Clicks」19 万回。音で動かす話が大きい。
- 仕上げは別のソフト。After Effects と AviUtl が上位で、Blender の動画も最後は戻っている。原悠和は Blender の MV でも文字だけ After Effects と語っている。
- 道具の買い物リストを見せる人は少ない。見せているのは売る側。Scenefiller の Grainy、MattePainter、Lens Sim、Botaniq、ToOptimize Tools は紹介料の付く物が混じる。Max Novak は自分の店。

人によって型が分かれる：

| 型 | 例 | 作り方 | 再生 |
|---|---|---|---|
| ラップの 3D 映像 | Max Novak、salfolio | 人形のキット、Mixamo の動き、魚眼、被写界深度、カメラ追跡、After Effects | 6 万〜25 万 |
| ボカロと文字 PV | 藍瀬まなみ、Coloso の講座、盆地 | After Effects か AviUtl が主。Blender は一部 | 1 万〜41 万 |
| 実写合成 | 涌井嶺 | 緑の幕で撮って全背景を CG。遠景だけ買い、目立つ所は自作。写真を板に貼って切る | VFX-JAPAN の優秀賞 |
| VTuber | 海月とうみ、月森蝶華 | BOOTH のモデル、VRM Live Viewer と背景効果の設定集 | 16〜10.8 万 |
| 商業 | サンジゲンの宝鐘マリン MV | モーションキャプチャを手付けで直す。Grease Pencil、質感は撮影で作る | — |

「最初からモデリングしない」は型による。ラップと VTuber は出来合いが中心。涌井嶺は目立つ所を自分で作る。

## デザインが動く映像（Stack）

ユーザーは Codex との話で、MV からモーグラ、VJ を経て、Kimathi Flavien の『Stack』の型に行き着いた（11/1）。Codex の話は本人の記事で確かめた。

本人の記事で確かめられた事：

- 2D で 4 年働いた人の初めての 3D。この作品のために Cinema 4D を覚えた。
- Cinema 4D で書き出して After Effects で混ぜた。記事にプラグインの名前は出ていない。
- 少ない形からできるだけ多くの場面を作る。1 日 1 場面以上。つなぎは形、動き、音のマッチカット。
- 3D を 2D に合わせるため、主に 200mm のレンズで奥行きを潰した。
- メトロノームに合わせて切った。
- 一番の苦労は、途中確認の絵が無く、After Effects と遅い書き出しを何度も往復した事。

読み取り：足りないのはプラグインより、2D と 3D の往復を無くす事。Motolii は 1 つの窓で両方を見られるので、この苦労が最初から無い。ユーザーの「元から 2D は存在していなくて、z が 0 なだけ」と同じ話。

Codex の話で確かめられなかった事：本人が Signal や Ricochet を使った証拠は無い。Codex 自身もそう言っている。

Signal は loop、ease、noise、flicker・bounce・vibration・spin の型、Fields、BPM。Motolii のふるまい、区間イージング、BPM、減衰と大きく重なる。別物として足す必要は薄い。

Stack の作り方で次に測る事：真似の 1 カットを窓で作り、何が足りないかを手数で見る。作品の絵と音は使わず、手法だけ真似る。

## ポチポチで作れる道具は界隈にあるか

ユーザーの問い：「大事な話はそうじゃない、簡単にポチポチ作れるかよ。そんでその道具はもう界隈では知られてるんじゃね？」。

ある。ただし 2D か 3D のどちらかに寄っている。

| 道具 | 場所 | ポチポチの中身 | 3D | 値段 |
|---|---|---|---|---|
| Jitter | ブラウザ | 動き、消え方、大きさ、マスク、ぼかしの型。テンプレート 300 超。キーの代わりに簡単な操作 | 説明に無い | 無料から |
| Cavalry | Mac、Win | 型ではなく仕組み。複製と減衰、1 つの値で全体が変わる。書き出し待ちが無い | 2D | 個人は無料 |
| Motion Bro | After Effects の中 | 型の棚と店。本体は無料。作者の言う利用者 100 万超 | After Effects 次第 | 無料と有料の型 |
| Animation Composer | After Effects の中 | 型の棚。School of Motion が紹介 | After Effects 次第 | 無料と有料 |
| Motion Studio | After Effects の中 | 道具 80 超。1 クリックの小道具が中心で、型の棚ではない | — | 値段は未読 |
| Apple Motion | Mac | ふるまいをドラッグするとキー無しで動く。3D の文字、複製 | 有る | 約 $50（古い情報） |
| Spline | ブラウザ | 形、材質、灯り、状態、物理。Web に載せる道具 | 有る | 無料から |

見えた事：

- 2D のデザインモーションは Jitter と Cavalry が知られている。どちらも説明に 3D が無い。
- 一番近い席は Apple Motion。2D と 3D が 1 つの窓で、ふるまいで動く。ただし Mac だけ。
- Spline は 3D だが、ページに動画の書き出しと音の話が無い。
- 探した範囲で、Stack の型（2D と 3D を混ぜて、音に合わせて切る）のための Win でも動くポチポチの道具は見つからなかった。

Motolii が取れそうな席は、Apple Motion の Win 版のような所。ふるまいと 3D の文字はもうある。

分からない事：どの道具も実際に触っていない。手数は読んだ説明からで、比べていない。売れた本数と利用者数は Motion Bro の自己申告だけ。aescripts の売上順は 403 で読めず、School of Motion の業界調査も見つからなかった。Reddit は写しのサイト経由で、一部だけ。Apple Motion の値段は古い情報。

## Motolii にある物と無い物

| 売れ筋 | Motolii の今 | 差 |
|---|---|---|
| カメラの動き（CinePack） | カメラ、ギズモ、区間イージング、Auto-Shake | 無し。棚にしても名前が増えるだけ |
| 素材の棚（Poly Haven） | `room` に 7 部屋。.gltf は読める | 中。読めるが棚に無い |
| 人の動き（Auto-Rig Pro、Mixamo） | .glb に入った動きは流せる（gltfio の Animator） | 大。別の人形へ移す口が無い。FBX も BVH も読まない |
| 火、電気、天気（Elemental） | 名前で探して Appear Rain、Smoke Screen、am_clouds、am_lightning、am_electricedges | 中。雨や雷はある。火と爆発は名前では見つからない |
| 散らす（Geo-Scatter） | Scatter Repeat と Scatter Field | 中。今は層を平面に散らす。3D の面に生やす物は無い |
| 群衆（Procedural Crowds） | 無い | 大。動きの移し替えが先 |
| 液体（Flip Fluids） | 無い | 大。出来た絵や動画を使う方が早い |
| 文字の動き（Quick Motion） | ふるまいと Look が有る | 小。売れ方も小さい |

## 順番

店の本数と YouTube を合わせて直した。
カメラの動きは外した。ユーザー：「区間イージングでどうにかできる話」。

1. 音で動かす。BPM と音の打点は有るが、音の大きさで物を動かす口は検索で見つからない。BeatBlender と M.V.P が持っている。
2. 舞台と灯りの組の棚。Poly Haven の素材と、Direktor 3D の舞台 14・灯り 20 が見本。
3. 人形と踊り。Mixamo の FBX と、日本の VRM、MMD の動きを読む口。VRM、VMD、BVH は検索で見つからない。一番重い。
4. 3D のトゥーン、カメラ追跡、火と爆発、面に散らす。

文字は After Effects に残っている所で、Motolii はここを抱えている。別のソフトへ戻らなくて済むのが、Blender に無い強み。

## 分からない事

- YouTube の 49 本は検索の上位で、全体の見本ではない。書いていない道具は数えられない。字幕は取れず、説明欄と題名だけ。
- 「ひっひっふー」の Hurray! の講演（Blender だけで編集まで）は案内の検索結果だけで、ページは 503 で読めなかった。CGWORLD の涌井嶺のインタビューも本文は読めず、検索の抜粋。
- 宝鐘マリン MV の話は聴いた人のメモで、本人の資料ではない。
- X は見ていない。Direktor 3D の売れた本数は載っていない。
- 本数は「その店で」の数字。Blender の無料の拡張の棚と、Gumroad などの他の店は入っていない。
- 本数には発売からの長さが効く。週間の順位でも Auto-Rig Pro は 1 位だったので、今も強いとは言える。
- Mixamo の条件は Adobe の FAQ の検索結果で読んだ。ページ本体は読めず（403）、書き出しの形式と動きの本数は確かめていない。

## 出典

- [Superhive の売上順](https://superhivemarket.com/products?sort_sales=desc)
- [Add-ons の売上順](https://superhivemarket.com/categories/addons?sort=sales)
- [Presets の売上順](https://superhivemarket.com/categories/presets/browse?sort_sales=desc)
- [Auto-Rig Pro](https://superhivemarket.com/products/auto-rig-pro)
- [Botaniq Trees](https://superhivemarket.com/products/botaniq-trees)
- [Flip Fluids](https://superhivemarket.com/products/flipfluids)
- [Poly Haven Asset Browser](https://superhivemarket.com/products/poly-haven-asset-browser)
- [Poly Haven の許諾](https://polyhaven.com/license)
- [Geo-Scatter](https://superhivemarket.com/products/scatter)
- [Procedural Crowds](https://superhivemarket.com/products/procedural-crowds)
- [CinePack](https://superhivemarket.com/products/cinepack-pre-animated-camera-moves)
- [Dynamic VFX Elemental Asset Pack](https://superhivemarket.com/products/blender-dynamic-vfx---elemental-asset-pack)
- [MoGraph Toolbox](https://superhivemarket.com/products/blender-mograph-toolbox)
- [Quick Motion](https://superhivemarket.com/products/quick-motion)
- [Dead Simple Motion](https://superhivemarket.com/products/dead-simple-motion)
- [Motion Animate](https://blendermarket.com/products/motion-animate---speed-up-motion-graphics/docs)
- [Mixamo FAQ](https://helpx.adobe.com/en/creative-cloud/faq/mixamo-faq.html)
- [Mixamo から Blender への手順（Cinevva）](https://app.cinevva.com/guides/mixamo-to-blender-2026)
- [CGWORLD の Blender 連載](https://cgworld.jp/regular/blender/index.html)
- [Stack の制作解説（Motionographer）](https://motionographer.com/2025/02/12/exclusive-kimathi-flavien-the-work-behind-of-chef-doeuvre-motion-design-film-stack/)
- [Stack の紹介（STASH）](https://www.stashmedia.tv/kimathi-flavien-solves-the-design-puzzle-in-stack-short-film/)
- [Signal](https://greyscalegorilla.com/plugins/signal)
- [What is Jitter](https://help.jitter.video/en/articles/12089209-what-is-jitter)
- [Jitter と After Effects](https://jitter.video/after-effects-alternative/)
- [Cavalry](https://cavalry.studio/)
- [Motion Bro](https://motionbro.com/plugin/)
- [Motion Studio](https://mtmograph.com/products/motion)
- [School of Motion の 2026 年のプラグイン](https://www.schoolofmotion.com/blog/best-after-effects-plugins-and-effect-packs-you-need-in-2026)
- [Spline](https://spline.design/)
- [Apple Motion の説明書](https://support.apple.com/guide/motion/motnb0278e80/mac)
- [Motion（Wikipedia）](https://en.wikipedia.org/wiki/Motion_(software))
- [Rocket Lasso の棚](https://rocketlasso.com/shop)
- [Direktor 3D](https://mediamonopoly.co/collections/video-presets-plugins/products/direktor-3d-ultimate-blender-plugin-for-creators)
- [Media Monopoly の店](https://mediamonopoly.co)
- [M.V.P Music Visualizer Pro](https://superhivemarket.com/products/mv)
- [5 EASY Ways To Create Music Videos with BLENDER](https://www.youtube.com/watch?v=hgtBeWa5esY)
- [Animate Your Music in 5 Clicks](https://www.youtube.com/watch?v=IQ3ABnImQvI)
- [Top 10 Music Video Effects in Blender](https://www.youtube.com/watch?v=nXtJDXoEhx8)
- [Creating 3D Music Visuals in Blender（salfolio）](https://www.youtube.com/watch?v=gLEtSbqYtGY)
- [Music Videos in Blender（Scenefiller）](https://www.youtube.com/watch?v=gmp-zfvprS0)
- [ボーカロイドの MV が出来るまで（藍瀬まなみ）](https://www.youtube.com/watch?v=2ttDoIA3uJU)
- [VTuber が自分で MV つくってみた（海月とうみ）](https://www.youtube.com/watch?v=aFkKIJyxv5U)
- [After Effects の代わりに Blender で作った MV（80 Level）](https://80.lv/articles/see-how-this-music-video-was-made-in-blender-instead-of-after-effects)
- [涌井嶺のバーチャル MV（ねとらぼ）](https://nlab.itmedia.co.jp/nl/articles/2106/12/news030.html)
- [涌井嶺のインタビュー（CGWORLD）](https://cgworld.jp/interview/202107-the6thlie-mv-2.html)
- [「ひっひっふー」の講演（CGWORLD）](https://cgworld.jp/special/blenderfes/vol5/?p=940)
- [宝鐘マリン MV の聴講メモ](https://scrapbox.io/motoso/%E3%80%90%E3%82%B5%E3%83%B3%E3%82%B8%E3%82%B2%E3%83%B3%E3%80%91%E5%AE%9D%E9%90%98%E3%83%9E%E3%83%AA%E3%83%B3MV%E5%88%B6%E4%BD%9C%E3%81%AE%E8%A3%8F%E5%81%B4%EF%BC%9ACG%E5%88%B6%E4%BD%9C%E3%81%AE%E3%82%B9%E3%83%86%E3%83%83%E3%83%97%E3%82%92%E5%AD%A6%E3%81%BC%E3%81%86%EF%BC%81%E3%80%90%E3%81%82%E3%81%AB%E3%81%A4%E3%81%8F2024%E3%80%91)
- [VRM Live Viewer](https://booth.pm/ja/items/1783082)
- [VRM Live Viewer の背景効果設定集](https://booth.pm/ja/items/3469388)
