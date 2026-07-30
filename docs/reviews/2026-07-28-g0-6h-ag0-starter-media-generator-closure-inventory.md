# G0-6H-AG0 Starter Media generator / output closure 棚卸しと責任処分

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-AG0: **DONE**

## 目的

generator / output closure の棚卸しと責任処分をちょうど1件確定する。byte / path / codec / schema / command を閉じない。

## 候補inventory表

実装せず、byteも生成せず、repo内の既存経路だけを根拠に分類する。

| 候補 | repo内の実在経路 | できること | できないこと・持ち込む責任 | 本fixtureへの扱い |
|---|---|---|---|---|
| `docs/mocks-ui` locked Node route | `docs/mocks-ui/package.json`（`"type": "module"`、devDependencies の locked `pngjs` / `pixelmatch` / `@playwright/test`、npm scripts `generate-reference` / `check-reference` / `test:reference-guard`）、`node:crypto` | reference generation の evidence route、guard 付き npm 動線、PNG 読み取りと hash 計算の既存境界 | 製品runtime、Document、Starter Media recipe の意味決定、ffmpeg 呼び出しの正本化 | **既存 route を WRAP の EXISTING ROUTE として参照する** |
| reference-generation の read-only hash と atomic generation pattern | `docs/mocks-ui/scripts/reference-generation.mjs`（`sha256()` / `pixelSha256()`、`publishReferenceGeneration()` の stage → fsync → rename → `CURRENT`、`readCurrentReferenceGeneration()` の closed schema と read-only 検証）、`docs/mocks-ui/scripts/reference-cli.mjs` の `check()` | byte / pixel の integrity 集約、atomic publish、repository 変更検出付き read-only check | Starter Media 固有の manifest、media byte 生成、cross-platform byte 決定性の保証 | **integrity 境界の pattern として WRAP 内で参照する** |
| 既存 repo ffmpeg setup と r9-smoke synthetic video 経路 | `scripts/setup-local-deps.sh`（`.tools/ffmpeg/bin` と PATH）、`scripts/r9-smoke.sh`（`command -v ffmpeg` 前提、`lavfi testsrc` 合成動画） | 外部 ffmpeg CLI 境界、ローカル toolchain 前提の synthetic video 生成例 | Node route からの統合 orchestration、version / OS 差の吸収、fixture recipe | **外部 CLI 境界として WRAP の EXISTING ROUTE に含める** |
| Rust test-local WAV helper | `crates/motolii-audio/tests/support.rs`（`write_pcm16_wav` / `sine_wave_i16`）、製品側 `crates/motolii-media/src/mux.rs` の `write_f32le_wav_stereo_48k` | cargo test harness 内での PCM WAV 合成 | Node / npm evidence route からの直接到達、JS 境界越え再利用、製品関数の fixture 生成への流用 | **棄却**（下記） |
| 新規 codec / media framework / package manager / process supervisor / 製品service の自作 | 該当なし | — | 依存優先ゲート §3-2 / §4 STOP に一致する汎用機能の新設責任 | **棄却**（下記） |

## 短票（依存優先・責任最小化ゲート §2）

```
RESPONSIBILITY DISPOSITION: WRAP
EXISTING ROUTE: 既存 `docs/mocks-ui` Node evidence route（locked `pngjs`、`node:crypto`、既存 npm script 動線）と、既存の外部 ffmpeg CLI 境界（`scripts/setup-local-deps.sh` / `scripts/r9-smoke.sh` が示す PATH 前提と前提条件検査）
OWNED RESIDUE: Motolii 固有の fixture recipe（どの Starter Media を何のために置くか）、provenance 記述、integrity 集約の orchestration だけ。汎用機能を1件も残さない
IMPORTED RESPONSIBILITY: locked `pngjs` の version / license / 供給網、外部 ffmpeg CLI の存在・version・OS差・出力差、Node runtime version 差
EXIT: fixture 専用 adapter と read-only integrity 境界に閉じる。交換・削除時に触るのは本粒が指す fixture 用 file と docs だけで、製品crate・公開API・Document・serde面・plugin契約・React source へ波及しない
RETIREMENT: FROZEN / DELETE-LATER
```

## 棄却の明示

- Rust test-local WAV helper の JS 境界越え直接再利用を**棄却**する。cargo test harness 専用 helper であり、Node route から到達する経路が存在せず、到達させるには新しい橋渡し責任を所有することになる（`crates/motolii-media/src/mux.rs` の製品関数を fixture 生成へ流用することも同様に棄却）。
- 新規 codec、media framework、package manager、process supervisor、製品service の新設を**棄却**する（ゲート §4 STOP条件、§3-2 に一致）。

## 決定論の主張範囲

`G0-6H-AF` AF-4 を継承し、**cross-platform の byte 決定性を主張しない**。後続の生成粒が実際に使った toolchain を記録し、その結果の byte を凍結する。capture / test 時の integrity 検査は network 非依存で、可能な範囲で tool 非依存に保つ。

## 確定しないこと

具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、正確な hash 値、生成command、tool / package の正確なversion、route / query shape、adapter API、media byte、実装file。

## 停止線の継承

`G0-6H-A` A-3 / A-7 / A-8 と `G0-6H-AF` AF-6。`Starter Media` は Project 外 fixture-only 源であり、Document / 製品runtime / 公開API / plugin契約 / 永続形式 / production Registered folder の正本にならない。

## `G0-6H-V0` の扱い

`WAIT` のまま。本粒は implementation ledger の状態語を変更しない。

## React / Browser authority（参照のみ・本粒は差分0）

