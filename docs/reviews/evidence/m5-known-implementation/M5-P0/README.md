# M5-P0 post algorithm fixture receipt

状態: **PASS / KEEP（algorithm contract）**（2026-08-02）

## Scope

製品workspace外の小さなCPU fixtureで、blurのRoI padding、Unknown全域、linear-light、LGG系の線形調整、
seed付きgrain、Draft／Final品質、Preview／Export同一評価関数を確認した。これはwgpu shader parity、
製品filter graph、M4 resource lifecycle、golden画像、色管理の全体契約を証明しない。

## Fixed source

| source | fixed version | checksum | license |
|---|---|---|---|
| `thiserror` | crates.io `2.0.19` | `09a43598840e33d5b0331f38c5e30d13bb11c11210a4b58f0d9b18a5a5eefcd9` | Apache-2.0 OR MIT |

algorithm本体は依存なしのprobe-only実装で、standalone `Cargo.lock`は6 packageを固定する。

## Commands and oracle results

```sh
cargo fmt --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml -- --check
cargo clippy --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path spikes/m5-known-implementation/M5-P0/Cargo.toml
```

| oracle | result | evidence |
|---|---|---|
| blur RoI padding | PASS | radius 1の全画面結果と3×3 finite RoIの内部pixelが一致 |
| Unknown region | PASS | Unknownは明示的に全画面へ展開、範囲外finiteは`InvalidRegion` |
| linear-light | PASS | black＋white平均がsRGB 0.5でなくlinear合流の0.7353569 |
| LGG／grain | PASS |同じlinear evaluator内で有限値clampとseed決定的grainを適用 |
| Draft／Final | PASS | 同一入力・seedで品質差は出るが、同一Qualityの再評価はbyte相当一致 |
| preview／export | PASS | 2経路を作らず`evaluate_post`一関数を共有 |

## Disposition

- RoI／Unknown／linear／grain／quality fixture: **KEEP / REUSE PATTERN**
- wgpu／pipeline cache: **REUSE候補を維持**。本receiptはGPU passの採択に昇格しない。
- Vello blur、scene engine post stack、product golden: **REJECT / 未採用**。別の製品oracleなしに持ち込まない。

## Remaining gates

実wgpu shader、premultiplied alpha、既存color conversion一箇所、M4 K0/K1 resource owner、Draft/Final human
quality、Preview／Export実データ、3 OS、M5-A0Sは未完了である。
