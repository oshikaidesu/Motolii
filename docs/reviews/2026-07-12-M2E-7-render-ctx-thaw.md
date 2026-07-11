# M2E-7 解凍手続き: Filter/Composite へ `RenderCtx` を導入する

日付: 2026-07-12  
対象: 凍結ゲート項目2(プラグインtrait) — [宣言](2026-07-10-freeze-gate-declaration.md)の解凍手続き3点セット

## 1. 変更理由と実証

**理由**: Filter/Composite の `render` が `t` を裸引数で受け、`Quality`・F-7 `InstanceIndex`・F-11 `CompLookbehind`・F-12 `TemporalFootprint` の口が無い。引数を増やすたびに全プラグインが破壊される(監査 P-1 / 入場条件 M2E-7)。`render_graph_cached` は Quality を解像度に畳み込むため、プラグインは TextureRef.desc から Draft/Final を判別できない。

**実証**:
- 参照 Filter/Composite を `ctx: &RenderCtx` 経由に改訂し、既存ゴールデン・purity・workspace テストが不変で緑
- `RenderCtx::new(t, quality)` でホストが Quality を渡し、予約フィールドはデフォルト(未配線)
- `#[non_exhaustive]` により以降の予約口追加はシグネチャ非破壊

## 2. 旧データの migrate 経路

- **Document / ProjectV1 JSON**: 変更なし(永続スキーマに触れない)
- **プラグインパラメータ**: 変更なし(`migrate_plugin_params` 不要)
- **ソースAPI**: コンパイル時改訂。Filter/Composite 実装は `t: RationalTime` → `ctx: &RenderCtx` に機械置換し、時刻は `ctx.t`、品質は `ctx.quality`

## 3. 影響ゴールデンの更新

- **期待**: ピクセル出力は不変(参照プラグインは Quality/予約を未使用)
- **実施**: ゴールデン参照画像の更新なし。既存 `cargo test --workspace --locked` 全緑で確認

## 契約の要約

```rust
#[non_exhaustive]
pub struct RenderCtx {
    pub t: RationalTime,
    pub quality: Quality,
    pub instance: Option<InstanceIndex>,       // F-7 予約
    pub lookbehind: Option<CompLookbehind>,    // F-11 予約
    pub temporal_footprint: TemporalFootprint, // F-12 予約(ゼロ窓 default)
}
```

Filter/Composite 以外(LayerSource / ParamDriver)は現行 Context 型のまま。
