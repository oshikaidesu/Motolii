# M5-R0 headless PBR／unlit compatibility receipt

状態: **PASS / KEEP**（2026-08-02）

## Scope

製品renderer外のstandalone `wgpu 29.0.4` pipelineで、unlit／dielectric／metal／normal／emissiveと
固定environment入力を同じoffscreen経路へ通した。これはglTF Sample Renderer／Blenderとの画像一致、
PBR conformance、M4 budget、Preview／Export、Layer Order接続を証明しない。

## Fixed sources

| source | version | checksum | license |
|---|---|---|---|
| `wgpu` | crates.io `29.0.4` | `76e8840e1ba2881d4cbb18d2147627a56af426ff064c0401eb0c8410c6325d07` | MIT OR Apache-2.0 |
| `bytemuck` | crates.io `1.23.1` | `5c76a5792e44e4abe34d3abf15636779261d45a7450612059293d1d2cfc63422` | MIT OR Apache-2.0 |

Khronos Sample Renderer／Blenderは参照資料であり、依存またはoracle実装として持ち込んでいない。

## Commands and result

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-R0/Cargo.toml
```

- `cpu_reference_covers_material_matrix`: PASS
- `gpu_matches_cpu_reference_or_returns_typed_refusal`: PASS
- macOS 15.5 / Apple arm64でadapter取得とGPU readbackを確認
- `Rgba8Unorm` 32×32、copy row 256 bytes、single render pipeline
- CPU referenceとの差分は各channel 2以内。既存goldenや製品thresholdは変更していない

## Negative oracle and boundaries

- adapter不在: `AdapterUnavailable`
- `max_texture_dimension_2d`／storage limit不足: `InsufficientLimits`
- device／pipeline／readback失敗: typed error
- uniformのenvironmentが不成立時にambient／unlitへ黙って縮退する経路は作っていない
- scene color format、neutral environmentの製品byte、resource budget、PBR extension受理範囲は未決のまま

## Disposition

- wgpu offscreen／pipeline／readback: **KEEP / REUSE**
- renderling: **未実行・任意比較のまま**。R0のgateやoracleにしない
- Khronos Sample Renderer／Blender: **PATTERN**。数値・shader・scene ownerを移植しない
- このprobeのshader／CPU reference: **probe-only**。製品material systemへ移植しない

## Remaining gates

Windows／Linux、low-spec hard floor、normal-map texture decode、metallic-roughness conformance、linear
scene-color／premultiplied合流、M4 lifecycle、Layer Order／Group Depth接続は未検証である。
