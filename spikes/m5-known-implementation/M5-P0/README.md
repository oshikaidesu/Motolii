# M5-P0 post algorithm fixture

製品workspace外のCPU fixture。blurのRoI padding／Unknown全域、linear-light計算、LGG系の線形調整、
seed付きgrain、Draft／Final品質とPreview／Export同一評価関数を比較する。wgpu passや製品filter graphは
このfixtureへ持ち込まない。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml
```
