# iced Timeline 視覚再現・第2ラウンド(2026-08-19)証拠

## 経緯

発注時の再現目標は `docs/mocks-ui/public/timeline-library.html` + 同 `.css`
(round1 の正本)だった。作業途中、供給者から2枚の egui スクリーンショットが
追加で届き、**再現目標が egui shell の実描画へ差し替わった**:

1. `/tmp/egui-shell-current.png`(別 document・m3 fixture) — 最初の訂正。
   ここで「css モックにあって egui に無い要素は入れない」方針へ切り替えた
   (OBJECT 見出し行・行頭アイコンを一旦外した)。
2. `/tmp/egui-same-doc.png`(**この worktree と同じ** `--project
   /tmp/verify-proj.json` で撮った egui shell、1960×1300) — 最終正本。
   同一 document なので行名・clip 配置まで並べて見比えられる。本 README の
   「一致」「差」はすべてこの2枚目を基準にしている。

## 並び絵

| 面 | 基準 | iced(このレーン後) |
|---|---|---|
| Timeline 全体 | `egui-reference-same-doc.png` | `iced-round2.png` |
| Timeline 帯だけ(拡大) | `crop-egui-reference.png` | `crop-iced-round2.png` |

`iced-round2.png` は
`cargo run -p motolii-shell-iced -- --project /tmp/verify-proj.json
--screenshot <path> 150` で撮った(1960×1300 = 980×650 論理 ×2 scale)。

## 自分の目で見た一致

- **transport 帯**: playhead 読み(等幅・時:分:秒)+ `N rows` + `view
  a.bb-c.ddss` + `grid n`。等幅フォントで横一列に並ぶところまで一致
- **ARRANGEMENT 俯瞰帯**: 灰色の丸みを帯びた1本の帯(基準は per-row の
  色分けセグメントではなく、これ1本だけ)。**押す/引きずると `OverviewSeek`
  が飛び、view がその時刻へ寄る**(既存の pan と同じ非 intent の view chrome)
- **ルーラ**: 等幅・小数第1位固定(`0:00.0 | 0:02.0 | …`)。主目盛の間に
  副目盛(小さい縦線)を足して、基準の「線が密に入る」手触りへ寄せた
- **行ヘッダ**: 角丸の色付き四角(bar と同じ色を流用・新色は発明していない)
  + 名前 + **M / S ボタン**(角丸・枠あり・押下でトークンの accent 色が乗る)。
  押すと本物の `UiIntent::ToggleItemFlag` が飛び、Document の
  `ItemEnvelope.visible` / `.solo` を反転する(M-4b で Inspector が既に
  持っていた intent への結線のみ — 新しい intent は作っていない)
- **bar**: 角丸・行の高さいっぱいではなく上下に余白。選択中の bar の枠は
  **金色**(`tokens.action_active`)— 前ラウンドの白(playhead 用に予約された
  色)からの取り違えを直した
- **下端の横スクロールバー**: 灰色の丸いバーを追加(表示専用。view/comp の
  比率を映す)

## 意図的に入れなかった物(次ラウンド送り・理由つき)

- **L(ロック)ボタン**: `UiItemFlag` は `Mute` / `Solo` の2値しか持たない
  (`crates/motolii-ui/src/blitz_shell/intent.rs:166-172` のコメント通り
  「Lock は Timeline の行だけの操作なので wire に載せない」)。iced 側に
  対応する intent が無いので、押しても何も起きないボタンを置かない(Q0)
- **transport 帯の再生ボタン・`space=play` `L=loop` `Cmd+G=group` の近道表示**:
  play / loop / group はいずれも iced Timeline pane に intent どころか
  `TimelineMsg` すら無い。`Del`(削除)だけは実在する近道だが、基準はこの
  文字列を一塊のヒントとして出しているので、半分だけ抜き出して独自の
  ヒント表示を発明するより、対応するボタン・ヒント自体を今回は置かない方を
  選んだ
