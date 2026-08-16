# Timeline の実行時基盤を egui へ戻す — 実測つき

日付: 2026-08-16
状態: **決定**

## 決定

利用者裁定(2026-08-16): **Timeline の実行時基盤を egui とする。** [同日の「BlitzがTimelineの正本」](2026-08-16-blitz-timeline-authority.md)を**撤回**する。

ただし **HTML/CSS のモックは捨てない**。役割を変える。

| | 役割 |
|---|---|
| `docs/mocks-ui/public/timeline-library.html` + `.css` | **UX の台本**。設計はここで詰める。製品経路ではない |
| Blitz | **ビルド時のコンパイラ**。カスケードとレイアウトを解かせ、値を取り出す。実行時には持たない |
| egui + `egui_taffy` | **実行時の基盤**。構造と描画と入力 |
| `timeline_projection` / `*_gesture` / `timeline_viewport_state` | renderer 非依存(1,071行)。**この決定の外側**。再実装しない |

**この座席は今日4回動いた**(egui指名 → Skia訂正 → Blitz裁定 → 本決定)。前3回と本決定の違いは1つだけで、**今回は実測が判断材料である**。以下は全部その日のうちに取った数字で、プローブが `spikes/blitz-probe/src/bin/` に残っている。

## 1. DOM仮想化の天井 (`virtual_dom_grid`)

Blitz を DOM のまま使う案を潰すために測った。窓の出入り(recycling)のコストが未知だった。

**recycling は問題ではなかった。** 毎フレーム可視423ノードを全消し全生成しても、常設ノードの属性差し替えと同じ 3.0ms/frame だった。

残る変数は可視ノード数だけで、60fps(16.6ms)に対する天井はこうなった。

| 可視ノード | TOTAL p50 | p95 |
|---|---|---|
| 423 | 2.97ms | 3.78 |
| 823 | 5.23ms | 6.08 |
| 1,623 | 10.09ms | 11.19 |
| 2,024 | **14.72ms** | 16.19 ← 限界線 |
| 2,423 | 17.69ms | 19.04 ✗ |
| 6,423 | 46.03ms | 49.60 ✗ |

**可視2,000ノードが上限。** 行高20px固定・ビューポート600pxなら30行なので、1行あたり約66個。clip なら余裕、**密なキーフレーム行では足りない**。台帳27行目の「DOM天井3,600」とオーダーは一致し、60fps基準ではもう少し手前だった。

なお文字をバーに載せると 2.97 → 4.69ms(+57%)。「名前は OBJECT 列だけ」の決定がそのまま性能予算になっている。

## 2. DOM の入力は本当にタダで返る (`mock_input`)

実物の `timeline-library.html` に対してヘッドレスで検証した。**この事実は撤回しても消えない**ので記録する。

- clip 中心の hit: **3/3 命中**(`doc.hit(x,y)` が要素を返す。座標計算コード 0 行)
- **CSS で7px しかない trim ハンドルを撃ち分けられた** — 左端+3px → `trimHandle trimStart` / 中央 → `objectBar` / 右端-3px → `trimEnd`
- hover の出入り(`pointerover/enter`、`pointerout/leave`)が祖先の連なりごと自動で飛ぶ
- down → move → up で `pointerdown` / `pointerup` / `click` が正しい相手へ届く
- move に合わせて `left` を書き換えると **Δx=201px(指示200px)** で実際に動き、移動先で掴み直しも効く

**egui へ移ると、この層は自前に戻る。** 特に trim 端の 7px は `classify_bar_edge` として書き直しになる。それを払ってでも下記3〜4を取る、という判断である。

## 3. Blitz(0.3.0-beta.1)で踏んだ制約 — 1日で4件

