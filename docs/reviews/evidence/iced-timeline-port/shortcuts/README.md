# iced 近道キー移植 — 証拠

2026-08-19。レーン: `claude/shortcuts-20260819`(Timeline の近道キーを egui →
iced shell へ移植)。表と写像の正本は `crates/motolii-shell-iced/src/shortcuts.rs`。

## 移植元と対応

egui 側の正本は `crates/motolii-ui/src/timeline_editor/mod.rs` の `egui::Key::` 各所。

| キー | 意味 | 状態 | 備考 |
|---|---|---|---|
| Cmd+Z | undo | **既に実装済み**(この移植レーンの前から) | `timeline/canvas.rs::Program::update` |
| Shift+Cmd+Z | redo | **既に実装済み** | 同上 |
| Escape | 進行中ジェスチャの取消 | **既に実装済み** | 同上。`TimelineMsg::GestureCancelled` |
| Delete / Backspace | 選択の削除 | **既に実装済み** | 同上。`TimelineMsg::DeletePressed` |
| ← / → | コマ送り(Shift で ±10) | **既に実装済み** | 同上。`TimelineMsg::PlayheadStepped` |
| Cmd+A | 全選択 | **このレーンで実装** | 下記参照 |
| Space | 再生 / 一時停止 | **置かなかった** | 再生機構(`playing` / audio clock)が iced shell に無い(Q0) |
| L | loop | **置かなかった** | 同上 |
| Cmd+K | 分割 | **置かなかった** | `UiIntent` に `SplitClip` の口が無い(下記) |
| M | playhead へマーカー | **置かなかった** | `UiIntent` に `AddLocator` の口が無い(同上) |
| Cmd+G / Shift+Cmd+G / Cmd+D / Enter | group/ungroup/複製/rename確定 | 担当外 | 構造操作レーン |

`timeline/canvas.rs` の既存キー処理は **cursor 位置に関係なく** 窓の keyboard
event を受ける(`iced_test::Simulator` の既存テストが hover なしで叩けている
のが証拠)ので、実質もう window-level の近道として機能している。この移植
レーンは Cmd+A だけを `window_input.rs` へ新規配線した(呼び出し1箇所、
`crate::shortcuts::additional_window_shortcut`)。

## 見つかった intent の欠け(D2 の口が無い)

- `motolii_doc::Command::SplitClip` / `AddLocator` は D2(`crates/motolii-doc`)に
  在るが、iced 側の唯一の口 `ShellGateway::dispatch(UiIntent)`
  (`crates/motolii-ui/src/blitz_shell/intent.rs`)に対応する `UiIntent` 変種が
  無い。egui 版 `TimelineEditor::split_selected` / `tap_locator` は D2 を直接
  叩けるが、iced 版にその道は無い。**新しい intent は作らない指示**だったので
  実装せず、ここに報告する。

## Cmd+A の実装

`crates/motolii-shell-iced/src/timeline/pane.rs` に `TimelineMsg::SelectAllPressed`
を追加。`plan()` が `motolii_ui::timeline_rows::rows(&ctx.document,
&TimelineFoldState::default())` を読み、`RowKind::Object` の行(= 見えている
top-level 行。iced shell はまだ Group の開閉 UI が無いので既定 fold state が
「全部閉じ」と同義)だけを、**まだ選ばれていないものに限って**
`UiIntent::SelectLayer { additive: true }` にする。

`additive: true` は `add_to_selection`(= Cmd+click と同じ**トグル**)なので、
素朴に全行へ毎回出すと**既に選ばれている行が外れる**。`red.txt` はこの回帰の
入口(何も配線していない状態でのテスト失敗)、`command_a_selects_every_visible_object_row`
のテスト後半は「2回目の Cmd+A で選択が変わらない」ことも確かめている
(このトグル回帰そのものの oracle)。

## 証拠一覧

| file | 何の証拠か | 生成元 |
|---|---|---|
| `red.txt` | Cmd+A の配線前、受入テストが落ちていた記録(red 先行) | `cargo test -p motolii-shell-iced --test drive_timeline command_a_selects` |
| `green.txt` | 配線後、crate 全体(lib unit test 26件 + 全 integration test)が緑になった記録 | `cargo test -p motolii-shell-iced` |
| `shortcuts-legend.png` | 座席ありで撮った窓全体。status 帯の下に近道キーの提示行 `Cmd+Z=undo   Shift+Cmd+Z=redo   Esc=cancel gesture   Delete=delete selection   ←/→=step 1 frame (Shift = 10)   Cmd+A=select all` が出る(**実際に効くキーだけ** — Space/L/Cmd+K/M は出さない、Q0) | `cargo run -p motolii-shell-iced -- --project <fixture>.json --screenshot shortcuts-legend.png 120` |

## PNG の再現

```sh
# fixture project(lab_fixture の Document)を適当な path に保存してから:
cargo run -p motolii-shell-iced -- --project <path>/project.json \
  --screenshot docs/reviews/evidence/iced-timeline-port/shortcuts/shortcuts-legend.png 120
```

(`120` フレーム待つ — 合成が間に合わないと status 帯の legend が誤診されるため。)

## 気づいたが触らなかったこと

Inspector pane(`inspector_pane.rs`)にも近道キーの提示行が既に在る
(`space=play L=loop Cmd+G=group Del=delete drag name=reorder`、
`shortcuts-legend.png` の右上に写っている)。これは Space/L/Cmd+G のような
**このレーンで機構が無いと確認したキー**を含んでおり、Timeline 帯の legend
(このレーンが足した分)と同じ Q0 の問題を抱えている。ただし Inspector pane
はこのレーンの担当外(触っていない)なので、修正はせず報告のみ。
