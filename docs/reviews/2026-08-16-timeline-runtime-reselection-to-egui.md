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

## 7. UI側の編集経路が同日に消えた — 繋ぐ先の現状

本決定と同じ日に、`document_edit_runtime/`(25ファイル)と `timeline_intent_adapter.rs` が削除された(`21cb8204`)。**C2 で繋ぐはずだった翻訳層である。**

朝の[Web窓とRN製品面の畳み込み](2026-08-16-web-window-and-rn-product-fold.md)は「`timeline_intent_adapter`(166行)は参照ゼロだが**残してある**。消すと同じ作業を書き直すことになる」と書いていた。**その指示に反して消した**ので、経緯として明記する。

ただし**繋ぐ両端は無傷**であり、「配線が全部消えた」は誤りである。

| | 状態 |
|---|---|
| ジェスチャ層(`timeline_move_gesture` 130 / `timeline_trim_gesture` 189) | **生存**。`TimelineMoveRequest` / `TimelineTrimRequest` を作るところまで動く |
| 投影・視野(`timeline_projection` 411 / `timeline_viewport_state` 341) | **生存** |
| 単一書き手の境界(`document_command_request.rs` 62) | **生存**。決定済みD2 command列を1回の編集要求へ畳む |
| D2 の意味論(`motolii-doc/src/command/`) | **無傷**。`AddPositionKey` と逆操作の持ち主は最初からここ |
| 翻訳層(`document_edit_runtime` + `timeline_intent_adapter`) | **削除**。コード2,556行 / テスト3,212行 |

消えた5,934行の**半分以上はテスト**(`apply_failures` 494 / `place_and_trim` 418 / `position_keys` 390 / `journal` 303 …)で、それらは「**どの編集が合法で、どれが拒否されるべきか**」の記録だった。痛いのはコードよりこちらである。

### 戻し方 — `git checkout` ではなく `git show`

```bash
git show 21cb8204^:crates/motolii-ui/src/document_edit_runtime/tests/place_and_trim.rs
```

**いま木へ戻さない。** 理由は `timeline_egui` を戻さないのと同じで、(1) 消費者ゼロのコードを押し戻すことになる(ビルド 23.4→3.10s の効きは自分のコード量だった)、(2) `timeline_intent_adapter` は平坦な行モデル前提で、**今日決めた Group 階層では作り直しになる**、(3) テストは runtime の型に依存するので単体では動かず、その runtime こそ作り直す対象。

**着手順は C2 に入る時点で: テストの意図を新しい行モデル向けに書き直す → それが通る最小の翻訳層を書く → `prepare_place.rs` / `process_keys.rs` を参照実装として開く。** D2 の API は変わっていないので、設計判断はほぼ済んでいる。

## 8. 開発動線 — UX(ジェスチャ・ショートカット)をどう通すか

**ビルド時間は問題ではない**。1ファイル触ってからの実測(`[profile.fast]`、温まった状態):

| | |
|---|---|
| `cargo check -p motolii-ui` | 4.67s |
| lib ビルド(コード生成) | 5.80s |
| `--bin motolii-blitz-dump` | **3.53s** |
| `--bin motolii-blitz-shell` | **4.67s** |
| `spikes/blitz-probe` の1本 | 2.47s |

冷えた初回だけ17秒かかる。**crate分割で速度を稼ぐ理由はない**([`motolii-timeline` crate は U3a-1 で `REJECT` 済み](2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md))。

### 既存の3本はBlitz向きで、そのままでは使えない

| | 現状 |
|---|---|
| `scripts/watch-timeline-widget-dump.sh` | **`timeline_blitz/` を監視** → `motolii-blitz-dump timeline` → PNG。監視先を差し替えれば骨格(41行)は再利用できる |
| `examples/timeline_widget_lab.rs`(416行) | eframe → Blitz texture → egui image の窓。経路ごと不要になる |
| `src/blitz_dump/`(bin) | Blitzパネル専用。egui は描けない |

