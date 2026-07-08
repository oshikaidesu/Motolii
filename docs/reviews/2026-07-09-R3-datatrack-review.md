# コードレビュー所見 2026-07-09 (R3/DataTrack統合)

対象: R3/R5/R6統合ブランチ。軽量パスで承認済み。次便で対応した軽微3件。

## 軽微（次便対応 → 完了）

- [x] **1. 未知パラメータキーの黙殺** — `build_data_tracks`でプラグイン未定義キーを`ProjectError::UnknownParam`で拒否
- [x] **2. nb_frames欠落時のDataTrack duration=0** — `export_frame_count`が`duration`へフォールバック、両方無ければ`IndeterminateExportLength`
- [x] **3. Vec2Axesの型不一致が0.0** — `eval_scalar_axis`でDataの`fallback`を使用

## 同便追加

- [x] **R4** — `ExternalTexture`→`VideoSource`リネーム、`linear_graph_with_video_source`
- [x] **testkit集約** — `gpu_or_skip`/`tmp_dir`を`oc-testkit`へ
