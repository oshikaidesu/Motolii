# U3a-1 headless Timeline owner/visibility分割決定

- 日付: 2026-07-26
- 状態: **決定**
- U3a-1: **SPLIT**
- U3a-1S: **DONE**
- U3a-1I: **DONE**

## 1. 分割の結論

`U3a-1`（headless Timeline projection）を実装前に内部分割する。直前の実装orderは「公開API変更STOP」と「多数のpub型を新規re-exportする成果」が同一order内で自己矛盾し、さらに`crates/motolii-ui/src/layout.rs`を「pub再export先例」と誤記したためCodex PRECHECKで拒否された。続く`U3a-1S` prepareは台帳未登録IDだったため`ORDER: STOP`になった。さらに直前の分割案は`U3a-1S`を`DO`、`U3a-1I`を`WAIT`のまま残し、次の`U3a-1S`粒が同じdocs decisionを再度書くno-opになるため、これもPRECHECKで拒否された。本変更でowner裁定とvisibility裁定を完全に確定し、`U3a-1S`を`DONE`にする。後続の同名docs粒は作らない。

| ID | 状態 | 一成果 | 依存 | 合格 | STOP |
|---|---|---|---|---|---|
| U3a-1 | `SPLIT` | 親。分割証跡のみ | U0a, U0b-1, U0b-2 | 発注依存証跡に`SPLIT`行が一意に存在する | — |
| U3a-1S | `DONE` | owner裁定とvisibility裁定を本文書で確定した | U0a, U0b-1, U0b-2 | 本文書§3/§4だけで`U3a-1I`のowner/visibilityが一意に決まる | — |
| U3a-1I | `DONE` | headless Timeline projection / layout / cull / hit-testの実装 | U3a-1S | 歴史回収§6の完成条件を小さな決定的Document fixtureで満たす | owner/visibility裁定の外へ出る必要が生じた |

## 2. 現行コード事実

BASE_SHA `4fdff7d459e2947e53cec45ff11e38b58d99ea88` 時点の`crates/motolii-ui`実測。本節以外のコード探索を`U3a-1I` closed orderの根拠にしない。

| # | 事実 |
|---|---|
| 1 | `crates/motolii-ui/src/lib.rs` は全moduleを非公開 `mod` で宣言し（`app` / `command_registry` / `diagnostic` / `display_slot` / `document_command_request` / `document_edit_runtime` / `domain_intent` / `input_router` / `interaction_state` / `keymap` / `keymap_codec` / `layout` / `layout_authority` / `layout_runtime` / `layout_runtime_adapter` / `render_worker` / `shell` / `state_ownership` / `static_preview`）、`pub use` で選択した型だけを再exportする |
| 2 | `interaction_state` は `pub use interaction_state::{InteractionState, InteractionStateMachine, InteractionTransitionError};` として再exportされ、integration test `crates/motolii-ui/tests/interaction_state.rs` が公開面から検証する |
| 3 | `state_ownership` は `pub use state_ownership::{UiStateLifetime, UiStateOwner};` として再exportされ、integration test `crates/motolii-ui/tests/state_ownership.rs` が公開面から検証する |
| 4 | `crates/motolii-ui/src/layout.rs`（U1a-2のtoolkit非依存panel layout正本）は `lib.rs` に `pub use layout::...` を**持たず**、型は `pub(crate)`（`pub(crate) enum PanelRole` 9行目、`SplitAxis` 35行目、`LayoutNode` 41行目、`PanelLayout` 56行目、`LayoutConstraints` 62行目、`SeparatorAction` 68行目、`LayoutAction` 76行目、`LayoutError` 88行目、`pub(crate) fn normalize_runtime_shares` 409行目）である。layout.rs はcrate内部限定moduleの先例であり、pub再exportの先例ではない |
| 5 | `crates/motolii-ui/tests/public_boundary.rs` が `src/` を走査し、公開itemに `egui:: / eframe:: / egui_wgpu:: / egui_winit:: / egui_tiles:: / winit:: / slint::` が現れることを拒否する |
| 6 | `crates/motolii-testkit/tests/ui_toolkit_dep_policy.rs` がCargo metadata経由でUI toolkit依存方向を検査する（GR-UI-5） |
| 7 | ルート `Cargo.toml` の workspace members に `motolii-timeline` は存在せず、`crates/` 配下にも当該crateは無い（実在は motolii-audio / cli / core / doc / eval / export / gpu / media / nodes / plugin / plugins-firstparty / render / testkit / transport / ui / ui-token-gen の16件） |

## 3. Owner裁定

- `U3a-1I` の実装owner は **`motolii-ui` crate内の、toolkit/renderer非依存な新規module**とする。
- 新規 `motolii-timeline` crate の新設は **`REJECT`**（[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)の裁定語）。理由は、現時点でconsumerが `motolii-ui` 一つしかなく、独立crateは長寿命の公開面と依存境界を先に増やすため。
- 再判定地点は **`U3a-2`**（windowed renderer / platform統合）に限定する。`U3a-2` で同ゲートを再適用し `PASS / REDUCE / STOP` を再判定する。本粒はcrate分離を恒久的に禁止しない。

