# CU-201P-HOST-INPUT 通常Product Host入力背骨決定

- 日付: 2026-08-04
- 状態: **SPEC DONE / ADOPT・WRAP**
- 親: `CU-201P` / M3 U3b / VS-2
- 後続: PRODUCT `CU-201P-HOST-INPUT`

## 1. 粒の目標

通常Product Hostで開始したnative Timeline gestureを、同じwindowの論理Escape、focus loss、
pointer lossで確実にcancelする。物理eventは一つのapproved adapterで既存のtoolkit非依存入力へ
正規化し、gesture開始・終了・cancelを既存`InputRouter`へ配送する。cancelはTransientだけを破棄し、
Document、journal、history、revision、published snapshotを変更しない。

この粒はtrim挙動を追加しない。先に入力の背骨だけを閉じ、その後に既存
`CU-201P-TRIM` WIPを再baseしてLeft/Right trimへ戻る。

## 2. 現行コード事実と矛盾

- `product_runtime_adapter.rs`は`CursorMoved`、`MouseInput`、`Focused(false)`、`CursorLeft`を
  `ProductApp`へ渡すが、raw input guardのapproved閉集合はwindow lifecycle 5 eventだけである。
- `product_runtime.rs`はraw `winit::event::ElementState`と`MouseButton`を受けるため、現行local mainの
  `raw_input_boundary::workspace_product_sources_have_no_raw_toolkit_input`は失敗する。
- `KeyToken::Escape`、`motolii.gesture.cancel`、`DomainIntent::CancelInFlightGesture`、
  `NormalizedInput`、`SafetyInterrupt`、`InputRouter`は既に存在する。
- Product HostのUndo/Redoは、parentへfocusを移した時だけ有効になる`host_pointer_capture.rs`の
  macOS AppKit local monitorが`EffectiveTrigger`を生成する。これは既存のWebView/parent focus橋であり、
  Escapeを生成しない。
- `CU-201P-MOVE`と`CU-201P-TRIM-S`はEscapeのzero-writeを要求するが、通常Product Hostから
  Timeline gestureへ届くEscape producerは無い。

したがってGAPは新しいCancel意味ではなく、既存cancel intentへ届く一つの通常Host adapterと、
既存raw event漏洩の収束である。

## 3. 既知実装検索と採択

