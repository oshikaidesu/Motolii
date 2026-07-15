//! U0V spike — 実行方法と検証の分界。

## 実行

```bash
cd spikes/u0v-visual
cargo test
U0V_EVIDENCE_DIR=/tmp/u0v-evidence U0V_EVIDENCE_ONLY=1 cargo run
cargo run   # GUI 環境のみ
```

## 自動判定

- DTCG token schema + 3テーマ key parity + contrast ≥ 4.5:1
- 生成物以外の raw color literal 禁止
- 6 情報領域 region-id (asset/preview/property/timeline/transport/context)
- ja/en/pseudo 翻訳 + pseudo 伸長
- timeline wgpu texture + `Image::try_from` ヘッドレス証跡

## 手動確認が残るもの

- v2 モックとの視覚的一致(ピクセル golden 未実施)
- テーマ実行時切替の見た目
- pseudo-locale 実画面での折返しゼロ(GUI 要)
