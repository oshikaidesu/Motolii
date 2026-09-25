# レンダー締め: 全画面 run の固定費の正体(2026-09-25、M4 / Metal、構造変更なし、測定のみ)

問い: Glass Garden の GPU p50 18.2ms のうち、「1 run をフルサイズの描画面として扱うこと自体の 1.2〜1.4ms」は何に支配されているか。

## 結論(先に)

**attachment(tile memory / MSAA / fp16 resolve / depth / store)ではない。** 固定費に見えていたのは、run の中の**画素ごとの fragment 仕事**(exact curve fill と、rectangle の手書き bilinear)と、**内容に関わらず全画面で走る補助 pass**(mix、Glow、Blur、backdrop の写しと mip、Look)の 2 つ。前者は面積に比例し、後者は内容に比例しない。

## 測定と結果

### 1. attachment 単体(`attachment_microbench.rs`、wgpu 30、1080p、全画面の三角形 1 枚を 100 回)

| 構成 | ms / pass |
|---|---|
| Motolii と同じ: MSAA 4× + Rgba16Float + Depth32Float、resolve、blend | 0.082 |
| 描かない(clear + resolve だけ) | 0.068 |
| depth 無し / blend 無し / Rgba8 / Depth16 | 0.070〜0.084 |
| MSAA 無し(Rgba16F、depth 有り / 無し) | 0.066〜0.068 |
| MSAA 4× を resolve せず store も | 0.240 |
| sample-rate shading(trivial な shader) | 0.080 |

Motolii の 1 run は 1.2〜1.4ms なので、**attachment 構成は 15 倍以上の差の説明にならない**。Motolii で MSAA を切ると 1.4 → 2.5ms に悪化した(別の shader 経路になるため)。

### 2. 実際の run を fragment shader の中で切る(Metal System Trace、`MOTOLII_GPU_LABELS=1`)

全画面の run 1 枚(単色の curve fill 矩形 = `full_plain`、texture の矩形 = `full_png`、Prism Orbit の層 = `only_stage`)。

| shader を切る位置 | mesh(curve fill / texture) | rectangle |
|---|---|---|
| 何も評価せず定数を返す | 0.15 ms(表示閾値未満) | 0.30 ms |
| mesh: footprint と clip の後 / texture 1 枚を読んだ後 | 0.15 / 0.16(texture の読みは安い) | — |
| mesh: `switch`(curve coverage と gradient)の後 | **1.08〜1.16 ms** | — |
| 全部(hook と tint まで) | 1.29〜1.37 ms | 1.18 ms |
| mesh: curve fill のコードを消す(表示は壊れる、測定用) | 0.37 ms | — |
| rectangle: minification を Nearest にする | — | 0.67 ms(1.18 → 0.67) |

- **mesh の全画面 curve fill(層の形が shape の時)**: exact curve coverage(band の header と curve を `textureLoad` して winding を足す、2 本の ray)が約 **0.9ms / 全画面**。Orbit の 2 層(272×272 の矩形 shape を 7 倍に拡大した canvas 大の mesh)はこの経路。
- **rectangle(層が texture の時)**: fragment が `textureLoad` 4 回と decode(bilinear を手で)。Nearest にすると 0.5ms 減る。ember(blur 後の絵)はこの経路。
- curve fill のコードを、curve を持たない model の program だけから外す変種(fork の `no_curve_fill`、押し出し solid・glTF・material の板が対象)を作って測ったが、Glass Garden は 18.3ms のまま**変化なし**(register 圧の仮説は否定)。fork と Motolii の変更は捨てた。

### 3. 内容に関わらない全画面の仕事(GPU trace の 1 コマ、18.4ms の内訳から)

| 仕事 | 1 回 | 回数 | 計 |
|---|---|---|---|
| mix(blend vism、下と上を読んで新しい canvas へ) | 0.15〜0.24 | 約 9 | 約 1.5 ms |
| Glow の 12 pass(plate 2 つ) | 0.55 / plate | 2 | 1.1 ms(Glow を描かないと −1.1) |
| Blur(ember、2151² の padded canvas、使わない段のゼロ書き 0.30 を含む) | — | 1 | 約 1.1 ms(Blur を外すと −1.1) |
| backdrop の写しと mip(ガラスの run の前) | 0.24〜0.39 + 0.1〜0.17 | 3 | 約 1 ms |
| Look(13 pass)+ 最後の合成 | — | 1 | 約 1.2 ms(空のシーンでも同じ) |

### 4. run を 1 つずつ描かない(混雑下の暫定値、掘る順を決めるためだけ。最終値ではない)

背景全面 Stage and Orbit 3.1 / 外側の輪 3.0 / 円盤+板 2.5 / 内側の輪 2.3 / Orbit Front 2.2 / ember 1.5 / 球 1.0 / ラベル 0.5 ms。

## 比較資料(A / B / C、実装しない)

- **A 平らな 2D の run を軽い合成経路にする**(MSAA・fp16・depth の ViewBuilder を通さない):
  固定費の主因が attachment ではないと分かったので、得られるのは shader 仕事の差だけ。texture の矩形なら手書き bilinear を避けられ(約 −0.5ms)、curve fill の層は結局 coverage を評価する。**縁が bit 一致でなくなる**うえ、「flat 2D だけ別経路」は、将来 Effect / Glass / Depth / Relation が乗ると境界条件が増える。**根拠が弱くなった**。
- **B run / plate の canvas を内容の範囲だけにする**(canvas の origin を持つ mix):
  内容に関わらない全画面の仕事(mix 約 1.5 + Glow 1.1 + Blur 1.1 + backdrop 約 1 + MSAA の clear / resolve 約 0.6 = **約 5ms**)が、内容の面積に比例するようになる。2D 専用の規則ではなく、小さい plate / run 全般(300 Jewel の Repeater 群、小さい Glow 付きの輪)に効く。**今回の測定で最も根拠が強くなった**。ただし compositor の構成変更(mix の入出力、`Window` の roi、screen pass の origin、backdrop の座標)で、設計と検収が要る。
- **C 今の 18.2ms(約 55fps)で受け入れる**: 画質回路を削らず、CPU の山は消え、構造は 300 Jewel で個数に比例しない。Glass Garden は 2 枚の全画面 curve fill + 2 つの Glow plate + ガラス 40 枚超の意地悪な fixture。

## 構造を変えずに取れる無駄(小さい、今回は実装していない)

- rectangle の手書き bilinear を、decode が恒等の texture では hardware bilinear にする(fork の rectangle shader、約 −0.5ms / 全画面矩形、重みの精度差で bit 一致ではないが画は同じ)。
- Blur が radius に応じて使わない段(0 を書く full-res の 2 pass)を走らせない(約 −0.3ms、ISF の PASSES を radius で選ぶ仕組みが要る)。
- どちらも Glass Garden を 16.7ms に入れるには足りない(合計 −0.8ms 程度)。

## 開発用の道具(コミット済み)

`MOTOLII_GPU_LABELS=1` で view run と effect pass に名前が付く(`xcrun xctrace record --template 'Metal System Trace'` の GPU interval で読む。1 コマの時系列は `metal-gpu-intervals` を xml で export して解析)。
