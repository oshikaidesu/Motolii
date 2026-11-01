# Blender で一番売れている簡単な拡張（11/1）

ユーザーの問い：「今、ブレンダーで一番売れてる簡単プラグインプリセット集みたいなのって何があるか調べてみて。MV でブレンダー使ってる人がいちいち最初からモデリングとかしてるわけないからな、動きとかも」。

## 結論

- MV の人が Blender へ行く話は 2 段に分かれる。3D が要るのは作品で見える。C4D でなく Blender を選ぶのはお金の説明が筋が通る（ユーザー：「c4dでもう既にいいのに金かかりすぎるからかも」）。「世界の中で撮りたい」は本人の言葉で、格好をつけた分は切り分けられない。文字と仕上げは After Effects に残る。
- 店で売れているのは、人形に動きを移す道具と、出来合いの素材の棚。MV の人に絞ると Direktor 3D のような専用の棚があり、After Effects か AviUtl で仕上げている。
- 1 位は Auto-Rig Pro で 70,700 本超。動きのプリセットそのものではなく、動きを別の人形へ移せる。
- 文字や図形の動きだけのプリセット集は桁が小さい。MoGraph Toolbox が 800 本超、CinePack が 3,900 本超。
- 売っている形はどれも同じ。絵の付いた棚から 1 手で入れて、数字で直す。Motolii の Look のダブルクリックと同じ形で、足りないのは棚の中身。
- ユーザーの見立ては半分当たり。出来合いは確かに売れる。ただし Hard Ops、MACHIN3tools、MESHmachine のようなモデリングの道具も上位にいる。Blender の買い手全体の話で、MV の人だけの数字ではない。

## なぜ MV を作る人は Blender へ行くのか

ユーザーの問い：「そこらへんはもう奪ってるだろ」「blenderに行く？なぜmv製作者は」。文字、BPM、型の棚の話ではなく、After Effects の人がわざわざ 3D のソフトへ行く理由を、本人の言葉で読んだ。

読んだ物：note の 14 本（作り手本人と講座の受講者。本文から関係する行を抜いて読んだ）。YouTube 16 本の説明欄と 1 ページ目のコメント 313 件。Blender Artists の議論 2 本（各 20 投稿まで）と MV の投稿 3 本。BCON（Blender の公式会議）の講演 4 本（MV 3 本、CM 1 本）の説明欄とコメント。ビデオサロンの講座のページ。

### 行く理由

| 理由 | 誰が言っているか | 中身 |
|---|---|---|
| 世界の中で撮りたい | 涌井嶺（note のスキ 707）、tamf_kazuki、無印かげひと | 涌井嶺は After Effects と Element 3D では物を並べるだけだった。Blender では人以外すべて 3D にして、影は灯りを当てて作った。tamf_kazuki は三脚で撮った素材を世界に置いて、カメラを後から付けた。かげひとはビル群の中に絵を置いて撮れる事を挙げる |
| 合成に見せたくない | 涌井嶺 | バンドに合成っぽいのは嫌だと言われた。世界の灯りで素材を馴染ませて消す |
| 絵コンテから 1 本で済ませたい | ぽぷりか（Hurray!） | Clip Studio と After Effects でやっていたコンテを Grease Pencil へ移したら快適になった。ヨルシカの MV、映画『数分間のエールを』も Blender が主。しぐれうい「ひっひっふー」（781 万回）は全編 Eevee |
| アニメ調の人形 | ぽぷりか、revirytic、かげひと | トゥーンを安く作れるとかげひとが書く。ぽぷりかの NPR 講座、セルルックの MV 制作ノート。Eevee の NPR の講演は 76,000 回再生 |
| 布、流体、物理 | hama | 布と流体は Blender、ボールは After Effects。レンダリング 24 時間でも使う |
| 待たずに見られる | tamf_kazuki（全編 Eevee）、涌井嶺 | 軽く作れば Eevee でその場で動く |
| お金 | Blender Artists、Karon の動画のコメント（いいね 74）、涌井嶺、revirytic、かげひと | Cinema 4D は年 $839、Blender は 0。Maxon のサブスク移行と教育版の打ち切りで離れた人がいる。涌井嶺は有料の拡張無しを売りに書き、revirytic は有料の Pencil+4 を標準機能で置き換え、かげひとはコスパと書く。海月とうみの MV へのコメント「こういうのやりてぇわ……C4Dか……」はいいね 216 |

英語圏のラップの MV も形は近い。Max Novak の動画は再生 7 万〜22 万回で、説明欄は緑バックの合成、カメラ追跡、3D の人形の話。説明欄しか読めていない。

### 行かない物、戻る物

