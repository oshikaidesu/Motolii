# 裁定159: `next/shell/motolii-shell` pane crate 分割 縫い目調査

読むだけの実測。判断・裁定はしない。対象 checkout: `/Users/member_ottoto/rust_ae/Motolii`
(main, `next/shell/motolii-shell/`)。行数は 2026-08-21 時点の `wc -l`。

## 0. ファイル一覧(行数)

```
src/lib.rs              2615   ← Shell/Message/Session の正本 + 全 pane の update ロジックの実体
src/inspector_pane.rs   1402
src/tokens.rs           1008
src/screenshot.rs        976
src/stage.rs              563
src/settings_pane.rs      533
src/clipboard.rs          409
src/fixture.rs            364
src/timeline/projection.rs 461
src/timeline/canvas.rs    292
src/timeline/mod.rs       282
src/timeline/key_rows.rs  329
src/timeline/key_gesture.rs 261
src/timeline/input.rs     258
src/timeline/lane_bar.rs  223
src/timeline/clip_gesture.rs 205
src/metrics.rs              70
src/main.rs                 86
tests/suite/*.rs (16本)    ~4700
```

timeline/ 合計 = 2,311 行(9ファイル。プロンプトの「8ファイル」は `mod.rs` を除いた数と一致)。

---

## 1. 結合地図

### 1.1 `Message` enum(lib.rs:153-334、48 腕)の pane 内訳

| 分類 | 腕数 | 内訳 |
|---|---|---|
| core(assembler に残る想定) | 9 | `Undo` `Redo` `ScrubTo` `Select` `AddLayer` `AdmitPaths` `DropReceived` `FlushDrops` `TokensFileChanged` |
| cross-cutting(2 pane が読む。core 残留が妥当) | 2 | `KeyboardModifiersChanged`(timeline drag と inspector drag 両方が読む) `EscapePressed`(`update()` 内で timeline→timeline-key→inspector の順に3種のキャンセルを試す、lib.rs:724-728) |
| Inspector | 8 | `InspectorFieldInput/Submit` `InspectorNameInput/Submit` `InspectorToggleHidden` `InspectorValuePressed` `InspectorPointerMoved` `InspectorPointerReleased` |
| Timeline | 14 | LaneBar 3(`ToggleMute/Solo/Lock`)+ clip drag 4(`BarGrabbed/DragMoved/DragReleased/DragCancelled`)+ key 行 2(`KeySelect/DeleteSelectedKeys`)+ key drag 5(`KeyGrabbed/KeyDragMoved/KeyDragReleased/KeyDragCancelled/NudgeKeyframe`) |
| Settings | 7 | `ToggleSettingsPanel` `ToggleCheckerboard` `SettingsBackgroundPreset` `SettingsBackgroundChannelInput/Submit` `UiScaleInput/Submit` |
| Stage(観測カメラ) | 2 | `StageObserve` `ResetToRenderCamera` |
| Clipboard | 6 | `CopyLayer` `PasteLayer` `CutLayer` `DuplicateLayer` `SelectAllLayers` `DeselectAllLayers` |

合計 9+2+8+14+7+2+6 = 48。

### 1.2 `Shell::update`(lib.rs:650-781)が触る内部状態

`update` 自体は 130 行の match 1本(lib.rs:655-778)で、全腕が `self.<field>` 直書き
か private メソッド呼び出しのどちらか。**pane ロジックの実体は private メソッドとして
lib.rs 側に置かれている**(pane モジュール側にはロジックが無い)。file:line と、
触る private field:

