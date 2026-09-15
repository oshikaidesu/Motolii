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
