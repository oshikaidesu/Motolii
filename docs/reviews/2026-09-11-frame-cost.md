# 再生負荷の調査 — 2026-09-11

## 結論

保存済み swiss.rrd の比較では、主な継続負荷は Text (LayerId 1) の Radiance。パスのCPU三角化を今回の主因とする証拠は得ていない。GPU処理であるRadianceの負荷と、GPU完了を編集キューで同期的に待つ構造を分けて対処する必要がある。

## 測定

- 起動中アプリ PID 14912 を再生中に `sample 14912 3 1`。共有文書キューの1217サンプル中1053が `EditorRuntime::render_into → Device::poll → Metal wait` 内。これは全CPU利用率でもGPU実行時間の直接測定でもない。
- 比較は保存ファイルを毎条件読み直し、メモリ上のIntentで素材／効果を除外。元ファイルと実窓の作品は変更していない。
- ファイル: `/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/100daa9e-6636-43f2-80a9-68d347adbb8a/scratchpad/swiss.rrd`
- SHA256: `34b396d04f0f68f25b16294a1cabd028b3d9e244c4fff8ec26d8cf0a6aa39cca`
- 正常ビルド済み `libmotolii_render-7646cbd50e3c9429.rlib` と、その依存fingerprintが指す `libwgpu-caac4f3cf7933f69.rlib` を用いて測定器をlink。現在の作業ツリーとは別の固定済みバイナリ。
- `Engine::render_frame_into` → `Device::poll(wait_indefinitely)`。1920×1080の保存comp、0〜145フレームを5フレーム刻み、2周目の30描画の平均。画像CPU readbackなし。CPU欄はrender呼び出しの経過時間で、純粋なCPU実行時間ではない。待ち欄もGPU timestampではない。
- 一部計測中に別のコンパイルが動いているためCPU側の絶対値には交絡がある。2回の比較で全体約33ms、Radiance除去約6msという差は明確。実窓の観測カメラ・未保存編集・連続全フレーム・Flutter表示時間までは一致させていない。

| 条件 | CPU側経過 ms | GPU完了待ち ms | 合計 ms |
|---|---:|---:|---:|
| 全体 | 6.259 | 26.872 | 33.131 |
| Radiance除去 | 2.199 | 3.959 | 6.159 |
| Glass除去 | 2.646 | 26.883 | 29.529 |
| 文字のGlassだけ除去 | 5.789 | 27.640 | 33.429 |
| 全エフェクト除去 | 0.625 | 2.519 | 3.144 |
| 文字除去 | 2.199 | 4.047 | 6.247 |
| 図形除去 | 6.234 | 27.444 | 33.678 |

生ログ: [素材比較](../stage5/evidence/frame-cost/ablation.log)、[効果比較](../stage5/evidence/frame-cost/effects.log)。測定器: `motolii/crates/motolii-render/examples/frame_cost.rs`。

## CPUとGPUの実際の分担

- fork `re_renderer/src/renderer/paths.rs`: lyonで塗り／線をCPU三角化し、頂点の色も生成してGPUメッシュへ渡す。
- `engine/texture.rs`: 文字／図形のGPUモデルをcache。同じ輪郭・スタイル・要求精度なら再使用。投影倍率で精度要求が上がると再三角化する。色や輪郭のアニメーションではcache keyが変わり得る。
- fork `shader/mesh_vertex.wgsl`、`instanced_mesh_base.wgsl`: 座標変換・fieldによる頂点変形・surface shadingはGPU。
- CPU三角化はGPU化不可能な処理ではない。VelloはGPU compute中心の既存方式。ただし2D出力をそのまま局所固定textureとして3Dへ貼るだけでは、最終投影まで輪郭を維持する要件を満たさない。GPU化するならこの境界も検証する。
- 一次資料: https://github.com/linebender/vello 、 https://github.com/linebender/gpu-stroke-expansion-paper

## 今回のGPU負荷

`vism/radiance.wgsl` は20パス（光源、jump flood、距離場、6段cascade、合成）。`effects/vism.rs` は中間targetを入力のextentと同じ寸法で確保し、全パスを記録する。毎フレームの解像度・領域・再利用条件の見直しが優先。上流由来は `reference/vgpu-vism.md`。

`ui/native/src/lib.rs` の `render_into` は最後に `wait_indefinitely`。Swiftの `motolii.port.shared-document` で編集と描画が直列なので、GPU待ちが編集要求にも波及する。単にwaitを除くと完成前のIOSurfaceを渡す危険があるため、完了通知と資源寿命を保つ非同期化が必要。

## 次の実装順

1. Radianceの中間解像度・有効領域・不変入力の再利用を、上流方式と画質比較で設計する。勝手に効果を無効化しない。
2. GPU完了待ちを編集要求の直列キューから外す。完成フレームの通知と資源寿命を保つ。
3. 動く輪郭・色・投影精度での再三角化を別計測し、GPU側で保持・処理するパス方式を検証する。今回の継続負荷の改善と混同しない。

## 最新ソースの制約

調査中に依存pinが006ce3ddからa0b0a610へ更新された。最新ソースの比較buildは `EffectProgram::record_in_frame` が無い E0599 で停止。固定済みライブラリの結果を最新ソースの性能とは主張しない。製品の描画方式・効果・キューの変更は本調査では行っていない。
