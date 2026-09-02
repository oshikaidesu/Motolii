# vgpu → Vism の系譜とホットリロード

## 何を借りたか

[vgpu](https://github.com/vercel-labs/vgpu) は MIT の WebGPU library。Motolii はruntimeを依存にせず、
`Triangle LED Hero` の表現を `vism/tri_led.wgsl` へ移した。元はLED emitter textureと
raycastの2pass・7 WGSL・host側TypeScript設定だった。Motolii版は1passへ畳み、importをinline化し、
texture参照を辺上のLEDを直接合成する式へ置換した。これは同じ作品の複製ではなく、
**shader + 宣言された型付きinput + hostが渡す時間/値**というVism境界の外部証拠である。

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