| private メソッド(lib.rs) | 行範囲 | 行数 | 触る Shell field |
|---|---|---|---|
| `commit_inspector_field` | 959-1007 | 49 | `doc` `session.selection` `inspector_field_draft` |
| `commit_inspector_name` | 1008-1024 | 17 | `doc` `session.selection` `inspector_name_draft` |
| `toggle_inspector_hidden` | 1025-1045 | 21 | `doc` `session.selection` |
| `start_field_drag` | 1693-1727 | 35 | `doc` `session.selection` `inspector_drag` |
| `continue_field_drag` | 1728-1765 | 38 | `doc` `inspector_drag` `keyboard_modifiers` |
| `finish_field_drag` | 1766-1796 | 31 | `doc` `inspector_drag` |
| `enter_field_editing` | 1797-1818 | 22 | `inspector_field_draft` |
| `cancel_inspector_interaction` | 1819-1838 | 20 | `doc` `inspector_drag` `inspector_field_draft` `inspector_name_draft` |
| **Inspector 小計** | | **233行** | |
| `toggle_layer_hidden` | 1046-1064 | 19 | `doc` |
| `toggle_layer_solo` | 1065-1085 | 21 | `doc` |
| `toggle_layer_lock` | 1086-1113 | 28 | `doc` |
| `start_timeline_drag` | 1114-1154 | 41 | `doc` `timeline_drag` |
| `continue_timeline_drag` | 1155-1218 | 64 | `doc` `timeline_drag` `keyboard_modifiers` |
| `finish_timeline_drag` | 1219-1239 | 21 | `doc` `timeline_drag` |
| `cancel_timeline_drag` | 1240-1249 | 10 | `doc` `timeline_drag` |
| `apply_key_selection` | 1250-1299 | 50 | `doc` `session.selected_keys` `session.key_anchor` |
| `delete_selected_keys` | 1300-1352 | 53 | `doc` `session.selected_keys` |
| `start_timeline_key_drag` | 1353-1419 | 67 | `doc` `session.selected_keys` `timeline_key_drag` |
| `continue_timeline_key_drag` | 1420-1478 | 59 | `doc` `timeline_key_drag` `keyboard_modifiers` |
| `finish_timeline_key_drag` | 1479-1502 | 24 | `doc` `timeline_key_drag` |
| `cancel_timeline_key_drag` | 1503-1509 | 7 | `doc` `timeline_key_drag` |
| `nudge_keyframe` | 1510-1543 | 34 | `doc` `session.selected_keys` |
| `commit_key_frames`(共通ヘルパ) | 1544-1629 | 86 | `doc` |
| **Timeline 小計** | | **584行** | |
| `apply_background_preset` | 1630-1642 | 13 | `doc` |
| `commit_background_channel` | 1643-1669 | 27 | `doc` `background_draft` |
| `commit_ui_scale` | 1670-1692 | 23 | `tokens` `ui_scale_draft` |
| **Settings 小計** | | **63行** | |
| `copy_layer` | 794-807 | 14 | `doc` `session.selection` `clipboard` |
| `paste_layer` | 808-824 | 17 | `doc` `clipboard` |
| `cut_layer` | 825-852 | 28 | `doc` `session.selection` `clipboard` |
| `duplicate_layer` | 853-874 | 22 | `doc` `session.selection` |
| `select_all_layers` | 875-882 | 8 | `doc` `session.selected_layers` |
| `deselect_all_layers` | 883-891 | 9 | `session.selected_layers` `session.selected_keys` |
| **Clipboard 小計** | | **98行** | |

**含意**: pane crate 分割は「モジュールを crate へ移す」だけでは終わらない。update ロジック
584+233+98+63 = **978行**が lib.rs から各 pane 側へ**移動**して初めて「pane が自分の
write ロジックを持つ」形になる(現状は timeline/inspector_pane/settings_pane モジュールは
**描画専用**で、書き込みは1つも持っていない)。

### 1.3 pane モジュール間の相互 import(誰が誰を使うか)