| 物 | 証拠 |
|---|---|
| 文字 | 原悠和は Blender の MV でも文字は After Effects。Ducky 3D のコメント：Blender の問題は文字。BYOL のコメント（いいね 124）：2D の文字と図形は After Effects の方が楽そう |
| 緑抜き | 涌井嶺は After Effects、tamf_kazuki は DaVinci |
| 仕上げ | 涌井嶺は白の調整、フレア、粒を After Effects で。tamf_kazuki は DaVinci で色 |
| 往復の苦しさ | hama は 120fps で書き出して After Effects で詰めた。涌井嶺は EXR が After Effects で重くエラーも出た。darenoki は BlenderAe でもモデルは渡せず、人の前後で分けて書き出す要があると書く |
| レンダリング待ち | 涌井嶺は平均 3〜4 分/コマ。Tarahese は自分の PC では 2 年かかると書く |

読み取り（ユーザーの指摘で直した。前の版は「一番の理由は世界」と書いたが、本人の言葉を数えただけだった）：3D が要るのは作品で見える。Blender を選ぶのはお金の説明が筋が通る。Motionographer では Cinema 4D 76 記事、Blender 13 記事で、お金のある所は Cinema 4D のままに見える。残る理由は文字と切り貼り。だから皆 2 本の間を往復している。ぽぷりかは逆に Blender の中に編集と文字を寄せている。BYOL の動画のコメントで、読んだ中で一番いいねが多い物（182）は、After Effects 風の 2D を Blender でやる講座が欲しいという物だった。

### Motolii と照らす

文字、切り貼り、緑抜き、仕上げはもう 1 つの窓にある。残りを実物で試した。

| 物 | Motolii の今 | 確かめ方 |
|---|---|---|
| 緑を抜いて世界に置く、灯りを当てる、人の形の影 | できる | 11/1 に緑の画像を Chroma Key で抜いて `space 3d` に置き、灯りを当てた。丸い人に丸い影が壁へ落ちた。静止画 1 枚だけで、動画では未確認 |
| カメラを後から付ける、何台も切り替える | できる | checklist 206 行 |
| 待たずに見る | できる | Filament で実時間 |
| 手で描く線 | shape の `drawing` がある | V コンテとして使えるかは未測 |
| 物が落ちて当たる | できる | `physics` |
| 布、流体 | 無い | 素材で済ますか未定 |
| アニメ調の灯り | 無い | T の 3D のトゥーン |
| 動きを別の人形へ、VRM | 無い | T に有り |
| カメラ追跡 | 無い | T に有り。三脚の素材には要らない |
| 魚眼 | 検索で見つからない | ひっひっふーの絵の核 |

大きな差はアニメ調、人形、追跡の 3 つ。

お金はもう差にならない。Blender、Autograph、Cavalry は個人なら 0。差は手数だけになる。手数の見本は Blender より Cinema 4D。Blender Artists で移行の一番の穴に挙がるのは Cinema 4D の Cloner で、物理もタグ 1 つと褒められている。Motolii の物理も値 1 つ（`physics`）で、Cloner に当たる物は平面の Scatter まで。

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

読み取り：Stack の苦労は 2D と 3D の往復だった。Autograph と Motolii はどちらも 1 つの窓でこれを解く。

Codex の話で確かめられなかった事：本人が Signal や Ricochet を使った証拠は無い。Codex 自身もそう言っている。

Signal は loop、ease、noise、flicker・bounce・vibration・spin の型、Fields、BPM。Motolii のふるまい、区間イージング、BPM、減衰と大きく重なる。別物として足す必要は薄い。

Stack の作り方で次に測る事：真似の 1 カットを窓で作り、何が足りないかを手数で見る。作品の絵と音は使わず、手法だけ真似る。

## 界隈の実数

ユーザーの言葉：「界隈じゃないね、それはいかがでしたか系の記事だ」。前にここへ書いた表はまとめ記事の寄せ集めだったので消した。

**前の版の誤り：**「一番近い席は Apple Motion」「Win でも動くポチポチの道具は見つからない」は間違い。Autograph を見落としていた。

文字、BPM、型の棚はユーザーの言うとおり奪い済みの話なので、数字だけ残す。Presets の棚が空なのは変わらない。

**BOOTH のスキ**（検索ごとにスキ順の上位 14 件）：MV-Effects α 7,892、トランジション 81 種 2,632、MV-Effects β 1,859、MultiEase 1,710、RandomMotionNS 1,615、リリックモーション素材 54 種 1,528、テキストアニメ 170 個 1,510、VegaroLyricMotion 1,357、ひらがなだけ小さく 1,289、文字分解 1,245、Aulymo 1,018。

**Gumroad の評価の件数**（無料込み）：Free Motion Bro Presets 10,840、Ben Marriott の Grain Brush 450、Textures in After Effects 242、Cyberpunk LED Glitch 196、Type Array 194、Key Cloner 188、Depth 177。Cavalry の物は 10〜21 件。

