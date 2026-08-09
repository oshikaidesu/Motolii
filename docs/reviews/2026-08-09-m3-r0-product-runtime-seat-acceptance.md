# M3 R0 product runtime seat 受入

日付: 2026-08-09

状態: **決定 / R0-ACCEPT DONE**

## 1. 判定

`R0-HOST`、`R0-MAC-SEAT`、`R0-STAGE-LIFECYCLE`を責任別に再照合し、通常のRN Release artifactで同じread-only revisionを表示するR0出口を確認した。R0対象bytesは基準commit `a1c2bbe49aa366d18871d79f6da6bf35053f2c89`から現行main `c9cab8e8`まで不変である。よって4 nodeを`DONE`とし、R1をcurrent codeから再compileできる状態へ移す。

これはR1のGPU binding、Stage描画、Browser編集、三面E2E、M3完成を受け入れる判定ではない。

## 2. 受入matrix

| node | 確認した出口 | 判定 |
|---|---|---|
| `R0-HOST` | 一つのHost、bounded snapshot、typed lifecycle／diagnostic、semantic write 0 | `DONE` |
| `R0-MAC-SEAT` | RN window、offline Hermes bundle、Rust static library、明示project path、fail-closed bootstrap | `DONE` |
| `R0-STAGE-LIFECYCLE` | native child viewのregister／mount／resize／focus／unmount／remount、late event拒否、同revision | `DONE` |
| `R0-ACCEPT` | 実projectを通常RN Release appで開き、read-only表示、invalid path拒否、network 0で起動継続 | `DONE` |

## 3. 非LLM oracle

Rust側は次を再現した。

```text
cargo test -p motolii-ui --test r0_rn_product_seat       5 passed
cargo test -p motolii-ui rn_product_host::tests          18 passed
cargo test -p motolii-ui document_edit_runtime           43 passed
cargo test -p motolii-ui --test cu110_product_place_commit 2 passed
cargo test -p motolii-ui --test cu111_product_undo_redo    1 passed
```

RN／macOS側は次を確認した。

- `corepack yarn install --immutable`: success
- `corepack yarn exec tsc --noEmit -p .`: pass
- Jest: 2 passed
- ESLint: error 0。既存の`no-void` warning 2件は残存
- `pod _1.15.2_ install --deployment`: success
- arm64 Release build (`CODE_SIGNING_ALLOWED=NO`): succeeded
- `main.jsbundle`生成とRust symbol link: confirmed
- 現行Document fixtureの通常app起動: Browser／Inspector／Timelineを表示し、`Host unavailable`なし
- 存在しないproject path: typed `Host unavailable / project path does not exist`、crashなし
- `sandbox-exec`の`deny network*`下で有効projectを起動: 5秒超生存

## 4. Stage境界の裁定

`R0-STAGE-LIFECYCLE`の「描画しない」は、R0 nodeが描画責任や描画完了を所有しないというnode境界である。同じfileに隣接する未受入のR1 GPU／draw候補が存在することはR0 lifecycle違反ではない。top-seatが作ったblind evidence envelopeをfresh Grok 4.5 sessionがread-onlyで再照合し、`NODE_BOUNDARY_VERIFIED`、P0なしと返した。

R1では一つのDevice／Queue、surface lifecycle、第二renderer 0を別途compileし、code存在を受入へ繰り上げない。

## 5. 状態遷移と非目標

- `R0-HOST`、`R0-MAC-SEAT`、`R0-STAGE-LIFECYCLE`、`R0-ACCEPT`: `DONE`
- `R1`: `READY-RECHECK`。各nodeはcurrent mainからclosed orderへ再compileする
- R1〜R4、Windows実機、人間受入、署名／配布、remote push: 本判定の非目標
- R0 artifactに隣接するR1候補は未受入のまま保持し、再実装も自動採用もしない
