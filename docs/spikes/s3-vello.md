# S3(R8): Vello採否評価 結果報告

日付: 2026-07-10
検証コード: `spikes/vello-eval/`(使い捨て、ルートworkspaceから隔離)

## 結論: **採用**(条件3つ付き)

ベクター描画基盤(SVG読み込み+プロシージャル図形)としてVello+usvgを採用する。
Apple M4(Metal)でのヘッドレス実測に基づく。

## 判定の根拠

1. **wgpuバージョン互換(最重要ゲート): 合格**
   vello 0.9.0はwgpu `^29.0.3`依存で、本体固定のwgpu 29.0.4と完全一致。同一`wgpu::Device`で
   本体のレンダパスとvelloが同居し、`render_to_texture`で`wgpu::Texture`へ直接描ける(CPUバウンスなし)
2. **実レンダ確認**: 矩形・半透明円・usvg経由のSVG三角形をRgba8Unorm(STORAGE_BINDING)ターゲットへ
   レンダし、読み戻しでピクセル値一致を確認(`cargo run --release`でPASS)
3. **usvg 0.47**: 保守中(2026-02更新)。パース→velloシーンへの変換は自前アダプタ約60行で成立を確認

## 条件(統合時に守ること)

1. **`Renderer::new`は約900msかかる(シェーダ初期化)** — 毎フレームはもちろん、毎エクスポートでも
   作り直さない。`RenderSession`と同じ寿命で1回だけ生成して使い回す。macOSは
   `num_init_threads: Some(1)`(vello公式推奨)。`pipeline_cache`の口もあるので起動最適化はv2で検討可
2. **出力はstraight alpha**(実測: 半透明青が`[0,0,255,128]`)。本体の正規形はpremultipliedなので、
   **vello出力→合成の境界でstraight→premul変換を1回入れる**(既存の「UI入力色はstraightで受けて
   境界でpremul化」規約と同じ扱い)
3. **vello_svgは使わない** — 最新0.9.0がvello `^0.7`固定でvello 0.9と型が合わない。
   usvg→velloの変換層は自前で書く(スパイクの`append_group`/`to_kurbo`が雛形。
   v1スコープはパス+単色fill+透明度+変形。グラデ/クリップ等は必要になった時に追加)

## リスク(A-3の拡張)

バージョン結合が **Slint↔wgpu↔vello の三者**になった。wgpuメジャー更新はSlintとvelloの
両方が追従してから(現在は三者とも29系で整合)。references.mdのバージョン方針に追記済み。

## 実測値(Apple M4, Metal, 64x48ターゲット)

| 項目 | 値 |
|---|---|
| `Renderer::new`(初回のみ) | ~895ms |
| render+読み戻し | ~18ms(読み戻し込み。実運用はVRAM内で完結するためさらに軽い) |
| 依存解決 | vello 0.9.0 / usvg 0.47.0 / wgpu 29.0.4 で衝突なし |