**Motionographer 1,054 記事**（17 語で集めた）に言葉が出る記事の数：After Effects 217、storyboard 114、rig 80、Cinema 4D 76、stop motion 70、style frame 58、plugin 50、hand-drawn 48、grid か layout 33、Blender 13、template 11、preset 5、Cavalry、Jitter、Figma、Motion Bro は 0。plugin の話も最近は音か昔話が多い。Nickelodeon のリブランドは Illustrator でなぞって Cinema 4D へ入れ、スクリプトは自分たちで書いた。

**YouTube** のコメント 36 本 408 件で Flow 5、Overlord 4、Limber 4、FX Console 4。少なくて当てにならない。

### Autograph

2026 年 4 月 15 日に Maxon が無料で出し直した。翌日に Cavalry も Canva の元で個人無料になった。5 月 7 日からチームも無料。2 週間前に 2027 が出た。

- Mac と Win。2D と 3D が同じタイムライン。文字を 3D に押し出せる。Cloner がある。
- 3D の描画は Filament と 2023 年の記事にある。Motolii と同じ。土台は OpenUSD。
- 2027 で MCP サーバー、Cinema 4D との連携、文字の組版、テンプレート。
- 公式ページに型の棚と音で動かす話は無い。
- コメントの声：Adobe 離れ、Live Link の要望、講座が足りない。
- 2025 年の無料版の制限は、Maxon が無くしたと言っている。

世界と文字が 1 つの窓にある席は Motolii だけではなくなった。触っていないので手数は比べられない。

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

Blender へ行く理由から直した。

1. 3D のトゥーン。日本の MV が Blender へ行く入口で、Motolii に無い一番大きな物。
2. 緑バックの 1 カットを Blender の手順と比べて手数を測る。部品はあるので、動画で影まで通るかを含めて確かめる。
3. 人形：VRM、動きを別の人形へ。
4. カメラ追跡、魚眼。
5. 3D の Cloner。Cinema 4D が見本。
6. Autograph を触って手数を比べる。

文字、BPM、型の棚は外した。ユーザー：「そこらへんはもう奪ってるだろ」。カメラの動きも外した。ユーザー：「区間イージングでどうにかできる話」。

## 分からない事

