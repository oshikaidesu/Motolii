# CU-G04S VS-1 edit runtime session source

- 日付: 2026-07-27
- 状態: **決定**
- CU-G04S: **DONE**

## 1. 目的

VS-1 edit runtime の **session source** と **interim no-session / action disposition** だけを、
[CU-G04S0選定](2026-07-27-cu-g04s0-session-source-selection-decision.md)で選定された4問の結論として docs で閉じる。
UX、typed shape、公開 API、実装機構は本粒で発明しない。

## 2. authority と参照行

| ID | 内容 | 参照 |
|---|---|---|
| A1 | AGENTS.md 作業規約・発注境界 | [AGENTS.md](../../AGENTS.md) |
| A2 | docs 入口・登録規則 | [docs/README.md](../README.md) |
| A3 | 決定逆引き台帳 | [decision-index.md](../decision-index.md) |
| A4 | implementation ledger「現在の並列レーン」「発注依存証跡」 | [implementation-ledger.md](../implementation-ledger.md) |
| A5 | M3 VS-1 運用順・PRODUCT-ASSET lane | [M3-ui-integration.md](../specs/M3-ui-integration.md) |
| A6 | reviews 登録規則 | [reviews/README.md](README.md) |
| A7 | GR-UI 状態所有・command 境界 | [M3 UI境界予防](2026-07-14-m3-ui-boundary-prevention.md) |
| A8 | VS-1 縦slice blocking decision 表 | [縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md) |
| A9 | 快適利用粒度化 §8 W1 依存表 | [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) |
| A10 | CU-G03 edit durability ordering・§3.1 recover base・§4 failure・§7 base 作成 lifecycle | [CU-G03決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) |
| A11 | CU-109S Undo/Redo prepared-action 順序再確認 | [CU-109S順序再確認](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) |
| A12 | CU-109SP P1 precedence・acceptance evidence 限定 | [CU-109SP prerequisite決定](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) |
| A13 | CU-G04S0 session source 選定・4問の handoff | [CU-G04S0選定](2026-07-27-cu-g04s0-session-source-selection-decision.md) |

## 3. 現行コード事実（BASE_SHA b35bcbac）

不在（caller / field / API が無いこと）は順序・所有・可否の肯定証拠にしない（§7 N12）。

| ID | 事実 |
|---|---|
| CF1 | `crates/motolii-ui/src/shell.rs:48` `pub fn run_shell() -> Result<(), ShellError>` は引数を取らず、project path を受け取らない |
| CF2 | `shell.rs:53-62` は `bootstrap_document()` / `bootstrap_document_for_edit_smoke()` から in-memory Document を作り、`DocumentWriter::new` → `DocumentEditRuntime::new(writer)` を構築する。`ProjectSession` は開かない |
| CF3 | `crates/motolii-ui/src` に `ProjectSession` / `save_with_journal` / `commit_edit` の呼出は無い（`state_ownership.rs` / `domain_intent.rs` の variant 名のみ） |
| CF4 | `shell.rs:24` `const DOCUMENT_EDIT_SMOKE_ENV: &str = "MOTOLII_TEST_U2B1_DOCUMENT";` |
| CF5 | `crates/motolii-ui/tests/u1a1_window_smoke.rs:144` が実 binary を `MOTOLII_TEST_U2B1_DOCUMENT=1` で起動し、`U2B1_DOCUMENT passed` 行に対して `registrations=1` / `generation=4` / `revisions=1,2,3` を assert する |
| CF6 | `crates/motolii-ui/src/app.rs:552-555` がその log 行を出力する（`revisions=1,2,3` は literal） |
| CF7 | `crates/motolii-doc/src/journal/session.rs:72` `ProjectSession`、`session.rs:129` `save_with_journal`、`crates/motolii-doc/src/journal/wal.rs:151` `commit_edit` は存在する |
| CF8 | ledger「現在の並列レーン」で `状態` が `DO` の行は全 lane 通算 1 件（`CU-G04S`、L161）。`CU-109` は L162 で `WAIT` |
| CF9 | ledger「発注依存証跡」に `CU-G03D` / `CU-G03R` / `CU-109S` / `CU-109SP` / `CU-109SP-R1` / `CU-G04S0` / `D1m` / `D2` / `U2b-1` の `DONE` 行が一意に存在する。`CU-G04S` / `CU-109` / `CU-110` / `CU-111` の行は無い |
| CF10 | `./scripts/check-docs.sh` は `OK`。`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` は 118 pass / 0 fail |
| CF11 | 本 worktree に `node_modules` は無い。`npm run test:reference-guard` と `inspector-read-model-inventory.test.mjs` は BASE_SHA 時点の環境前提で実行不能／`ERR_MODULE_NOT_FOUND`。本粒の diff が原因ではない |
| CF12 | `git status --porcelain` は空。HEAD = `b35bcbac06d6d66759bce4458b1153929fed88cc` |

## 4. FACTS / INFERENCES

