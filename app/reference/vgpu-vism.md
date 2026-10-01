# vgpu → Vism の系譜とホットリロード

## 何を借りたか

[vgpu](https://github.com/vercel-labs/vgpu) は MIT の WebGPU library。Motolii はruntimeを依存にせず、
`Triangle LED Hero` の表現を `vism/tri_led.wgsl` へ移した。元はLED emitter textureと
raycastの2pass・7 WGSL・host側TypeScript設定だった。Motolii版は1passへ畳み、importをinline化し、
texture参照を辺上のLEDを直接合成する式へ置換した。これは同じ作品の複製ではなく、
**shader + 宣言された型付きinput + hostが渡す時間/値**というVism境界の外部証拠である。

2026-09-08 に `examples/radiance-cascades` を `vism/radiance.wgsl` へ移した(jump flood → 距離場 →
6 段の cascade → 合成)。vgpu では TypeScript が ping-pong する 6 枚の target を、ISF の「同じ TARGET 名は
同じ buffer」で manifest に畳み、段の分岐は PASSINDEX で shader 側に持たせた。探針の間隔を 2px にして
atlas を素材と同じ大きさに収めた。生成器ではなく、素材の明るい所と形が無ければ何も起きない効果。

同日、`examples/transmission`(背後のフレームを mip ピラミッドにして屈折先を読む)と `examples/clipping`
(平面で切って断面に蓋)も移した。前者は fork の網 shader に背後の絵を読む口を足し、後者は世界の平面を
fork の 1 式にして板・点群・網が同じ式で切れるようにした。生成器(black-hole・fractal)は「素材があってこそ
輝く物を取る」裁定で採らない。

一次の調査・変形の全記録は
[vgpuとVism構想](../../docs/reviews/2026-08-29-vgpu-vism-viability.md)。

## vgpu から残す体系

- `.wgsl`をmoduleとして扱い、reflectionでbinding名・型・layoutを手書きから外す
  ([README](https://github.com/vercel-labs/vgpu/blob/main/README.md))。
- 値と時間はshader内のambient stateでなく、hostが`set()`と`clock()`から明示して渡す
  ([Getting started](https://github.com/vercel-labs/vgpu/blob/main/docs/topics/getting-started.docs.md))。
- bundlerのwatch graphへ推移的なWGSL importを登録し、変更時に依存も読み直す
  ([WGSL HMR](https://github.com/vercel-labs/vgpu/blob/main/packages/wgsl/README.md#hmr-behavior))。
- `check`→headless実画素→browserの順で検証し、見た目だけを根拠にしない
  ([Shader workflow](https://github.com/vercel-labs/vgpu/blob/main/docs/topics/shader-workflow.docs.md))。
- docs・examples・validationをCLIと機械可読なagent資料から発見できるようにする。

Motoliiで対応する物は、shader先頭のISF互換JSON、ISF/WGSL共通の`VismProgram`、
build時に`vism/`から生成するinventory、`re_renderer`のshader/pipeline poolと`FileServer`である。

## 現在のリロード境界

| 変更 | 現在 | 完成条件 |
|---|---|---|
| 既存のplain WGSL本文 | debug窓で監視。`RenderContext::begin_frame`がshaderとpipelineを再生成 | 実窓で次描画に反映する回帰試験 |
| 構文・pipeline生成error | `re_renderer`は失敗時に旧pipelineを置換しない | 窓へ診断を返し、last-goodの絵を保つ |
| `DEFAULT`/`MIN`/`MAX`/`MAPS`などmanifest | 起動時の`OnceLock`。再起動が必要 | 本文+manifestを一transactionでparse/validateし、Inspectorも同時更新 |
| Vismファイルの追加・削除 | `build.rs` inventory生成。再buildが必要 | directory watch→ID衝突/admission→追加/撤去を原子的に公開 |
| preludeを前置きするblend/matte | 生成物を一時fileへ置くためwatch対象外 | import/preludeを依存graphとして監視する |

したがって「WGSL hot reload済み」は**本文だけ**なら正しいが、vgpuが持つmodule/HMR体系や
Vismの宣言・catalog・Inspectorまで含むhot reloadは未完成。完成形は、file変更を
**安定読取 → manifest+shader検証 → layout/pipeline準備 → catalogと描画を同時交換**する一手にし、
途中の失敗では現在のVismを1箇所も進めないこと。
