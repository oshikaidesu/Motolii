# M5-C0 Observation semantics receipt

状態: **PASS / KEEP（private semantic fixture）**（2026-08-02）

## Scope

`glam 0.33.2`をprivate math leafとして、M5-C0意味決定のprojective Observation、typed failure、
provider exchange oracleを検証した。製品workspace、Document、serde、公開API、renderer、M4 resource
ownerには接続していない。

## Fixed source and license

| source | fixed version | license |
|---|---|---|
| `glam` | crates.io `0.33.2` | MIT OR Apache-2.0 |

## Commands and results

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-C0/Cargo.toml
```

| oracle | result |
|---|---|
| Planar compatibility projection／no parallax | PASS |
| Perspective projective projection／depth parallax | PASS |
| missing provider／version mismatch typed refusal | PASS |
| failed provider swap: binding／undo unchanged | PASS |
| invalid output dimensions rejected before output | PASS |
| unit test | **5/5 PASS** |
| clippy `-D warnings` | PASS |

## Disposition and remaining gates

Private semantic fixtureを**KEEP**する。fixture内の`ProviderPin`、`Observation`、matrixは製品型や
wire identityではない。Camera Object／active bindingのDocument schema、provider package／entry／version
identity、M4 K1a resource owner、3 OS／GPU／Preview／Export、製品runtimeは未成立であり、次は`C0-Schema`
を別契約境界で仕様化する。
