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
set 2 key 1
key 2 x 0 -0.6
key 2 x 3 0.6
add glass ring
set 3 y -0.7
at 2.5
still out/now.png
undo
redo
say 2 lay screen
film out/mine.mp4
list
```

- `add 種類 ファイル` … 置く。種類は film（動画）、picture（AEのファイル）、glass（ring、gem、cube）。返事に物の番号が出る。
- `set 番号 名前 数` … 値を変える。`key 番号 名前 時刻 数` … その時刻にキーを打つ。`unkey 番号 名前 時刻` … キーを消す。
- `say 番号 名前 言葉` … 言葉の値を変える（`lay` に add／multiply／screen）。`remove 番号` … 消す。
- `undo`、`redo` … 取り消し、やり直し。閉じて開き直した後でも効く。
- `at 秒` … 舞台の時刻。`still 絵.png` … 今の絵。`film 動画.mp4` … 最初から最後までの動画。`list` … 文書の中身。
- 値の名前：x、y、z（場所）、turn（傾き、度）、high（高さ）、start（出る時刻）、inpoint・duration・speed（動画のどこから・どれだけ・速さ）、key（1で地の色を抜く）。

絵と動画は `filament/out/wgpu_kit/out/` に出る。保存の操作は無い（打った時にもうファイルに入っている）。

## 出来上がった場面を見る

`reports/show/` の動画。今夜の物：`neko_v2.mp4`（猫の民衆）、`mask_v5.mp4`（マスク）、`lay_v1.mp4`（重ね方）、`edit_v1.mp4`（上の文書）、
`puppet_v4_awkward.mp4`（変な動画12本）、`led_v4_arch.mp4`（AEの動きを3Dの物に）。

## 検査を走らせ直す

`cd filament && ./records.sh`（20分ほど。結果は `reports/ae_rubric/records/`）。
