# reference/vello-blend.wgsl の出所

- `vello_shaders 0.10.0`(crates.io、checksum
  `abf943bd2920bfd22928a9c1bad39866f7ffcc1109b6ed924ab52773e3868a83`)の
  `shader/shared/blend.wgsl` を**原文のまま**置いた。改変していない
- ライセンス: `Apache-2.0 OR MIT OR Unlicense`(先頭の SPDX と同梱の
  `vello-blend.UNLICENSE`)
- **この crate は既に依存ツリーに居る**(`Cargo.lock`: blitz → anyrender_vello →
  vello → vello_shaders)。窓が開いている間、同じ式が同じプロセスで動いている

## なぜ reference に置くのか

2つの役目を兼ねる。

1. **技術の幹の語彙**。`const MIX_*` / `const COMPOSE_*` が W3C Compositing and
   Blending Level 1 の語彙をそのまま持つ。`lottie.schema.json` が Bodymovin 経由で
   AE の意味を運ぶのと同じ関係で、こちらは vello 経由で W3C の技術語彙を運ぶ。
   `tests/compositing_coverage.rs` がここから地図の骨を機械生成する
2. **借りる式そのもの**。`blend_mix_compose(backdrop, src, mode)` は束縛ゼロの
   純関数で、呼び手の器を選ばない(裁定「Motolii は何も持たない」2026-08-30)
