# M5-A1 GLB preflight compatibility receipt

状態: **PASS / KEEP**（2026-08-02）

## Scope

`gltf`をGLB parser、`mikktspace`をtangent生成のprivate leafとして使えるかを、製品workspace外で検証した。
これはImporter、faithful asset、Document接続、renderer、3 OS、製品P1の完成証拠ではない。

## Fixed sources and licenses

| source | fixed version／commit | checksum | license |
|---|---|---|---|
| `gltf` | crates.io `1.4.1`、調査HEAD `50d65229477fe5f785c2c90df21eb59c93ea2261` | crate `e3ce1918195723ce6ac74e80542c5a96a40c2b26162c1957a5cd70799b8cacf7` | MIT OR Apache-2.0 |
| `mikktspace` | crates.io `0.3.0`、調査HEAD `6275cc4f15cff8be29819fb34ae8be3b9129dae1` | crate `7d0b56b403871a8f992ca626d52cc0a690d4841baea8955dc4af6304ac62f8b0` | MIT OR Apache-2.0 |
| Khronos Asset Generator | release `v0.6.1`、現行調査HEAD `3d99767e9a67fbfe109f0d298c1e8d909bcac9db` | zip `4134d685f6ee4e7a27f3ec826c62566c86cca07ca23edba1150222b04ea074d1`、113,848,238 bytes | MIT |

standalone `Cargo.lock`は22 packageを固定する。`gltf`はdefault `import` featureを無効にし、image decoder、
base64、URI loaderを依存へ入れていない。`mikktspace 0.3.0`の`glam` featureはprivate transitiveとして
`glam 0.15.2`を導入する。これは将来のcamera math用`glam 0.33`と同じ型にせず、公開面へ漏らさない。

## Environment

- macOS 15.5 (24F74), Apple arm64
- rustc 1.96.1 (`31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd`)
- cargo 1.96.1

## Commands

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml
cargo tree --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml --depth 2
```

Khronos corpusは公式release assetを取得し、上記SHA-256を照合して展開した。

```sh
curl -L --fail -o GeneratedAssets-0.6.1.zip \
  https://github.com/KhronosGroup/glTF-Asset-Generator/releases/download/v0.6.1/GeneratedAssets-0.6.1.zip
shasum -a 256 GeneratedAssets-0.6.1.zip
unzip GeneratedAssets-0.6.1.zip -d GeneratedAssets-0.6.1
cargo run --manifest-path spikes/m5-known-implementation/M5-A1/Cargo.toml \
  --bin khronos_corpus -- GeneratedAssets-0.6.1
```

## Oracle results

| oracle | result | evidence |
|---|---|---|
| embedded triangle＋normal map＋tangent欠落 | PASS | MikkTSpaceが3 cornerへ有限・単位長・符号付きtangentを生成 |
| malformed GLB | PASS | panicせず`InvalidGltf` |
| required extension | PASS | validation前に名前を回収し、辞書順の`UnsupportedRequiredExtensions` |
| oversize | PASS | parser起動前に`Oversize` |
| `../escape.bin` | PASS | file accessせず`ExternalResource { kind: "buffer", ... }` |
| JSON glTF at GLB boundary | PASS | `NotBinaryGlb` |
| Khronos positive | PASS | 171件中core 156件を全受理、required-extension 15件を別分類、core reject 0 |
| Khronos negative | EXPECTED LIMIT | 14件を`gltf` validationは全受理。Validatorを別oracleから外せない |

unit testは6/6 PASS、clippy `-D warnings` PASS。warm test commandはmacOS固定機で0.56秒だったが、
単発時間値を性能保証や3 OS証拠にしない。

## Disposition

- `gltf 1.4.1`: **KEEP**。raw `Gltf` APIをprivate preflightへ使い、filesystem convenience importを
  製品policyにしない。
- `mikktspace 0.3.0`: **KEEP**。normal map＋tangent欠落時だけprivate normalizationで使う。
- Khronos Asset Generator: **REMAP**。releaseは`.gltf`＋外部resource corpusなので、embedded-only
  GLBの直接fixtureではなくcapability coverageと負例分類へ使う。
- Khronos Validator: **KEEP / REQUIRED EXTERNAL ORACLE**。parser成功をasset適合へ読み替えない。

## Remaining gates

- この結果は現在のMac一台だけであり、Windows／Linux、32-bit index上限、実巨大asset、decode bomb、
  画像format／色semantic、全supported extension、license収集自動化は未検証。
- GLB-onlyをv1製品入力として確定するか、外部URI付き`.gltf`を許すかは`M5-A0S`の意味decisionへ残す。
- 製品依存追加は別commitとし、faithful private asset、Host resource budget、diagnostic UIを同時に発明しない。
