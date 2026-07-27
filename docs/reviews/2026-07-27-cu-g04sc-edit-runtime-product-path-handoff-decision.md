# CU-G04SC VS-1 edit runtime product path handoff

- 日付: 2026-07-27
- 状態: **決定**
- CU-G04SC: **DONE**

## 1. 目的

[CU-G04SC0選定](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md) §4 が列挙した4問
（carrier、entry境界、failure処分、test-flag containment）を、docs-only 粒として閉じる。
Rust、公開 API 凍結、UX、recovery 実装は本粒で発明しない。

## 2. authority と参照行

| ID | 内容 | 参照 |
|---|---|---|
| A1 | AGENTS.md 作業規約・発注境界 | [AGENTS.md](../../AGENTS.md) |
| A2 | docs 入口・登録規則 | [docs/README.md](../README.md) |
| A3 | 決定逆引き台帳 | [decision-index.md](../decision-index.md) |
| A4 | implementation ledger「現在の並列レーン」「発注依存証跡」 | [implementation-ledger.md](../implementation-ledger.md) |
| A5 | M3 VS-1 運用順・GR-UI 割当表 | [M3-ui-integration.md](../specs/M3-ui-integration.md) |
| A6 | reviews 登録規則 | [reviews/README.md](README.md) |
| A7 | GR-UI 状態所有・toolkit 隔離 | [M3 UI境界予防](2026-07-14-m3-ui-boundary-prevention.md) |
| A8 | VS-1 縦slice blocking decision 表 | [縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md) |
| A9 | 快適利用粒度化 §8 W1 依存表 | [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) |
| A10 | CU-G03 edit durability ordering・§3.1 recover base・§4 failure・§7 base 作成 lifecycle・§8 STOP | [CU-G03決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) |
| A11 | CU-109SP P1 precedence・acceptance evidence 限定 | [CU-109SP prerequisite決定](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) |
| A12 | CU-G04S D1〜D7 session source / no-session / interim disposition | [CU-G04S session source決定](2026-07-27-cu-g04s-edit-runtime-session-source-decision.md) |
| A13 | CU-G04SC0 product path handoff 選定・§4 四問 | [CU-G04SC0選定](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md) |

## 3. 現行コード事実（BASE_SHA 943279b1）

不在（caller / field / API が無いこと）は順序・所有・可否の肯定証拠にしない。