- YouTube の 49 本は検索の上位で、全体の見本ではない。書いていない道具は数えられない。字幕は取れず、説明欄と題名だけ。
- 「ひっひっふー」の Hurray! の講演（Blender だけで編集まで）は案内の検索結果だけで、ページは 503 で読めなかった。CGWORLD の涌井嶺のインタビューも本文は読めず、検索の抜粋。
- 宝鐘マリン MV の話は聴いた人のメモで、本人の資料ではない。
- X は見ていない。Direktor 3D の売れた本数は載っていない。
- Reddit は内蔵ブラウザの安全制限で開けず、読んでいない。Core4D は証明書の不一致で読めなかった。aescripts の売上順は取れなかった。Discord は入っていない。
- BOOTH のスキは欲しいの数で、売れた数ではない。検索語の取りこぼしもある。Gumroad の評価は無料の物が多い。
- Motionographer は 17 語で引いた分だけで、全記事ではない。古い記事も混ざる。
- Autograph は触っていない。.aep の読み込みは 2025 年の記事で、今の条件は確かめていない。
- ぽぷりかがコンテを移した経緯は、公開の制作メモを読んだ人の note と、CGWORLD の記事の検索結果で読んだ。CGWORLD は 11/1 にメンテナンス中で本文は読めず、FANBOX の本人の記事も読めていない。
- ビデオサロンの講座の「Blender を使うメリット」は項目名だけで、中身は有料で読んでいない。
- BCON の講演は説明欄とコメントだけ。字幕は取れない。OK Go はジオメトリノードと実時間の入力の話で、After Effects から来た理由は書かれていない。
- コメント 313 件でお金と世界の話を数えたが、お金 11 件、世界 12 件で誤検出も多く、どちらが強いかは決められなかった。お金の説は確かめた結論ではなく、一番筋が通る説明。
- Cinema 4D の値段は Maxon のページと CG Channel の検索結果で読んだ。公式ページ本体は開いていない。
- 灯りと影の試しは静止画 1 枚。露出は調整していない。動画と Chroma Key の組は試していない。
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
- [MV-Effects α](https://booth.pm/ja/items/4112016)
- [RandomMotionNS](https://booth.pm/ja/items/3036337)
- [VegaroLyricMotion](https://booth.pm/ja/items/6856099)
- [Aulymo](https://booth.pm/ja/items/6403113)
- [BOOTH の After Effects スクリプト、スキ順](https://booth.pm/ja/search/AfterEffects%20%E3%82%B9%E3%82%AF%E3%83%AA%E3%83%97%E3%83%88?sort=wish_lists)
- [After Effects で「ずっと使っている」プラグイン 10 選（未都いのり）](https://note.com/mito_inori/n/n523ac8b7fad4)
- [モーショングラフィックス どんなツールで作る？（ヨシダタカユキ）](https://note.com/takayukiyoshida/n/n5d37f2db0629)
- [考えないで作る「それっぽい」モーショングラフィックス（しげぞう）](https://note.com/shige518/n/n05b4abe22f65)
- [Cavalry ってどんなツール？（Iori Iwaki）](https://note.com/ioriiwaki/n/n5f9f1a782330)
- [REEL 2025 の裏側（hama）](https://note.com/hama_mov/n/n7bf1c1ad592f)
- [Layr（しあさ）](https://note.com/shiasa/n/n258eeabb559d)
- [Nickelodeon のリブランド（Motionographer）](https://motionographer.com/2024/02/27/nickelodeon-global-rebrand-project-breakdown/)
- [Autograph](https://www.maxon.net/en/autograph)
- [Blenderで実写合成MVを作った記録（涌井嶺）](https://note.com/raywakui/n/na7cd1ee70030)
- [Blenderで実写合成MVを制作した話（tamf_kazuki）](https://note.com/tamf_kazuki/n/n0c8fa4c5c3ca)
- [Ae × Blender の制作の裏側（hama）](https://note.com/hama_mov/n/n8e69b97a2eea)
- [BlenderとAEの連携テスト（darenoki）](https://note.com/darenoki/n/n8ee90a3c4cf0)
- [blenderでセルルックアニメ制作（revirytic）](https://note.com/revirytic/n/neeaaaea70de3)
- [3D系MVも作りたいから（無印かげひと）](https://note.com/kagehito_muji/n/nfdafdfcb118d)
- [Hurray! コンセプトムービー 2 のメモ](https://note.com/lab1092/n/n751d390fe949)
- [ひっひっふー](https://www.youtube.com/watch?v=VXUbEH0DvSM)
- [ひっひっふー MV のセミナー（ビデオサロン）](https://videosalon.jp/premium/vsw307/)
- [映像作家 ぽぷりかの講座（Coloso）](https://coloso.jp/products/videocreator-popurika-jp)
- [Hurray! の映画は Blender が主（ITmedia）](https://www.itmedia.co.jp/news/articles/2401/24/news182.html)
- [Cinema 4D の値段（Maxon）](https://www.maxon.net/en/cinema-4d)
- [Cinema 4D 2027.0 と値段（CG Channel）](https://www.cgchannel.com/2026/10/maxon-releases-cinema-4d-2027-0/)
- [VTuber が自分で MV を作った（海月とうみ）](https://www.youtube.com/watch?v=aFkKIJyxv5U)
- [Eevee の NPR（Blender Fes）](https://www.youtube.com/watch?v=c9v2GjmylEg)
- [This Is Why Blender Is Replacing After Effects（BYOL）](https://www.youtube.com/watch?v=5QaW7oI5rTw)
- [I've Used Both Blender and Cinema 4D（Karon）](https://www.youtube.com/watch?v=9Vs5MQ0EogE)
- [Can Blender Still Compete（Ducky 3D）](https://www.youtube.com/watch?v=bDHdUT2oiZE)
- [5 EASY Ways To Create Music Videos（Max Novak）](https://www.youtube.com/watch?v=hgtBeWa5esY)
- [10-year C4D veteran moving to Blender（Blender Artists）](https://blenderartists.org/t/10-year-c4d-veteran-moving-to-blender/1178612)
- [Thinking of switching to Blender from C4D（Blender Artists）](https://blenderartists.org/t/thinking-of-switching-to-blender-from-c4d/1486541)
- [Tarahese の MV（Blender Artists）](https://blenderartists.org/t/tarahese-manifestations-an-animated-music-video/1392739)
- [OK Go の MV（BCON 2025）](https://www.youtube.com/watch?v=SQhJllRhOTU)
- [音で動く MV（BCON 2024）](https://www.youtube.com/watch?v=WV_zD_KppjY)
- [Autograph を無料に（CG Channel）](https://www.cgchannel.com/2026/04/maxon-releases-motion-graphics-software-autograph-for-free/)
- [チームも無料に（80 Level）](https://80.lv/articles/maxon-makes-autograph-free-for-all-users)
- [Autograph の描画は Filament（80 Level）](https://80.lv/articles/autograph-motion-design-visual-effects-tool)
- [2025 年の無料版の制限（Digital Production）](https://digitalproduction.com/2025/03/10/autograph-starter-free-motion-graphics-tool-with-a-twist/)
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
