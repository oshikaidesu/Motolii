# M5-D0 stable instance evaluator receipt

状態: **PASS / KEEP（test-only meaning fixture）**（2026-08-02）

## Scope

explicit `source_id`／`parent_id`／`depth`と`user_seed`からslot key、semantic `InstanceId`、3 channelを
決定するprivate fixtureを検証した。count増減、reorder、nested parent、thread順、typed invalid inputと
golden vectorを含む。これはP0I/P7 schema、Document persistence、Behaviour runtime、3 OS実測、製品
Duplicator接続の完了証拠ではない。

## Fixed sources and licenses

| source | fixed version | checksum | license |
|---|---|---|---|
| `rand_pcg` | crates.io `0.10.2` | `caa0f4137e1c0a72f4c651489402276c8e8e1cf081f3b0ba156d2cbeef09e86a` | MIT OR Apache-2.0 |
| `rand_core` | crates.io `0.10.1` | `63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69` | MIT OR Apache-2.0 |

standalone `Cargo.lock`は8 packageを固定する。seed mixerはprobe-ownedで、`sha2`やOS entropyを導入しない。

## Commands and oracle results

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-D0/Cargo.toml
```

| oracle | result | evidence |
|---|---|---|
| count growth／reorder | PASS | source ID単位のInstanceId／channelsが追加・並べ替えで不変 |
| nested parent | PASS | parent／depthをkeyへ含め、同sourceでも親が違えばidentityが変わる |
| thread order | PASS | 同じ入力を別threadで評価してsource mapが一致 |
| invalid input | PASS | source 0、nested parent 0をtyped errorで拒否 |
| golden vector | PASS | seed 77／source 11／parent 1でInstanceId `237390148889641753`と3 channelを固定 |

## Disposition

- `rand_pcg 0.10.2`: **KEEP / PRIVATE LEAF**。P0I/P7の候補乱数として使用可能性を確認。
- owned stable mixer: **KEEP / REFERENCE FIXTURE**。入力shape→slot key→InstanceIdの責任を明示する。
- `sha2` seed hash、OS entropy、時計、thread／GPU順: **REJECT**。identityの入力へ混ぜない。
- Product schema／Document／Behaviour: **REDUCE / WAIT**。fixtureの意味を公開永続形式へ昇格しない。

## Remaining gates

P0I/P7の仕様decision、migration／roundtrip、Behaviour cache、Instance channel型、3 OS golden、製品
Duplicator／Document接続、M5-A0Sは未完了である。
