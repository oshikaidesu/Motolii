# 箱のブロックの口 — 外のブロックで机上の試し

2026-09-15。利用者「コアが持つものは最小で、箱」「ファーストパーティの方が言葉が近い、サードパーティの外部エフェクトでも行えるべき」「ブロックは探る必要はありません。外部を参考にしてシミュレーションをすればいい」。
判定語: 案(机上)。先例の名前と振る舞いは各製品の取説の記述から。実装はしていない。

## 口の案(1 版)

ブロック = 時刻ごとの純関数。

```
block(t, params, items, world) -> outputs
```

| 入力 | 中身 |
|---|---|
| `t` | 時刻 |
| `params` | 欄の値(t で評価済み) |
| `items` | ブロックが掛かった物ごと: id、番号(写しの番号)、箱(親の空間の lo/hi)、中心、回転、大きさ、不透明度、自分の時刻 |
| `world.box(id)` | 他の物の箱(欄で指した物、親、comp) |
| `world.shape(id)` | 他の物の輪郭(道) |

| 出力 | 中身 |
|---|---|
| ずれ | 物ごとの位置・回転・大きさ・不透明度の差 |
| 時刻のずれ | 物ごとに、別の時刻の姿を見せる |
| 写し | 1 つの物から N 個、それぞれにずれ |
| 形 | 線・札など、描き足す輪郭(Push Trace、つなぐ線) |

## 外のブロックを通す

凡例: ○ 1 版の口で通る / △ 口に足す物が要る(右端の列) / × 口の外(別の道)

| 出どころ | ブロック | 振る舞い | 通るか | 足す物 |
|---|---|---|---|---|
| Cavalry | Duplicator + Grid / Circle / Linear | 写しを並べる | ○ | |
| Cavalry | Distribution: Path / Shape Points / Shape Edges | 道・形の上に並べる | ○(`world.shape`) | |
| Cavalry | Distribution: Sub-Mesh / Text の字 | 字・子の輪郭ごとに | △ | **字や子を items として出す**(文字の箱の中の小さな箱) |
| Cavalry | Stagger | 番号で値を振る(曲線つき) | ○ | |
| Cavalry | Falloff | 箱・形からの距離で強さ | ○ | |
| Cavalry | Noise / Oscillator(値) | t・番号・位置の関数 | ○ | |
| Cavalry | Oscillator / Noise(Deformer) | 輪郭を法線へ揺らす | △ | **出力に「形の変形」**(道を返す) |
| Cavalry | Random / Value Array / Color Array | 番号で値を選ぶ | ○(色は下) | |
| Cavalry | Color Array で色を振る | 物ごとの色 | △ | **出力に「物ごとの色」**(今の描く道は写しごとの色を持たない) |
| Cavalry | Look At / Distance | 他の箱の中心へ向く、距離を値に | ○ | |
| Cavalry | Connect Shape | 点どうしを線で結ぶ | ○(形) | |
| Cavalry | Align / Layout Group | 箱を揃える・並べる | ○(コアの配置を読んでずれ) | |
| Cavalry | Trails | 過去の位置の軌跡 | △ | **`world.at(t')` で別の時刻を読む**(巡り止め) |
| Cavalry | Forge Dynamics | 積み上げの剛体 | △ | **状態を持つブロックの約束**: t0 から解いた結果の覚え(cache)を口が持ち、同じ入力は同じ結果 |
| Cavalry | Color Collision Event | ぶつかった時に色 | △ | 上の状態のブロックが「出来事」を出す |
| C4D | Plain Effector + Field | 場の強さでずれ | ○ | |
| C4D | Step / Random Effector | 番号・種でずれ | ○ | |
| C4D | Target Effector | 的へ向ける | ○ | |
| C4D | Push Apart | 重なりを解く(全員を同時に読む) | ○(items 全部を読む) | |
| C4D | Delay Effector | 動きを遅らせてなめらかに | △ | `world.at(t')` |
| C4D | Shader Effector | 絵の明るさでずれ | × | 絵を読むのは解析の橋(Blob Track と同じ道)。口には解析の結果だけ渡す |
| C4D | Sound Effector | 音でずれ | × | 同上(音の解析の結果を値で) |
| AE | Repeater | 写し | ○ | |
| AE | Wiggle / expression の `valueAtTime` | 揺らぎ、別の時刻の値 | ○ / △ | `world.at(t')` |
| CSS | Flex / Grid / 箱の大きさ | 配置 | コア | (箱の規格そのもの) |
| CSS | anchor positioning / offset-path / transform-origin | 他の箱に付く、輪郭を道に、変形の中心 | ○ | |
| Motolii | 押す(間合い)・Bounce・見つけた格子・Push Trace・つなぐ線 | | ○ | |

## 分かったこと

口の 1 版に足す物は 5 つ:

1. **字や子を items に**(Sub-Mesh)。文字の箱の中に字の箱がある、を規格に
2. **出力に形の変形**(Deformer)
3. **出力に物ごとの色**(Color Array)。描く側が写しごとの色を受ける必要がある
4. **別の時刻を読む** `world.at(t')`(Trails・Delay・valueAtTime)。巡りを止める約束つき
5. **状態を持つブロックの約束**(Forge Dynamics・衝突の出来事)。時刻の純関数を、覚え(cache)で外から見て純に保つ

口の外に置く物: 絵・音を読むこと(解析の橋が先に解き、結果の値だけ渡す)。見た目(fx)。

「Sync」の芯(押し合って落ち着く・交わりの箱・同心・残像・鏡)は、1 版 + 4(別の時刻)+ 5(状態)で通る。