| # | 症状 | 原因 | 回避 |
|---|---|---|---|
| 1 | `z-index` を持つ絶対配置が**文書原点へ飛ぶ**(描画・hit の両方) | 包含ブロックが stacking context を作っていないと `hit_inner` / 描画が親のオフセットを失う | 包含ブロックへ `z-index:0` |
| 2 | 最初の `hit()` が親(`rowTrack`)に当たる | 1回目の `resolve` では stacking context が揃わない | `resolve` を2回回す |
| 3 | **0.5px 未満の clip が消える** | `blitz-dom/src/resolve.rs:376` の `taffy::round_layout()` が整数へ丸める(0.2px→0.0、1.5px→2.0)。設定で切れない | 投影側で最小幅を1pxにクランプ(AE/Premiere と同じ) |
| 4 | ドラッグがポインタから遅れる／時刻を進めないと**動かないように見える** | `.objectBar{transition:left 80ms}` | ジェスチャ中は transition を外す |

加えて、前日のコミット `1376b49a` が **CSS px と物理 px の単位境界**でバグを出している(retina でのみ再現、scale 1.0 のヘッドレス検査をすり抜けた)。**この境界は塗り手を替えても消えない** — Blitz である限り CSS ピクセルの層があるため。

### Blitz + Skia を検討して棄却した

`blitz-paint` は `anyrender` 越しに描くのでレンダラ可換であり、`PaintScene` の必須メソッドは8本、`null_backend` は159行。`skia-safe` は既にワークスペース依存。入口は安い。

**しかし採らない。**

- 上の4件を**1つも解決しない**(丸めはレイアウト、単位境界は CSS 層、stacking context は blitz-dom)
- `vello_hybrid` は**こちらの wgpu Device/Queue をそのまま受ける**(コピーなし)。`skia-safe` は wgpu を話さず、既存の `timeline_skia_raster` も `skia_safe::surfaces` = CPU ラスタ。**接ぎ目が増える**
- beta の HTML エンジンを使いながら自前レンダラバックエンドも保守することになる

Skia へ動く正当な引き金は「Vello の描画品質か成熟度が律速になったとき」であり、まだそうなっていない。**逃げ道として記録する。**

## 4. egui 側の検証 (`egui_taffy_lab`)

