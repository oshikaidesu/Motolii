# 軽量レビュー 2026-07-10 (R8/Vello採否スパイク)

対象: 人間実施のR8スパイク。ドキュメント・使い捨てコード `spikes/vello-eval/`。

> 2026-07-23歴史監査: cutoff全1版と結果報告全1版を[Unit 5G回収](2026-07-23-historical-vello-adoption-lineage-recovery.md)で処分した。採択はdirect wgpuを主、Velloを複雑path／textの局所passとする後続決定へ縮小して維持する。spikeは同一device／textureの成立性であり、現行workspace統合、K6／P6／U3a-2完了、Apple M4時間値のSLO化、semantic goldenの一斉再生成を証明しない。

## 判定: **承認**（採用決定を支持。ブロッカーなし）

### 合格理由

- **最重要ゲート(wgpu29同居)**: vello 0.9.0 / wgpu 29.0.4 の依存一致をCargo.lock相当で確認。`render_to_texture`→同一`wgpu::Texture`でVRAM内完結の経路が成立
- **実レンダ検証**: 矩形・半透明円・usvg三角形のピクセルassertがスパイクにあり、straight alpha判定も明示(`[0,0,255,128]`)
- **統合条件が具体**: Renderer長寿命(~900ms)、straight→premul境界変換、vello_svg不採用+自前アダプタ雛形 — いずれもAGENTS.mdの絶対規律(VRAM常駐・色変換一元化)と矛盾しない
- **M1方針との整合**: R7は自前シェーダで完走、Vello本番統合は凍結ゲート後 — 「細く完走」に合致

### 軽微（記録のみ・ブロッカーではない）

- **スパイクはMetal(M4)のみ実測** — CI(lavapipe)でのVello動作は本番統合チケットで再検証が必要。採否判断自体はヘッドレス+wgpu29一致で十分
- **usvgアダプタは最小形** — グラデ/ストローク/クリップ未対応。M4-K6で段階拡張でよい
- **ファイル名 `s3-vello`** — M0のS3(有理数時間)と番号が紛らわしいが、`docs/README.md`で「S3(R8)」と区別済み

### M1残タスク（タスク台帳更新済み）

| ID | 状態 | 備考 |
|---|---|---|
| R7 | 未着手 | 円/線+add/multiply。Velloは触らない |
| R9 | 未着手 | 実素材+GUI。人間必須 |

### 次便(Cursor/R7)へのプロンプト用メモ

Vello統合は**今回やらない**。将来統合時は [spikes/s3-vello.md](../spikes/s3-vello.md) の条件3つを守る:

1. `Renderer::new` は `RenderSession` 寿命で1回（~900ms）
2. vello出力は straight alpha → 合成境界で premul 化1回
3. `vello_svg` 不使用 — `spikes/vello-eval/src/main.rs` の `append_group`/`to_kurbo` を雛形に