## 最小コアの線(2026-09-15 利用者)

- コアはレイヤー(在る・順・在る度合い)・箱・時間。色は中身、不透明度はレイヤー(重なりの計算が読む値)
- **taffy(CSS の配置)はコアに入れる**: 利用者「css もなぜ簡単かというと web の標準で、ユーザはわざわざ考えません」。標準は規格の一部で、ブロックではない
- それ以外(押し合い・Bounce・場・つなぐ線・Trace・見つけた格子・Repeater・形の効果)は build 無しで載る外のブロック。先例 AE の Tracery / Furikake(本体を build し直さない)、手本は vism の fx

## ブロックは GPU、天井を作らない(2026-09-15 利用者)

- 利用者「わたしの推奨は gpu。web にできないことをしたいから」「これはコンポジットソフト。ユーザはさらに上に詰める(ガラスのシェーディング、シェイプのトラッキング)。x10 レイヤーで 4000 シェイプ。この天井を作るべきでない」。JS は人の手のヘッドレス用(数百行)で、ブロックを載せない
- 形: 箱の並びを GPU のバッファに置き、ブロックは vism の fx と同じ置き場・頭の JSON・再読み込みの計算シェーダー。描く所まで GPU から出さない
- 今の天井(コードから、測っていない): 関係を CPU で物ごとに解く(押し合いは全組 × 32 回、移り方は 1 コマに過去の配置を最大 120 回)/ 形は層ごとに輪郭を絵に描く / 写しは 1 つずつ層として解く / Blob Track は絵を CPU へ読み戻して塊を数える
- taffy はコア(CPU)のまま。並べる計算は物の数に線形で、関係の天井ではない

## 2 番の前半: ブロックの段(2026-09-15、実装)

- `vism/` に置く fx と同じ頭の JSON に `"STAGE": "block"`。作者は `fn block(i: u32, p: BlockParams) -> Offset` だけを書く。箱の並び `items`(親の空間の lo / hi、住む箱、角丸、番号)・`host`(時刻と数)・欄の struct・全員に掛ける外枠は `block_program.rs` が組み、naga で確かめてから棚に載せる。読み直し・壊れた時に前の正しい物を残すのは fx と同じ
- 1 本目 `vism/bounce.wgsl`(欄 Strength)。試験: 4000 個の箱で GPU の結果が doc の `layout::bounced` と 0.05px 以内 `the_bounce_block_folds_boxes_like_the_cpu_law`、棚に段 Block で並ぶ `a_block_sits_on_the_shelf_like_an_effect`
- 踏んだ天井: device の上限を選択の枠の分だけ小さく借りていた(storage buffer 1 本・64 KiB・workgroup 16)。adapter の範囲で広げた(8 本・1 GiB・256)
- まだ: 描く所へ届いていない(試験だけ読み戻す)。次の段で fork の頂点の口(`motolii_field`)が物ごとのずれの buffer を読む。書類の Bounce(CPU)はその後に消す

## 2 番の後半: 読み戻さずに描く所まで(2026-09-15、実装)

- fork(`03801b1c`): `TargetConfiguration::motion`(物ごとの world のずれの storage buffer、全体の束ねの binding 11)。欄の最後(`params[23]`)が n > 0 の物は、網・板 2 つの頂点の段で n − 1 番のずれだけ動く。`MotionBuffer::new(ctx, 個数)` で Motolii が pool から借りて計算シェーダーで書く
- Motolii: `engine/blocks.rs`。層を組む前に住む箱(親の Group の箱を comp で)と物の箱(書類の `layer_box`)を集め、`build_layer` の後に描く時と同じ置き方で comp の箱と comp → world の向きを出して並べ、24 個目の欄に番号を入れる。組み終えたらブロックの種類と欄の値ごとに計算シェーダーを回し、view の設定 5 か所に buffer を差す。fx の hook の欄は 23 個まで(1 つ空けた)
- 試験: `the_bounce_block_draws_where_the_cpu_bounce_does`(Overflow = Bounce の CPU と、子に Bounce のブロックの GPU を 4 コマ描き、四角の中心が 1px 以内)
- 踏んだ物: 形の素材の枠は縁のにじみの余白で 2px 大きい → 書類の `layer_box` を読む
- 今の限り: 住む箱は comp で軸に沿う(親を回すと違う)。ブロックはつながらない(1 つの物に 1 本)。鏡の中の写し(反射)と選択の枠・当たり判定はずれを知らない。書類の CPU の Bounce はまだ残している

## 3 番の前半: 全員の state と、ブロックのつながり(2026-09-16、実装)

- 作りを直した: GPU に「全員の箱(`objects`)」と「全員の今のずれ(state、2 本を交互)」を置き、ブロックは掛かった物(`members`)の state にずれを足す。`now_lo(k)` / `now_hi(k)` で他の物の今の箱を読める(押し合いのように全員を読むブロックが書ける)。`"ROUNDS": n` で n 回続けて解く
- 物ごとの効果の列の順に段を分け、段の中はブロックと欄の値ごとに 1 回の計算。最後に固定の計算で state を world のずれにして motion へ
- 2 本目 `vism/push_apart.wgsl`(C4D の Push Apart、欄 Margin、ROUNDS 32)。試験 `the_push_apart_block_pushes_like_the_margin_law`(書類の間合いの押し合いの nudge と 0.05px 以内)
- つながり: `blocks_chain_in_effect_order`(Push Apart → Bounce で 40 個が箱の中に収まる)
- 押し合いは全組(1 回 n²)。4000 個 × 32 回は GPU でも重い見込み — 升目で近い物だけ読む口は次
