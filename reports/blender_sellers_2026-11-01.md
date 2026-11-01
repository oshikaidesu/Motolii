# Blender で一番売れている簡単な拡張（11/1）

ユーザーの問い：「今、ブレンダーで一番売れてる簡単プラグインプリセット集みたいなのって何があるか調べてみて。MV でブレンダー使ってる人がいちいち最初からモデリングとかしてるわけないからな、動きとかも」。

## 結論

- 売れているのは、人形に動きを移す道具と、出来合いの素材の棚。
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

## Motolii にある物と無い物

| 売れ筋 | Motolii の今 | 差 |
|---|---|---|
| カメラの動き（CinePack） | カメラとギズモは有る。動きの棚は無い | 小。AbsentMotion の `CameraMove` 6 種が手元にある（`CameraRig.kt`）。T に行あり |
| 素材の棚（Poly Haven） | `room` に 7 部屋。.gltf は読める | 中。読めるが棚に無い |
| 人の動き（Auto-Rig Pro、Mixamo） | .glb に入った動きは流せる（gltfio の Animator） | 大。別の人形へ移す口が無い。FBX も BVH も読まない |
| 火、電気、天気（Elemental） | 名前で探して Appear Rain、Smoke Screen、am_clouds、am_lightning、am_electricedges | 中。雨や雷はある。火と爆発は名前では見つからない |
| 散らす（Geo-Scatter） | Scatter Repeat と Scatter Field | 中。今は層を平面に散らす。3D の面に生やす物は無い |
| 群衆（Procedural Crowds） | 無い | 大。動きの移し替えが先 |
| 液体（Flip Fluids） | 無い | 大。出来た絵や動画を使う方が早い |
| 文字の動き（Quick Motion） | ふるまいと Look が有る | 小。売れ方も小さい |

## 順番

1. カメラの動き。売れていて、見本がそのまま手元にあり、手が一番小さい。
2. Poly Haven の棚。無料で CC0。作るのは棚だけ。
3. 人の動きの移し替え。一番売れている所だが、一番重い。既存の実装を探す所から。
4. 火と爆発の棚。

## 分からない事

- MV 制作者が実際に何を使っているかの調査は見つからなかった。CGWORLD の連載は本文を読んでいない。
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
