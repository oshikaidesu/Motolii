# iced Timeline キー編集レーン(2026-08-19 M-6)証拠

egui → iced Timeline 移植の**キー編集**担当分。構造操作(Group/Rename/Lock/
畳み開閉)は別レーン(`claude/tl-structure-20260819`)が同時に走っている —
下記の柵を守って衝突を避けた。

## 撮った絵と、なぜ一時パッチが要ったか

property 行(`RowKind::Property`)は `TimelineFoldState::params_open` が
立った時だけ画面に出る。**その開閉のトグルは構造レーンの担当**で、この
worktree の時点(base `0d0b079d`)ではまだ配線されていない —
`canvas.rs::scene()` は `TimelineFoldState::default()`(全部閉じている)を
固定で使う。

自分の目で見て検収するために、`canvas.rs::scene()` の1行だけを

```rust
let mut fold = TimelineFoldState::default();
for (layer, _) in document.layers.iter() {
    fold.open_params(layer);
    fold.open_children(layer);
}
let rows = rows(&document, &fold);
```

へ**ローカルで一時的に**差し替え、`lab_fixture()`(egui 版と共用の fixture —
`param_keys` の正当性を審判するテストが使っているのと同じ document)を
`cargo run -p motolii-shell-iced -- --project <fixture>.json --screenshot
iced-property-rows.png 120` で撮った。**この一時パッチは撮影後に revert
済みで、提出した diff には入っていない**(`canvas.rs` は最終的に「描画1行の
呼び出し」「hit 判定の統合」だけが変更点)。

| ファイル | 内容 |
|---|---|
| `iced-property-rows.png` | Timeline pane 全体(1960×1300)。`Title scene`(Group)/`Shared left` が Position・Scale・Opacity の property 行を開いて表示 |
| `iced-rail-crop.png` | レール側の拡大(チップ色 + ラベル。M/S ボタンが object 行だけに出ることが見える) |
| `iced-diamonds-crop.png` | トラック側の拡大(菱形の形・色・位置) |
| `egui-same-doc-reference.png` | `/tmp/egui-same-doc.png` の写し(発注が指した目標画像)。**このドキュメントは params_open を開いていない**ので、bar/ルーラ/transport 帯など Property 行**以外**の一致確認に使い、菱形そのものの見比べには使えない(下記「egui との見比べで妥協した点」参照) |

## 自分の目で見た一致

- **菱形の形・寸法**: 8px 四方相当のひし形(egui 版 `d = 4.0`)。行の縦中央に
  正しく乗る
- **色**: 未選択はほぼ地に溶ける暗いグレー地(`#353535`)+ 明るい枠
  (`#eee`)— `docs/mocks-ui/public/timeline-library.css:6` `.key` の正本値
  そのもの。egui 版のスクリーンショットは無いが、egui 側のソース定数
  `KEY_IDLE = Color32::from_rgb(0x35,0x35,0x35)` /
  `ACCENT = Color32::from_rgb(0xe9,0xcf,0x72)`
  (`crates/motolii-ui/src/timeline_editor/mod.rs:89-94`)と**同じ数値**を
  そのまま使っているので、色の出所は egui 版と共通(新しい hex は発明していない)
- **レール**: チップ(param 種別色)+ ラベル(`Position` / `Scale` /
  `Opacity`)だけが出て、object 行の M/S ボタン・名前入力は出ない —
  egui 版 `RowKind::Property` 描画(チップ + ラベルのみ)と同じ絞り方
- **並び**: object 行の直後、`depth + 1` の字下げで並ぶ(`timeline_rows::rows`
  が既に持っている並び規則をそのまま使っている — 行モデルは作り直していない)

## egui との見比べで妥協した点(正直に列挙)

- **同一 document・params_open 済みの egui スクリーンショットが無い**。
  発注が指した `/tmp/egui-same-doc.png` は params を開いていない状態で
  撮られたもので、この worktree からは egui 側の別レーンを起動して
  「◇ を押して開いた」状態を新しく撮り直す時間を取らなかった。代わりに
  **egui 側のソースコードの色定数を直接引用**することで色の一致を保証した
  (上記)。次ラウンドで egui 側の `--screenshot` に params_open の状態を
  1枚追加できると、より直接的な比較になる
- **吸着(snap)が clip 端・他キーへ効かない**: egui 版のキー drag は
  `commit_drag_snapped`(`snap_candidates`)を通るが、この版はフレーム境界
  への吸着だけ(`pane.rs::note` の `TimelineDrag::Key` 腕、`frame_snapped`)。
  v1 の残差として `pane.rs` にコメントを残した
- **Shift による範囲選択が無い**: 凍結された `KeyGrabbed` Message は
  `additive: bool` だけを運ぶ(`range` は無い)ので、Cmd の足し引きだけを
  実装した。egui 版の `select_key` は Shift 範囲選択も持つ

## 検収

`cargo test -p motolii-ui -p motolii-shell-iced -j 5`: **motolii-shell-iced は
全 green**(lib 9 + 統合テスト13本、うち `drive_timeline_keys.rs` 6本が
このレーンの新規分)。`motolii-ui` は357 passed / 1 failed —
落ちたのは `timeline_editor::audio_seat::tests::
a_real_device_session_starts_and_reseeks_at_the_playhead`
(実オーディオデバイスの callback を要求するテストで、このレーンのコード
[`timeline_editor/mod.rs` の `remove_param_key_at` 等]とは無関係。
`audio_seat.rs` は今回のレーンで1行も触っていない — sandbox に実オーディオ
出力が無い/複数 worktree が同時に device を取り合っている環境要因と判断)。

red→green は `crates/motolii-ui/src/timeline_editor/mod.rs` の
`remove_param_key_at_removes_only_the_named_key` /
`remove_param_key_at_a_missing_time_rejects_without_writing` /
`remove_param_key_at_a_locked_layer_rejects` /
`move_param_key_changes_only_that_keys_time` /
`set_param_key_interp_is_position_only`(D2 呼び出しの単体)+
`crates/motolii-shell-iced/tests/drive_timeline_keys.rs` の6本
(`Shell::update` 経由の統合)。どちらも実装前は存在しない関数/Message
を呼ぶので、書いた時点で red だった。