| ID | 事実 |
|---|---|
| CF1 | `crates/motolii-ui/src/shell.rs:48` — `pub fn run_shell() -> Result<(), ShellError>` takes no argument and receives no project path. |
| CF2 | `shell.rs:54-66` builds an in-memory Document from `bootstrap_document()` / `bootstrap_document_for_edit_smoke()`, then `DocumentWriter::new` → `DocumentEditRuntime::new(writer)`. No `ProjectSession` is opened. |
| CF3 | `shell.rs:24` — `const DOCUMENT_EDIT_SMOKE_ENV: &str = "MOTOLII_TEST_U2B1_DOCUMENT";`, read at `shell.rs:51` via `std::env::var_os(...).is_some()` (boolean presence only). |
| CF4 | `crates/motolii-ui/src/lib.rs:54` — `pub use shell::{run_shell, ShellError};` is the only shell re-export. `mod shell;` at `lib.rs:21` is private. |
| CF5 | `crates/motolii-ui/src/bin/motolii_ui_shell.rs:1-15` — the binary imports `motolii_ui::{run_shell, ShellError}`, calls `run_shell()` with no arguments, reads no `std::env::args`, and exits `77` on `ShellError::Gpu(_)` else `1`. It is a `src/bin` target, i.e. a separate crate that cannot call a `pub(crate)` library item. |
| CF6 | `crates/motolii-ui/tests/u1a1_static_viewport.rs:10` — `let entry: fn() -> Result<(), ShellError> = run_shell;` pins the exact zero-argument signature; `:117` matches the function name string `"run_shell"` during AST scanning. |
| CF7 | `crates/motolii-ui/tests/u1a1_window_smoke.rs:142-163` launches `CARGO_BIN_EXE_motolii_ui_shell` with `.env("MOTOLII_TEST_U2B1_DOCUMENT", "1")` and **no** positional argument, asserting `registrations=1` / `generation=4` / `revisions=1,2,3`. |
| CF8 | `run_shell` occurrences in the repo are exactly: `shell.rs:48`, `lib.rs:54`, `bin/motolii_ui_shell.rs:1,6`, `tests/u1a1_static_viewport.rs:5,10,117`. The only non-test caller is the binary. |
| CF9 | ledger "現在の並列レーン": the row whose `状態` is exactly `DO` is `CU-G04SC` (L163). `CU-109` is `WAIT` (L164). Other-lane rows carry qualified states (`DO / HUMAN` L174, `DO / SPEC` L175, `DO / CHECK-PATH` L181) and are not exactly `DO`. |
| CF10 | ledger "発注依存証跡" contains unique `DONE` rows for `CU-G04SC0` (L278), `CU-G04S` (L277), `CU-G03D` (L247), `CU-G03R` (L248), `CU-109SP` (L274), `CU-109SP-R1` (L275), `D1m` (L244). There is no `CU-G04SC` / `CU-109` / `CU-110` / `CU-111` row. |
| CF11 | `./scripts/check-docs.sh` → `OK: docs整合チェック全項目通過`. `node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` → 118 pass / 0 fail. |
| CF12 | This worktree has no `node_modules`. `npm run test:reference-guard` and `inspector-read-model-inventory.test.mjs` are not runnable here (`ERR_MODULE_NOT_FOUND`); this is a BASE_SHA environment property, not a defect of this diff. |
| CF13 | `git status --porcelain` is empty. HEAD = `943279b1f8590b243735d8117c15a8ff250e703d`. |

## 4. FACTS / INFERENCES

### FACT

