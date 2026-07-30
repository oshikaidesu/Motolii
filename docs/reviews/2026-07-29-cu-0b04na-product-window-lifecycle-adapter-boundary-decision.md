# CU-0B04NA product window lifecycle adapter境界決定

- 日付: 2026-07-29
- 状態: **決定・guard実装完了 / DONE**
- 前提: `CU-0B04P`、`CU-0B04N`

`CU-0B04N`のdirect winit Hostは、既存U0d-3 raw input guardが許可するegui
layout adapter閉集合に含まれず、`WindowEvent`参照を正しく拒否した。

新しい承認境界を`motolii-ui` private
`product_runtime_adapter.rs`一ファイルへ限定する。許可するwinit eventは
`CloseRequested` / `Resized` / `ScaleFactorChanged` / `Occluded` /
`RedrawRequested`だけである。keyboard、pointer、device event、alias / glob、
egui raw inputは引き続き拒否する。

これは製品input意味、公開API、Document、D2、Undoを追加せず、1 Surfaceのconfigure /
acquire / presentとlayout epoch更新に必要なwindow lifecycleだけを既存guardへ追加する。
guard fixtureは許可5 variantの正例とKey/Device/KeyboardInputの負例を持つ。

次は同じ`CU-0B04N`実装へ戻る。guardを広げてraw pointerやshortcutを通さない。
