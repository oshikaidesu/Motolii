# iced M-3 Stage frame seat — 証拠

2026-08-19。レーン: `claude/stage-frame-seat-20260818`(iced shell の Stage 島に、
評価済み Document フレームの席を結線する)。

## 直った隙間

`crates/motolii-shell-iced/src/stage_island.rs` の `Embed::frame()` は
`present_probe_frame()`(テスト専用の既知絵)を無条件に呼ぶだけで、
**評価済みの Document フレームを一度も流していなかった** — M-2 が席だけ空け、
M-3 の発注書がそこに触れていなかった隙間。`cargo run -p motolii-shell-iced` で
project を開いても Stage に何も映らなかったのはこれが原因。

## 直し方

`motolii_ui::stage_frame_seat::StageFrameSeat`(egui shell の
`blitz_shell::pane::StagePane::frame_seat` が使っているのと**同じ**評価済み
フレームの席。export と同じ評価一本 = `build_document_frame_graph` +
`render_graph_cached`)を、`mod stage_frame_seat` を `pub` にして iced 側から
使えるようにし、`StageIsland{ document, playhead, .. }` が座席の Document と
playhead を運ぶ。`document.is_some()` の間だけ `StageFrameSeat` を回し、
`document: None`(fixture 展示・pixel oracle)は従来どおり `present_probe_frame`
を通る — probe 経路は消していない。

## red → green(`red.txt` / `green.txt`)

赤2回・原因が別々だった:

1. **retry loop の早期break**: `wait_for_dominant` が「share > 0.7 なら止める」
   だけで判定していたため、評価が届く前の Rerun シーン背景(`Other`扱い)
   自体が1発目から share=1.00 になり、1周目で止まって「評価済みフレームが
   一度も届いていない」ことを見逃していた。`expected` 色との一致を条件に
   直した(テストのバグ、配線のバグではない)。
2. **camera aspect の不一致**: `motolii_core::camera` は評価する frame の
   寸法が `Composition` の camera aspect(`Document::new_current()` の既定は
   16:9)と食い違うと評価そのものを `frame WxH does not match camera aspect
   N/D` で蹴る。fixture の `resolution` だけを独立に 4:3 へ差し替えていたのが
   原因(`Composition` に aspect の setter は無い)。fixture の解像度を
   16:9(96x54)へ揃えて直した — **これは iced 固有ではなく、export と共有する
   評価経路(`build_document_frame_graph`)の正しい検証**なので、同じ罠は
   他の fixture でも起こりうる。

## PNG(生成元はすべて `cargo test -p motolii-shell-iced --test stage_island_live_frame`。
macOS / Metal、`iced_test::Simulator` の headless wgpu renderer)

| file | 何の証拠か |
|---|---|
| `live-frame-lands-red-wgpu.png` | 赤一色の fixture Document(`RECT_LAYER_SOURCE`、96x54)を座らせただけで、`StageFrameSeat` が export と同じ評価を走らせ、その結果が iced の widget を通って出てくる(outcome 1)。`an_evaluated_document_frame_lands_on_the_stage_island` |
| `live-frame-follows-t0-red-wgpu.png` | 2ショット fixture(赤→青)の playhead=0.0 |
| `live-frame-follows-t1-blue-wgpu.png` | 同じ Document の playhead=1.5。t0 と別の色が出ることが「playhead を動かすと絵が変わる」の証拠(outcome 2)。`moving_the_playhead_changes_the_frame_on_the_stage_island` |

3枚とも画枠の内側(縁から15%逃がした格子16x16点)を多数決で分類し、期待色が
70%超を占めることをテストの合否そのものにしている(`matches_image` はPNGを
書けたことの確認だけで、pixel の判定は別)。

## 再生成

```sh
cargo test -p motolii-shell-iced -j 5 --test stage_island_live_frame
```

(このディレクトリの `-wgpu.png` は毎回消してから書き直すので、常に最新の
走行の絵である。)

## 実窓での確認方法(手動)

`cargo run -p motolii-shell-iced -- <project.json>` で project を開き、
Timeline の playhead を動かす(クリック/ドラッグ/矢印キー)と、Stage 島に
その時刻の合成結果が出て、playhead が動くたびに絵が追従する。評価・texture
import に失敗した場合は帯(status log)に一言出る(`Message::StageReported`
経由、黙らない)。**この lane では実窓のスクリーンショットは撮っていない**
— 上記の headless pixel oracle が同じ評価経路(export と同一)を通っている
ので、実窓での見え方はこの3枚と同じになるはずだが、実機での目視確認は
残作業として残る。
