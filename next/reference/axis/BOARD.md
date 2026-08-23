# 軸台帳の消化ボード(状態の正本)

軸台帳(`A01`〜`A12`)は**掃討の記録**で、判定はその時点の実測。
このボードは**その後どうなったか**を1枚で持つ。混乱を避けるための唯一の現在地。

**規約**: 各 `A##-*.tsv` の行は**その軸を担当したレーンだけ**が書き換える。
着地・結線待ち・担当の情報はこのボードに集約し、tsv を書き換えて回らない。

最終更新: 2026-08-23(**波D 着地時点**)

**穴の残り: 135 → 116**(閉じたのは19件。ただし**全部『結線済(実機未確認)』であり、窓を一度も開けていない** — 裁定219)

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

## 波C — 着地

| レーン | 何が変わったか | 残り |
|---|---|---|
| **C-1 保存と復帰** | **Cmd+S が効く**(既知パスは無言で上書き)・**未保存●**・**×ボタンで確認**・**再起動で前回を開く**・autosave 復帰を聞く | MRU 一覧は未。User Settings 層そのものは未設計(Session の永続化先が無い) |
| **C-2 選択と一括** | **複数レイヤーを消せる**(専用動詞を新設)・複製が選択全員に効く・M/S/L 一括。`selected_layers` を正本、`selection` をその導出キャッシュにし**書き手を1本に**した(読み手40箇所は無改修) | `resolve_layer_selection` は**まだ呼び手ゼロ**・bar drag の非対称は Timeline 側 |
| **C-3 書き出し** | **mp4 に音が入る**・export 中も UI が止まらない・cancel で残骸なし | mux 直前の cancel 再チェック不可(`Cancel::is_cancelled` が非 pub) |
| **C-4 入口の結線** | 死んだ `SetSource` が生きた・**Zoom In/Out/Fit**・壊れた素材の理由が status 帯へ | Zoom 100% は viewport bounds が取れず未 |
| **A-1b** | **テキストのスタイルが本当に時間軸で評価される**(画素が変わることを実証)。track が正本 | — |
| **監督** | Cmd+S の実キー割当(C-1 が境界で置けなかった1本)・`AssetStatus` の公開・`Asset` の取り残し修正 | — |

## 波D — 着地

| レーン | 何が変わったか | 残り |
|---|---|---|
| **D-1 Preview=Export** | **プレビューと Lottie 書き出しが engine と同じ値を見る**(M15 の穴)。テキストの Key 列/drag を結線 | **Key ボタンが状態を表示しない**(◇/◆ の出し分けが `projection.rs` 側・write-set 外)・drag 中の値が欄に即時反映されない |
| **D-2 Backspace 二重発火** | 文字を消してもキーフレームが消えない。**柵の射程へ入れた**(`global_bindings` が対照に入り「pub でないため対象外」の逸脱が解消) | — |
| **D-3 Q0 一掃** | `resolve_status` を結線(素材差し替え時に実体の有無を理由つきで報告) | `WorkspaceBook`・Zoom 100% は write-set 超過で見送り(**入口は新設していない= Q0 違反を増やしていない**) |
| **D-4 verdict 再監査** | `採用済` 230行を照合。**確定不一致は1件**。`GOALS.md` が台帳より遅れていることを検出 | 全数のソース読解はしていない(grep 一次選別+35行の手読み)ので**下限** |
| **D-5 Timeline 縦カリング** | **見えない行を描かない**。`ViewportCanvas`(iced の Canvas と同形、`_viewport` を渡す1点だけ違う)を新設し1行差し替え | — |

---

## 未着手(次)

| 項目 | なぜ後か |
|---|---|
| **track を持たない値の drag**(Composition W/H/FPS/尺・AutoSave) | 既存機構が使えないので**設計から**。shell に drag state を足す形になる(構造的事実 1〜3) |
| **キーボード到達性(A02)** | 裁定217 で優先度を下げた。案B(`VerbId` へ直接キーを割る)が推し |
| **B 系の時間予算** | 器具が `next/` に存在しない(旧 egui ホストにしかない)。移植が先 |
| **可搬性**(フォント埋め込み・パス解決の UI・最近使ったファイル) | P3 後半の「意味が無い」17件。型から要る |
| **窓を開けた検分** | 手順書の `【未確認】`25件・A04/A11 の `未計測`6件。**利用者が一ユーザーとして最終チェックに来る(2026-08-23 宣言)。ここは利用者の席** |
| **`resolve_layer_selection` の結線** | 単体テスト付きで呼び手ゼロのまま。修飾キー付きクリックの配線が `input.rs` と `timeline-pane/write.rs` に跨る |
| **split(Cmd+K)** | `SplitAtPlayhead` が宣言のみで shell へ未統合(GOALS M6 の唯一の残り) |
| **Key ボタンの状態表示** | D-1 が結線したが `projection.rs` が track の有無を持たないため常に同じ見た目 |
