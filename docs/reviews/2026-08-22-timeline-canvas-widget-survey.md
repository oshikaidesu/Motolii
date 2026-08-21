# Timeline は canvas であるべきか — DOM/widget 分割線の調査(TL-arch)

読むだけの実測。**製品コード・モック・正典への変更は一切なし**(`cargo build`/`cargo test` は実行していない — `cargo metadata` のみ可の読み取り専用レーン)。判断・裁定はしない(ψ 台帳・R1 と同じ姿勢) — 白黒と選択肢を並べ、決めるのは利用者。

## 動機(利用者提起、2026-08-21)

転写元モック(`next/reference/mocks/timeline-semantics.html`)は DOM = widget ツリーであり、実装(`next/ui/motolii-timeline-pane/`)だけが canvas 手描きである。この非同型が

- **(a) 文字の手動処理** — φ(裁定168)の `truncate_to_width`/`is_wide_char`/`default_measure` という等幅近似の切詰め器具
- **(b) 機械検収の盲点** — υ(S 器具第一波)FINDING「Fitts 幾何は header ボタン5操作のみ、canvas 内は walker に構造的に不可視」(atlas walker/`q0_fence` の限界)
- **(c) 転写摩擦** — ψ(転写ギャップ台帳)が実測した25要素中「唯一の転写完結ファイルは `projection.rs`(0.52 比率、純数学)」で、絵を描く2ファイル(`canvas.rs`/`lane_bar.rs`)はいずれも複数世代の正典が混在(逸脱8件)

の共通根ではないか、という仮説の検証。

**追記(supervisor 注入、2026-08-22 途中)**: 調査中に2件の追加問いが入った——(1) clip trim を「端/ハンドル drag」の一般形の特殊化として見た時、widget 化後に pane 非依存の共有部品として括り出せるか(§3b)、(2) shell 固定レイアウトを iced 標準 `pane_grid` widget へ載せ替える成立性(§3c、パネルリサイズの native 候補が実在するという supervisor 実測に基づく)。どちらも Timeline 単体の canvas/widget 問いから派生した副次調査として追加した。

## 読んだもの

1. モック: `next/reference/mocks/timeline-semantics.html`(全129行)
2. iced ソース実測(**pin 固定 rev 実物**、`next/Cargo.toml:77`/`next/ui/motolii-inspector-pane/Cargo.toml:36` の `rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c"` が指す checkout): `~/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/` 配下の `widget/src/{stack,mouse_area,canvas,pin,scrollable,column,pane_grid}.rs`・`core/src/{layout,text}.rs`・`graphics/src/geometry/text.rs`・`graphics/src/text/paragraph.rs`
3. 現行実装全ファイル: `next/ui/motolii-timeline-pane/src/{lib,canvas,lane_bar,key_rows,hit,input,clip_gesture,key_gesture,nav,write,projection}.rs`(3,936行)+ `next/shell/motolii-shell/src/lib.rs`(`Shell::view`、固定 `row!`/`column!` レイアウト)+ `next/reference/mocks/stage-semantics.html`(boxcam ハンドル意匠)
4. 先例: `docs/reviews/2026-08-21-timeline-grammar-surveys/{r1-egui-extraction,r4-neoutl,r5-lottie-modern}.md`
5. 走行状態の正本: `docs/reviews/2026-08-21-lane-board.md`(T-rail/T-canvas/I-ratio/M4 の write-set 確認)
6. `docs/reviews/2026-08-21-timeline-transcription-gap-survey.md`(ψ)・`docs/reviews/2026-08-21-pane-split-survey.md`(裁定159/160、crate 境界)

---

## §1. モック = 参照アーキテクチャ

`timeline-semantics.html` の DOM 階層をそのまま図式化する(これが実装と比較する時の「設計図」そのもの — CSS ではなく**要素の親子関係**が主張):

```
.pane (position:relative)
├─ .grid (display:grid; grid-template-columns:150px 1fr)
│  ├─ .corner                       (grid-column:1 — rail の corner、地のまま)
│  ├─ .ruler#ruler (grid-column:2)  (連続生成: .tick × N)
│  └─ .rows#rows (grid-column:1/3, position:relative)
│     └─ .row × N (display:grid; grid-template-columns:150px 1fr)
│        ├─ .rail                   (flex: swatch + name text + .g×3(M/S/L))
│        └─ .field (position:relative)
│           ├─ .band × N (position:absolute — 時間帯)
│           ├─ 縦線 × N (position:absolute — 全目盛)
│           └─ .bar (position:absolute; left/width = 時間→px)
│     └─ .row.keys (property 行 — 同型 .rail + .field)
│        └─ .field
│           ├─ (bands 共有)
│           └─ .dia × N (position:absolute — キー菱形、rotate(45deg))
└─ .play (position:absolute — playhead、pane 直下)
```

**主張しているのはこれだけ**: rail(名前・スウォッチ・M/S/L)と field(bar・菱形・playhead)は**別要素**であり、field 内の bar/菱形/縦線/帯はすべて「絶対配置された個別ノード」として存在する。DOM ツリーである以上、各要素は原理的にホバー・クリックを個別に持てる(実際に持たせるかは別問題)。**「時間軸上の連続的な位置」と「絶対配置された離散ノードであること」は矛盾しない** — CSS の `position:absolute` も iced の `pin` も、同じ理由で両立する(§2 参照)。