- **対象面**: product-owned React module `DiscoveryBrowserCandidate` の `Media` surface（`#plugin-browser-candidate` route）。移管契約は [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime境界は [ui-runtime-architecture.md](../ui-runtime-architecture.md)（Browser は bundled first-party Host module）。対応spec IDは M3 / VS-1 / G0-6H 系列（`G0-6H-A` scenario契約、`G0-6H-AF` 媒体源裁定、次粒 `G0-6H-V0`）。
- **SOURCE ASSET**: 固定 commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`（`ui/motolii-web/source-provenance.json` の `fixedSourceCommit`）。対象 export は `DiscoveryBrowserCandidate` at `ui/motolii-web/src/index.js`。旧 path は `docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx` 等、現行 path は `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` 等。closure は対象 component / CSS / pattern、`ui/motolii-web/source-provenance.json`、`docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`。旧 generation 限定の再現 authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は現行 product surface の authority ではない。
- **PRESERVE / REPLACE / DIAGNOSTIC ROUTE / NEGATIVE ORACLE**: 本粒は React source を1 byteも変更しない。Motolii Studio Preview は未実装。現行 G0-6H 人間審判入力 route は `#plugin-browser-candidate` のみ。generator / integrity check は製品 runtime から到達しない証拠カプセルとして後続粒に委ねる。

## 非目標

- media byte、file、fixture、asset の生成・取得・追加・commit。
- 具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、hash値、生成command、tool / package の正確なversion、route / query shape、adapter API、公開APIの決定。
- generator の実装、script、CI 配線、npm script 追加、依存追加、`docs/mocks-ui/package.json` / `package-lock.json` の変更。
- `G0-6H-AG` の実装内容（出力set、path、check command）の先取り。
- repo全体での vendoring 方針、既存 vendoring / 依存採否の撤回・変更、`docs/references.md` の候補処分の変更。
- token値、theme、threshold、golden、snapshot、期待値、component、iconの選定・変更。
- `Project` および production `Registered folders` の意味の新設・変更・拡張。
- 現行route実装、route名、入場条件、`docs/mocks-ui/README.md`、`docs/mocks-ui/src/main.jsx`、`ui/motolii-web/**`、hash fixture の変更。
- `docs/implementation-ledger.md` の変更。
- 隣接チケット（`CU-107*` / `CU-110*` / `CU-111` / `U3a-*` / `U2h-*` / `G0-9*` / `U0e-*` / `CU-0B0*` / `G0-6H-V0`）への波及。

## 必須負例

- §ALLOWED_FILE 以外の file を変更・追加・削除する。
- 4 file のうち一部だけを変更する部分適用、または TODO スタブ・「後で判断」で処分を置き換える。
- `RESPONSIBILITY DISPOSITION` を `WRAP` 以外にする、複数選ぶ、hybrid にする、または短票6行のいずれかを欠落・改名・順序変更する。
- `RETIREMENT` を空、「後で判断」、または `FROZEN / DELETE-LATER` 以外にする。
- Rust test-local WAV helper の JS 境界越え直接再利用を許容・保留・条件付き採用と書く。
- 新しい codec、media framework、package manager、process supervisor、capture framework、retry framework、製品service を採用・提案・新設する。
- cross-platform / OS横断 / toolchain横断の byte 決定性を主張する。
- 具体path、file名、codec、寸法、尺、byte数、manifest schema、hash algorithm、hash値、生成command、tool version、route / query、adapter API、media byte を1つでも確定する。
- media byte、fixture file、生成物を commit する、または generator の実装file・script・npm script・CI job を作る。
- `docs/mocks-ui/package.json` / `package-lock.json` / `node_modules` を変更する（`browser-catalog-decoder.test.mjs` の `AUTHORITY_SHA256` が落ちる）。
- `Starter Media` を Project asset、Document、公開API、plugin、永続形式、production Registered folder の正本として扱う記述。
- label または opaque ID から欠落意味を推測して補う。
- `reference-handoff.md` の既存節を変更・削除・並べ替えする、または `未記入` / `[ ]` を埋める。
- 承認済みnormal 5画面、`check-reference` 成功、Git ancestry を、visual parity・人間承認・route同一性・empty-project成立の根拠とする。
- `docs/implementation-ledger.md` を本ticket差分に含める、または ledger の状態語を変更する。
- reviews索引未登録のまま新規文書を置く、相対リンク切れを残す、`decision-index.md` に固定語彙外の状態語を書く。
- lint / test 抑制、期待値・golden・threshold・snapshot・fixture special-case の追加・変更、生JSON/文字列走査による型付き境界の迂回、公開raw API、invented serde default、重複planner/helper の新設。
- `G0-6H-AG` 以外の後続粒を新設する、または次の一粒を2件以上・0件にする。

## 次の一粒（ちょうど1件）

**`G0-6H-AG`** — bounded implementation grain。固定された Starter Media evidence capsule（固定出力 + raw provenance）と read-only integrity check を作る。route / React 統合を含めない。`WRAP` 処分と `FROZEN / DELETE-LATER` の retirement を継承し、製品runtimeから import されない。2件目の後続粒を起票しない。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-A` | **DONE** | 前提。scenario / fixture所有契約 |
| `G0-6H-AF` | **DONE** | 前提。媒体源・provenance class 裁定 |
| `G0-6H-AG0` | **DONE** | 本粒。generator / output closure inventory と WRAP 責任処分（byteなし） |
| `G0-6H-AG` | **DO** | 固定 evidence capsule + read-only integrity check（bounded implementation） |
| `G0-6H-V0` | **WAIT** | 本契約のCodex統合まで維持 |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-A scenario / fixture契約](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-AF 媒体源・provenance class 裁定](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)
- [依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)
- [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
- [ui-runtime-architecture](../ui-runtime-architecture.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
