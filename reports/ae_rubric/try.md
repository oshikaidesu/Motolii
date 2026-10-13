# やってみる（使う人が打つ物だけ）

## 文書を開いて、呼び出しで編集する

```
cd /Users/motolii_maker/motolii/filament
./edit.sh out/mine.motolii
```

開いたら、1行に1つ打つ（ファイルの場所は `filament/out/wgpu_kit/` から見た形）：

```
add picture ../../third_party/thorvg_example/res/lottie/gradient_background.json
set 1 z -1
set 1 size 4.2
add film ../vid/happy.mp4
set 2 keyed 1
key 2 x 0 -0.6
key 2 x 3 0.6
add glass ring
set 3 y -0.7
add solid ../../third_party/filament_src/third_party/models/Fox/Fox.glb
add picture ../../../reports/ae_rubric/stills/lay_v1.jpg
add text ../arial_black.ttf
say 6 text MOTOLII
say 6 fill ffffff
set 6 y 1.3
set 6 size 0.24
at 2.5
still out/now.png
undo
redo
say 2 blend screen
write out/mine.mp4
list
```

- `add 種類 ファイル` … 置く。返事に物の番号が出る（番号は使い回さない）。種類：
  film（動画）、picture（AE のファイル、写真の .jpg／.png）、text（文字。ファイルはフォント .ttf）、solid（3Dのファイル .glb／.gltf）、glass（ring、gem、cube）。
  フォントは `filament/out/arial_black.ttf` と `arial_unicode.ttf`（日本語も入っている）に、この Mac の物への近道が置いてある。
- `set 番号 名前 数` … 値を変える。`key 番号 名前 時刻 数` … その時刻にキーを打つ。`unkey 番号 名前 時刻` … キーを消す。
- `say 番号 名前 言葉` … 言葉の値を変える（`blend` に normal／add／multiply／screen。文字なら `text` に行の終わりまでの文、`fill` に6桁の色 ff3366）。文字の `size` は1行の高さ。`remove 番号` … 消す。
- `undo`、`redo` … 取り消し、やり直し。閉じて開き直した後でも効く。
- `at 秒` … 舞台の時刻。`still 絵.png` … 今の絵。`write 動画.mp4` … 最初から最後までの動画。`list` … 文書の中身（自分で変えた値だけ出る）。
- 数の名前（ThorVG が自分で使ってる名前に合わせた）：x、y、z（場所）、rotate（傾き、度）、size（高さ）、start（出る時刻）、duration（出ている長さ。言わなければずっと出ていて、終わりまで行ったら頭から繰り返す）、
  begin・speed（動画のどこから・速さ、負は逆再生）、keyed（動画の緑の地を抜くなら 1）。
- 単位：カメラは決め打ちで、z＝0 の所で画面は高さ 3.2、幅 1.8。真ん中が x＝0、y＝0。上が y の正、手前が z の正。置いた物は高さ 1 で真ん中に出る。前後は z で決まる。
- 出来ない呼び出しは「NOT done: 理由」と返って、文書は変わらない（無いファイル、無い番号、その種類の物が持たない名前、範囲の外の数、舞台が出せない値）。断られた `add` は番号を使わない。
- 形の違う行（言葉が足りない・多い、数の所に言葉）は「not a call」と返って、呼び出しの一覧が出る。`help` でも一覧が出る。
- **キーとキーの間は、それ自体を決められる**（Motolii のやり方。AE は値から値へだけ）。`way 番号 名前 時刻 行き方` で、その時刻のキーから次のキーまでの行き方を言う。
  行き方は過去の倉庫（`rust_ae/Motolii`）の9種類そのまま：`Hold`、`Linear`、`Bezier x1 y1 x2 y2`、`Bounce first_dip dip`、`Elastic limit period damp`、
  `Cyclic period peak linear envelope_end`、`Random seed grain center_u center_v bias`（揺らす。両端では止まる）、`Steps width smooth`、`ElasticSteps width elasticity`。
  名前だけ言えばいつもの数、数を言うなら全部。何も言わない間は `Bezier`（ゆるく出てゆるく止まる）。`ways` で一覧。見本：`reports/show/ways_v2.mp4`。
  9種類に無い行き方は式で足せる：`define 名前 式`（u＝間のどこまで来たか 0〜1、t＝間に入ってからの秒、d＝間の長さ、a・b＝両端のキーの値、p1〜p4＝後に言う数）。両端が合わない式は断る。
  `value 番号 名前 時刻` で、その時刻の値が読める。
- キーを打てるのは x、y、z、rotate、size。キーのある値に `set` すると、キーは全部消えてその数で止まる（そう返事が出る。取り消しで戻る）。
- 動画に書き出す長さは、いちばん長く出ている物の終わりまで（長さを言ってない物は、自分を1回通す長さ）。終わる物が1つも無ければ 8秒。`list` の最後に出る。
- 同じ場所（同じ z）に2つ置くと、後から置いた方が前。
- ファイルの名前は行の終わりまで（空白があってもいい）。文書の中の色は、ファイルの色のまま出る（白 255 は 255）。
- フォントは .ttf だけ（.ttc は読めない）。フォントに無い字は出ない（Arial Black に日本語は無い。日本語は arial_unicode.ttf で）。

絵と動画は `filament/out/wgpu_kit/out/` に出る。保存の操作は無い（打った時にもうファイルに入っている）。
文書からはまだ出来ない事：型（マスク）、カメラを動かす、ぼかし、影。音は書き出した動画に入る（作業中は鳴らない）。

## 出来上がった場面を見る

`reports/show/` の動画。それぞれ一番新しい版：`neko_v4.mp4`（猫の民衆）、`mask_v7.mp4`（マスク）、`lay_v3.mp4`（重ね方）、`edit_v5.mp4`（文書：AEの絵、動画、ガラス、文字2つ、3Dファイル、写真）、
`puppet_v6.mp4`・`puppet_v6_forty.mp4`・`puppet_v6_awkward.mp4`（人形劇、40本、変な動画14本）、`led_v5_arch.mp4`（AEの動きを3Dの物に）。
音は動画の素材そのものの音（動画の無い `mask_v7`、`led_v5_arch` は無音）。
古い版（番号の小さい物）は直す前の記録として置いてあるだけ。

## 検査を走らせ直す

`cd filament && ./records.sh`（20分ほど。結果は `reports/ae_rubric/records/`）。

## 同じ事を C の関数で（Flutter のガワが呼ぶ形）

`filament/motolii.h` が口。`filament/check/api.c` が素の C からの呼び方の見本（`filament/out/wgpu_kit/` で `../api_check ../mine2.motolii out/api.png`）。
上の行を読む道具（`edit.sh`）も、中ではこの関数だけを呼んでいる。