### 検証は3層に分ける

**1. 純関数** — hit の分類 / トリム端 / ショートカット解決。窓もフレームも要らない。**UX の不変条件の大半はここ**。道具は無傷で残っている(`motolii-input` 2,413行: `keymap` / `keymap_codec` / `command_registry` / `input_router`)。削除された 339 行のテストが見ていたのもこの層(`egui_shortcut_mapping_rejects_wrong_modifiers` / `classify_bar_edge_narrow_bar_is_always_body` 等)。

**2. [`egui_kittest`](https://docs.rs/egui_kittest/0.35.0/egui_kittest/struct.Harness.html) 0.35.0** — egui 公式(emilk/egui)。依存は `egui ^0.35.0` / `egui-wgpu ^0.35.0` / `wgpu ^29.0` で**このワークスペースと全一致**。

```rust
harness.drag_at(from); harness.hover_at(to); harness.drop_at(to);  // 座標でジェスチャ
harness.key_combination(&[Key::Cmd, Key::Z]);                      // 修飾キーつき
harness.get_by_label("Mute").click();                              // AccessKit で引く
harness.snapshot("clip_moved_200px");                              // tests/snapshots/ と差分
```

今日 `mock_input`(300行)を手書きして確かめた「down → move → up → 実際に動いた」が、**手書きのハーネスなしで書ける**。`blitz_dump` との差は、**食い違ったらテストが落ちる**こと(dump は人が見るだけだった)。

**3. 窓の Lab** — 手触り。ここだけは人間が要る。

### 設計への跳ね返り

`get_by_label` が効くのは **egui ウィジェットとして作った部品だけ**(AccessKit ノードが出るため)。**painter で描いた矩形はラベルで引けない。**

```
chrome(ボタン・M/S・transport・fold三角)  → egui ウィジェット。ラベルで叩ける
密な面(clip・key・グリッド・playhead)      → painter。座標で叩く
```

座標で叩く以上、**テストが幾何を知る**必要がある。§6 で `geometry.rs` から「一本の変換を共有する契約」だけを写すと決めたのは、ここに効く。

### Blitz との役割分担 — 補強は設計時であって実行時ではない

```
設計時: HTML/CSS で見た目を詰める → Blitz(Stylo+Taffy)が解く → mock_tokens が値を吐く
実行時: egui だけ。**Blitz は出てこない**
```

egui は視覚設計の反復が弱い(CSS が無い)。そこを Blitz で補うが、**それはビルド時の話**である。実行時に Blitz を混ぜる案は §3 の理由(接ぎ目・単位境界・beta の露出)で棄却済みであり、この分担を「実行時の補強」と読み替えないこと。

### 着手順

**フォント同梱 → 行モデル＋純関数テスト(339行の意図を写す) → その時点で `egui_kittest` を入れてジェスチャとスナップショットを閉じる。** 純関数テストが先なのは、一番安く、行モデルの作り直しを安全にするため。スナップショットは**撮る対象ができてから**入れる。

## 残余（2026-08-16 深夜 更新）

> 更新履歴: 夜の版は `0d79a139` 時点で書かれており、その後 `81ab958e` `071fb5bc` `4b71de05` `1d26879d` `6e9de3d4` の5本が着地したため、次の席が読む前にここを実状へ戻した。**「進行中」は空になり、「未着手」からは選択・playhead・横ズーム/パンが抜けた。**

### できている（テストが押さえている）

| | oracle |
|---|---|
| 行モデル(`timeline_rows.rs`) — 1 Layer = 1行、2軸独立の開閉、キー無しパラメータは行を出さない | `cargo test -p motolii-ui --lib timeline_rows` → **7 passed** |
| Lab で clip / group を動かす、トリム、キー単体のドラッグ、M/S の書き込み、選択、playhead のスクラブ、Undo/Redo/Esc | `cargo test -p motolii-ui --example timeline_egui_lab` → **14 passed** |
| clip 移動に**5パラメータ全部のキーが追従する**(Position / Anchor / Scale / Rotation / Opacity)。Position だけが自前の command で、残りは `SetTransformParamKeyTime`(`81ab958e` で D2 に追加)。**KeyframeId は編集をまたいで生き残る**(remove+add にしない) | 同上 |
| 横ズーム・パン。カーソル下の時刻が動かない、0.25秒より寄れない、composition より広く引けない、`x→time` と `time→x` が往復で一致 | 同上 |
| **編集の時刻がフレーム境界に乗る。** `try_from_frame` / `try_to_frame_round` を通す(fps は有理数なので `(秒*fps).round()` を自前で書かない) | 同上 |
| **Cmd+D 複製。** 再帰(Group の子・入れ子 Vector)と id の採り直しは `duplicate.rs` が持ち、Lab は子を辿らない | 同上 |
| 外部の編集が次フレームで映る(revision 監視。ブラウザからのシェイプ配置が別プログラムに見えないため) | 同上 |
| **複数選択**(素のクリック / `Cmd` で足し引き / `Shift` で範囲)。移動・複製・削除がまとめて効く。親 Group と子を同時に選んでも**子は二重に動かない**(`selection_roots`) | 同上 |
| **並べ替え**(左列を上下へドラッグ)。落とし先は**行と行のあいだ**で決まり、開いた Group の中へも出し入れできる。**自分の中へは落とせない**。時刻は変えない | 同上 |
| **削除**(`Delete`/`Backspace`)。Group は中身ごと。1回 = 1 Undo で、**同じ LayerId と表示名が戻る** | `cargo test -p motolii-doc --test d2_command removing_` → 2 passed |
| **縦スクロール**(ホイール / 右端のつまみ)。面からはみ出した行は描かず、触りもしない | `--example timeline_egui_lab` |
| **ピンチで横ズーム**、下端の**時間ナビゲータ帯**(掴んで横パン、両端6pxでズーム) | 同上 |
| **再生**(`Space`)。終端で止まり、終端で押すと頭から。**再生中は面が流れ、playhead は窓の中央に居続ける**(頭と終端では窓が止まり、playhead のほうが窓の中を動く)。スクラブは再生を止め、playhead もフレームに乗る。**窓が隠れていた分の時間はまとめて進めない**(1フレーム50msまで)。追従は同日に「相対位置を保つ」「DAW のページ送り」と2度作り替えたが、**利用者の指定でこの中央固定へ戻した** — 詳細と経緯は[DAWのplayhead追従 先例調査](2026-08-16-daw-playhead-follow-prior-art.md)の「撤廃」 | 同上 |
| **Group 自身のキーが追従する**(2026-08-16 利用者裁定で未決から決定へ)。掴んだ subtree の中の Group envelope のキーも子と同じ delta で動く。追従の集合は「clip だけ」から**subtree の全 layer**へ広げた | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **グループ化**(`Cmd+G`)。「空の Group を置く」+「選んだものを `ReparentClip` で入れる」の組み合わせで、**新しい意味の command を足していない**(D2 に `prepare_add_group` を追加、返すのは `AddTrackItem`)。1回 = 1 Undo。**親が揃っていない選択は断る** — 位置が言えなくなるため | 同上 + `-p motolii-doc --test d2_command an_empty_group` |
| **キーフレーム削除**。キーをクリックで選び(`Cmd`で足し引き)、`Delete` は**キーが選ばれていればキーを消す**(無ければ層)。行を選び直すとキーの選択は落ちる — Delete の対象を2つ持たない | 同上 |
| **クリックしただけで選択が変わる**(clip bar / キー)。掴んで動かすまで選択が変わらないのは、押した手応えが無いのと同じ | 同上 |
| **分割**(`Cmd+K` / メニュー)。playhead で切る。端では切れないが**それは断りであって失敗ではない**(status で言う) | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **キーの追加**(メニュー → パラメータを選ぶ)。playhead の時刻へ打ち、打ったキーを選んでキー行を開く。同じ時刻には二重に打たない。**`prepare_add_transform_param_key` が同じ `KeyframeId` を返す穴を先に塞いだ**(カウンタのコピーではなく本体から採る。`d3cfdb9a`) | 同上 + `-p motolii-doc --test transform_param_key` |
| **イージング**(キーのメニュー → Hold / Linear / Ease in-out)。**Position だけ**。他パラメータは `prepare_set_*_key_interp` が D2 に無いので席のまま(キー編集APIを1本へ畳む話と同じ穴) | 同上 |
| **吸着**(clip の端・キー・playhead・ループの端・0・終端)。間合いは**画面の距離**(7px)なので寄れば細かくなる。掴んでいる当人へは吸わない。**Alt で切れる**。フレーム丸めより吸着が優先 | 同上 |
| **transport**(頭出し / 再生停止ボタンと**タイムコード `M:SS:FF`**)。記号は painter で描く — `▶` `⏮` はフォントに無く豆腐になる | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **矩形選択**(何も無いところをドラッグ)。**bar と時間方向に重なった行だけ**選ぶ(行に掛かるだけでは拾わない)。何も無いところのクリックで選択解除、`Cmd+A` で見えている行を全選択 | 同上 |
| **掴んでいるあいだ、時刻がポインタの脇に出る**(status 行まで目を運ばせない) | 同上 |
| **行の高さ切替**(小24px / 大34px、メニュー)。**意味は変わらない**、見やすさだけ | 同上 |
| **ロック**(`L` 列 / メニュー)。D2 に `SetItemLock` を新設(`SetItemVisible` と同型、`ItemEnvelope.lock` は元からあった)。**D2 は lock を見ない**(評価・描画に影響しないフラグ)ので、触らせないのは UI の仕事 — 移動・トリム・キー・削除・複製・分割・グループ化の入口で外す。**選択は許す**(見て確かめたいことがある)、外したときは status で言う | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **表示名の変更**(右クリック → `Rename…` / `⏎`。**ダブルクリックは使わない** — 同じ場所に選択・並べ替えが重なっており、2回目の押下を別操作の途中と区別できない)。D2 に `SetLayerName` を新設し、`LayerIdTable::rename` を足した。**動くのは台帳だけ** — ツリーも ID も参照(`transform.parent` / `LookAt` / journal)も動かない。空の名前は断る、ロック中は始まらない、その場が入力欄になる(別窓を出さない) | `cargo test -p motolii-ui --example timeline_egui_lab` + `-p motolii-doc --test d2_command renaming` |
| **ロケータ**(Ableton の Locator 相当)。**タイムラインを右クリック → その位置に置く**(仮の playhead のような印)。押すと playhead がそこへ跳び、右クリックで名前を直し、掴んで動かせる(吸着つき)、**名前を空にすると消える**。ルーラにロケータの段を1本足し、面には薄い縦線だけ。D2 は `Document.locators: Vec<Locator>{t, text}` と4 command(Add/Remove/SetTime/SetText)。**識別子を持たず保持順が宛先**、空なら書き出さないので既存文書のバイト列は不変、旧readerは未知キーとして往復するので**版は上げない**。※最初「メモ(marker)」として作ったが、利用者の求めは**構成管理と再生位置のナビゲーション**で、定義を確かめる前に作ったのが誤り | `cargo test -p motolii-ui --example timeline_egui_lab` + `-p motolii-doc --test d2_command locator` |
| **左列の部品は1本の入口に揃えた**(`rail_button` / `rail_glyph`)。**`Sense::click_and_drag()` にするのが要点** — クリック専用だと、下に敷いた行(選択＋並べ替え)のほうが掴みの相手になり、**指が数px動いた瞬間にボタンの `clicked()` が消える**(「M/S/L がたまに効かない」の正体)。**ただしそれだけでは今度は自分が「掴んで離した」になり `clicked()` が立たない**(2026-08-17「Group が閉じられない」の正体) — 的の中で離したなら押下として拾う(`pressed`)。**外へずらして離す取り消し**はそのまま残る | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **行の色**。D2 に `SetItemColor` と `ItemEnvelope.color: Option<u32>`(**選ぶまで載らない / 空なら書き出さない / 版を上げない**)。**既定は `LayerId` から導く** — 採番後に変わらず再利用もされない唯一の値なので、並べ替えでも Group への出し入れでも動かない(行番号から導くと並べ替えで総入れ替えになる)。**選んだ色は Document なので複製にも付いていく**(envelope ごと写る)。選択が複数なら全部に付く。**「色を出すか / 白で統一するか」は Document に入れない** — 製品では Workspace profile の持ち物(白で統一したい人の好みが、他人の付けた色を消してはいけない)。パレットの値は**仮**で、正本は `mock_tokens` 側 | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **選択の点灯は白**。行が色を持つ以上、アクセント色だと「行の色のひとつ」に見える — 選択は状態であって持ち物ではない | 同上 |
| **ロックは枝ごと効く**(`effective_lock`)。Group を掛けると中も掛かる — 各行の `lock` を単体で読んでいたのがバグだった。子の `L` は**自分が掛けた分だけ点け**、親から受けている分は薄く出す(押しても外れないものを点灯させない。押したら理由を status へ) | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **掴み物は `Hold` 1つに畳んだ**(clip/キー/トリム/並べ替え / ロケータ / ループ帯 / ナビゲータ / 矩形選択)。以前は Option が4つ並び、「何か掴んでいるか」を聞くたびに4つ確かめる必要があった。毎フレーム同じ3つ(ポインタの時刻・端で流す量・px/秒)は `Surface` 1つへ、単発の書き込みは `apply_one` / `apply_in` へ集約 — **写経が4+7箇所消えた** | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **キーは全パラメータ掴める**(2026-08-17)。D2 に `SetTransformParamKeyTime` が入った時点で Position 縛りの理由は消えていたのに、**掴む側だけ Position のままだった** — clip 移動の追従は全パラメータ効くのに単体で掴むと動かない、という食い違いになっていた | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **ロケータのドラッグが1 Undo になった**(畳む前は毎フレーム `begin_gesture` していたので、フレーム数だけ Undo が積まれていた)。併せて D2 の `merge_pair` に `SetLocatorTime` / `SetLocatorText` の腕を足した — 腕が無いと catch-all の「後着をそのまま」に落ち、**Undo が掴む直前ではなく1フレーム前へ戻る** | 同上 + `-p motolii-doc --test d2_command locator` |
| **パンと縦スクロールは生のホイール量で動かす**(2026-08-17)。`smooth_scroll_delta` は egui が時間で均した値で、**指を止めても数フレーム流れ続ける** — 面を掴んで動かす操作では上乗せがそのまま遅延になる。OS の慣性はイベントに含まれて来るので、捨てているのは egui の均しだけ。**ズームだけは均した値のまま**(倍率は1フレームの差が指数で効くので、生値だと段が見える) | 目視 |
| **畳んだ Group は中身をその bar の中に見せる**(子の占める範囲を色付きの帯で)。開けば行として見えるものが閉じると消えるので、何が入っているか掴めなかった。**左列には入れ子の背骨**(深さぶんの縦線)。どちらも**文字を足さずに**構造を見せる | 同上 |
| **右クリックメニュー**(行 / キー / 何も無いところ の3種)。**egui のウィジェットで作る** — AccessKit のラベルが出るので `egui_kittest` の `get_by_label` で叩ける(§8「chrome はウィジェット」)。トンマナは面の定数(`CELL`/`INK`/`ACCENT`)を `visuals` へ渡して合わせた。**まだ無い操作は灰色の席として並べる**(空欄だと「この面には無い操作」に見える)。右クリックは選んでいない行なら選び直してから開く | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **ループ再生**。ルーラの**上端12px**を引いて区間を作る(右→左でも同じ区間、端はフレームに乗る、最短1フレーム)。端8pxで伸縮(**掴んだ瞬間に反対側の端を固定**するので追い越しても畳まれない。判定を外すと新規作成に落ちて区間が消えるため、端は甘く見る)、中を掴んで移動、`L`で効きの入り切り(帯は残る)。**折り返しの判定は「お尻を越えたか」だけ** — 区間の外から再生を始めても頭へ引き戻さず、前から入ればそこまで通し、後ろから始めたら一度も折り返さない。行き過ぎた分は頭へ足す(捨てると毎周ぶん周期が伸びる) | `cargo test -p motolii-ui --example timeline_egui_lab` |
| **掴んだまま面の端まで運ぶと窓が流れる**(playhead / clip / キー / ループ帯)。端に近いほど速い。窓の中にある時間しか指せないと、寄っているとき遠くへ運べない | 同上 |
| **目盛は時刻に貼り付く**(窓のN等分ではない)。数字つきの主目盛＋数字なしの細目盛の2段で、寄るとフレームの倍数、引くと秒・分の倍数へ移る。方眼はルーラと同じ列から引く | 同上 |
| **1区間おきに下地の明暗を変える**(Ableton の Arrangement と同じ)。線だけだと区間が全部同じ面に見え、どこからどこまでが1目盛か目で掴めない。濃淡は**絶対時刻の偶奇**で決めるのでパンしても縞が入れ替わらない | 同上 |
| playhead は**時間面の中だけに描く**。窓の左端より前の時刻に居ると x がレールへ入り、レイヤー名の列を縦に貫いていた | 目視 |
| 記号の豆腐(`egui_fonts.rs`) | Hack を fallback へ。追加フォント0 |

**面の動かし方は AE / Premiere と同じ割り当てにした**(2026-08-16 深夜)。素のホイール＝縦スクロール、`Shift`＝横パン、`Cmd`＝横ズーム、ピンチ＝横ズーム。
夜の版では素のホイールが横ズームだったが、**縦スクロールを入れる時点で素のホイールは縦へ渡すのが普通**であり、
横方向の手掛かりが消える分をナビゲータ帯が埋める。

**レイヤーの全体地図(minimap)は作らない**(2026-08-16 利用者指摘、採用)。1 Layer = 1行なので**行の一覧そのものが全体図**であり、
縮小版を別に持っても情報が増えない(利用者の言い方では「行ベースだと死に機能になる」)。
一方、**時間方向のナビゲータ帯は死なない** — 寄ると全体のどこに居るか分からなくなるのは時間軸だけだからである。

**触れる:** `cargo run --profile fast -p motolii-ui --example timeline_egui_lab`

### 決めていない（実装させずに止めてある）

1. **ドラッグ中に1本弾かれたときの後始末。** gesture ごと巻き戻すか、部分適用を許すか
3. **コピー／ペーストの持ち出し形式。** Cmd+D(複製)とは別物で、**貼り付け先が変わる**ぶんだけ決めることが多い — キーを別パラメータへ貼れるか(型が違う)、別レイヤーへ貼ったとき時刻は絶対か playhead 基準か、シェイプを別 Group へ貼ったとき親の Transform をどう扱うか。**下請けに渡す前に決める話**

### 進行中

なし（2026-08-16 深夜時点。夜の版にあった `SetTransformParamKeyTime` は `81ab958e` で着地）。

### 未着手

- **`rows()` の毎フレーム全走査。** 描画と hit は可視範囲だけになったが、行の列は毎フレーム全部作る。1,000行なら問題なく、100k で効く
- **`TimelineView`(Lab, f32) と `timeline_viewport_state`(341行, f64 + `RationalTime`) の二重化。** ズーム／パン／snap を両方が持っている。`snap_time` は「近くの端やキーへ吸着」であってフレーム量子化とは別物なので、畳むときに混ぜない
- **並べ替えの落とし先が横位置を見ない。** 開いた Group の**末尾**と、その Group の**次**は同じ境界になる(いまは常に「中」を選ぶ)。Finder のようにインデントで撃ち分けるかは未決
- **B(ロック / 表示名 / マーカー)は全部着地した**(2026-08-16)
- `egui_kittest` の導入(§8)。**撮る対象はできた**
- CJK フォントの取得と `docs/references.md` への登録(コードではなく取得の仕事)
- chrome を egui ウィジェットへ(AccessKit から叩けるように。§8)
- `timeline_projection` の扱い。行の構造は `timeline_rows` が持ったので、投影に残すのは時間→座標の変換だけ
- `mock_tokens` の修飾class対応(発注文は用意済み)

### 決定待ち — キー編集APIを `ScalarPropertyId` 1本へ畳む(2026-08-16 提起)

**利用者の指摘**: 「`transform_key_target` にかぎらず、それぞれ汎用的に単純なAPIにすべき。
拡張できるように、今後の VISM のためにも、動的な拡張ができるように」。

現物を見ると**セレクタの語彙は既にある**。

```rust
pub enum ScalarPropertyId {           // command/ids.rs:10
    Position, Anchor, Scale, Rotation, Opacity,
    EffectParam(EffectId, String),    // ← 実行時に増える。VISM が要るのはこれ
    SourceParam(String),
}
```

使われていないのは command 側で、キー編集の入口が**9本に割れている**。

| | |
|---|---|
| `prepare_{add,remove}_position_key` / `prepare_set_position_key_{value,interp,time}` | **Position 直書き 5本** |
| `prepare_{add,remove}_transform_param_key` / `prepare_set_transform_param_key_{value,time}` | `ScalarPropertyId` を取る 4本。ただし `transform_key_target` が **Scale/Rotation/Opacity しか受けない** |

**Anchor のキーが動かせないのも、EffectParam にキーが打てないのも、この1関数の受け付け集合が原因**である
(2026-08-16 の `SetTransformParamKeyTime` 発注で `NOT_DONE` として返ってきた)。
発注先はさらに、受け付け集合が `transform_param`/`transform_param_mut` と
`transform_key_target`/`clone_transform_param` に**二重に置かれている**ことも報告している。

**畳む方向は正しいが、置き換えてはならない。** 既存プロジェクトの journal には
`SetPositionKeyTime` 等が記録済みで、variant を消すと replay できなくなる。移行は:

1. `ScalarPropertyId` を**全 variant 受け付ける**汎用版を足す(add / remove / set_value / set_time / set_interp の5本)
2. 旧 variant は **replay 用に残す**(emit はやめる)
3. 新規の書き込みは汎用版だけを使う

**未決**: 汎用版の command 名と payload、`CommandKind` を新設するか
(`diagnostic_projection.rs:250` の網羅 match が UI 側にあるため UI と同時に決める必要がある)、
`prepare_add_transform_param_key` の `next_stable_id` 非対称(下記)を移行に含めるか。

**併せて直すべき既存の穴**(2026-08-16 発見、未修正): `prepare_add_transform_param_key` は
`doc.next_stable_id` の**コピー**から `KeyframeId` を採り reservation を載せないため、
同じ doc に2回呼ぶと**同じ id が返る**。`prepare_add_position_key` は reservation で
counter を commit するので、非対称は transform param 側だけ。
現在キー追加を UI から呼んでいないので害が出ていない。

### 引き継ぐときに読む順

1. この文書の §7(編集経路)・§8(開発動線)
2. `git log --oneline` の 2026-08-16 分。**判断の理由は commit message に書いてある**
3. `crates/motolii-ui/src/timeline_rows.rs` の冒頭コメント(行モデルの規則)
4. Lab を起動して触る
