# M5-C0 private Observation semantics probe

M5-C0意味決定のconformance候補を製品workspaceから隔離して検証する。`glam`と独自のprivate
fixture型だけを使い、Document、serde、公開API、plugin facade、renderer、M4 runtimeへ接続しない。

## Verified boundary

- Planar compatibility projectionはdepthで画面位置を変えない。
- Perspective projective projectionはZに応じたparallaxを持つ。
- provider missing／version mismatch／invalid requestをtyped failureへする。
- provider swapはpreflight成功時だけ1 undoを積み、失敗時bindingとundoを不変にする。
- private fixtureはray／differential／shutter、provider wire、Document schemaを持たない。

## Commands

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml
```

## Boundary

これは意味fixtureであり、`glam`の製品依存追加、Observation公開型、Camera Object schema、
provider identity wire、M4 K1a resource owner、3 OS／GPU／Preview／Exportの証明ではない。
