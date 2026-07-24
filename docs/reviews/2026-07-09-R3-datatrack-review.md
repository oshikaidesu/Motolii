# コードレビュー所見 2026-07-09 (R3/DataTrack統合)

2026-07-23歴史監査: cutoff全3版を[Unit 5M回収](2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md)で処分した。この[x]は当時のbranch到達を示す。未知paramのfail-closed、duration fallback＋typed indeterminate、`VideoSource`語彙は現行にも残る。一方、export区間は後続M2E-17の半開総尺が正本で、Vec2Axes fallbackはvalidation後の防御へ縮小する。testkit集約は方針を維持するが、後発media testにlocal `tmp_dir`が再発しており全workspace恒久達成とは数えない。

対象: R3/R5/R6統合ブランチ。軽量パスで承認済み。次便で対応した軽微3件。

## 軽微（次便対応 → 完了）

- [x] **1. 未知パラメータキーの黙殺** — `build_data_tracks`でプラグイン未定義キーを`ProjectError::UnknownParam`で拒否
- [x] **2. nb_frames欠落時のDataTrack duration=0** — `export_frame_count`が`duration`へフォールバック、両方無ければ`IndeterminateExportLength`
- [x] **3. Vec2Axesの型不一致が0.0** — `eval_scalar_axis`でDataの`fallback`を使用

## 同便追加

- [x] **R4** — `ExternalTexture`→`VideoSource`リネーム、`linear_graph_with_video_source`
- [x] **testkit集約** — `gpu_or_skip`/`tmp_dir`を`motolii-testkit`へ

## 現行での読み方

- `build_data_tracks`は現行PluginRuntime／NodeDesc解決を通り、未知plugin／param／型不一致をtyped errorで拒否する。旧ProjectV1 APIを製品正本へ戻さない。
- `export_frame_count`は`nb_frames`を優先し、欠落時はexact rational durationをfloorして半開総尺を得る。旧「最終PTS+1」解釈は復活させない。
- Data入力のfallbackはmissing runtime data時の防御であり、Documentの型不一致をsilent coercionする規則ではない。
- `motolii-testkit::tmp_dir`は存在するが、`motolii-media/tests/framereader_cancel.rs`と`roundtrip.rs`にlocal helperが残る。回収commitへcleanupを混ぜない。
