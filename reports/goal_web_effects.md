# ゴール：ネットの WebGPU 効果を、作者のレンダラーのまま Motolii の動画に（10/26）

## ゴール

shader-effects-inc/shaders（shaders.com、MIT、200 種）の効果を、Motolii の動画の中で、**作者のレンダラーのまま**、**Filament と同じ Dawn の装置で**、60fps で使える。

## できたと言える条件

1. `reports/splat_orbit.mp4`（Motolii が書き出した動画）に、作者の Glitch を掛けた動画が、Motolii の中で撮れる。動画は作者の VideoTexture か ImageTexture を通る。
2. 物の効果の段の 1 つとして載る（`filament/thing_fx.h` の段の並び。ThorVG と同じ道：Filament の描く糸の順番に入れる `door::on_driver`、結果は `Texture::Builder::import` で Filament へ）。
3. つまみは作者の定義（各効果の props の範囲と初期値）をそのまま使う。時間は作者の口 `renderFrame({ deltaSeconds })` で Motolii の時間軸の 1 コマにつなぐ。
4. 1920×1080 で 1 コマ 16.7 ms 未満。QuickJS の分と GPU の分を分けて測る。

## 今あるもの（動かして確かめた）

- 作者のレンダラー（npm の組み立て済み `shaders@4.0.5` の `createRendererFromJSON`）が、Motolii の QuickJS-ng 0.17 の上で、Filament の装置を借りて描く。Plasma の上の Glitch が Node で描いた絵と 1 バイトも違わない。640×360 で 1 コマ 3.4〜4.5 ms。
  - 動かし方：`./filament/dawn_probe/js_effects.sh <preset.json> <w> <h> <frames> <out.rgba>`（preset の例：`{"components":[{"type":"Glitch","children":[{"type":"Plasma"}]}]}`）
- 部品：
  - `filament/tsl/jsrh_napi/build.sh` と `jsrh_qjs017_dawn.patch`：BabylonJS/JsRuntimeHost（980cfb0、MIT）の QuickJS の Node-API を、Motolii の QuickJS に `Napi::Attach` で付ける。0.17 との違い 1 か所と、足りない関数 3 つ。
  - `filament/tsl/dawn_lend.patch`：Dawn の JS の口（dawn.node）に Filament の Dawn の呼び先の表（`motolii_dawn_procs`）を使わせ、装置とテクスチャを `wrapDevice`・`wrapTexture` で包む。
  - `filament/tsl/jsrh_napi/shaders.ts`、`web_globals.ts`：作者のレンダラーを回す包みと、QuickJS に無い Web の部品（core-js、event-target-shim、abort-controller、fast-text-encoding）。
- 経緯と数字：`reports/shaderport_video.md`。

## 残っている穴

1. **動画の入力**：dawn.node に外部テクスチャが無い。Dawn の本家の main（0cf3049c5b、2026-10-26）でも、`GPUDevice::importExternalTexture`（GPUDevice.cpp:398-402）、外部テクスチャを bind group に入れる所（Converter.cpp:1489-1493、`TODO(crbug.com/dawn/1129)`）、`GPUQueue::copyExternalImageToTexture`（GPUQueue.cpp:119-124）がまだ UNIMPLEMENTED。他の倉庫で埋めた物は、まだ探し終えていない。
2. **Motolii の画面の中で毎コマ回すこと**：今は Filament の仕事を全部終えてから JS を回している。画面の中では Filament の描く糸と同じ装置を使うので、`door::on_driver` の順番に入れる。
3. **時間を戻す・飛ばす**：前のコマを使う効果（TimeTrail など）をどう描き直すか。作者のレンダラーに初めから描き直す口があるかを先に調べる。

## 決まり（Motolii の partner）

- 自作は負債。書く前に既存の物（本家、分けて持っている倉庫、コミュニティ）を探す。見つからなければ、問題の切り方を疑う。
- 借り物と借り物の継ぎ目だけを、直しのファイル（patch）として書く。出典（ファイルと行、版）を書く。コミットごとに「借りた物／書いた物」を書く。
- 動かしていないものを「できた」と言わない。数字は測った物だけ。
- 説明は短く、見えた進み具合だけ。ユーザーに判断を投げる問いが出たら、切り方が間違っている合図。

## やめた道（もう一度やらない）

- Node を部品として埋め込む：JS の土台（V8）をもう 1 つ抱えるのは大きすぎる。
- NativeScript の QuickJS の Node-API：dawn.node の最初の試しまでに穴が 4 つあった。
- Dawn の記録と流し直し（Replay）：前に流すだけで、時間を変えて同じコマを繰り返せない。
- 作者の WGSL を書き出して、自作の再生役で段を回す：Surface3D の compute、TimeTrail の前のコマ、マウスなどが抜ける。消した。
- Filament の描く係（screen_door）で作者の WGSL を描く：入口の名前が `main` 固定、テクスチャとサンプラーを同じ set に置く決まり、`texture_external` が無い。
