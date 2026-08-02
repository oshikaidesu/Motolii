# M5-I0 dense picking comparison

製品workspace外のCPU fixture。10,000 primitiveのdense sceneで、obvhs CWBVHとowned flat traversalの
semantic hitを比較し、generation付きreadback tokenでstale結果を即時拒否する。GPU traversalやStage／
selection接続はこのprobeに含めない。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml
```
