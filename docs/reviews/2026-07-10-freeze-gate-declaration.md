# 凍結ゲート宣言(2026-07-10)

ステータス: **宣言**

根拠: M1 R1–R9消化、プラグイン境界入場チェック全緑、G-1入場条件のコード実証 FG-C1〜C6全緑([2026-07-10-freeze-gate-remaining.md](2026-07-10-freeze-gate-remaining.md))。
これは外部ユーザーへの semver 1.0 約束ではなく、**並列エージェントへの内部契約**である。

## 凍結する口(実証済み・狭い約束)

| # | 項目 | 凍結面(狭い) | 明示的に凍結しないもの |
|---|---|---|---|
| 1 | `FrameDesc` / 色・premul | wgpu実装で検証した形 | — |
| 2 | プラグインtrait + 純関数契約 | Filter/ParamDriver/Compositeのレジストリ経由ディスパッチ、出力= f(t, inputs, params) | dylib/WASM(v2)、lifetime/`&'static`(項目16) |
| 3 | ドキュメントスキーマ枠 | version + migrate 枠組みの存在 | 本スキーマ(トラック/クリップ)=M2-D1 |
| 4 | `RationalTime` / `ParamEval` | 有理数時間・キーフレーム/DataTrack評価API | — |
| 5 | Cargo workspace分割 | クレート境界 | — |
| 6 | 正準座標系(F-1) | 単位なし・原点中央・Y-up・高さ=1、Overlay実証+Draft/Final重心一致 | — |
| 7 | 所有権(F-2) | 単一writer + `Arc`スナップショット。`DocumentWriter::edit`は参照を外に返さない | 本Documentスキーマ・並行テストの本実装=M2-D8 |
| 8 | 単一評価モデル(F-3) | M1部分集合: ソース→オーバーレイ→合成 | マスク/グループ/変形スタック全文=M2 |
| 9 | TimeMap(F-4) | **報告口**: `try_map`でsource_timeを解決する契約。型+恒等/offset/定数速度 | **実デコード/シーク再写像=M2**(未実証のため固めない) |
| 10 | インスタンスインデックス(F-7) | 型`InstanceIndex`の予約 | 配線・Repeater実証=M2以降 |
| 11 | プラグイン表示メタデータ(F-8) | `NodeDesc`の display_name/category/tags/version | GUIブラウザ=M3 |
| 12 | プラグイン可搬性(F-9) | 安定文字列ID+version | 未知IDパススルー=M2-D1 |
| 13 | plugin-authoring規約 | 禁止事項を契約として読む | — |
| 14 | PipelineCache(F-10) | ホスト所有キャッシュ+TintFilter実証 | GpuAssetCache結線=M2 |
| 15 | AssetRef(F-10) | `ValueType::AssetRef`予約 | Importer実装・点群=将来 |
| 16 | lifetime非凍結 | id/paramのセマンティクスのみ安定。`&'static`→String化は互換変更 | — |
| 17 | CompLookbehind(F-11) | 型予約 | 配線=M4後 |

## 解凍手続き(改訂の扉)

凍結は「変更禁止」ではない。**変更はこの扉を通る**:

1. **変更理由と実証** — なぜ広げる/狭めるか、ゴールデンまたはレビュー所見で示す
2. **旧スキーマの migrate 経路** — `migrate_plugin_params` / ドキュメントversion移行など、旧データを壊さない経路を同時に出す(FG-C4の型紙)
3. **影響ゴールデンの更新** — 壊れるテストを新契約に合わせて緑にする

この3点セットが無い改訂PRは受けない。外部ユーザーがいない今、この扉は並列エージェント間の現実の手続きとして機能する(Rust edition / Blender `do_versions` のミニ版)。

## 並列化の解禁

本宣言をもって **M2〜M5 の並列レーンを開いてよい**。ただし上表「明示的に凍結しないもの」を勝手に製品経路へ焼き込まないこと。未実証の口を広げたくなったら、実装を止めて仕様改訂(解凍手続き)を先に。

既知の解凍手続き待ち(2026-07-10マージ時点): **F-12 時間軸自由度の口の予約**(`SimulationPlugin` trait叩き台・`TemporalFootprint`・スキーマのシムノード席。[simulation-model.md](../simulation-model.md)、backlog FG-2)。設計は確定済みだが、項目2(プラグインtrait)と項目3(スキーマ枠)への口の追加になるため本手続きを通す。

実施済み: **M2E-7 `RenderCtx`**(2026-07-12、[解凍記録](2026-07-12-M2E-7-render-ctx-thaw.md)) — Filter/Composite に Quality と予約口(InstanceIndex/CompLookbehind/TemporalFootprint)を `#[non_exhaustive]` 文脈へ畳んだ。F-12 の NodeDesc 宣言・SimulationPlugin・スキーマ席は未実施。

## 参照

- G-1入場条件: [pitfalls-and-roadmap.md](../pitfalls-and-roadmap.md)
- 残件消化記録: [2026-07-10-freeze-gate-remaining.md](2026-07-10-freeze-gate-remaining.md)
- プラグイン境界入場: [2026-07-10-M1-plugin-boundary-review.md](2026-07-10-M1-plugin-boundary-review.md)
