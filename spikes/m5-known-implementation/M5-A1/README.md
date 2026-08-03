# M5-A1 GLB preflight compatibility probe

M5の製品runtimeから隔離した、`gltf`と`mikktspace`のprivate leaf互換性検証である。
Document、serde、公開API、renderer、ルートworkspaceの`Cargo.lock`には接続しない。

## 検証する境界

- 入力byte上限をparse前に拒否できる。
- JSON glTFをGLBへ偽装せず、embedded-only GLB境界で外部buffer／image URIを拒否できる。
- malformed GLBをpanicせず型付きで拒否できる。
- `extensionsRequired`を名前付き・安定順で拒否した後、通常schema validationを省略しない。
- normal mapがありtangentがないtriangleへMikkTSpace tangentを生成できる。
- Khronos Asset Generatorのcore positive corpusとnegative corpusを別集計し、parserとValidatorの
  証明範囲を混同しない。

## Commands

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml
cargo run --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml \
  --bin khronos_corpus -- /path/to/GeneratedAssets-0.6.1
```

固定source、取得方法、実測値、採否は
[receipt](../../../docs/reviews/evidence/m5-known-implementation/M5-A1/README.md)を正とする。
