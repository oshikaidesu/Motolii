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
set 1 high 4.2
add film ../vid/happy.mp4
set 2 keyed 1
key 2 x 0 -0.6
key 2 x 3 0.6
add glass ring
set 3 y -0.7
add solid ../../third_party/filament_src/third_party/models/Fox/Fox.glb
add picture ../../../reports/ae_rubric/stills/lay_v1.jpg
at 2.5
still out/now.png
undo
redo
say 2 lay screen
write out/mine.mp4
list
```

- `add 種類 ファイル` … 置く。返事に物の番号が出る（番号は使い回さない）。種類：
  film（動画）、picture（AE のファイル、写真の .jpg／.png）、solid（3Dのファイル .glb／.gltf）、glass（ring、gem、cube）。
- `set 番号 名前 数` … 値を変える。`key 番号 名前 時刻 数` … その時刻にキーを打つ。`unkey 番号 名前 時刻` … キーを消す。
- `say 番号 名前 言葉` … 言葉の値を変える（`lay` に over／add／multiply／screen）。`remove 番号` … 消す。
- `undo`、`redo` … 取り消し、やり直し。閉じて開き直した後でも効く。
- `at 秒` … 舞台の時刻。`still 絵.png` … 今の絵。`write 動画.mp4` … 最初から最後までの動画。`list` … 文書の中身（自分で変えた値だけ出る）。
- 数の名前：x、y、z（場所）、turn（傾き、度）、high（高さ）、start（出る時刻）、duration（出ている長さ。言わなければずっと出ていて、終わりまで行ったら頭から繰り返す）、
  inpoint・speed（動画のどこから・速さ、負は逆再生）、keyed（動画の緑の地を抜くなら 1）。
- 単位：カメラは決め打ちで、z＝0 の所で画面は高さ 3.2、幅 1.8。真ん中が x＝0、y＝0。上が y の正、手前が z の正。置いた物は高さ 1 で真ん中に出る。前後は z で決まる。
- 出来ない呼び出しは「NOT done: 理由」と返って、文書は変わらない（無いファイル、無い番号、知らない名前、舞台が出せない値）。

絵と動画は `filament/out/wgpu_kit/out/` に出る。保存の操作は無い（打った時にもうファイルに入っている）。
文書からはまだ出来ない事：型（マスク）、カメラを動かす、文字を打つ、ぼかし、影、音。

## 出来上がった場面を見る

`reports/show/` の動画。それぞれ一番新しい版：`neko_v3.mp4`（猫の民衆）、`mask_v6.mp4`（マスク）、`lay_v2.mp4`（重ね方）、`edit_v2.mp4`（文書：AEの絵、動画、ガラス、3Dファイル、写真）、
`puppet_v5.mp4`・`puppet_v5_forty.mp4`・`puppet_v5_awkward.mp4`（人形劇、40本、変な動画12本）、`led_v4_arch.mp4`（AEの動きを3Dの物に）。
古い版（番号の小さい物）は直す前の記録として置いてあるだけ。

## 検査を走らせ直す

`cd filament && ./records.sh`（20分ほど。結果は `reports/ae_rubric/records/`）。
