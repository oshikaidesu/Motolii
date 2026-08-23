# 軸台帳の消化ボード(状態の正本)

軸台帳(`A01`〜`A12`)は**掃討の記録**で、判定はその時点の実測。
このボードは**その後どうなったか**を1枚で持つ。混乱を避けるための唯一の現在地。

**規約**: 各 `A##-*.tsv` の行は**その軸を担当したレーンだけ**が書き換える。
着地・結線待ち・担当の情報はこのボードに集約し、tsv を書き換えて回らない。

最終更新: 2026-08-23(波A+B 着地時点)

---

## 波A+B — 着地

| レーン | 仕事 | 状態 | 残り |
|---|---|---|---|
| **A-3** | `Asset` に `AssetStatus`(Unchecked/Present/Missing/Unreadable)+ `resolve_status`(canonicalize 経路) | **着地** | **結線待ち** — `resolve_status` の呼び手ゼロ。UI 表示も再リンクも無い |
| **A-4** | engine のレイヤー隔離。1枚の decode 失敗でフレーム全体を落とさない+`layer_failures()` | **着地** | **結線待ち** — shell が `layer_failures()` を読んで status 帯へ出す配線が無い |
| **A-1** | `text_style.{id}.{size,line_height,tracking,fill_color,stroke_color}` / `text_justify` / `solo` / `speed` の `PropertyId` 名前空間 | **着地(器のみ)** | 評価側が読まないため**未消費**。A-1b が担当中 |
| **A-5** | Timeline キー描画の時間軸カリング(実描画を画面幅で頭打ちに) | **着地** | **行(縦)方向は未対応** — iced の `canvas::Widget::draw` が `_viewport` を捨てているため、Canvas の自前ラップが要る |
| **A-2** | 色/Composition/AutoSave の drag 化 | **コード変更ゼロ(正しい判断)** | 構造的に pane 単独では不可能。波C 送り(下記) |
| **B** | `shell/lib.rs` を11モジュールへ分割 | **着地**(6,228→2,127行・`cargo check --tests` EXIT=0) | `keymap_equivalence.rs` が `inspector_pointer_event` をまだ import していない |

### 波A+B が確定させた構造的事実(KNOWN.md 記帳済み)

1. **drag 機構は `layer + PropertyId + track` に固く結合**。track を持たない値(色・Composition・AutoSave)には流用できない。→ **A03 は A02 の前提**
2. **drag の状態と継続購読は `Shell` が所有**(fork の `mouse_area` は bounds 外の cursor を追えない)。pane 単独で drag 対象は増やせない
3. **pane の Message enum を shell が wildcard 無しで網羅 match**。バリアント1本の追加で shell が即コンパイル不能
4. **`PropertyId`+`KeyframeTrack` は一般機構ではない** — 効くのは**評価側が明示的にクエリしている物だけ**。text style と solo にはクエリが無い(`bm`/`matte`/`ao` と同型)
5. **iced の `canvas::Widget::draw` は `_viewport` を捨てている** — 縦のビューポートカリングは標準 canvas では不可能

---

## 波C — 発注中

分割で生えたファイルが**そのまま write-set の境界**になる。

| レーン | 仕事 | write-set | 対応する軸/手順書 |
|---|---|---|---|
| **C-1 保存と復帰** | Cmd+S・未保存●・閉じるボタンの確認・再起動で前回を開く・autosave の読み返し | `document_io.rs` + `view.rs` | A06 全件・P1/P2/P3 の Cmd+S(4本一致) |
| **C-2 選択と一括** | 選択の正本を1本へ・`resolve_layer_selection` の結線・複数選択の削除/複製/一括編集・bar drag が選択を潰す非対称 | `selection.rs` | A08 全件・P1/P2 |
| **C-3 書き出し** | 同期ブロッキングの解消・cancel を効かせる・**音声 mux を呼ぶ** | `export_ops.rs` | GOALS M9・P2 |
| **C-4 入口の結線** | 死んだ `SetSource`・`zoom_step`・`WorkspaceBook`・`is_project_dirty`・**A-3 の `resolve_status`**・**A-4 の `layer_failures()`** | `create.rs` + `input.rs` + `render.rs` | A01・A10 の Q0 違反4件・A-3/A-4 の結線待ち |

**A-1b(走行中)**: `store/view.rs` + `engine` の text 経路 + `inspector-pane/text.rs`。波C と write-set は交わらない。

---

## 未着手(波C の後)

| 項目 | なぜ後か |
|---|---|
| **track を持たない値の drag**(Composition W/H/FPS/尺・AutoSave) | 既存機構が使えないので**設計から**。shell に drag state を足す形になる(構造的事実 1〜3) |
| **Timeline の縦カリング** | iced の Canvas を自前ラップする必要がある(構造的事実 5) |
| **キーボード到達性(A02)** | 裁定217 で優先度を下げた。案B(`VerbId` へ直接キーを割る)が推し |
| **B 系の時間予算** | 器具が `next/` に存在しない(旧 egui ホストにしかない)。移植が先 |
| **可搬性**(フォント埋め込み・パス解決の UI・最近使ったファイル) | P3 後半の「意味が無い」17件。型から要る |
| **窓を開けた検分** | 手順書の `【未確認】`25件・A04/A11 の `未計測`6件。**波C 完了時が開くべき瞬間** |