- [CU-G04SC0選定](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md) §4 は carrier・entry 境界・failure 処分・test-flag containment を `CU-G04SC` の問いとして列挙した。
- [CU-G04S session source決定](2026-07-27-cu-g04s-edit-runtime-session-source-decision.md) D2 は明示既存 project path と `ProjectSession` open を要求し、D4 は zero-path bootstrap を diagnostic 専用とした。
- [CU-G03決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3.1 / §4 / §7 / §8 は recover base、failure authority、base 作成 lifecycle、STOP を規定する。
- [CU-109SP prerequisite決定](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) §5 は P1 と Apply roundtrip 限定 acceptance を裁定した。
- M3 spec GR-UI 割当表は本粒に UI surface を追加しない。適用 discipline は GR-UI-1（project path は caller 供給引数であり Document/User settings 状態ではない）と GR-UI-5（entry は toolkit-free）のみ。

### INFERENCE

- C1〜C6 は A10・A11・A12 を書き換えず充足する。
- `run_shell_with_project(path: &Path) -> Result<(), ShellError>` は **期待 shape の記録のみ**であり、最終公開名・signature は `CU-109` 実装時に固定する。本粒は公開 API signature を凍結しない。
- Fable 5 read-only 助言（CU-G04SC0 事実節 O1 相当: 0/1 positional argv を最小 caller 候補とする推奨）は **未検証助言であり authority ではない**。主担当 Codex が再照合したが、本文の証拠として引用しない。

不在（CF の「無い」）を順序・所有・可否の肯定証拠として結論に使わない。

## 5. DECISION

- **C1（Carrier と閉じた grammar）**: `motolii_ui_shell` は positional 引数を **最大1個** 受け付ける。
  - 1 引数 = 既存 project path をそのまま session-backed entry へ渡す;
  - 0 引数 = 既存の zero-path diagnostic baseline;
  - 2 引数以上、または flag 様の引数 = typed usage failure、非ゼロ exit、window なし、session なし;
  - env、config、cwd、recent-project、default-project から path を取らない。

- **C2（Entry 境界）**: 明示的な既存 project path を取る **additive・public・toolkit-free** な entry を **ちょうど1つ**、既存 shell 境界（`crates/motolii-ui/src/shell.rs`）に置き、`crates/motolii-ui/src/lib.rs` で `run_shell` の隣へ re-export する。caller は `src/bin` 別 crate のため public 必須（CF5）。
  - 期待 shape は `run_shell_with_project(path: &Path) -> Result<(), ShellError>`。**期待 shape の記録のみ**。最終名・signature は `CU-109` 実装時に固定し、本 docs 粒は公開 API signature を凍結しない。
  - binding は additive・1 つ・public・toolkit-free・明示既存 project path・既存 shell 境界・`run_shell` 隣 re-export のみ。
  - `run_shell()` の signature と zero-path diagnostic 意味は不変（CF1、CF6）。
  - merged `Option<&Path>` entry は作らない。
  - argv 解析は binary `main` のみ。library は process argv を読まない。

- **C3（Selector mapping）**: 1 引数は `CU-G04S` D2 session-backed disposition のみへ。0 引数は `CU-G04S` D4 diagnostic disposition のみへ。第三の startup mode は無い。New / Open / last-project / chooser は `CU-G04` が所有する。

- **C4（Path opacity）**: `motolii-ui` は path codec、canonicalization、存在 heuristic、初期化、migration 意味を追加しない。`&Path` を既存 `ProjectSession::open` へ渡す。失敗は additive な typed `ShellError` variant とし、既存 structured error を文字列へ潰さない。

- **C5（暫定 failure 処分・fail-closed）**: usage / open / recover 失敗は typed error と非ゼロ exit を返し、**製品 edit runtime も製品 edit window も起動前**に終了する。fallback Document、retry、auto-init、save、migrate、optional durability、live-only edit route は無い。recovery UX 提示は後続 `CU-G04` が所有し、本粒は fail-closed default のみ固定する。

- **C6（Test-flag 降格と CU-109 証拠）**: `MOTOLII_TEST_U2B1_DOCUMENT` は boolean test evidence のまま、**有効な session-backed path entry の後**でのみ尊重する。path・Document・session の選択や運搬には使わない。flag + 0 引数は smoke も製品 edit path も作らない。実 smoke の再係留は `CU-G04S` D6 に従い `CU-109` が所有する。`CU-G04SC` 完了後、唯一の PRODUCT-ASSET `DO` は `CU-109` とし、acceptance evidence は Apply roundtrip のみとする。

## 6. 非目標

- Rust、test、fixture、golden、script、package、lock、CI、asset の変更
- New/Open chooser、Save / Save As、Unsaved Changes、read-only-newer、recovery UX、checkpoint policy
- path codec / canonicalization、OS file association または double-click、一般 CLI、`--help`、`--version`、現行 VS-1 を超える互換・恒久 promise
- Document schema、journal / serde / min-reader、plugin 契約、`Command` / `JournalEdit`、公開 raw writer / journal / stack / peek API
- 具体 `Healthy` / `Poisoned` state、`CU-109` runtime 配線、`CU-110`、`CU-111`
- Rerun または React product-asset 作業
- 親 `CU-G04` を `DONE` にすること（`SPEC / DECIDE` 維持）
- `CU-110` / `CU-111` / `U3a-2Q-V` / `CU-106P` / `CU-106F` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `G0-6H` / `G0-9*` / `U4a` / VS-2 または無関係 lane の状態変更

## 7. 必須負例（負 oracle）

### 将来 `CU-109` order が記録すべき負例

1. test を削除しても session entry が製品 binary `main` から到達可能なままであること;
2. `MOTOLII_TEST_*` および製品 env 読取が project source を選択しないこと;
3. argv が唯一の製品 carrier であること — cwd / recent / default / config fallback なし;
4. `run_shell()` の exact signature と zero-path diagnostic disposition が不変であること;
5. missing / uninitialized / open-failure / recovery-failure が project data、edit runtime、受理された edit または publish、empty-editor fallback を作らないこと;
6. 2 引数以上または flag 様引数が window も session も作る前に失敗すること;
7. optional durability と第二 publish path が無いこと;
8. 本 docs 粒で test、golden、threshold 期待を変更しないこと。

### 本粒ローカル負例

- **N1**: 本粒で Rust / test / fixture / golden / script / package file を変更する。
- **N2**: `MOTOLII_TEST_*` または製品 env が path・Document・session を選択できると書く。
- **N3**: merged `Option<&Path>` entry、第三 startup mode、`run_shell()` signature または zero-path 意味の変更を「決定済み」と書く。
- **N4**: optional durability route、fallback Document、auto-init、retry、第二 publish path を許可と書く。
- **N5**: PRODUCT-ASSET lane の `DO` を 0 件または 2 件以上にする。
- **N6**: 「発注依存証跡」の既存行、過去 decision 本文 / PR / hash を書き換える（`CU-G04SC` 新規 1 行 append のみ可）。
- **N7**: guard / `check-docs.sh` 通過のため期待値・threshold・除外・固定 hash・fixture・依存を触る、lint 抑制を足す。
- **N8**: 不在を肯定証拠として結論に使う。
- **N9**: allowlist 外 file を 1 byte でも変更する、または allowlist 外へ新規 file を作る。
- **N10**: 結論を TODO / 保留で複数候補のまま残す、または mirror の一部だけ更新して stale を残す。

## 8. STOP 条件

- **S1**: C1〜C6 を閉じるのに `CU-G03` §3 / §4 / §7 / §8、`CU-109SP` §5、`CU-G04S` D1〜D7、粒度化 §8 W1 依存表の書き換えが必要になる。
- **S2**: 結論に concrete typed shape、凍結公開／非公開 API signature、payload、transport、journal 形式、具体 `Healthy` / `Poisoned` state、poison 実装が必要になる。
- **S3**: carrier を閉じるのに未決 New / Open / Save / Unsaved UX 判断、path codec / canonicalization / OS-association 意味が必要になる。
- **S4**: ALLOWED_FILE 以外へ 1 byte でも変更が要る。
- **S5**: guard / `check-docs.sh` を通すために期待値・threshold・除外・固定 hash・fixture を触りたくなる。
- **S6**: 変更後に PRODUCT-ASSET `DO` が 1 件（`CU-109`）にならない形しか書けない。
- **S7**: 本 order、AUTHORITY 一覧、CF1〜CF13 以外の repo 横断調査や別 worktree が判断に必要になる。
- **S8**: ledger の `CU-G04SC` 行が `DO` でない、または DEPENDENCY のいずれかが `DONE` でないことを発見した。
- **S9**: `CU-110` 配置、UI surface、製品 window、`G0-6H`、`G0-9*`、`U4a` / VS-2 の状態を動かす必要が出る。
- **S10**: 1 positional grammar を書くのに一般 CLI、`--help` / `--version`、OS file-association 行為も同時に決める必要が出る。

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| **`CU-G04SC`** | **DONE** | argv carrier、entry 境界、path opacity、fail-closed failure disposition、test-flag demotion を docs で閉じた |
| **`CU-109`** | **DO** | 共有配線実装粒。acceptance evidence は Apply roundtrip のみ。session-backed entry は明示 project path を要求し、no-session では edit runtime を構築しない |
| `CU-G04`（親） | **SPEC / DECIDE** | 維持。New/Open/Save/Unsaved/read-only-newer/recovery UX/checkpoint policy を引き続き所有 |
| `CU-110` / `CU-111` | **WAIT** | 据え置き |
| `U3a-2Q-V` / `CU-106P` / `CU-106F` | **WAIT** | 据え置き |

PRODUCT-ASSET lane の `DO` は `CU-109` ただ一件とする。