```
inspector_pane.rs
  → crate::tokens::{Colors,Dimensions,Ink,TextWeight}   (56)
  → crate::{Message, Session}                            (57)

settings_pane.rs
  → crate::inspector_pane::{section_header, value_input_style}  (40)  ★pane間依存
  → crate::tokens::{Colors,Dimensions}                    (41)
  → crate::{button_style, Message}                        (42)  ★button_style は lib.rs 定義(pub(crate) fn, lib.rs:2593)

timeline/mod.rs
  → crate::tokens::{Colors,Dimensions}                    (76)
  → crate::{Message, Session}                             (77)
timeline/projection.rs → crate::Session                   (7)   ★Session←→timeline 循環の起点(§2.3)
timeline/key_rows.rs   → crate::Message                   (33)
timeline/input.rs      → crate::Message                   (28)
timeline/lane_bar.rs   → crate::tokens::Dimensions         (22)
timeline/key_gesture.rs → crate::timeline::clip_gesture::snap_frame (27, timeline 内部のみ)

stage.rs
  → crate::tokens::{Colors,Dimensions}                    (36)
  → crate::Message                                        (37)

screenshot.rs (pane ではなく検分器具)
  → crate::inspector_pane::SelectionProjection            (35)
  → crate::tokens::{Colors,Dimensions}                    (36)
  → crate::{settings_pane, timeline_pane, Shell}           (37)  ★4方向(3 pane + Shell 本体)に依存する唯一のファイル

tokens.rs / metrics.rs / clipboard.rs / fixture.rs
  → crate:: 依存ゼロ(motolii_store / std / iced のみ)      ★既に自己完結
```

`timeline/input.rs` と `timeline/key_rows.rs` は `crate::Message` の**中の core 腕**
(`Message::Select`, `Message::ScrubTo` — input.rs:95,139,160)も直接組み立てている。
pane 側 widget が「自分の腕」だけでなく「core の腕」も発行している、という点は
分割後の pane-local `Message` 設計で吸収が要る(§3 参照)。

---

## 2. crate 化の障壁

### 2.1 Shell の private 状態への直接触れ

pane モジュール(inspector_pane.rs / timeline/* / settings_pane.rs / stage.rs)自体は
`&Shell` を受け取らない。受け取るのは `&StoreView` / `&Session` / 各種投影型
(`Dimensions`/`Colors`/`FieldDraft`/...)だけ — **これ自体は分割に有利**(pane が
Shell の private field を直接読む箇所はゼロ、既に一枚岩ではない)。

障壁は逆方向: **書き込み側の実ロジック(§1.2 の978行)が pane モジュールにではなく
lib.rs の `impl Shell` に private メソッドとして置かれている**ため、これらは
`self.doc`(`Document`、private field)を直接叩く。`Document` への書き口を
「pane crate から呼べる」形にするには、`&mut Document`(または同等の trait)を
関数引数として明示的に渡す形へ書き換える必要がある(現状はレシーバ `&mut self` 経由
の暗黙アクセス)。

### 2.2 `tokens`(全 pane 共有)

`tokens.rs` は crate:: 依存ゼロで完全に自己完結(§1.3)。**共有 crate 化の最有力候補**
かつ**最も安全な最初の切片**(依存する側はいるが、tokens 自身は誰にも依存しない=
純粋な leaf)。`tokens::watch_subscription`(debug のみ)が `notify` crate を direct 依存
に持つ(Cargo.toml コメント参照)ので、tokens crate はこの依存も一緒に引き継ぐ。

### 2.3 `Session` ⇄ `timeline::KeySelector` の循環

- `Session`(lib.rs:120-138)は `selected_keys: Vec<timeline::KeySelector>` と
  `key_anchor: Option<timeline::KeySelector>` を持つ → **Session → timeline 型への依存**
- `timeline::projection::rows`(timeline/projection.rs:35)は `session: &Session` を
  引数に取る → **timeline → Session への依存**

この2つが同時に立っているため、「Session を timeline-pane 配下に置く」も
「Session を root にそのまま残して timeline-pane が root に依存する」もどちらも
成立しない(前者は root/他 pane が Session を読めなくなる、後者は root→pane→root の
循環になる)。`KeySelector`(timeline/projection.rs:109 定義、fields は
`LayerId`/`PropertyId`/`i64` のみで timeline 固有の構造は無い)を Session と同じ層へ
**移設**すれば解ける — 既存の `pub use timeline as timeline_pane;`(lib.rs:38)と同じ
「型 alias で外部参照を壊さない」手口がそのまま使える。

### 2.4 `settings_pane` → `inspector_pane` の pane間依存

`settings_pane.rs:40` が `inspector_pane::{section_header, value_input_style}` を、
`settings_pane.rs:42` が `crate::button_style`(lib.rs:2593 定義)を再利用している
(settings_pane.rs 冒頭 doc comment: 「2箇所で別の意匠を発明しない」)。これは
settings-pane crate が inspector-pane crate + root(button_style の置き場)の**両方**
に依存する形になり、「レーンが自分の pane crate だけを買える」目的に反する
(settings だけ触りたいレーンが inspector-pane のフルビルドも背負う)。
`button_style`/`section_header`/`value_input_style` は tokens(`Dimensions`/`Colors`)
だけを読む純関数(スタイル計算のみ、Session/Document 非依存)なので、共有スタイル
crate(tokens 隣接 or tokens 内)への吸い上げで切れる。

### 2.5 `screenshot.rs` は pane ではなく4方向依存の検分器具

`screenshot.rs`(976行)は `inspector_pane` + `settings_pane` + `timeline_pane` +
`Shell` 本体の4つ全部に依存する(§1.3)。pane crate のどれか1つには属せない —
分割後も **assembler crate(root `motolii-shell`)側に残す**しかない。
`tests/suite/settings_drive.rs` がこの screenshot 経由で settings を検分しているため
(`use motolii_shell::{screenshot, ...}`)、settings-pane crate 単体のテストとは別に
screenshot 経由の統合テストは root crate に残り続ける。

### 2.6 tests の pane 跨ぎ

`tests/suite/*.rs` は**全ファイルが** `use motolii_shell::{Message, Shell}` — つまり
今の drive 系試験は**例外なく** Shell(root)を直接叩く形で、pane crate 単体に
閉じたテストは1つも無い(pane-local 単体テストは別に `src/**/*.rs` 内の
`#[cfg(test)] mod tests` として既に9ファイルに存在 — clip_gesture.rs / hit.rs /
key_gesture.rs / projection.rs / stage.rs / clipboard.rs / inspector_pane.rs /
tokens.rs(5ブロック)/ settings_pane.rs。これらは既に pane 内で自己完結しており
**分割の追加コストなしでそのまま pane crate 側の unit test になる**)。