### FACT

- [CU-G04S0選定](2026-07-27-cu-g04s0-session-source-selection-decision.md)は session source 未決を `CU-G04S` へ分割し、4問（path 渡し・no-session 処分・CU-111 前 Undo/Redo・U2b-1 smoke 再配置）を次粒の問いとして列挙した。
- [CU-109SP prerequisite決定](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) §5 は P1：`CU-109` 先行、acceptance evidence は Apply roundtrip に限定、`CU-111` は後続接続のみと裁定した。
- [CU-G03決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3.1 は recover 可能な main または generation base の確認を要求し、§4 末尾は Undo/Redo command 取得を `CU-111` 非公開境界へ、§7 は初期 base 作成を CU-G04 lifecycle へ割り当て、base 不在 session への edit-only commit を `CU-109` に禁ずる。
- 主担当 Codex が本粒向けに D1〜D7 を確定した（binding order §3）。Spark は意味を足さず・削らず写すだけとする。

### INFERENCE

- D2〜D4 は A10（CU-G03 §3.1 / §4 / §7）を書き換えず充足する。
- D5 は A10 §4 末尾および A12 §5 P1 を書き換えず充足する。
- D6 の U2B1 smoke 期待値変更（revision 1 維持、publish 1 維持、`CU-111` が後に revision 2/3 を復元）は、本 decision で事前承認された唯一の経路であり、`CU-109` code diff 側で即興的に変更してはならない。

不在（CF3 等の「呼出が無い」）を、順序・所有・可否の肯定証拠として結論に使わない。

## 5. DECISION

- **D1（分割）**: 親 `CU-G04` は `SPEC / DECIDE` を維持し、New/Open chooser、Save / Save As、
  Unsaved Changes、read-only-newer、recovery UX、checkpoint policy、一般 project lifecycle を
  引き続き所有する。子 `CU-G04S` は **VS-1 edit-runtime session source と interim
  no-session / action disposition だけ**を決める。

- **D2（session source）**: 実 session-backed 製品 shell entry は、**呼出側から明示的な既存
  project path を要求する**。entry は `ProjectSession` を open して保持し、recover された
  Document を使う。temp/default path を捏造しない。base 不在の project を初期化しない。
  新規 project / base 作成は後続 CU-G04 lifecycle の作業である。

- **D3（no-session 処分）**: path 欠落、project が存在しない／未初期化、recovery 失敗のいずれでも、
  **session-backed edit runtime を構築せず、編集を一切受理も publish もしない**。
  optional durability flag、live-only の製品 edit fallback、in-memory edit route、
  二本目の durability / publish 経路のいずれも作らない。

- **D4（現行 zero-path bootstrap shell）**: 現行の path 無し bootstrap shell は
  **diagnostic / native-shell baseline 専用**であり、`CU-109` の製品証拠にしない。
  `CU-109` 後に semantic な Apply/Undo/Redo bypass を残してはならない。
  本粒は docs のみを変更し、実装機構は `CU-109` が所有する。

- **D5（CU-111 前の Undo/Redo）**: `CU-111` が非公開 typed prepared-action 境界を供給するまで、
  製品 Undo/Redo は **typed な pre-mutation rejection** とする。queue action は 1 回だけ消費し、
  journal 0、live Document / history / revision 不変、publish 0、poison なし、retry なし。
  `CU-109` は将来の Undo/Redo durability / poison / reconcile / publish 配線を引き続き所有し、
  この暫定 rejection は所有権を `CU-111` へ移さない。

- **D6（U2b-1 smoke の再係留・期待値変更の事前承認）**: 実 binary の
  `MOTOLII_TEST_U2B1_DOCUMENT` smoke は **`CU-109` の中で** test 所有の初期化済み project path
  へ再係留する。Apply 証拠を保持し、kill/reopen 後の Document 同値と duplicate retry の
  拒否／不可能性まで拡張する。`CU-111` 前の Undo/Redo 期待は typed-rejection assertion へ変わり、
  **revision は 1 のまま、publish 回数は 1 のまま**となる。`CU-111` が後に durable な
  revision 2 と 3 を復元する。**この期待値変更は本 decision で事前承認された唯一の経路であり、
  code diff 側で即興的に変更してはならない。**

- **D7（次粒）**: `CU-G04S` 完了後、次の唯一の PRODUCT-ASSET `DO` は `CU-109` へ戻る。
  `CU-109` の acceptance evidence は **Apply roundtrip のみ**。
  `CU-110` / `CU-111` および他の待ち行は `WAIT` のまま。

## 6. 非目標

- Rust、test、fixture、golden、script、`package.json` / lock、CI 設定、visual asset、threshold の変更
- 公開 API、`Document`、journal / serde 形式、min-reader、plugin 契約、`Command`、`JournalEdit` の変更
- `CU-109` / `CU-110` / `CU-111` の実装、着手、または同一粒への束ね
- New/Open chooser、Save / Save As、Unsaved Changes、read-only-newer、recovery UX、checkpoint policy、
  project path codec の意味決定
- `Healthy / Poisoned` の具体 state、typed prepared-action の shape / API / payload / transport の設計
- U2B1 smoke の**実際の**編集（D6 は将来の `CU-109` diff への事前承認であり、本粒では 1 byte も test を触らない）
- `U3a-2Q-V` / `CU-106P` / `CU-106F` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `G0-6H` / `G0-9*` /
  `U4a` / VS-2 の状態変更
- 隣接チケット（`CU-110` Place、`U2h-1P`、`U3a-2*`、`GAP-23/24/25`）への拡張
- 親 `CU-G04` 全体を `DONE` にすること
- 会話履歴、外部 model 助言、旧粒度化の候補分類、過去 order 文面を authority にすること

## 7. 必須負例（負 oracle）

- **N1**: 本粒で Rust / test / fixture / golden / script / package file を変更する。
- **N2**: U2B1 smoke の `revisions=1,2,3` / `generation=4` / `registrations=1` を本粒で書き換える。
- **N3**: temp project、default path 捏造、missing base の自動初期化、optional durability flag、
  live-only 製品 edit fallback、in-memory edit route、二本目の durability / publish 経路を採択する。
- **N4**: 現行 zero-path bootstrap shell を `CU-109` の製品証拠として書く。
- **N5**: `CU-111` 前の暫定 Undo/Redo rejection を根拠に、`CU-109` の Undo/Redo 所有権を `CU-111` へ移す。
- **N6**: `CU-109` の acceptance evidence を Apply roundtrip 以外へ広げる。
- **N7**: 親 `CU-G04` を `DONE` にする、または CU-G04 lifecycle UX（New/Open/Save/Unsaved 等）を決める。
- **N8**: PRODUCT-ASSET lane の `DO` を 0 件または 2 件以上にする。
- **N9**: 「発注依存証跡」の既存行、過去 decision の本文 / PR / hash、歴史 receipt 行を書き換える
  （`CU-G04S` の新規 1 行 append のみ可）。
- **N10**: raw stack / raw writer / 汎用 peek / 公開 raw mutation API を「必要」として本文へ書く。
- **N11**: guard test / `check-docs.sh` を通すために期待値・threshold・除外・固定 hash・fixture・
  依存を触る、lint 抑制や個別除外を足す。
- **N12**: 不在（CF3 等の「呼出が無い」）を順序・所有・可否の肯定証拠として結論に使う。
- **N13**: 結論を書かず TODO / 保留 / 「後続で決める」で複数候補を残す、または一部 file だけ更新して
  他の current mirror を stale のまま残す。
- **N14**: allowlist 外の file を 1 byte でも変更する、または新規 file を allowlist 外へ作る。

## 8. STOP 条件

- **S1**: CU-G03 §3 / §4 / §7 / §8、CU-109SP §5（P1）、粒度化 §8 W1 の依存表のいずれかを
  書き換えないと §5 の D1〜D7 を書けない。
- **S2**: 結論に typed shape、公開／非公開 API、payload、transport、journal 形式、
  `Healthy / Poisoned` の具体 state、poison 実装が必要になる。
- **S3**: session source を閉じるのに New/Open/Save/Unsaved などの未決 UX 判断が必要になる。
- **S4**: 許可 file 以外へ 1 byte でも変更が要る。
- **S5**: guard / `check-docs.sh` を通すために期待値・threshold・除外・固定 hash・fixture を触りたくなる。
- **S6**: 変更後に PRODUCT-ASSET `DO` が 1 件（`CU-109`）にならない形しか書けない。
- **S7**: 本 order、AUTHORITY 一覧、CF1〜CF12 以外の repo 横断調査や別 worktree の状態が判断に必要になる。
- **S8**: ledger の `CU-G04S` 行が `DO` でない、または DEPENDENCY のいずれかが `DONE` でないことを発見した。
- **S9**: `CU-110` 配置、UI surface、製品 window、`G0-6H`、`G0-9`、`U4a` / VS-2 の状態を動かす必要が出る。

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| **`CU-G04S`** | **DONE** | VS-1 edit runtime session source / no-session 処分 / CU-111 前 typed rejection / U2B1 smoke 再係留の事前承認を docs で閉じた |
| **`CU-109`** | **DO** | 共有配線の実装粒。acceptance evidence は Apply roundtrip のみ。session-backed entry は明示 project path を要求し、no-session では edit runtime を構築しない |
| `CU-G04`（親） | **SPEC / DECIDE** | 維持。New/Open/Save/Unsaved/read-only-newer/recovery UX/checkpoint policy を引き続き所有 |
| `CU-110` / `CU-111` | **WAIT** | 据え置き |
| `U3a-2Q-V` / `CU-106P` / `CU-106F` | **WAIT** | 据え置き |

PRODUCT-ASSET lane の `DO` は `CU-109` ただ一件とする。
