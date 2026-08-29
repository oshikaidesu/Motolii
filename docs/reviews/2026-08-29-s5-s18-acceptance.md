# S5(キー打刻・値スクラブ)/ S18(ギズモ第一波)検収

対象: `62683103` — `motolii/probe/src/{inspector,fixture,stage_widget,app}.rs`
方法: `cargo build` 緑 → 窓(`dioxus-native-probe`)を CGEvent で実操作 → `PROBE room=write` ログと画面で照合。

## 通った

- 層クリック → Inspector が層名・`solid · N keys`・TRANSFORM/APPEARANCE の実値を出す
- Key ◇ クリック → `key-added`、◇が◆になり `0 keys` → `1 keys`
- 値セル内で完結する横drag → `value-scrub axis=0 dx=30.0`、Position X が 30.0、Stage の絵も枠も一緒に動く
- Stage に選択枠(accent 4辺 + 四隅ハンドル)、ヒット判定が効く
- `has_position_track` の層は `gizmo-move-skipped-keyed` で書かれない(第一波の裁定どおり)

## 破れ

1. **ギズモ移動が Inspector に返らない。** `gizmo-move (0.0,0.0)->(755.9,604.7)` がログに出ても
   Inspector の Position は 0.0 のまま。S5 の再描画は `inspector.rs` 内の `tick` signal 一本で、
   `stage_widget.rs` からは叩けない。S5 と S18 の縫い目が繋がっていない
2. **枠だけが動いて絵が動かない。** 現在時刻に生きていない層(timeline に band が無い層)にも
   枠が出て、掴めて、書ける
3. **値スクラブがセルの外で離すと無反応。** `onmouseup` が押した span でしか発火しない。
   セル幅は論理 39px しかないので、実用的な引き幅はほぼ全部外へ出る
4. **ドラッグ中に何も動かない。** 値も枠も、離すまで変わらない(`PointerMove` は空)
5. **セルを単クリックしただけで `dx=0` の SetTrack が1回書かれる。** 空編集が Document に入る

## コード側

- 選択枠の矩形計算が `selection_box()` と `paint()` 内に同じ形で二重にある
- `current_rt()` の `3000` が `paint()` の時刻経路と別立て