| 候補 | 確認した収束解 | 裁定 |
|---|---|---|
| [winit 0.30.13 `WindowEvent`](https://docs.rs/winit/0.30.13/winit/event/enum.WindowEvent.html) / [`Key`](https://docs.rs/winit/0.30.13/winit/keyboard/enum.Key.html) | window宛て`KeyboardInput`の`logical_key`で`NamedKey::Escape`を扱い、`KeyEvent`にpress/release、repeat、textを持つ | **ADOPT**。physical keyやplatform scancodeでEscapeを判定しない |
| [winit 0.30.13 control-flow example](https://github.com/rust-windowing/winit/blob/v0.30.13/examples/control_flow.rs#L89-L114) | `KeyboardInput`のlogical keyとPressedを一つのwindow event adapterでactionへ変換 | **PATTERN**。action自体は既存Motolii `EffectiveTrigger`へ写す |
| [egui-winit 0.35.0 event adapter](https://github.com/emilk/egui/blob/0.35.0/crates/egui-winit/src/lib.rs#L282-L421) | raw mouse/cursor/IME/keyboard/focusをadapter一箇所で正規化し、synthetic key pressを捨てる | **PORT / REDUCE**。private helperの閉じたkey集合だけを移し、egui stateやevent型は製品runtimeへ持ち込まない |
| [egui-winit 0.35.0 IME処理](https://github.com/emilk/egui/blob/0.35.0/crates/egui-winit/src/lib.rs#L562-L635) | WindowsのIME処理済み`NamedKey::Process`を通常shortcutから除外する | **PATTERN**。press-only adapterで`Process`を捨て、physical fallbackしない |
| [Qt 6 `QKeySequence::Cancel`](https://doc.qt.io/qt-6/qkeysequence.html#StandardKey-enum) | Cancelを機能固有key branchでなくstandard commandとし、全主要desktopでEscapeへ収束 | **PATTERN**。既存stable command `motolii.gesture.cancel`を使う |
| [Blender modal operators](https://docs.blender.org/manual/en/4.5/interface/operators.html#modal-operators) | interactive operationはconfirmとcancelを分け、Escapeでmodal operationをcancelする | **PATTERN**。未commit Transientを破棄し、適用後Undoへ読み替えない |
| 既存`layout_runtime_adapter.rs` | raw egui eventをlayout専用actionと`SafetyInterrupt`へ限定変換 | **REUSE PATTERN**。fileやegui runtimeをProduct Hostへ流用しない |

### 採択しない候補

- `product_runtime.rs`内でfeature別にwinit key/mouseを読む: raw型がownerへ漏れ、同じ変換を増やすため棄却。
- AppKit local monitorへEscapeと一般shortcutを追加する: macOS専用でwinitのwindow/IME意味と二重化するため棄却。
- Browser Placeのpointer captureをTimelineへ流用する: source、capture generation、WebView tracking loopの責任が異なるため棄却。
- egui-winit `State`を通常Product Hostへ導入する: 製品runtimeへegui stateを戻すため棄却。
- raw input guardへ既存違反pathを文字列追加するだけの修正: typed seamとretirementを伴わず、違反を正当化するため棄却。

## 4. 背骨

```text
winit WindowEvent
  -> product_runtime_adapter.rs (唯一のraw owner)
  -> existing KeyToken::Escape / EffectiveTrigger
     or InputPhase / SafetyInterrupt / ImeGateState / primitive coordinates
  -> ProductApp
  -> existing InputRouter
  -> existing CancelInFlightGesture
  -> drop Timeline Transient + preview, semantic write 0
```

### 4.1 adapter境界

`product_runtime_adapter.rs`だけが通常Product Hostの次のraw inputを読める。

- `CursorMoved`: physical `[f64; 2]`へ薄く写し、scale factorによるlogical変換は既存ProductAppの一箇所を使う。
- primary `MouseInput`: `Pressed / Released`を既存`InputPhase::Press / Release`へ写す。
- `Focused(false)`: `SafetyInterrupt::WindowFocusLost`。
- `CursorLeft`: 現行gestureのcapture-loss相当として`SafetyInterrupt::PointerCaptureLost`。
- `ModifiersChanged`: winitのControl / Super / Alt / Shiftを既存
  `Modifier::Control / Meta / Alt / Shift`へ写し、ProductAppのprivate current modifiersだけを更新する。
- `KeyboardInput`: non-synthetic、non-repeatのPressedかつlogical `Key::Named(NamedKey::Escape)`だけを、
  既存`KeyToken::Escape`とcurrent modifiersの`EffectiveTrigger`へ写す。`NamedKey::Process`、文字key、
  unidentified/dead keyは捨て、physical key fallbackを行わない。
- `Ime`: Preeditが非空の間だけ既存`ImeGateState::PreeditActive`、Commit / DisabledでInactiveへ戻す。
  preedit中のEscapeをcommandへ配送しない。

raw型、winit enum、device identity、scancode、platform keycodeはadapterの外へ出さない。公開型、serde、
Document、journal、plugin契約へ追加しない。

### 4.2 commandとgesture lifecycle

- product builtin keymapを不変base version 2へ上げ、既存`motolii.gesture.cancel`のmodifierなしEscape
  bindingを追加する。新CommandIdは作らない。wire codecはv1のままとし、source builtin version 1の
  deltaは既存`SourceVersionMismatch`で拒否する。存在しない移行や暗黙適用を発明しない。
- Timeline gestureが実際に開始した時だけ`NormalizedInput::Phase(DragStart)`を既存`InputRouter`へ渡す。
- mouse releaseでgestureをtakeした時は`DragEnd`、focus/pointer lossは既存`SafetyInterrupt`、Escapeは
  `NormalizedInput::Command`として同じrouterへ渡す。
- `RouterOutput::Intent { intent: CancelInFlightGesture, .. }`または`SafetyCancel`だけが既存
  `cancel_window_pointer_gesture`へ届く。gesture不在のEscapeは`CancelCommandIgnored`でwrite 0。
- release / cancel後のduplicate eventはactive gesture不在によりwrite 0。
- AppKit `mac_history_trigger`、command inbox、host-command enable flag、poll routeは変更しない。
  WebView/parent focusを跨ぐhistory commandは別責任であり、この粒へ混ぜない。

## 5. 実装粒

### `CU-201P-HOST-INPUT`

```text
MECHANISM_CLASS:
  desktop window input normalization and modal cancel delivery
KNOWN_IMPLEMENTATION_SEARCH:
  repo InputRouter/KeyToken/EffectiveTrigger/layout adapter;
  winit 0.30.13; egui-winit 0.35.0; Qt StandardKey::Cancel; Blender modal cancel
ADOPTION_ROUTE: ADOPT / WRAP / PORT
BUILD_JUSTIFICATION: NONE
BUILD: FORBIDDEN

ALLOWLIST:
  crates/motolii-ui/src/product_runtime_adapter.rs
  crates/motolii-ui/src/product_runtime.rs
  crates/motolii-ui/tests/raw_input_boundary.rs
  inline tests in changed src files only; no other test file

PRIMARY_ORACLE:
  logical Escape press -> existing cancel intent once during active Timeline gesture;
  product BuiltinKeymap version 2 binds modifier-free Escape to the existing command;
  source builtin version 1 delta keeps the existing typed mismatch behavior;
  existing Undo/Redo route remains byte-for-byte outside the diff;
  pointer coordinate and press/release meaning remain unchanged
NEGATIVE_ORACLE:
  synthetic/repeat/release/Process/preedit/unknown key and modified Escape -> no cancel command;
  gesture absent, duplicate Escape, focus loss followed by release -> semantic write 0;
  raw winit type outside product_runtime_adapter.rs -> guard failure;
  AppKit history command files absent from the diff
REPO_LANES:
  cargo test --locked -p motolii-ui --test raw_input_boundary
  cargo test --locked -p motolii-ui input_router
  cargo test --locked -p motolii-ui product_runtime
  cargo test --locked -p motolii-ui
  cargo clippy --locked -p motolii-ui --all-targets -- -D warnings
  cargo fmt --all --check
  git diff --check
EXTERNAL_GATES:
  normal Mac product window: body drag and trim-edge drag each cancel by Escape with write 0;
  focus loss and pointer loss each cancel with write 0;
  existing Undo/Redo still reach the existing commands without duplicate dispatch
```

`EXTERNAL_GATES`未実施は`EXTERNAL_GATE_PENDING`であり、repository greenで置換しない。

## 6. 非目標

- trim edge hit、delta、Trim command、snap、move/trim Document意味の変更。
- generic gesture coordinator、新しい公開input API、新しいCommandId、keymap wire codec version変更、
  builtin version 1 deltaの暗黙migration。
- Browser / Inspectorのtext editing、candidate window位置、U1d全OS IME受け入れ完了。
- Stage Place capture、WebView pointer bridge、layout egui adapterの統合。
- raw event allowlistの包括許可、device event、physical key、scancode、platform shortcut special-case。
- push、PR、main統合、M3完了宣言。

## 7. 状態遷移

- `CU-201P-HOST-INPUT-S`: `DONE / ADOPT・WRAP`
- `CU-201P-HOST-INPUT`: `DO`
- `CU-201P-MOVE`: 実装diffは保持するがEscapeとraw guardが再締結するまで`REOPEN / WAIT_HOST_INPUT`
- `CU-201P-TRIM`: WIPを保持し、host input実装が閉じるまで`WAIT_HOST_INPUT`
- 親`CU-201P`: `SPLIT / WAIT_TARGET`を維持

Host input実装後は独立reviewとnamed external gateを別に判定し、成功した時だけMOVE再締結とTRIM再開を行う。