[`egui_taffy` 0.13.0](https://github.com/PPakalns/egui_taffy)(egui ^0.35.0 / taffy ^0.9.2、2026-07-08更新)で、モックの構造を Blitz 抜きで組んだ。

- `grid-template-columns: 196px minmax(0,1fr)` 相当(固定列＋可変列) — **成立**
- 行の積み上げ(layer 24px / property 20px の混在、階層インデント) — **成立**
- 密な面を leaf として受け取り、その矩形へ直接描く — **成立**
- grid(KEY TOOLS の3列) — **成立**

CSS の `display:flex` / `grid-template-columns` / `height:24px` が `taffy::Style` の同名フィールドへほぼ1対1で写る。

**hidpi はタダで正しく出た**(2560x1000 のスクリーンショットで1px罫線が保たれる)。制約3の単位境界に相当するものが egui には構造的に存在しない。単位系が1つだからである。

**フォントの穴が実物で出た。** KEY TOOLS の `←` `↔` `→` が豆腐(□)になる。[UI視覚言語 185行](../ui-visual-language.md)の「egui既定fontではCJKを表示できない」と同じ問題。**フォント同梱は最初に払う費用**として確定。

## 5. 変換機構 — 一つ一つの移植をしない (`mock_tokens`)

`timeline_blitz/html.rs` 冒頭には、色の出所を `timeline_egui/*.rs:行` で示す表がある。値の系譜は **`timeline_egui` → mock(HTML) → `timeline_blitz/html.rs`** と既に一周しており、egui へ戻すともう一周する。**そこを機械にやらせる。**

Blitz をビルド時のコンパイラとして使い、Stylo にカスケードを解かせ、Taffy に寸法を確定させた**計算済みの値**を読み出す。CSS のテキストは読まない。

**固定値と可変値の分離は、同じ文書を複数の面サイズで解いて突き合わせる。**

```
1280x500 / 1600x900 / 1000x600 で解き、全部で一致した値だけを定数にする
```

結果: **定数129本 / 面の大きさで動くもの60件(除外) / 一意に決まらないもの14件(人の宿題)**。

この方式でしか出ない発見が1つあった。

```
.layerCell width: 1280x500=196.0  1600x900=196.0  1000x600=172.0
```

**左レール幅196pxは固定値ではなく「1050pxより広いときの値」だった**(モックの `@media(max-width:1050px)`)。2サイズだけで解いていたら定数として焼き込んでいた。**egui 側にも同じ折り返しが要る。**

折り返しの位置そのものは3点の測定から逆算できないので生成できない。**人が書く場所を機械が指し示す**分担になる。

### 一方通行の規律

生成物を手で直すと二重管理で崩れる。**直すなら HTML/CSS を直す。** 生成物は編集しない。

## 6. `timeline_egui`(961行、`f209da9d^`)の扱い

**ディレクトリごと戻さない。** 中身を読んだ結果、今日の決定と衝突するものが混ざっている。

| 持ち込まない | 理由 |
|---|---|
| `rows.rs` の行モデル | 平坦(`property: Option<&'static str>` だけ)。**再帰 Group Layer を表現できない** |
| `theme.rs` | 配色が分岐(ACCENT `#ffad56` vs モック `#e9cf72`)。**ここは `mock_tokens` が生成する** |
| `geometry.rs` の数値 | `row_height = (h-72)/rows` の可変行高。[2026-08-08 の「行高は固定・最小(20px)」](2026-08-08-timeline-design-decisions-and-skia-fixtures.md)に反する。レール幅も `width*0.255` |
| `clip_band.rs` の文字描画 | バーに名前を描く(`painter.text` 3箇所)。「名前は OBJECT 列だけ」に反する |
| `mod.rs::apply_direct_manipulation` | **Document を経由せず席の状態を直接動かす**。意味の持ち主が2つになる |

| 持ち込む | 理由 |
|---|---|
| `input.rs`(163行)の型 | `EguiTimelineHit{Key,Body,Left,Right,None}` / `TimelineIntent{Pointer,Command,Wheel}`。`Left`/`Right` は trim 端で、Blitz が CSS でくれていたものの置き換え先 |
| `timeline_egui_interaction_tests.rs`(339行) | hit がギャップを拒む / キーを bar より優先する / 細い bar では端をトリムにしない / 未知コマンドを Document へ通さない。**UX が変わっても生き残る意味論** |
| `geometry.rs` の**考え方** | 描画・hit・入力が同じ一本の変換を共有する契約。数値は捨てる |

**リポジトリへ戻す必要はない。** `git show f209da9d^:<path>` で読める。

## 残余

- **入力(C2)は未配線のまま。** 基盤が変わっても、この穴は塞がっていない。前回 `timeline_egui` が止まったのもここ
- **同一性の対応づけが未設計。** `LayerId`/`KeyframeId` ↔ 面の要素。renderer 非依存の作業
- **フォント同梱**が未着手(豆腐)
- **行モデルの作り直し**が要る(group 階層)。着手順は「テスト339行 → 行モデル → トークン取り込み → `input.rs` の型」
- `mock_tokens` の限界: グラデーション・擬似要素・box-shadow は取れない。修飾class(`objectBar toneGreen`)を先頭classで集計するため14件が宿題として残る
- **Blitz を実行時から外しても、ビルド時の依存としては残る**(`spikes/blitz-probe`)。beta のバグは踏むが、踏んでも壊れるのはビルドであって実機ではない

## プローブ(すべて `spikes/blitz-probe/src/bin/`)

| | 何を測ったか |
|---|---|
| `virtual_dom_grid` | DOM仮想化の天井と recycling コスト(§1) |
| `dom_inspect` | 計算済みスタイルと確定矩形の抽出。§3-1 の切り分けに使った |
| `mock_input` | DOM の hit / hover / イベント配送 / ドラッグ結果(§2) |
| `mock_host` | モックを窓で触る器(DOM の hit だけで掴む。器は幾何を持たない) |
| `egui_taffy_lab` | egui 側の構造検証(§4) |
| `mock_tokens` | 値の変換機構(§5) |
