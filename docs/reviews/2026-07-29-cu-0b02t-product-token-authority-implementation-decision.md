# CU-0B02T 製品token単一authority実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 親: `CU-0B02` / `U0e-3` **SPLIT**
- 次の一粒: `CU-107PV` **PRODUCT-ASSET / DO**

## 1. 成果

`ui/motolii-tokens/sources/motolii-dark.json`を製品Dark tokenの単一DTCG正本とし、
`crates/motolii-ui-token-gen`の明示profile `v2-product`から
`tokens.rs`、`tokens.css`、`manifest.json`を決定生成した。

既存`v1-fixture` profileは2生成物、v1 header、manifestをbyte不変で維持する。
profileはpath/theme IDから推測せず、未知値、CSS variable衝突、
profileと出力directoryの不一致を型付き拒否する。

製品sourceはaccepted routeのlegacy style supplierとparallel mock supplierで
意味と値が一致する21 Dark roleだけを持つ。Light、custom、high-contrast、
font、spacing、radius、motion、`--object-1`〜`--object-6`は、二supplier間の
一意な照合または既存4型の根拠が無いため追加していない。

## 2. guard

既存`RG-RAW-COLOR`判定を
`docs/mocks-ui/scripts/raw-color-scanner.mjs`へ共有抽出した。
既存reference guardの49 testは挙動不変で全緑である。

新しいproduct token guardは次を固定する。

- product rootはsource JSON 1件とgenerated 3件だけ
- manifest v2、theme 1件、role 21件、object token 0
- generated Dark roleがlegacy/mock両supplierと一致
- handwritten CSS/JS supplierのraw colorを`RG-RAW-COLOR`で拒否

generated CSSは決定生成物なのでraw-color scannerへ掛けず、generatorの
read-only byte `check`が手編集を拒否する。既存product leaf CSS、accepted route、
archived HTML、visual threshold、goldenは変更していない。

## 3. 証拠

- `cargo test -p motolii-ui-token-gen`: 17 pass
- `cargo clippy -p motolii-ui-token-gen --all-targets -- -D warnings`: pass
- `cargo test -p motolii-ui --test u0e3_product_tokens`: 1 pass
- v1 `check --profile v1-fixture`: pass
- v2 `check --profile v2-product`: pass
- `node --test docs/mocks-ui/guard-tests/product-token-guard.test.mjs`: 3 pass
- `node --test docs/mocks-ui/guard-tests/reference-guard.test.mjs`: 49 pass
- reference registry/manifest check: pass
- Browser ownership: 7 pass
- `./scripts/check-ui-toolkit-deps.sh`: pass

全`guard-tests/*.test.mjs`は519/521 passで、失敗2件はBTP/ITP後から既知の
`CR2-SCHEMA` current-route provenance不一致である。これは独立
`G0-6H-V1G-RP`の既存負債であり、本粒はgeneration、`CURRENT`、source manifest hashを
変更して隠していない。

## 4. 次

2026-07-29のユーザー優先順位に従い、token後続`CU-0B02R/N/C/I`は`WAIT`へ戻す。
次のPRODUCT-ASSET `DO`はPlace配送連鎖の先頭`CU-107PV`だけとする。
`CU-107PV → CU-107TC → CU-107AD → CU-107TD → CU-110 → CU-0A08BTI`
の既決縦切りを優先し、token consumer、component state、icon、provenance修復を
Rectangle配置の前へ追加しない。