- **行の畳み開閉(▶)・Property 子行(Position/Parameters)・`Inbox` 行**:
  同じ base commit(`abf59aa0`)の `motolii-ui` には egui 版のこれらの機能に
  対応するコードが見当たらなかった(`timeline_egui.rs` という名のファイルも
  無い — 供給側で開発中の別ブランチの絵と思われる)。この shell の
  `timeline_rows::rows()` 自体が畳み開閉状態や Property 行を返さないので、
  データが無いものを描くことになり Q0 に反する。見送った

## 精度で妥協した点(正直に列挙)

- **ARRANGEMENT 帯・下端スクロールバーの窓の幅**: 基準画像は view(16s)が
  トラック幅の約27%だけを占める narrow な pill(同一 document のはずなのに)。
  このレーンの実装は `document.composition.duration`(verify-proj.json は
  10s)をそのまま分母に使うので、pill はほぼ全幅になる。基準側が何を分母に
  取っているか(egui 側のソースは見つからず確認できなかった)より、
  「実際の comp 秒数を正直に映す」方を採った — 見た目は基準と違うが、
  意味としてはこちらの方が誠実だと判断した
- **RAIL_W を 196→210px へ拡張**: M/S 2個 + 角丸スウォッチを名前と重ねずに
  収めるための調整(`semantics.rs` の `RAIL_W` doc コメントに出所を記載)
- **行頭の角丸四角のサイズ・M/S ボタンの正確な寸法**: 基準はブラウザで
  レンダリングされた絵、こちらは canvas 手描き。ピクセル単位の一致ではなく
  「同じ要素が同じ相対位置にある」レベルで揃えた

## 色の出所(Tokens に新規追加なし)

- 選択中 bar の枠・ARRANGEMENT 窓・下端スクロールバー: `tokens.action_active`
  / `tokens.border_strong`(いずれも `crates/motolii-shell-iced/src/theme/mod.rs`
  の既存 role)。M/S ボタンの accent(S=`action_active`、M=`text_muted`)は
  `inspector_pane.rs::flag_button_style` が既に採用している対応をそのまま
  流用した(Timeline と Inspector の M/S が同じ手触りになる)
- 行頭スウォッチ・bar 本体の色は元から在った `LAYER_COLORS` / `HEAD_BG` を
  そのまま流用(新しい hex は1つも足していない)

## 座標を持つテストの扱い

`crates/motolii-shell-iced/src/timeline/semantics.rs` の
`hit_test_resolves_bar_zones_on_the_fixture` と
`crates/motolii-shell-iced/tests/drive_timeline.rs` の
`scrubbing_the_ruler_seeks_the_playhead` 等は、ルーラの絶対 y 座標
(旧: `10.0` 固定)をハードコードしていた。transport 帯 / ARRANGEMENT 帯を
足した分レイアウトが下へ寄ったので、`PaneGeometry::ruler_top()` /
`transport_bottom()` / `overview_bottom()` から実 bounds を引く形へ直した
(指示にある「`Simulator::find` で実 bounds を引く形へ直してよい」と同じ
主旨— こちらは `PaneGeometry` の公開メソッドを直に呼べたので、それを使った)。
`hit_test_resolves_bar_zones_on_the_fixture` には Overview 帯 / M ボタンの
hit-test を確かめる assertion も足した。

## テスト集計

`cargo test -p motolii-shell-iced -j 5`: **全 green**
(lib 23 + 統合テスト群、drive_timeline.rs 14 本を含む。0 failed)。

## 撮った絵

- `egui-reference-same-doc.png` … 基準(`/tmp/egui-same-doc.png` の写し)
- `iced-round2.png` … 実物(`/tmp/tl-round2b.png` の写し)
- `crop-egui-reference.png` / `crop-iced-round2.png` … Timeline 帯だけを
  拡大して並べたもの