`Message::` 使用回数を pane 別に集計(`tests/suite/*.rs` 全体、grep実測):

| pane | 回数 | 主な出現ファイル |
|---|---|---|
| core(Undo/Redo/ScrubTo/Select/AddLayer/Admit/Drop/Flush/Tokens) | 100+ | 全ファイル(AddLayer 52, Undo 27, ScrubTo 16 が突出) |
| Timeline | 130 | drive.rs, timeline_key_gesture_drive.rs, timeline_preview_drive.rs, timeline_key_rows_drive.rs |
| Inspector | 33 | inspector_drive.rs, inspector_pixel_fence.rs |
| Settings | 21 | settings_drive.rs, ui_scale_fence.rs |
| Stage | 15 | observation_camera_drive.rs |
| Clipboard | 20 | clipboard_drive.rs |
| cross(KeyboardModifiersChanged/EscapePressed) | 6 | drive.rs 他 |

`drive.rs`(828行、Message:: 76回)は core + Timeline(LaneBar/ClipDrag)の混在で、
単一 pane に属さない「core シナリオの統合テスト」。分割後も root crate 直下の
integration test に残る想定。

---

## 3. iced 標準型(pane-local `Message` を親が畳む形)の確認

### 3.1 現状の障壁が示す結論

§1.3 の通り `timeline/input.rs`・`timeline/key_rows.rs`・`inspector_pane.rs`・
`settings_pane.rs`・`stage.rs` は**すでに** widget コールバックの中で `Message::Xxx`
を直接構築している。crate 分割後、これらのファイルは root の `Message`(root crate に
定義)を参照できない(pane crate → root crate の依存は循環になるので禁止)。
**したがって「pane ローカル Message を親が畳む」構成は、選択可能な案の1つではなく、
分割を成立させるために構造上必須になる**(現状の直接構築を維持したいなら pane は
crate 化できない)。

### 3.2 機械的に済むか

