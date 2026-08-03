# M5-I0 dense picking receipt

状態: **PASS / KEEP + REDUCE**（2026-08-02）

## Scope

10,000 primitiveの同一triangle fixtureをowned flat traversalと`obvhs 0.3.2` CWBVHへ通し、semantic
IDと距離の一致、generation付きstale拒否を比較した。これはGPU traversal、async readback実測、Stage／
Transient selection接続、bounds／gizmo公開契約、3 OS性能保証を証明しない。

## Fixed sources and licenses

| source | fixed version | checksum | license |
|---|---|---|---|
| `obvhs` | crates.io `0.3.2` | `3194269f7697a676e6b6d93b3a8c9558727ce8f2425c6fdd01b6e364fdbbdb2e` | MIT OR Apache-2.0 |
| `glam` | crates.io `0.33.2` | `7f22fb22f065b308be0d8724e3706c7fa3fc2a6c7d6899df4cad7860e7a75436` | MIT OR Apache-2.0 |

standalone `Cargo.lock`は89 packageを固定する。`glam`はobvhsとの型境界を閉じるため同じprivate probe内で
固定し、Motolii公開型へ漏らさない。

## Commands and oracle results

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-I0/Cargo.toml
```

| oracle | result | evidence |
|---|---|---|
| dense scene | PASS | 10,000 primitiveを生成し、6本のcamera rayを比較 |
| owned flat vs obvhs | PASS | semantic ID一致、距離差0.0001未満 |
| non-index identity | PASS | IDは`index`ではなく`index * 17 + 3`のtyped semantic value |
| stale generation | PASS | requested/current/token generation不一致を`StaleGeneration`で即時拒否 |
| readback stall | REDUCE | 待機／GPU readbackをfixtureへ入れず、stale resultをcommitしない状態機械だけを検証 |

## Disposition

- `obvhs 0.3.2`: **KEEP / PRIVATE ADOPTION PROBE**。dense CPU BVHの候補として再利用する。
- owned flat traversal: **KEEP / REFERENCE ORACLE**。BVH採択後もsemantic hitの比較基準として残す。
- Rerun-style async GPU traversal: **PATTERN / REDUCE**。GPU readbackとgeneration ownerが閉じるまで採用しない。
- `parry3d`: **REJECT**。このdense object picking境界へ物理／collision意味を追加しない。

## Remaining gates

moving cameraの連続frame、GPU／CPU readback latency、stale generationの実GPU証拠、bounds contract、same
semantic IDのStage projection／Transient selection、3 OS、M5-A0Sは未完了である。
