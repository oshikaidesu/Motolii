# build の置き場所 — 物理の調整と shader の編集で Rust を組み直さない(2026-09-17)

利用者「毎回 build が要るのは『しかたない』ではなく置き場所の問題」。組み直しが要るのは**世界**(揺らがない Rust)だけで、
解き手の欄・shader の本文は棚(vism/)に置けば保存が次のコマに載る。4 つを 1 つずつ commit した。

## 何をどこへ移したか

| 前 | 後 | commit |
|---|---|---|
| `engine/physics.rs` に直書きの `num_solver_iterations = 12` / `max_ccd_substeps = 4` / `normalized_allowed_linear_error = 0.0005` | `vism/field.wgsl` の `PHYSICS` 札の欄 `NUM_SOLVER_ITERATIONS` / `MAX_CCD_SUBSTEPS` / `NORMALIZED_ALLOWED_LINEAR_ERROR`(Rapier の IntegrationParameters と同じ名前)。書かなければ Rapier の既定 4 / 1 / 0.001、札には今の値 12 / 4 / 0.0005 を書いたので絵は変わらない。`ITERATIONS` は `NUM_SOLVER_ITERATIONS` へ改名 | 物理: 解き手の 3 欄を Rust から棚の札へ |
| `build.rs` が無条件に `rerun-if-changed=vism/` — debug でも .wgsl の保存で motolii-render を組み直す | disk から読む構成(`load_shaders_from_disk`: workspace + debug + 非 wasm)では vism/ を見張らない(build.rs 自身と env だけ)。焼き込む構成(release / wasm / workspace 外)だけが見張る | build.rs の見張りを狭める |
| ブロックの test が `include_str!("../../vism/bounce.wgsl")` 等で本文を焼き込む | `program_for(device, "bounce")` — 棚の札(`catalog_snapshot`)の `vertex_text` から組む(描く時 `blocks.rs:597` と同じ道)。描く側は元から棚を通っていた(include_str! は test だけだった) | ブロックの test も棚を通す |
| 描く側の見張り `zz_watch` は `--release`(棚は焼き込み、shader の保存は次の build まで載らない) | root `Cargo.toml` に `[profile.watch]`(release 継承 + `debug-assertions = true` → build.rs の `cfg!(debug_assertions)` が立ち、棚は disk)。`zz_watch` は棚の見張り(`watch_effect_catalog`)を立て、起きたら `refresh_effect_catalog` して描き直す | 描く側の見張りを watch profile へ |

## 計測(2026-09-17、M 系 Mac、warm cache)

.wgsl を touch してからの再 build:

| 構成 | 前 | 後 |
|---|---|---|
| `cargo build -p motolii-render --example zz_watch`(debug) | 19.7 s(Compiling motolii-render) | 0.3 s(Compiling 無し) |
| `cargo test -p motolii-render --lib --no-run`(debug) | 組み直し(include_str! の分) | 2.5 s(Compiling 無し) |
| `cargo build --release -p motolii-render --example zz_watch` | 18.9 s | 18.9 s(焼き込みなので今まで通り) |

同じ書類(gumball.js + Gain 1 行、1080×1350、150 コマ step 5 shrink 2 = 31 枚保存・151 コマ描画)の 1 回の描画:

| profile | 棚 | 描画 | 初回 build |
|---|---|---|---|
| `release` | 焼き込み | 13.5 s | — |
| `watch`(新設) | disk | 13.9 s | 4 m 02 s(別 target dir `motolii/target/watch`、以後は差分) |
| `dev` | disk | 15.6 s | — |

以前の「366 s」(dev)はこの書類では再現せず(dev は `opt-level = 1`、依存は 2)。重い書類・動画で差が出るかは別途。

shader の保存が次のコマに載る証拠(cargo 無し): `scratchpad/placement/sheet-before.png` → `gain.wgsl` の 1 行を保存 → `sheet-after.png`
(`shelf: generation 2` → 次の `shot:` で色が変わる)。編集は戻した。

## 手順(2 本の見張り)

```sh
# 台本の側(debug、書類だけ): 台本の保存 → MOTOLII_OUT/shot.rrd
MOTOLII_SCRIPT=motolii/ui/native/src/editor/script/examples/gumball.js MOTOLII_OUT=/tmp/shot \
  cargo test -p motolii-ui --lib watch_shot -- --ignored --nocapture

# 描く側(watch profile、棚は disk): 書類か vism/ の保存 → コマ → sheet.png
cargo build --profile watch -p motolii-render --example zz_watch
MOTOLII_LAST=150 MOTOLII_STEP=5 MOTOLII_SHRINK=2 motolii/target/watch/examples/zz_watch /tmp/shot/shot.rrd /tmp/shot/out
```

起動時に `shelf: disk` と出れば棚は disk(`baked` なら焼き込み — `--release` で組んだか、`.cargo/config.toml` の `IS_IN_RERUN_WORKSPACE` が無い)。

## 決めた事(指示に無かった分)

- `PHYSICS` 札は field.wgsl の manifest にあり、台本(.js)には解き手の欄を押す口が無い。今の値は台本ではなく **field.wgsl の札**に書いた(既存の `SUBSTEPS` / `ITERATIONS` と同じ置き場)。台本から押せる形にするのは別件(欄を INPUTS にするか、Effect の param にするか — 意味を足すので相談)
- 描く側の `include_str!` は test にしか無かった(描く側は元から棚経由)。item 3 は test の道を揃えた
- `begin_from` の hunk が作業樹に取り残されていて HEAD が組めなかったので、先に別 commit で入れた(他 lane の作業、そのまま)
- 証拠の書類: 物理の例はどれも vism の pass 効果を使っておらず(場の block は Rapier が引き受けて GPU の block は走らない、blend/matte も Normal の合成では通らない)、shader の編集が絵に出ない。gumball.js に `Gain` を 1 行足した写し(scratchpad、repo の例は触らない)で証拠を取った
- `blocks::tests::things_stop_moving_once_they_have_settled` は item 1 の前から落ちる(他 lane の途中)。`a_field_moves_*` 2 本は並走させると落ち、単独では通る(GPU の test の並走)