現行実装(`next/ui/motolii-timeline-pane/src/lib.rs:239-245`)はこの全体を**単一の `iced::widget::canvas(self)`**に畳んでいる。DOM のノード相当物(rail の swatch/name/M/S/L、field の bar/菱形)はすべて `Frame` への `fill_rectangle`/`fill_text`/`stroke` 呼び出しになり、「どのノードか」という情報はランタイムに一切残らない(`hit.rs`/`lane_bar.rs::hit_test` が実行時に再計算して復元する)。この二重化 ——「絵を描く」と「絵の中の場所を当てる」を別の関数に分けて自前で同期する—— こそが (a)(b)(c) の共通根であるという仮説は、§3 の台帳で裏付けられる。

---

## §2. iced 0.15(pin 73e686e)の道具 — ソース実測

### 2.1 `Stack`(`widget/src/stack.rs`)— 重ね

- `push()`/`push_under()` で子を積む。**描画順は index 昇順(後に push した方が上)**、**event 配送順は index 降順(topmost が先)**(`update()` 実装、行221-247)。
- 各層は `self.children[self.base_layer]` の `layout()` から得た `limits`(0..size)で**独立にレイアウト**される(行164-180)— つまり各層は**同じ矩形範囲全体**を与えられ、層自身が中でどう配置するかを決める。
- **event の透過(fall-through)が構造として存在する**: `update()` は topmost から順に `child.update()` を呼び、`shell.is_event_captured()` が真になった時点で `return`(行233-235)。**どの層も capture しなければ、その下の層へ同じイベントがそのまま渡る。**
- `draw()` はカーソルが乗っている層を検出し(`mouse_interaction() != None` の最上層)、それより下は `mouse::Cursor::Unavailable` として描く(hover 状態を1層にしか渡さない設計、行288-305)— **hover の奪い合いも構造として調停済み**。

### 2.2 `canvas::Canvas`(`widget/src/canvas.rs`)— 選択的 capture が可能

`Widget::update()`(行215-258)は `Program::update()` が `event::Status::Captured` を返した時**だけ** `shell.capture_event()` を呼ぶ(行240-242)。**Program 自身が「拾わない」と言えば、Stack の下の層へイベントが素通しになる。** 現行 `TimelinePane::update`(`lib.rs:251-262`)は `key_rows::update(...).or_else(|| input::update(...))` で、当たらなければ `None` を返す経路が既にある——つまり「canvas を最上層に重ねつつ、当たらない部分は下の real widget へ流す」設計は**既存コードの capture 判断ロジックを拡張するだけ**で成立する(新規のイベントルーティング機構は要らない)。

### 2.3 `Pin`(`widget/src/pin.rs`)— 絶対配置プリミティブが標準搭載

**これが本調査の最大の発見**: iced 0.15 には CSS の `position:absolute` に相当する `pin(content).x(50).y(50)` が**既に core widget として存在する**(公式 doc コメントに「This text is displayed at coordinates (50, 50)」と明記、行16-19)。`layout()`(行133-151)は「自分の position 分を差し引いた残り領域を子に与え、子のノードをその座標へ `move_to()` する」— まさに `.bar{position:absolute;left:...px}` と同じ意味。

これは**タスク前提(「Phase 2 は自作 layout container 1個が要る」)を覆す実測**である。bar/菱形の絶対配置は新規の `Widget::layout()` 実装を書かなくても、`Stack` の中に `pin(bar_widget).x(start_x).y(row_top)` を並べるだけで成立する。自作が要るとすれば「複数の pin 済み要素を効率よく並べる」ための薄いヘルパー関数(x/y を計算して `pin()` を返す純関数、EXACT TARGET 型のコードと同じ形)程度で、**新規 Widget trait 実装は不要**。

### 2.4 `MouseArea`(`widget/src/mouse_area.rs`)

単一 content をラップし、on_press/on_release/on_double_click/on_scroll/on_enter/on_move/on_exit を持つ。`ButtonPressed(Left)` は `on_press` か `on_double_click` が `Some` の時だけ `capture_event()`(行350-370)——**ここでも「反応しないなら黙って通す」が既定**。ただし **`on_move` はカーソルが自分の bounds 内にある間しか呼ばれない**(`update()` 内 `state.is_hovered` 判定、行316-341)— bar の端を掴んで pane 全体を横断するドラッグを実現するには、**個々の bar の `MouseArea` だけでは移動継続を追えない**(cursor が bar の外に出た瞬間 `on_move` が止まる)。この制約は現行の gesture overlay 設計(§4 参照)を Phase 2 でも残す理由になる——「掴む」は個々の widget が持てても、「掴んだ後の move 追従」は pane 全体を覆う1枚の受け皿が要る。

### 2.5 テキストの ellipsis — canvas と widget で挙動が違う(具体的なコード上の欠落)

`graphics/src/geometry/text.rs`(`canvas::Text` の実体)は `ellipsis: Ellipsis` フィールドを持つが、`draw_with()`(行52-65)は実際に `Paragraph` を組む時 **`ellipsis: Ellipsis::default()` をハードコードしており、`self.ellipsis` を読んでいない**(行63、`self.wrapping` も同様に無視、行62)。つまり **この iced pin では `canvas::Text` に ellipsis を設定しても効かない** — `lane_bar.rs` が `truncate_to_width`/`is_wide_char`/`default_measure`(等幅近似、CJK/半角の粗い判定)という自前の文字幅測定を書かざるを得なかったのは、実装判断の甘さではなく **canvas 経路そのものの構造的な穴**だったことがソースで確認できた。

