# 凍結ゲート残件(2026-07-10 監査)

対象: M1 R1–R9消化・プラグイン境界入場チェック([2026-07-10-M1-plugin-boundary-review.md](2026-07-10-M1-plugin-boundary-review.md))完了後。
観点: [pitfalls G-1入場条件](../pitfalls-and-roadmap.md)テーブルのコード実証。
**宣言**: [2026-07-10-freeze-gate-declaration.md](2026-07-10-freeze-gate-declaration.md)

修正したらチェックを入れ、修正コミットにこのファイルの項番(FG-C1等)を書くこと。全緑で全面凍結宣言→M2並列解禁。

## 達(段階凍結可)

- [x] F-10 PipelineCache + TintFilter(`tint_filter_uses_pipeline_cache_without_recompile`)
- [x] Filter の `RenderStep::Plugin` レジストリ経由ゴールデン
- [x] ParamDriver 製品経路(E2E DataTrack)
- [x] `ValueType::AssetRef` / `NodeDesc` メタデータ予約
- [x] Overlay 正準座標の解像度横断ゴールデン(T7/R7)

## 残チケット(実装対象)

| ID | 内容 | G-1項目 | 完了条件(自動判定) |
|---|---|---|---|
| FG-C1 | Composite を `RenderStep::Plugin` 経由でゴールデン | プラグインtrait | `core.composite.normal`(または同等)がレジストリ経由で premul over し、直呼び `CompositeNormal` と一致 |
| FG-C2 | TimeMap を VideoSource/export 製品経路に通す | F-4 | `BackgroundTextureRequest`/`ExportOverlayRequest` が `try_map` で source_time を解決。非恒等写像のテストが通る。**実デコード再写像はスコープ外(M2)** |
| FG-C3 | F-2 単一writer最小骨格 | F-2 | `motolii-doc`: `DocumentWriter` のみが `&mut Document` を持ち、読み手は `Arc<Document>`。型/単体テストで確認 |
| FG-C4 | param 移行枠 + 旧JSON roundtrip | param同一性 | 参照プラグインの param 改名を `migrate` で吸収し、旧JSONがロード→現行スキーマで壊れない |
| FG-C5 | Draft/Final 正準一致ゴールデン | F-1 | 同一グラフの Draft(半解像度)と Final で、非背景ピクセル重心の正準座標が一致(許容誤差内) |
| FG-C6 | 口の予約(型のみ) | F-7/F-11 | `InstanceIndex` / `CompLookbehind` を Rust 型として予約(配線は後) |

## 実装状況(2026-07-10)

- [x] FG-C1 `plugin_composite_dispatches_via_registry_golden`
- [x] FG-C2 `BackgroundTextureRequest.time_map` + export/ProjectV1 貫通
- [x] FG-C3 `motolii-doc` DocumentWriter骨格
- [x] FG-C4 `migrate_plugin_params` + `old_sine_amp_param_migrates_on_load`
- [x] FG-C5 `draft_and_final_share_canonical_overlay_centroid`
- [x] FG-C6 `InstanceIndex` / `CompLookbehind` 型+serde

## スコープ外(M2以降・明示)

- 未知プラグインIDの警告+パススルー(F-9) — M2-D1
- GpuAssetCache 結線 — M2
- F-3 全文(マスク/グループ) — M2-D3/D7
- TimeMap の可変速・逆再生 — スキーマ互換の将来拡張
- Document の本スキーマ(トラック/クリップ) — M2-D1。FG-C3は所有権骨格のみ
- **TimeMapの実デコード/シーク再写像** — 報告口(`try_map`)のみ凍結。ピクセル供給の再写像はM2

## 宣言方針

1. FG-C1〜C6 全緑 + レビュー3点(F-2参照非漏洩・TimeMap Result・F-4スコープ明示)対応済み
2. **宣言文書**: [2026-07-10-freeze-gate-declaration.md](2026-07-10-freeze-gate-declaration.md)(解凍手続き付き)
