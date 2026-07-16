# timeline-bench (M3 実装ガード2)

タイムラインを Slint ListView ではなく **wgpu テクスチャ1枚への自前描画**で、
クリップ 1,000 + キーフレーム 100,000 のパン/ズームが 60fps を満たすか検証するスパイク。

製品 `Document`/schema には触れない。`motolii-gpu` はヘッドレス `GpuCtx` 取得のみ。

## 実行

```bash
cd spikes/timeline-bench
cargo run --release
cargo run --release -- --json
```

環境変数:

| 変数 | 既定 | 意味 |
|---|---|---|
| `TIMELINE_BENCH_WARMUP` | 120 | ウォームアップフレーム数 |
| `TIMELINE_BENCH_FRAMES` | 600 | 計測フレーム数 |
| `TIMELINE_BENCH_EVIDENCE` | (なし) | 設定時、最終フレーム PNG を出力 |

## 計測方法

各フレームで `ViewState::animate` によりパン/ズームを更新 → CPU で可視クリップ/キーを抽出 →
インスタンスバッファを GPU へアップロード → 単一レンダパスで 1920×512 テクスチャへ描画 →
`queue.submit` + `device.poll(Wait)` までの **壁時計時間**を記録。

合否: 計測区間の **p95 フレーム時間 ≤ 16.667ms (60fps)**。

## 合否記録

`docs/spikes/timeline-bench.md` (INF-1 形式)
