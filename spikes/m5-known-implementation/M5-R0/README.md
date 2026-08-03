# M5-R0 headless PBR／unlit probe

製品rendererへ接続しないstandalone wgpu 29 probe。1つのoffscreen pipelineでunlit、dielectric、metal、
normal、emissiveを描き、CPU referenceとreadbackを比較する。environmentは入力として明示し、
adapter／limit不足はtyped refusalにする。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml
```

これはglTF PBR conformance、Blender parity、M4 resource budget、Preview／Export接続を証明しない。