## 4. Visibility裁定

- production consumerがまだ存在しないheadless moduleについて、**`lib.rs` からの `pub` 再export + `crates/motolii-ui/tests/` のintegration test** を許す。
- 根拠は §2-2 / §2-3 の `interaction_state` / `state_ownership` 先例のみ。§2-4 のとおり `layout.rs` は `pub(crate)` 先例であり、本裁定の根拠にしない（旧orderの誤記をここで訂正する）。
- 同時に、次の制約6点を裁定に含める。
  1. `motolii-ui` 以外のworkspace crateから参照・再exportしない
  2. `serde` 派生・serde属性・永続形式を持ち込まない
  3. 公開面にUI toolkit型を出さない（`public_boundary.rs` / `ui_toolkit_dep_policy` に適合）
  4. 外部依存（新規crate依存）を追加しない
  5. lint抑制（`#[allow(...)]` / `#![allow(...)]` / clippy allow）を 0 件にする
  6. 最初の実product consumer粒でvisibilityを再判定し、可能なら `pub(crate)` へ縮小する

本裁定は確定済みであり、`U3a-1I` orderは本節をauthorityとして直接引用する。

metrics / viewport のexact Rust signature（型名・関数シグネチャ・フィールド構成）を仕様へ焼かない。これらは [歴史回収 §6](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md) の現行semantics（`&Document` + caller注入metrics/viewport、`RationalTime` 正本、最終座標のみ `f64`、Manhattan diamond key > bar > none、typed unsupported、typed reject）の内側で、`U3a-1I` のclosed orderが閉じる。旧 `CU-G02` の「次PRODUCT-ASSET粒を `U3a-1` に固定」という選定判断は否定・撤回しない。本粒は同じ選定の内側で `U3a-1S` / `U3a-1I` へ分けるだけである。

## 5. 非目標

- Rustコード、`Cargo.toml`、`Cargo.lock`、workspace members、crate新設、module追加、型追加、関数シグネチャ確定
- `crates/` / `ui/` / `plugins/` / `spikes/` / `samples/` / `scripts/` 配下の一切の変更
- `docs/mocks-ui/` および `docs/mocks/` 配下の一切の変更（JSX / CSS / guard-test / fixture）
- `U3a-2`、`U2h-1`、`U2h-2`、`U3b`〜`U3f`、`U4a-*`、`U4c`、`U2c-2`、`CU-0A08IT`、Rectangle系CU粒の意味・状態・依存の変更
- `CU-G02` の選定判断の否定・撤回・再判定
- metrics / viewport / hit-test の exact Rust signature、tolerance、閾値、px値、色、DPIの決定
- 新しい状態語彙の新設、`状態語` 表の拡張、decision-index状態語彙の拡張
- serde面、Document意味、plugin契約、公開APIの新設・変更
- `U3a-1S` の後続docs粒、第二の分割決定文書、readiness map（`2026-07-25-parallel-lane-readiness-map.md`）の更新

## 6. STOP

次のいずれかに到達した時点で実装を止め、`ORDER: STOP` としてCodexへ返す。

1. 決定を書くために、公開API・Document意味・serde面・plugin契約・恒久形式の新規決定が必要になった
2. allowlist外のfileを1つでも変更する必要が生じた
3. `crates/` 配下のコードを読む以上のこと（編集・追加・削除）が必要になった
4. `U3a-1I` の完成条件を歴史回収§6より狭くする／広げる必要が生じた
5. metrics / viewport の exact Rust signature を仕様へ書かないと決定が閉じないと判明した
6. `CU-G02` の選定を否定しないと整合が取れないと判明した
7. [../implementation-ledger.md](../implementation-ledger.md) の `現在の並列レーン` に `U3a-1` の `DO` 行が存在しない、または `発注依存証跡` の `U0a` / `U0b-1` / `U0b-2` が `DONE` でないと判明した
8. `scripts/check-docs.sh` を通すために既存文書の状態語彙・索引規則を変更する必要が生じた
9. 新しい状態語（`SPLIT` 以外）を台帳へ導入する必要が生じた
10. owner/visibility裁定を本commitで閉じきれず、`U3a-1S` を `DO` のまま残したくなった
11. 変更後に `git status --porcelain` がallowlist外のpathを出力した

## 7. 完了証跡（U3a-1I）

`U3a-1I`は`motolii-ui` crate内の`timeline_projection` moduleとしてheadless Timeline projection / layout / cull / hit-testを実装し、[../implementation-ledger.md](../implementation-ledger.md)の`発注依存証跡`で`DONE`とする。次のPRODUCT-ASSET粒は別の明示選定待ちであり、本変更はowner/visibility裁定（§3/§4）と制約6点を変更しない。`U3a-1S`の後続同名docs粒は作らない。