`Message::TimelineBarGrabbed{..}` → `Message::Timeline(timeline::Message::BarGrabbed{..})`
のような、腕名のプレフィックス剥がし+ wrap は名前衝突が無いため
**正規表現による機械的置換で済む**(実測: `Message::Timeline` prefix を持つ腕は
`TimelineBarGrabbed`/`TimelineDragMoved`/... の形で他腕と衝突しない命名になっている
— pane 名がそのまま腕名の prefix なので、`s/Message::Timeline(\w+)/Message::Timeline(timeline::Message::\1)/`
に近い1パスで足りる)。対象は `tests/suite/*.rs` の Timeline 130箇所・Inspector 33箇所・
Settings 21箇所・Stage 15箇所・Clipboard 20箇所 = **219箇所**。

**機械的に済まない例外が2つ**:
1. `Message::Select`/`Message::ScrubTo`(core 腕)が `timeline/input.rs` 内部からも
   発行される(§1.3)。pane-local `Message` に `Select`/`ScrubTo` 相当の腕を複製するか、
   root の `Message` をそのまま `Task<Message>` の型引数として pane の canvas
   `Program` に持たせるか(=完全な分離を諦める)の判断が要る — **これは判断要**。
2. `KeyboardModifiersChanged`/`EscapePressed` は core 側で複数 pane のキャンセルを
   順番に試す(lib.rs:724-728)ため、pane-local Message へ分解すると
   「どの pane のキャンセルが先か」という**意味を持つ順序**を assembler 側の
   `update` がどう再現するかの設計判断が要る(機械的ではない)。

### 3.3 pane 側ロジック移設(§1.2 の978行)は判断要

Message の wrap 自体は機械的だが、私法メソッド(`start_timeline_drag` 等)の中身が
`self.doc`/`self.session` を直接読み書きしている形からの書き換え——
「`&mut Document` と `&mut Session` を明示引数で受け取る自由関数、または
pane crate 内の小さな struct(現行の `TimelineDragState`/`FieldDragState` 相当)の
メソッドへ変換する」——は**構造判断そのもの**であり、機械的な置換の範囲を超える。

---

## 4. ビルド税の見積り

### 4.1 各 pane crate の推定行数(移設後、§1.2 の lib.rs 側ロジックを加算)

| crate 案 | 推定行数 | 内訳 |
|---|---|---|
| `motolii-shell-tokens` | ~1,008 | tokens.rs そのまま(依存ゼロ leaf) |
| `motolii-shell-metrics` | ~70 | metrics.rs そのまま(debug cfg leaf) |
| `motolii-shell-fixture` | ~364 | fixture.rs そのまま(dev-instrument leaf) |
| `motolii-shell-clipboard` | ~409 | clipboard.rs そのまま(leaf、Session 触る glue は残留) |
| `motolii-shell-state` | ~150(新規) | `Session`(19) + `KeySelector`(移設、~15) + 周辺 doc。leaf、motolii_store のみ依存 |
| `motolii-shell-inspector-pane` | 1,402 + 233 ≈ **1,635** | inspector_pane.rs 全体 + lib.rs 私法 Inspector ロジック |
| `motolii-shell-settings-pane` | 533 + 63 ≈ **596** | settings_pane.rs 全体 + lib.rs 私法 Settings ロジック(+ 共有スタイル crate 依存) |
| `motolii-shell-timeline-pane` | 2,311 + 584 ≈ **2,895** | timeline/ 9ファイル全体 + lib.rs 私法 Timeline ロジック(最大) |
| `motolii-shell-stage` | ~563 | stage.rs そのまま(純関数、Message 依存は pane-local 化) |
| root `motolii-shell`(assembler) | 2615 − 978(移設分) ≈ **1,637** | `Shell` struct・root `Message`・`view` 組立・`refresh_frame`・`admit`・undo/redo・`screenshot.rs`(976、cross-cutting のため残留)・`main.rs` |

移設後の総行数は現状とほぼ同じ(978行が lib.rs から3 pane crate へ移るだけ)。
**crate 数の増分でコンパイル単位が増える分のオーバーヘッドはあるが、コード総量は
増えない**(型の重複は `KeySelector` の1箇所移設のみ)。

### 4.2 レーンが払う `-p` 集合(分割後)

