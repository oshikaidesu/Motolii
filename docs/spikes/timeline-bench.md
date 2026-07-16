# タイムライン1枚描画ベンチスパイク結果 (M3 実装ガード2)

作成日: 2026-07-15 / Issue #57

## 結論 (M3 実装ガード2 / Issue #57 **合格**)

記録形式は [s1-slint.md](s1-slint.md)(INF-1) を参考にした。**本スパイクのラベルは INF-1 ではない**。

クリップ **1,000** + キーフレーム **100,000** の合成データで、パン/ズーム更新時の wgpu 自前描画が **p95 ≤ 16.667ms (60fps)** を満たす。Slint ListView / 大量 Slint エレメントでタイムラインを組む方針は採用しない（M3 実装ガード2どおり）。

| 層 | 内容 | 状態 | 証拠 |
|---|---|---|---|
| 1 構造 | 1枚の `wgpu::Texture` へインスタンス矩形描画・ビューポート CPU カリング | **合格** | [bench-manifest.json](timeline-bench-evidence/bench-manifest.json) + [frame-sample.png](timeline-bench-evidence/frame-sample.png) |
| 2 性能 | パン/ズーム 600 フレーム計測、p95 60fps 以上 | **合格** | 下表・[bench-report.json](timeline-bench-evidence/bench-report.json) |
| 3 U3 含意 | カスタム描画面 + Slint はイベント/シェル | **合格** | 本ドキュメント「U3への含意」 |

再現: `cd spikes/timeline-bench && cargo run --release`（ヘッドレス可。Vulkan ICD 要。CI では `mesa-vulkan-drivers`）

## 方針 (仕様どおり)

- **Slint ListView は使わない** — タイムライン/波形/グラフは 1 枚のカスタムレンダリング面 (wgpu テクスチャ) に自前描画
- プレビューと同様、完成テクスチャは `slint::Image::try_from` で埋め込む想定（本スパイクは Slint 非連結・描画コアのみ計測）
- 製品 `Document`/schema 変更なし (`spikes/` 完結)
- wgpu **29** (workspace 固定)

## 計測方法 (明記)

| 項目 | 値 |
|---|---|
| データ | 32 トラック・クリップ 1,000・キーフレーム 100,000（クリップあたり約 100 キー） |
| ビューポート | 1920×512 px |
| 操作 | 各フレーム `ViewState::animate` でパン(秒・トラック)とズーム(0.55〜1.0)を正弦変調 |
| 描画 | CPU で可視クリップ/キー/グリッドを抽出 → インスタンス SSBO → 単一レンダパス |
| フレーム時間 | `draw_frame` 全体の壁時計（カリング+upload+`submit`+`poll(Wait)`） |
| ウォームアップ | 120 フレーム |
| 計測 | 600 フレームの中央値・p95 |
| 合否 | **p95 ≤ 16.667ms** |

## 実測 (2026-07-15)

環境: Linux x86_64 クラウドエージェント / **llvmpipe (Mesa Vulkan, ソフトウェアレンダラ)**

| 指標 | 値 |
|---|---|
| median | **8.78 ms** (113.9 fps) |
| p95 | **16.00 ms** (62.5 fps) |
| CPU カリング+upload (最終フレーム) | ~7–8 ms |
| 可視インスタンス (最終フレーム例) | 210 clips + 20,005 keyframes (+ グリッド) |
| 合否 | **PASS** |

llvmpipe での p95 は **16.00ms / 予算 16.667ms（マージン約 4%）** であり、余裕付きではない。実 GPU は未実測（速い蓋然性はあるが本記録の根拠にしない）。**アーキテクチャ判断 (ListView 回避 + wgpu 1枚面) は本スパイクで確定**する。U3 着手前に開発主機 GPU で再実測し、マージンを記録することを推奨する。

再現コマンド（証跡一式）:
```bash
cd spikes/timeline-bench
cargo run --release -- --json > ../../docs/spikes/timeline-bench-evidence/bench-report.json
TIMELINE_BENCH_EVIDENCE=../../docs/spikes/timeline-bench-evidence cargo run --release
```

## U3 への含意

1. **タイムライン UI は wgpu 自前描画クレート**（`ui` シェル外・Slint 非依存）として実装し、`Image::try_from` で埋め込む
2. **インスタンス矩形 + ビューポート CPU カリング**で 100k キー規模は現実的。ズームアウト時の可視キー増加は今後の LOD（間引き表示）候補だが、本ベンチ条件では未満足でも合格
3. U3 完了条件に「本スパイク合否を満たす描画経路を使用すること」を追加可能（本記録が根拠）

## 不合格時の代替案 (参考・今回は不適用)

本 run は合格のため採用しないが、仕様どおり記録する:

| 代替 | 内容 |
|---|---|
| A | GPU コンピュート / 間接描画でカリングを GPU へ移し CPU upload を削減 |
| B | キーフレーム LOD（ズームアウト時はクラスタ表示・最大描画数キャップ） |
| C | 空間インデックス (時間軸 B-tree) で可視クエリを O(log n) 化 |

## 検証コード

- `spikes/timeline-bench/` — README 参照
- 証拠: `docs/spikes/timeline-bench-evidence/`

## 関連

- [M3-ui-integration.md 実装ガード2](../specs/M3-ui-integration.md)
- [s1-slint.md](s1-slint.md) (INF-1 記録フォーマット参考)
- Issue #57
