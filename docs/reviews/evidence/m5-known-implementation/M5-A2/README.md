# M5-A2 OBJ lowering compatibility receipt

状態: **PASS / KEEP（private leaf）**（2026-08-02）

## Scope

`tobj`でOBJをM5-A1後段のprivate faithful asset境界へlowerできるかを、独立probeで検証した。
製品workspace、Document、serde、公開API、renderer、M4 resource ownerには接続していない。

## Fixed source and license

| source | fixed version | license |
|---|---|---|
| `tobj` | crates.io `4.0.5` | MIT |

standalone `Cargo.lock`はprobe内だけに置き、root workspaceの依存解決へ追加していない。

## Commands

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/m5-known-implementation/M5-A2/Cargo.toml
```

## Oracle results

| oracle | result |
|---|---|
| triangleをsingle-index private meshへlower | PASS |
| normal／UVありのoptional attribute保持 | PASS |
| normal／UV欠落を`None`で保持し推測値を生成しない | PASS |
| MTL欠落／未解決参照を`MaterialBinding::Missing`で保持 | PASS |
| malformed OBJ | PASS。`ObjLoweringError::Parse` |
| clippy `-D warnings` | PASS |
| unit test | **4/4 PASS** |

## Disposition

- `tobj 4.0.5`: **KEEP（private OBJ input leaf）**。A1のfaithful asset境界へlowerする入口候補とする。
- MTLの値をglTF PBRへ変換しない。OBJ／MTLの忠実性、画像色意味、normal map、URI／size policyは別の
  importer admission fixtureで閉じる。
- OBJを製品入力へ追加する依存、Asset cache、GPU compiled asset、公開型、3 OS適合は未成立。

## Remaining gates

`M5-A0S`で未決のGLB-only／外部URI入力意味を再決定しない。A2の結果はP1 faithful assetの
private lowering候補を閉じただけで、M5-C0 Observation、M4 resource gate、製品runtime接続を解禁しない。