一方、実 widget の `text()` が使う経路(`graphics/src/text/paragraph.rs`)は `ellipsis` を `Paragraph` の構築(行95,118,200)とキャッシュ無効化判定(行239 `paragraph.ellipsis != text.ellipsis`)の両方で正しく使っている——**cosmic-text(実グリフ幅による本物のシェーピング)で ellipsis が機能する**。rail を widget 化すれば、`truncate_to_width` の「近似であることを明記」という注記ごと、手書き測定器が丸ごと不要になる(φ の根治がより根治される)。

### 2.6 `Scrollable`(`widget/src/scrollable.rs`)— 仮想化なし(性能見積もりの前提)

`layout()` は content 全体を毎フレーム完全にレイアウトし、`draw()` 側だけが `bounds.intersection(viewport)` で可視範囲へクリップする(行1033、1127-1129)。**画面外の子も毎フレーム `layout()` を通る** — Flutter の `ListView.builder` のような組み込み仮想化は無い。これは §5 の性能見積もりの直接の前提になる。

---

## §3. 現行実装の3分類台帳

離散(widget 候補)/連続(canvas 残留が妥当)/gesture 所有(誰が拾うか)で分類する。

| 要素 | 現在の実装箇所 | 分類 | 理由 |
|---|---|---|---|
| ruler 帯+目盛(小/大) | `canvas.rs:237-277`(`draw_ruler_ticks`) | **連続** | 1800フレーム/30刻みで数十本、個別ノード化する意味的単位が無い(クリックターゲットではない) |
| hairline(行境界・ruler境界) | `canvas.rs:279-300` | **連続** | 純粋な chrome、非対話 |
| ゼブラ(奇数行 wash) | `canvas.rs:85-102` | **連続** | 背景、非対話 |
| 時間帯(大目盛周期) | `canvas.rs:103, 302-345` | **連続** | 背景、非対話 |
| 時間方向縦線(全目盛) | `canvas.rs:110, 347-376` | **連続** | 背景、非対話 |
| **bar(clip)** | `canvas.rs:142-161`(絵)+`hit.rs`(当たり)+`lane_bar::Hit`ではなく`super::hit::hit_test` | **離散候補(強)** | move/trim という個別ジェスチャの対象そのもの。`pin(bar).x(start_x).y(row_top)` で絶対配置は成立(§2.3)。本体/端(TRIM_EDGE)の当たり分岐は widget 内部の相対座標判定に素直に落ちる |
| playhead | `canvas.rs:190-199` | **連続** | 自身への直接ジェスチャは無い(ルーラー/blank 経由の scrub が動かすだけ)。**現状のまま canvas 残留が妥当** |
| マーカー | `canvas.rs:64-76` | 連続(現状) | 視覚正本無し(ψ #22 EVIDENCE_GAP)。将来クリックジャンプ等を持たせるなら離散化の余地あり、今回は対象外 |
| **rail: スウォッチ** | `lane_bar.rs:246-254` | **離散候補** | 色チップ1個、`pin`+`container`で表現可能。将来の色ピッカー起点にもなる |
| **rail: 名前(text)** | `lane_bar.rs:256-274` + `truncate_to_width`(同113-139) | **離散候補(強)** | §2.5 の理由で widget 化が φ の根治そのもの。`iced::widget::text().width(Length::Fixed(w)).wrapping(Wrapping::None)` 相当+実 ellipsis に置換可能 |
| **rail: M/S/L glyph** | `lane_bar.rs:276-307`(絵)+`lane_bar::hit_test`(当たり、同161-206) | **離散候補(最強)** | これは実質的に「手で再実装された `Button`」— 状態(`row.hidden/solo/locked`)を Document から読み、押されたら `Message` を発火するだけの部品。iced の `button()`(押下フィードバック・disabled 表現・カーソル予告が標準装備)へそのまま置換できる意味論 |
| 行選択(row click) | `input.rs:82-100`(`lane_bar::Hit::Row`) | **gesture 所有: 今は canvas が拾う** → widget 化後は各行コンテナの `mouse_area.on_press` へ分散 | Phase 1 の核心の変更点 |
| **キー菱形(diamond)** | `key_rows.rs`(自己完結の draw+hit、`KEY_DIAMOND_SIZE=8.0`/`KEY_HIT=12.0`) | **離散候補** | bar と同じ理由で `pin` 化できる。ただし個数が bar よりずっと多くなり得る(property × key 数)ので Phase 2 の中でも優先度は bar より低い |
| property 行の帯 | `key_rows.rs`(bands 共有) | **連続** | bar 同様、背景は canvas 残留 |
| move/trim/retime の**意味計算** | `clip_gesture.rs`・`key_gesture.rs`・`nav.rs`(全関数が `pub fn`、`&self`/canvas 型を一切引数に取らない純関数) | **すでにアーキテクチャ非依存** | この3ファイルは canvas 云々と無関係にモデル値だけを受け取る純関数の集まり — widget 化しても**1行も変更不要**。転写摩擦(c)の反例であり、「canvas を捨てても意味論は無傷」の直接証拠 |
| drag 継続(掴んだ後の move 追従) | `input.rs`(`Interaction::drag: Option<DragKind>`) | **gesture 所有: pane 全体を覆う受け皿が要る(§2.4 の `MouseArea.on_move` bounds 制約により)** | widget 化しても「掴んだ後、カーソルが元の bar の外へ出ても追従する」機能だけは pane 幅の透明レイヤーが必要 — ここは Phase 2 でも変わらない |
| M/S/L の状態描画 | Document 直読み(`row.hidden`等) | 該当なし | widget 化・canvas 残留どちらでも「ボタンに状態を持たせない」設計原則は不変 |

**まとめ**: 「連続」に分類されるのは全て**非対話の背景装飾**(ruler目盛・hairline・ゼブラ・時間帯・縦線・playhead)。**対話対象は全て「離散候補」**(bar・菱形・スウォッチ・名前・M/S/L)。これは偶然ではなく—— DOM モックが「対話する物は要素、対話しない物は CSS 背景」という文法で組まれていたこと(§1)の裏返しである。

---

## §3b. 端/ハンドル drag プリミティブ(supervisor 注入 2026-08-22)

supervisor から追加された問い: clip trim(`clip_gesture.rs` の端 drag)を Timeline 固有と見ず、「矩形の端かハンドルを掴む→制約つき寸法変更」の**一般形の特殊化**として見た時、widget 化した世界でこれを pane 非依存の共有部品として括り出せるか。当初提示された3実例のうちパネル境界 drag は後続の supervisor 訂正で `pane_grid` native(§3c)へ切り離されたため、**ここではクリップトリム × boxcam ハンドルの2実例のみ**を比較する。

### 一般形の骨格

「押した瞬間の座標で掴んだ場所(本体か端かハンドルか)を確定 → ドラッグ中は絶対値で出し直す(delta 蓄積禁止) → 制約(最小寸・snap・lock)を通してから確定 → Esc/RMB でキャンセル」という骨格そのものは、`clip_gesture.rs`(モジュール doc 12-20行)が既に明文化している設計原則で、`hit.rs::classify_bar_part`(§2 で確認済み、`point_x`/`start_x`/`end_x`/`TRIM_EDGE` だけを受ける**次元非依存の純関数**)が「押した場所の分類」をタイムラインの型(`LayerId`・`RowProjection`)から既に切り離している。**この分類だけを見れば、supervisor の指摘どおり「一般形」は既に半分抽出済みの形でコードに存在する。**

### 共有できる部分 / できない部分

| 要素 | クリップトリム(実装済み) | boxcam ハンドル(意匠確定のみ、未実装) | 共有可否 |
|---|---|---|---|
| 掴む場所の分類(座標→zone) | `hit.rs::classify_bar_part`(1軸・2端点、TRIM_EDGE=8px) | 未実装(`stage-semantics.html:110` は「破線ボックス+コーナーハンドル」とだけ記述、ハンドル数・掴み幅は未確定) | **形は共有できる**(「矩形境界+許容幅→zone」の関数シグネチャは同型に書ける)が、**次元が違う**——トリムは水平1軸2値(EdgeIn/EdgeOut)、boxcam はコーナー(角)を含む2軸(N/S/E/W/corner、AE 型なら最大8方向)。1軸版の関数をそのまま2軸へ拡張するのではなく、上位に「N 方向のハンドル集合から最近傍を選ぶ」層を足す必要がある |
| カーソル予告 | `ResizingHorizontally`(端)/`Grab`(本体)/`NotAllowed`(ロック) | 未実装(コーナーなら `ResizingDiagonally` 系、AE は方向ごとに8種のカーソルを出す) | **写像テーブルとして共有可能**(zone→cursor の対応表を一般化すればよい、iced 側に `mouse::Interaction` の方向別バリアントは揃っている) |
| 最小寸でハンドルを隠す | `TRIM_EDGE*3=24px` 未満の bar は端を出さない | 未確定(EVIDENCE_GAP — 「解禁束」と明記されるだけで具体値なし) | 閾値の**存在**は一般形として共有できるが、閾値の**値**はドメインごとに別 |
| 押した瞬間の座標で確定する規律 | 明文化済み(`hit.rs` モジュール doc 6-10行) | 未実装 | 一般原則として共有可能 |
| 制約つき値への変換(dimension) | frame(`clip_gesture::moved_start`/`trimmed_in_start` — px→frame 写像+comp 尺 clamp) | カメラの FOV/frustum 座標(未実装、px→カメラ空間の逆射影が要る——ピクセル→フレームの線形写像とは数学的性質が違う) | **共有不可**——ここがドメイン固有の核。時間軸の線形写像と、カメラ投影の逆写像は同じ「制約つき寸法変更」という言葉で括れても計算そのものは別物 |
| スナップ候補 | 他 clip の start/end・playhead・0/終端(`clip_gesture::snap_candidates`、Document 形状に依存) | 未確定(セーフエリアガイド等が候補になり得るが未設計) | 共有不可(候補集合の型がドメインごとに違う) |
| ロック/許可判定 | `row.locked`(層固定、M13) | 未確定 | 共有不可(意味が違う——カメラに「ロックされたレイヤー」という概念はそのまま持ち込めない) |
| move 継続の受け皿 | pane 全体を覆う `canvas::Program`(現行)、widget 化後も同型の transparent overlay が必要(§2.4) | 未実装だが同じ制約(`mouse_area.on_move` は自分の bounds を出ると止まる) | **これも一般形として共有できる**——「掴んだ後、当たり判定の小さい zone の外へカーソルが出ても追従する」ための pane 幅 overlay は、トリムでも boxcam ハンドルでも同じ構造问题 |

### 括り出せるか — 結論

**「zone 分類+カーソル予告+move 継続の受け皿」という3点は pane 非依存の共有部品(1個の純関数ライブラリ+薄い overlay widget)として括り出せる。**「制約つき値への変換(px→frame vs px→カメラ空間)・スナップ候補・ロック判定」は**括り出せない**——ここは呼び手ごとに別物を書く。

これは偶然の線引きではない: 現行 `next/ui/motolii-timeline-pane/` が既に採用している分割(`hit.rs`=幾何、`clip_gesture.rs`=ドメイン数学、`input.rs`=両者をつなぐ薄い翻訳層)と**同じ境界線**が、pane を跨いだ「端/ハンドル drag プリミティブ」の括り出しにもそのまま適用できる、ということが分かった。

**Phase 2 の設計への載り方**: 新規の小さな共有 crate(例: `motolii-tokens-rs` と並ぶ薄い gesture-geometry crate、または `motolii-tokens-rs` 自体への追加)に、(1) `classify_edge<const N: usize>`-型の zone 分類関数群、(2) zone→cursor の写像表、(3) pane 全体を覆う「掴んだら capture・掴んでいなければ透過」の overlay widget(§2.1/2.2 の Stack 選択的 capture パターンをそのまま部品化したもの)を置く。Timeline の bar/菱形(§3 の Phase 2)と将来の boxcam ハンドルは、この3点を呼び出しつつ、それぞれ自分のドメイン数学(`clip_gesture.rs` 相当・未来の camera gesture module)を別に持つ——**現行コードの分割方針をそのまま2つ目の呼び手(boxcam)に複製できる形**であり、新しい設計原則を発明する必要はない。

**括り出せない理由(邪魔している物)**: (1) boxcam ハンドルは意匠だけ確定していて挙動が「予約地・b動詞」(`stage-semantics.html:35,110`)——**実例が実質1本(トリムのみ)であり、n=1 から抽象を切り出すのは時期尚早**(汎化しすぎた共有層を先に作ると、boxcam の実際の要件が固まった時に作り直しになるリスクがある)。(2) 1軸(トリム)と N軸(boxcam コーナー)という次元の違いは、共有関数のジェネリクス設計(§表の1行目)に手間を要求する——不可能ではないが、トリムしか実例が無い今は過剰設計になりやすい。

---

## §3c. `pane_grid` によるパネルレイアウト置き換えの成立性(supervisor 注入 2026-08-22)

supervisor 実測(`widget/src/pane_grid.rs`、pin rev 実物で確認): `pane_grid` は分割・ドラッグによる並べ替え・マウスによるリサイズ(`on_resize(leeway, f: Fn(ResizeEvent) -> Message)`、行263)・`on_drag(f: Fn(DragEvent) -> Message)`(行244)・`ResizeEvent{split, ratio}`(行1184)・`state::State`(split木を programmatic に操作可能)を持つ**フル機能の native widget**。パネルリサイズについては自前プリミティブ不要——**`pane_grid` が第一候補**という supervisor の縮小訂正は妥当。

### 現行 shell レイアウトの実測

`next/shell/motolii-shell/src/lib.rs:1866-1934`(`Shell::view`)は固定の `column![header, [row![inspector, stage].height(FillPortion(3))], timeline.view(), transport, status_band]`。リサイズ機構は無い(`Length::FillPortion`/`Length::Fixed` の**コード内固定値**のみ)。Browser pane は θ(lane-board)の段階で「shell 埋め込みは `Message::Browser` 腕のみ・**view 未配線**」——**実行時に画面へ出る pane は今のところ Inspector/Stage/Timeline の3枚**(4枚目の Browser は未接続)。

### 評価(4観点)

1. **4 pane 構成**: 現状は実質3 pane。`pane_grid` への載せ替えを検討する場合、Browser の view 配線(η/θ の後続)を待つか、3 pane から始めて後で split を1本足すかの選択になる——**pane_grid の split木は動的に足せる**ので、段階移行(3→4)は構造的に無理なく可能。
2. **比率の tokens 化**: `pane_grid::Configuration`(初期木)は各 `Split` に `ratio: f32` を持たせて構築できる。現行の `FillPortion(3)` のような token 由来の初期値をそのまま `Configuration` の初期比率へ注入するのは機械的な置換で済む(EVIDENCE_GAP: 実装コストの実測はしていない)。ただし `pane_grid` の比率は**利用者が動かせば変わる**(現行の固定 `FillPortion` と違い、これは仕様変更——「利用者が手でパネル境界を動かせる」という新機能を伴う。tokens はもう「唯一の正本」ではなく「初期値」に格下げされる、という設計判断が要る)。
3. **q0 柵/atlas との相性**: `pane_grid` の各 `Content` は通常の `Element` をラップするだけで、中身(Inspector/Stage/Timeline)は今までどおり widget ツリーのまま——atlas walker(`q0_fence`/`collect_targets`)から見て `pane_grid` 自体が特別な遮蔽を作る理由はない(canvas のような「中身が不可視になる」構造的な穴は無い、§3 のνFINDINGとは無関係な階層)。**ただし実際に atlas 器具を通した検証はしていない**(read-only レーンのため、EVIDENCE_GAP)。
4. **M4 StagePresenter widget が pane の中で生きるか**: `stage_pane(...)` は既に `row![inspector, stage_pane(...)]`(通常の `Element` として)に同居しており(`lib.rs:1908-1919`)、`pane_grid::Content::new(element)` は任意の `Element` をそのまま包むだけ——**StagePresenter(τ/M4 が組んでいる shader widget)を pane_grid の Content にするのに構造的な障壁は見当たらない**(shader widget 自身は自分がどんな親コンテナに置かれているかを知らない設計のはず——通常の layout プロトコルに従うだけ)。ただし実ビルドでの確認はしていない(EVIDENCE_GAP)。

**成立性の結論**: 構造的な障壁は見当たらない(4観点とも「機械的に可能」寄り)。ただし「利用者がパネル境界を動かせるようになる」は現行に無い新機能であり、**Phase 1/2(Timeline 内部の canvas→widget)とは独立の、別粒度の意思決定**(shell 全体のレイアウト哲学の変更)である——本調査のスコープ(Timeline pane の内部構造)を超えるため、ここでは成立性の白止まりとし、着手判断は持ち越す。

### 裏取り: `pane_grid` の gesture はクリップトリムへ流用できない

`pane_grid` のリサイズ(`hovered_split`+`ResizeEvent{split, ratio}`)は**split 木に登録された、常に隣接する2 pane 間の境界**だけを動かす設計——境界の「向こう側」は必ず木構造上のもう1枚の pane で、比率(`ratio`)はその2者間で閉じた値。clip trim は「自由配置された時間区間の端」であり、隣接候補が構造的に固定されていない(同じ端でも隣に別 clip が無いことも、複数の非隣接候補にスナップすることもある)。**pane_grid の resize gesture はデータモデルが「分割木」を前提にしており、Timeline の「フリーフォームな区間の端」には構造的に流用できない**——§3b の一般形も、pane_grid の resize 実装からは何も継承していない(別物)。

---

## §4. 先例

### 4.1 旧世界 egui `timeline_editor`(R1、8,566行)

**重要な補正**: egui 版は「widget 分割されたタイムライン」の先例では**ない**。egui は immediate-mode で widget ツリーを持たない GUI ライブラリであり、`timeline_editor` は `egui::Painter` への直接描画+自前 `Rect::from_center_size` 当たり判定(bar端8px・菱形12×12px・M/S/L 16×16px、R1「定数」節)という、**現行 `next/` 実装と同型の「絵と当たりを自前で同期する」構造**を持つ。つまり Motolii は egui 時代からずっと「canvas 手描き」を続けてきており、`next/` 実装はその継承(様式の借用元が2つの mock 世代に分裂している、というψ #6 の指摘と符合)。**egui 版は「widget 化した先例」ではなく「canvas 手描きを続けてきた実績」の方の証拠**。

### 4.2 NeoUtl(R4、AGPL — 構造のみ参照可)

egui ベース(Shipyard ECS)。`src/ui/timeline/{view,clip_item,layer_header,ruler,...}.rs` とファイル名は機能別に割れているが、R4 の記述(「クリップ本体に3つの当たり判定帯がある」「プレビュー座標だけが更新され」)は**egui の immediate-mode 描画+自前当たり判定**の記述であり、NeoUtl も widget ツリー方式ではない。AviUtl 系タイムラインは意味論(リップル・グループ制御・シーンタブ)の先例にはなるが、**DOM/widget アーキテクチャの先例にはならない**。

### 4.3 Lottie 圏(R5、LottieFiles Creator/Lottielab/Rive)

いずれもブラウザネイティブの Web アプリ(R5 冒頭「ネイティブブラウザ機能として外部ツール不要」等の記述)。**ソースを読める立場になく、内部実装が DOM か `<canvas>` かは今回のドキュメント調査だけでは確定できない**(EVIDENCE_GAP、§7)。ただし間接証拠が2点ある: (1) Lottielab の「Transition bar をクリックすると右パネルにイージングが出る」「レイヤー行はネストした disclosure tree」という記述は、CSS の hover/focus と親和的な UI パターンで書かれている典型的な Web app の語彙、(2) LottieFiles Creator の「Keyframe Thumbnails を恒久的に無効化した」という変更履歴は、個々のキーフレームが実体(要素かレンダーオブジェクト)として存在し ON/OFF できたことを示唆する。**これらは「DOM/widget ベースのタイムラインが実在する」ことの直接証明ではなく、状況証拠止まり**として記録する。

### 4.4 先例調査の結論

**「widget ベースのタイムラインの直接的な実装先例」は今回の3系統からは得られなかった**。egui版・NeoUtlはどちらも canvas/Painter型。Lottie圏は間接証拠のみ。**この調査の結論は先例の有無ではなく、§2 のソース実測(iced 自身が `Stack`/`Pin`/選択的 capture を道具として持つ)に立脚する**——「他社が widget 化しているから」ではなく「iced 0.15 の道具立てで機械的に成立するから」が Phase 1/2 の唯一の論拠になる。

---

## §5. 性能見積もり

### 5.1 widget 数の桁

100 layer × (bar 1 + M/S/L chip 3 + name 1 + swatch 1) = 600 leaf widget。各要素を包む `pin`/`mouse_area`/行コンテナを加えると、1 layer あたり概ね 8〜10 ノード相当 → **100 layer で800〜1,000ノード程度**。property 行(菱形)を含めれば選択レイヤーの展開時にさらに加算されるが、菱形自体は Phase 2 の中でも bar より優先度が低い(§3)。

### 5.2 iced 側のコスト構造

- `layout()` は `view()` を呼ぶたびに**全ノードを毎フレーム計算し直す**(§2.6 で確認した通り `Scrollable` にすら組み込み仮想化が無い——iced の Elm 的宣言型 UI は「diff で state を保つ」のであって「diff で layout をスキップする」わけではない)。
- ただし **文字シェーピング(cosmic-text)のコストは canvas でも widget でも同じ** — 現行 `lane_bar.rs::draw` も毎フレーム `frame.fill_text` を rows.len() 回呼んでおり、この部分の重さは移行で増えない。純増するのは **Widget trait の vtable 越し `layout()`/`draw()`/`update()` 呼び出しのオーバーヘッド**(800〜1,000ノード分)。
- 参考(間接情報、ベンチマーク未実施): `docs/reviews/2026-08-18-iced-track-record-survey.md`(調査)が COSMIC デスクトップ環境が iced 上で半年運用されている実績を記録している——デスクトップ環境の UI は画面上の対話要素数が Timeline の 1,000ノード級と同程度かそれ以上になり得るため、**「iced が widget 数千個級で実用にならない」という反証は見つからなかった**。ただし COSMIC のどの画面が何ノードかという直接の数値比較はしていない(推論の強さは中程度)。
- **参考(supervisor 注入、利用者実測の記憶)**: 利用者の記憶では、過去の処理検証(Blitz/mock 系と推定)で**500 DOM レイヤーを出しても重くなかった**という実感がある。これは (1) 出典が「利用者の記憶」であり再確認可能な一次資料ではない、(2) DOM(ブラウザ)と iced(ネイティブ widget tree)は描画パイプラインが別物、(3) 500 という数値が「レイヤー=Timeline の1行」と同じ意味かも未確認、という3点で§5.1の800〜1,000ノード見積もりを裏付ける確定的な証拠にはならない——ただし「同程度のオーダー(数百〜千ノード級)を人間の主観で問題視しなかった前例が少なくとも1つある」という**弱い傍証**としては記録に値する。レビュー規律(`docs/reviews/README.md`「出典は再確認可能な公開恒久文書に限定する」)に照らし、**これを唯一の論拠にはしない** — §5.3 の実測 probe が必要という結論は変わらない。

### 5.3 EVIDENCE_GAP

**この調査ではベンチマークを実行していない**(`cargo build`/`test` 禁止のレーンのため計測不能)。100 layer が Motolii の想定上限か典型値かも本調査の範囲外(EVIDENCE_GAP)。**Phase 1/2 着手前に、実測値(現行 canvas 版のフレーム時間)と widget 化 spike のフレーム時間を同一 fixture(例: 100 layer 合成)で比較する専用 probe が要る** — これは「やって良い」の判定材料ではなく「やる前に測る」の対象。

---

## §6. Phase 1 / Phase 2 の成立性と推奨

### Phase 1: rail の widget 化

**成立する。§2.3(pin)・§2.5(text ellipsis)・§3(M/S/L=手製Button)の3点がすべて Phase 1 の対象領域に集中しており、iced の標準部品(`button`/`text`/`container`)へ機械的に置換できる意味論しか rail には無い。**

- M/S/L → `button()`(押下フィードバック・disabled 表現が標準装備、現行の手書き `mouse::Interaction::NotAllowed` 分岐(`input.rs:247-248`)が iced 標準の disabled state に素直に写る)
- 名前 → `text()` + `Ellipsis::End`(φ の手動測定器を丸ごと撤去できる、§2.5)
- スウォッチ → 単色 `container`
- **rail の比率定数(T-rail が今まさに転写中の値、寸 0.308・角丸 0.25 等)はそのまま widget のスタイルへ移せる**(lane-board 自身の注記どおり)

Atlas walker/`q0_fence`(υ)は M/S/L を実 widget として検出できるようになる(b) の直接根治。文字幅の近似測定(a)も消える。

### Phase 2: bar/菱形の絶対配置 widget 化

**成立するが、Phase 1 より複雑度が一段高い。** §2.3 の `pin` が「自作 layout container」という前提を覆したため、当初想定より工数は軽い。残る複雑さは:

1. **drag 継続**(§2.4): 個々の bar/菱形を `mouse_area` にしても、掴んだ後カーソルがその bar の外へ出た時の move 追従は pane 全体を覆う透明レイヤー(現行 `input.rs::update` のような一枚岩)が要る——**タスク前提の「ゼブラ/格子/playhead は canvas 残留+gesture は透明 overlay」という設計は §2.1/2.2 のソース実測と整合する**(Stack の topmost-first + 選択的 capture で、overlay が「掴んだ」時だけ capture し、それ以外は下の real widget(M/S/L 等)へ透過する設計は無理なく組める)。
2. **性能が未計測**(§5.3)。bar は100個規模、菱形はそれ以上になり得るため、Phase 1 より widget 数の増分が大きい。
3. bar 端(TRIM_EDGE 8px)の当たり判定は widget 内部相対座標で素直に持てるが、**「幅24px未満の bar は端を出さない」(`hit.rs:99`)という現行ロジックはそのまま純関数として残せる**(widget 側が呼ぶだけ)。

**推奨**: Phase 1 を先に(独立して)進める根拠は強い(工数・確度とも高い)。Phase 2 は「iced の道具では通る」という設計的な白は出たが、**性能 probe を挟んでから着手判断する**のが妥当——ここは「やらない方が良い」ではなく「測ってから決める」に留める。

### canvas 維持の擁護論(同じ強度で)

- 現行 canvas 実装は**すでに動いており、visual regression は screenshot SHA(`01f71f0820d4…`)で守られている**。widget 化は描画パイプラインの入れ替えであり、既存の PNG oracle 群を作り直す必要がある(小さくない移行コスト)。
- **走行中の T-rail/T-canvas が今まさに現行 canvas 実装へ比率転写している**(§7)。Phase 1/2 は T-rail/T-canvas の成果(比率定数)を消費する側であり、着手順序を誤ると手戻りになる。
- ruler目盛・ゼブラ・時間帯・縦線・playhead という「連続」要素群(§3 で対話性ゼロと確認済み)は widget 化しても得るものが無い——**全面 widget 化ではなく rail(Phase 1)止まりという選択も十分に合理的**。「canvas であるべき、の前提を見直せるか」という問いへの答えは、**「全否定でも全肯定でもなく、対話する物だけを剥がす」が最も証拠に整合する**。

---

## §7. 走行中レーンとの衝突

`docs/reviews/2026-08-21-lane-board.md`(2026-08-21〜22走行中、メイン repo 側)を実測した結果:

- **T-rail(裁定172)**: write-set = `next/ui/motolii-timeline-pane/src/lane_bar.rs` のみ。中身はスウォッチ寸法・角丸・M/S/L 12px 塗りチップ化——**canvas 描画のまま**、比率をモックへ合わせる転写作業。
- **T-canvas(裁定172)**: write-set = `next/ui/motolii-timeline-pane/src/canvas.rs`/`lib.rs`(`lane_bar.rs` は触らない)。bar inset・角丸・ruler高・目盛り長比——**同じく canvas 描画のまま**の転写。
- **I-ratio(裁定172 §3)**: write-set = inspector-pane + 比率台帳 doc。Timeline pane のファイルには触れない(領域が別)。

**ファイルレベルの衝突は明確に存在する**: 本調査の Phase 1 推奨(rail の widget 化)は T-rail と同じ `lane_bar.rs` を write-set に持ち、Phase 2 推奨(bar/菱形の widget 化)は T-canvas と同じ `canvas.rs`/`lib.rs` を write-set に持つ。**両者が同時に同じファイルへ書けば衝突する。**

ただし **値レベルの衝突は無い**: lane-board 自身が TL-arch の項に「T-rail の比率定数は widget 化しても流用可(constants はどちらの実装にも載る)」と明記している。T-rail/T-canvas が今転写している寸法・比率・色(スウォッチ 0.308・角丸 0.25・bar inset 0.154 等)は、Phase 1/2 が widget 実装へ切り替える時にそのままスタイル値として持ち込める——**転写作業は無駄にならない**。

**推奨する順序**: T-rail/T-canvas を先に完了させ(視覚が現行 canvas のまま正しくなることを先に確定させ)、その転写済みの比率定数を Phase 1/2 の widget 実装が継承する形にする。Phase 1/2 を T-rail/T-canvas と並走させるのは write-set 非素の直接衝突になるため避けるべき。M4(ゼロコピー presenter)は Stage pane 側であり Timeline とは無関係(衝突なし)。

---

## EVIDENCE_GAP

1. **性能の実測値が無い**(§5.3)。100 layer(または Motolii が想定する典型/上限 layer 数)での widget 化 spike と現行 canvas 版のフレーム時間比較が未実施。
2. **Lottie 圏3製品の内部実装(DOM か canvas か)を直接確認していない**(§4.3、R5 が公式ドキュメントの操作記述からの推測に留まるため、本調査もソースを見ていない)。
3. **Motolii が想定する Timeline の layer 数の上限/典型値**が正典のどこにも見当たらなかった(性能判断の分母が未確定)。
4. **`pin` widget の実運用実績**(iced 本体のショーケース以外での大規模利用例)は今回のソース実測だけでは確認できていない——存在確認はできたが、大量インスタンス化時の実績は別問題。
5. **`mouse_area`/`pin` を組み合わせた「掴んだら pane 全体の transparent overlay が move を拾う」設計の具体的な API 形**(`Action::capture()` と個々の widget の capture 状態をどう共存させるか)は、設計の骨格(§2.1/2.2)は確認したが、コード試作はしていない(read-only レーンのため)。
6. **boxcam ハンドルの挙動が未設計**(§3b)——掴み幅・方向数(コーナーのみか辺も含むか)・カーソル種別・スナップ候補のいずれも `stage-semantics.html` に明記が無く(「予約地・b動詞」とだけ)、§3b の「共有可否」表はトリム(実装済み)側からの一方的な推測を含む。boxcam 側の要件が固まった時に表の判定が変わる可能性がある。
7. **`pane_grid` 移行のビルド確認をしていない**(§3c)——`Configuration`/`Content`/`on_resize` を実際に `Shell::view` へ組み込んだ試作はしておらず、比率の tokens 注入・atlas 通過・StagePresenter 同居のいずれも構造読みからの推論に留まる(read-only レーンのため)。
8. **shell レイアウト全体を `pane_grid` 化する是非**(§3c)は本調査のスコープ外(Timeline pane 内部構造の調査から派生した副次的な問いのため、成立性の白止まりで判断は持ち越し)。

---

## RETURN 用メモ

- push はしていない。commit のみ worktree 内(このファイルの新規作成+ `docs/reviews/README.md` への索引1行追加のみ)。
- 製品コード・モック・正典ファイルへの変更は一切なし。`cargo build`/`cargo test` は実行していない(`cargo metadata` も今回は不要だった — 依存 tree 確認は `Cargo.toml` の `rev` 読み取りだけで足りた)。
- 状態語彙: **調査**(ψ・R1〜R9 と同じ、決定はしていない)。
