# CU-206C ordinary Timeline viewport接続実装

- 日付: 2026-08-02
- 状態: **実装完了**
- 対象: `CU-206C`（`U3a-2Q-V`の既知target接続）
- 実装commit: `4c982bec`（固定viewport接続）＋本receiptと検収修理を同一完了commitへ収録

## 利用者出口

通常製品windowのnative Timelineを、固定row・固定時間scaleのviewportとして表示し、Timeline上にcursorがある時だけwheelの縦横deltaでProject sessionの表示範囲を移動する。

- row: 34 logical px
- time scale: 80 logical px/s
- `CursorMoved`はlogical座標へ、`MouseWheel`は`LineDelta`／`PixelDelta`を有限な2軸logical deltaへ正規化
- ruler、bar、key、playhead、hit-test、move／trim／snapは同じ`horizontal_start`／`vertical_offset`変換を使用
- viewportはprivate Project session state。Document、journal、Undo/Redo、project JSON、公開APIは変更しない

## 既知targetと修理

新しいcarrier、第二Timeline、汎用scroll frameworkは作らず、既存targetだけを接続した。

- `crates/motolii-ui/src/product_runtime_adapter.rs` — approved raw input closed set
- `crates/motolii-ui/src/product_runtime.rs` — private session viewport、projection／hit-test、clamp、scroll context
- `crates/motolii-ui/src/native_timeline_renderer.rs` — renderer instanceごとのviewport setterとfixed geometry
- `crates/motolii-ui/src/product_runtime_tests.rs` — existing move／trim負例を維持し、panned TrimOutを追加

初回独立検収で見つかったP0/P1（trim-out負例の弱化、`viewport.start`欠落、dead validation、keyの縦clamp、scroll経路不足）を、既存target内の修理として閉じた。修理後のOpus 5 fresh read-only検収は`VERDICT: ACCEPT / P0: 0 / P1: 0`。

## 検証

```text
cargo fmt --all --check                         PASS
cargo test -p motolii-ui --lib --quiet          PASS (205 tests)
cargo test -p motolii-ui --test raw_input_boundary --quiet  PASS (6 tests)
cargo clippy -p motolii-ui --all-targets --all-features -- -D warnings  PASS
git diff --check                                PASS
./scripts/check-docs.sh                         PASS
```

純粋な正負oracleは、cursor外のwheel no-op、LineDelta／PixelDeltaの一回だけの単位変換、上下左右の同時移動、viewport clamp、非zero viewport startのhit identity、scroll後のbar identity、bar endが画面外の時の`Move`、pan後の`TrimOut`を検証する。

## 通常製品window確認

以下の現行worktree binaryをproject fixtureで起動し、通常製品windowのnative routeを確認した。

```text
MOTOLII_UI_TRACE=1 target/debug/motolii_ui_shell \
  /private/tmp/cu206c-window.Bgj0oo/project.json
```

観測値は`scale_factor=2.000`、`timeline_x=0.000 y=600.000 width=1200.000 height=200.000`、`timeline-scene rows=1 bars=1 physical_y=1200 physical_height=400`で、Retinaでも固定rowを維持した。AX bridgeはnative wgpu面へraw wheelを配送しないため、wheel eventそのものの画面差分は取得できなかったが、上記の製品binary起動と、同じrenderer／projection routeを通るscroll→hit identity oracleを分離して記録した。未観測のmouse/trackpad挙動を推測していない。

## 非目標

zoom、minimap、scrollbar、track reorder、track header、viewport永続化、React Timeline、Document writer、runner／AGENTS／route contract改修は行わない。
