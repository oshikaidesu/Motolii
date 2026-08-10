# M5-R0 headless PBR／unlit probe

製品rendererへ接続しないstandalone wgpu 29 probe。1つのoffscreen pipelineでunlit、dielectric、metal、
normal、emissiveを描き、CPU referenceとreadbackを比較する。environmentは入力として明示し、
adapter／limit不足はtyped refusalにする。

追加のprivate二pass fixtureは、透明clear上の投影済みmaterial三角形をRGBA textureへ保持し、
composition正規化座標のhalftoneをfullscreen passで適用する。dot coverageは入力alphaと乗算するため、
3D silhouette外へ漏れない。frequencyは出力高さ当たり8 cellで、解像度ではなくcomposition内の配置を固定する。

Glowのprivate fixtureは`Rgba16Float` sourceからbright-pass、半径2の横縦blur、additive compositeを行う。
source／2枚のtransient／output、pipeline、bind group、readbackはfixture生成時に一度だけ作り、連続評価で
再利用する。1.0超highlight、haloによるalpha extent拡張、radius外透明をreadbackで自動確認する。

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml
```

これはglTF PBR conformance、Blender parity、M4 resource budget、Preview／Export接続、binary dot edgeの
anti-alias、Glow pyramid／tone mappingを証明しない。readbackは自動oracle専用であり、製品filterのCPU
pixel routeではない。