| レーン | `-p` 集合 | 備考 |
|---|---|---|
| timeline レーン | `motolii-shell-tokens` + `motolii-shell-state` + `motolii-shell-timeline-pane` | 現状: `motolii-engine`(GPU 初期化含む)・`iced_test`・`inspector_pane`・`settings_pane`・`screenshot` を道連れにフルビルドする 15,858行。分割後: timeline 内部の unit test(既存 4ファイルの `mod tests`)は上記3 crate だけで完結 — **motolii-engine 非依存**(engine は `Shell::new()` の GPU 初期化でのみ要る) |
| inspector レーン | `motolii-shell-tokens` + `motolii-shell-state` + `motolii-shell-inspector-pane` | 同上、timeline/settings/stage 非依存 |
| settings レーン | `motolii-shell-tokens` + `motolii-shell-state` + (共有スタイル crate) + `motolii-shell-settings-pane` | §2.4 のスタイル共有 crate が要る。inspector-pane crate には依存しない形にできる(スタイル関数の吸い上げ後) |
| stage レーン | `motolii-shell-tokens` + `motolii-shell-stage` | 最小。`motolii_core`/`motolii_engine`(`ObservationCamera`)には依存するが、他 pane 非依存 |
| 統合(drive.rs 等)・screenshot 経由の検分 | root `motolii-shell`(全 pane crate + `motolii-engine`) | 現状と同じ規模のフルビルドのまま — §2.5/2.6 の通り、これらは1 pane に閉じないので分割の恩恵を受けない |

---

## 5. crate 分割案(依存方向)

```
layer 0(leaf、motolii_store/motolii_core のみに依存)
  motolii-shell-tokens
  motolii-shell-metrics
  motolii-shell-fixture
  motolii-shell-clipboard
  motolii-shell-state         … Session + KeySelector(timeline/projection.rs から移設)
  motolii-shell-chrome(新規, 小)… button_style/section_header/value_input_style(tokens 依存のみ)

layer 1(pane crate。tokens + state (+ chrome) に依存、pane 同士には依存しない)
  motolii-shell-timeline-pane  … tokens, state
  motolii-shell-inspector-pane … tokens, state, chrome
  motolii-shell-settings-pane  … tokens, chrome            (state 不要 — Session を直接読まない)
  motolii-shell-stage          … tokens, motolii_engine(ObservationCamera)

layer 2(assembler、既存 crate 名を維持)
  motolii-shell … 上記全部 + motolii_engine + motolii_media + clipboard の glue
                   + screenshot.rs(cross-cutting) + main.rs + 統合 tests/
```

`tokens`/`timeline_pane` は既存コードの `pub use X as Y;` 慣用(lib.rs:38)を使えば、
root crate 側で `pub use motolii_shell_tokens as tokens;` のように再輸出できる —
**モジュール path を変えない分割なら `tests/suite/*.rs` の `use motolii_shell::tokens::...`
等は無改修で済む**(Message の wrap だけが唯一、機械的とはいえ書き換えを要る箇所)。

---

## 6. 分割切片の割り案

各切片の検収条件は共通: **挙動ゼロ変更・`cargo test -p motolii-shell` のテスト集合が
分割前後で一致**(既存 timeline 第2波第1切片=純ファイル分割、と同じ基準)。
重み3軸 = (a) 行数(ビルド税) (b) 対外 file:line 参照数(コードレビューの負荷) (c) 判断要度
(§3.3 のような設計判断が要るか、純移動で済むか)。write-set は「触るファイル」。

| 切片 | 内容 | 重み a/b/c | write-set | 依存順 |
|---|---|---|---|---|
| **切片1** | `motolii-shell-tokens` crate 抽出(コード無改変、`pub use` で `tokens` path 維持) | 低/低/低(§2.2: 既に依存ゼロ) | `Cargo.toml`(新crate)、`src/tokens.rs`→移動、`lib.rs`(mod宣言→pub use) | 最初(他の全切片の前提) |
| **切片2** | `motolii-shell-metrics` crate 抽出 | 低/低/低(依存ゼロ、cfg(debug_assertions) 分岐ごと移動) | `Cargo.toml`、`src/metrics.rs`→移動、`lib.rs`(48-73) | 切片1と独立、並行可 |
| **切片3** | `motolii-shell-fixture` crate 抽出(dev-instrument) | 低/低/低(依存ゼロ) | `Cargo.toml`(dev-dep)、`src/fixture.rs`→移動 | 切片1と独立、並行可 |
| **切片4** | `motolii-shell-clipboard` crate 抽出(`LayerSnapshot`/`Clipboard` 型のみ。§1.2 の glue 6関数は lib.rs 残留) | 低/低/低(依存ゼロ) | `Cargo.toml`、`src/clipboard.rs`→移動 | 切片1と独立、並行可 |
| **切片5** | `motolii-shell-chrome` 新設: `button_style`(lib.rs:2593)+ `section_header`/`value_input_style`(inspector_pane.rs から抽出)を tokens 依存の小 crate へ | 低/**中**(inspector_pane.rs・settings_pane.rs・lib.rs 3箇所の呼び出し元を書き換え)/低 | `inspector_pane.rs`(該当関数を切り出し)、`settings_pane.rs`(import 差し替え)、`lib.rs`(button_style 削除・import) | 切片1の後 |
| **切片6** | `motolii-shell-state` 新設: `Session` を lib.rs から移設 + `KeySelector` を `timeline/projection.rs` から移設(§2.3 の循環解消) | 中/**高**(Session 参照は `timeline/mod.rs`・`timeline/projection.rs`・`inspector_pane.rs`・`lib.rs` 全体に及ぶ)/**中**(型の置き場は変わるが計算ロジックは無改変) | `lib.rs`(Session定義削除・re-export)、`timeline/projection.rs`(KeySelector定義削除・re-export)、上記全 import 元 | 切片1・5の後、切片7-9の前提 |
| **切片7** | `motolii-shell-timeline-pane` 抽出: `timeline/` 9ファイル移動 + §1.2 Timeline private メソッド584行を pane-local `Message`/`update` へ移設(§3.3 の判断要含む) | **高**/**高**/**高** | `timeline/*.rs`(9)、`lib.rs`(Message腕14個の wrap、584行の移設先書き換え)、`tests/suite/*.rs`(Timeline 130箇所の Message:: wrap) | 切片6の後 |
| **切片8** | `motolii-shell-inspector-pane` 抽出: `inspector_pane.rs` 移動 + Inspector private 233行の移設 | 中/中/中(Timeline ほど深い状態機械ではないが drag state の移設判断は要る) | `inspector_pane.rs`、`lib.rs`(Message腕8個の wrap、233行の移設先)、`tests/suite/*.rs`(Inspector 33箇所) | 切片5・6の後、切片7とは独立(並行可) |
| **切片9** | `motolii-shell-settings-pane` 抽出: `settings_pane.rs` 移動 + Settings private 63行の移設 | 低/低(§2.4 解消済みなら inspector-pane 非依存)/低 | `settings_pane.rs`、`lib.rs`(Message腕7個の wrap、63行の移設先)、`tests/suite/*.rs`(Settings 21箇所) | 切片5の後、切片7・8とは独立(並行可) |
| **切片10** | `motolii-shell-stage` 抽出: `stage.rs` 移動(純関数のみ、私法メソッドの移設は無い — `refresh_frame`/`observation` 直書きは assembler 残留が妥当) | 低/低/低(既に純関数化済み) | `stage.rs`、`lib.rs`(Message腕2個の wrap)、`tests/suite/observation_camera_drive.rs`(15箇所) | 切片1の後、他切片と独立(並行可) |

**依存順の要旨**: 切片1(tokens)が全ての前提。切片6(state: Session/KeySelector 移設)が
timeline-pane 抽出(切片7)の必須前提(§2.3 の循環を解かないと timeline crate 化が
成立しない)。切片5(chrome)は settings-pane 抽出(切片9)の前提(§2.4)。
切片7・8・9・10 は互いに独立で並行実行できる — ここが並列の天井を上げる本体
(4レーン同時に動かせる)。screenshot.rs(§2.5)と統合 tests(§2.6)は最後まで
assembler crate に残る = どの切片でも触らない。
