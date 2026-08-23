# 台帳 verdict の再監査(D-4)

裁定158 の「未判定0」は判定が**正しい**ことを保証していなかった。[2026-08-23-trunk-sweep-results.md](2026-08-23-trunk-sweep-results.md) §3
が独立4レーンで見つけた4件の不一致(id1224 Save・id504/505 Next/Prev Keyframe・id1393 Find Missing Footage・GOALS M9 cancel)を起点に、
`normal-map.tsv` の `採用済` 行(HEAD 時点で **230行**)と `GOALS.md` の状態欄を、**現 HEAD**(`main` マージ後、C-1〜C-4・A-1b・B 分割・A-5 まで着地済み)に対して機械+実読で照合した。

## 0. 方法(発明しない・機械で辿れる形)

- `awk -F'\t' '$13=="採用済"'` で230行を抽出(`/private/tmp/.../scratchpad/adopted.tsv`)
- 各行の `理由` 列からコード識別子(`` `...` ``・`Type::Variant`・`snake_case`・`*.rs`)を正規表現抽出し、`grep -rl --include=*.rs -F <識別子> next/` で該当ファイルを機械集計するスクリプトを2本書いた
  (`/private/tmp/.../scratchpad/extract_audit.py`, `extract_audit2.py` → `audit_results.tsv`, `audit_results2.tsv`)。
  **結果は使い物にならなかった**: `理由` 列の大半は日本語の説明文で、`Copy`/`Darken`/`Solo` のような一般名詞そのものは
  コード中に大量にヒットするか(過検出)、逆にコード識別子が命名変更(例: `LaneBarToggleSolo`→実際は `ToggleSolo`)されていて
  文字列一致が外れる(過小検出)。**自動抽出は補助シグナルに留め、最終判定は個別に `grep`+ソース読解で行った**
- `next/check.sh` の既存3検査(depends/weight の「顔だけ実装0件」自己申告フェンス・Intent到達可能性grep・bundle整合)を先に走らせ、
  ベースラインとして確認した(全通過、後述)
- 監査は **230行の全数リストアップ+機械 grep 一次選別 → 疑わしい行を個別に手読み** という二段構成。
  **全230行についてソースコードを1行ずつ手で追い切ったわけではない**(網羅は主張しない)。手読みで実際に検証したのは
  「trunk-sweep の4件」「理由が `既決との衝突なし` のみで裏付けが無い4行」「理由が空欄の代表サンプル(層順・work area・shuttle等)」
  「本日の C-1〜C-4 着地に対応する行・GOALS.md 該当行」の計 **約35行**

## 1. 総数と不一致

**`採用済` 総数 = 230行。うち確定した不一致 = 1件**(型(c)寄り、下記)。

| id | canonical | 書かれていた状態 | 実体 | 型 |
|---|---|---|---|---|
| **1192** | Master Settings | 採用済(理由=`既決との衝突なし`のみ) | `next/reference/normal-map.tsv` の意味欄は「タイムライン形式/ビデオモニタリング/最適化メディア/作業フォルダ」(DaVinci Resolve のプロジェクト設定)。実際の `ui/motolii-settings-pane` は Stage 背景色 + `ui_scale` の2項目のみで、意味欄が指す4項目はどれも実装が無い。**同じ「Settings パネル」という外形はあるが、意味欄が主張する中身は0件** | (c)別物寄り(正確には意味欄と実装の中身が一致しない実装ゼロ) |

**verdict列を `採用済` → `採用予定` へ修正済み**(`next/reference/normal-map.tsv` の該当行、verdict列のみ)。

### 1192 と同型で当たった疑い(裏付けは取れた・不一致ではない)

`理由 = 既決との衝突なし` だけの `採用済` 行はもう3件あった(834 Effects Controls・900 New solid layer・959 Solid…)。
個別に実装を確認し、**この3件は実際にコードで裏付けが取れた**(834=`ui/motolii-inspector-pane/src/effects.rs` が Effect Controls 相当を表示、
900/959=`shell/motolii-shell/src/create.rs` の `CreateKind::Solid` が `ui/motolii-browser-pane/src/model.rs:1293` の `("solid", CreateKind::Solid)` カードから実際に生成可能)。
**`理由` 列が薄いこと自体は不一致の証拠ではない** — 1192だけが実際に中身ゼロだった。

## 2. trunk-sweep §3 の4件の現在地(HEAD時点で全て解消・一部は文書だけ古い)

| id/対象 | trunk-sweep 時点の判定 | 現HEADでの実体 | 現在の verdict |
|---|---|---|---|
| id1224 Save | 採用済(誤り。UI到達路なし) | **解消**。`shell/motolii-shell/src/input.rs:321` `Cmd+S(shift無し) → Message::SaveRequested`(C-1、`dbfee056`/`90ac5d03`) | 採用済(正しい)。ただし `理由` 列が `Document::save 実装済み(裁定55/56、GOALS M11「バックは済」)` のままで Cmd+S 結線に触れていない(**理由列は write-set 外なので未修正**、supervisor へ委ねる) |
| id504/505 Next/Prev Keyframe | 採用済(誤り。実装はクリップ編集点ジャンプのみと報告) | **P3の指摘は現HEADでは再現しない**。`shell/motolii-shell/src/playback.rs::jump_meaning_point` は選択レイヤーの `timeline_property_rows` のキー菱形時刻(+ shift無しならマーカー)を集めて最近傍へ跳ぶ — **キーフレーム位置そのものを見ている**。`,`/`.` に既定割当(shift付き=layer_only=純粋なキーフレームのみ)。`jump_clip_edge`(`i`/`o` = In/Out)は完全に別のMessage。P3が見た「別物」は `fef3bc37` 以前の状態か別経路の誤読の可能性がある | 採用済(妥当)。ただし `理由` 列の `既定割当K` は誤り(実際は `,`/`.`。`k` は `ShuttleStop` に使用中) — **理由列は書き換えない(write-set外)。supervisor 修正推奨事項として記録** |
| id1393 Find Missing Footage | 採用予定(実装0件の指摘) | 変化なし。現HEADでも `採用予定`・`理由=既決との衝突なし`。実装識別子は引き続き0件 | 元から `採用予定` — **不一致ではなかった**(trunk-sweep の記述「採用予定」どおりで整合) |
| GOALS.md M9 cancel | 済(誤り。書き出し中UIスレッドが止まり中断が効かなかった) | **解消**。`shell/motolii-shell/src/export_ops.rs` が `Task::run(export_stream(...))` で非同期化、`cancel.cancel()` で `out_path` を残さず打ち切る(C-3、`bf26c612`/`d182313f`) | GOALS.md 本文が未更新(§3参照、要修正) |

## 3. GOALS.md 状態欄の照合 — 本日の着地で複数行が古くなっている

`git log --oneline -25` で確認した本日の着地順: A-4→A-1→B(分割)→A-5→C-2→C-4→A-1b→C-1→C-3→(取り残し修正)。
GOALS.md はこれらの多くに対して**着地前の文言のまま**。実装を直接確認した上で、書き換えるべき行と文言案:

| 箇所 | 現在の文言 | 実装確認の根拠 | 書き換え案 |
|---|---|---|---|
| **M6** | `未` | Delete=`shell/motolii-shell/src/selection.rs::delete_selected_layers`(全`selected_layers`をループ)・Duplicate=同ファイル`duplicate_...`(同)・複数選択=`Cmd+A`/`Shift`/marquee 実装済み(`input.rs:299,302`)。**Split(Cmd+K)だけ未結線**: `ui/motolii-menubar/src/context.rs:55` が明記「`SplitAtPlayhead` は宣言のみで write::Message/shell へ未統合」 | `**部分**。Delete/複製/複数選択(Shift・Cmd+A・marquee)はC-2で結線済み。split(Cmd+K)のみ未統合(`split.rs`冒頭doc「統合手順(次波)」参照)` |
| **M7** | `未`(「旧eguiはmenuに項目があるのに何も起きない」) | `ui/motolii-menubar` の Edit メニューに Copy/Paste/Cut/Undo/Redo/SelectAll/DeselectAll/Duplicate が実配線(MB-0、id429-437の`理由`列)、既存shortcutと併存 | `**済**。MB-0でEditメニューへ結線、既存shortcutと併存(S6安全)` |
| **M9** | `**部分**(報告=現物・cancel は済。音声mux は未結線)` | `engine/motolii-media/src/mux.rs::mux_mixed_pcm` が `shell/motolii-shell/src/export_ops.rs:508` から実際に呼ばれている(C-3)。cancelも非同期化・残骸なしを確認済み | `**済**。音声muxは`mux_mixed_pcm`結線済み(C-3)、cancelは非同期・残骸なし` |
| **M11 / 「バック優先」表の保存・読込行** | `**部分**。... UI側(Cmd+S・未保存●・閉じる確認)が残り` / `... shellのCmd+S結線が残り` | `input.rs:321` でCmd+S実結線済み(C-1)。未保存●・閉じる確認もC-1のwrite-set(`document_io.rs`+`view.rs`)対象 — 個別未確認だが同一波での着地が既決事項 | `**済**(未確認箇所=未保存●表示・閉じる確認ダイアログの実見。C-1が結線したと申告、次回実機確認を推奨)` |
| **M18** | `未` | `shell/motolii-shell/src/input.rs:392-396` に `Cmd+=`/`Cmd+-`/`Cmd+9` → `ZoomIn`/`ZoomOut`/`ZoomToFit`(C-4、`stage::zoom::zoom_step`/`named_zoom_level`実装呼び出し) | `**部分**。Zoom In/Out/Fit(カーソル下時刻保持は`zoom_step`のドキュメント参照、要再確認)はCmd+=/-/9で結線済み(C-4)。Zoom to 100%は未結線(`input.rs`コメント「id1490はキーを発明しない」)` |

**未確認のまま残した行**(本監査では手が回らなかった、次回への申し送り): M3(Timeline第1波の一周確認)・M12(Q0全数)・M13(全操作拒否確認)・M19(Inspector Transform全行のキーフレームUI、`key_rows.rs`/`input.rs`のdrag実装は確認したがUI経由の追加/削除まではソース読解が届いていない)・M20。

## 4. normal-map.tsv 側で「本日の変更で新たに採用済にすべき」行(RETURN #2、tsvは変更していない)

verdict列は「不一致が確定した行」のみを書き換える約束のため、**下記は 採用予定 → 採用済 への"昇格"であって"訂正"ではない**と判断し、
tsvは書き換えず本文書にリストするに留めた(supervisor判断を仰ぐ)。全て実装+実キー割当をソースで確認済み:

| id | canonical | 現verdict | 根拠(ファイル:行) |
|---|---|---|---|
| 616/617/618 | Replace selected footage item 他2行 | 採用予定(理由「UI(Browser差替)は未」) | `ui/motolii-browser-pane/src/lib.rs:1376` に実ボタン `.on_press(Message::ReplaceSelectedLayerSource(asset_id))`、`shell/motolii-shell/src/create.rs:341` で `Intent::SetSource` dispatch(C-4) |
| 1441/1442 | Zoom In / Zoom Out | 採用予定(理由「M18未」) | `input.rs:392-395` の `Cmd+=`/`Cmd+-` |
| 1491(・1492は要個別確認) | Zoom to fit | 採用予定 | `input.rs:396` の `Cmd+9`。コード側コメントが「id1441/1442/1491の結線」と明記 |

## 5. 機械検出の柵にできるか

**できない(現状は)**。理由:

1. `理由` 列が自然文であり、コード識別子の抽出が構造的に安定しない(1で述べた過検出/過小検出)。`lottie-coverage.tsv` 方式(evidence欄に**検証可能な識別子だけ**を書く規約+`cargo test`でgrep)を`normal-map.tsv`にも適用すれば原理的には可能だが、1,551行・230採用済行を今から evidence欄形式へ書き直すのは本レーンの write-set(verdict列のみ)を超える
2. **「実装がある」はgrepで機械化できても「利用者から到達できる」の判定は機械化しにくい**。今回の手読みは「その識別子が `shell/`・`ui/` のイベントハンドラ(`Message::X => ...`)や `on_press`/キー比較式の中で使われているか」を人間が文脈判断した。`check.sh` の Intent到達可能性検査(`Intent::X` が `ui`/`shell` の非testコードに出現するか)はこの種のgrepを**既に部分的に自動化**しており、今回1192のような「意味欄と実装内容の不一致」型は検出対象外だが、**「識別子ゼロ」型((a))は今の仕組みでも拾える**
3. **提案**: `normal-map.tsv` に evidence 相当の列を足す代わりに、`check.sh` の Intent到達可能性検査と同じ発想を「Message variant 版」でもう1本足す(`ui`/`shell` の `enum Message` の全variantに対して非testコードでの出現有無をgrepする)。これは実装コストが低く(Intent版のコードをほぼ流用できる)、id1224型(「実装はあるがVariantへの入口がゼロ」)を機械的に洗い出せる。**やらなかった理由**: このレーンはコード非改変(write-setがdocsとtsvのverdict列のみ)なので、`check.sh` への追記は今回の範囲外 — 次レーンへの発注候補として記録するに留めた

## 6. 新発見の事実・迷った判断

- **`resolve_layer_selection`(`ui/motolii-timeline-pane/src/rows.rs:95`)は本日のC-2着地後も呼び手ゼロのまま**。C-2は同じ目的を`shell/motolii-shell/src/selection.rs::set_selected_layers`で別実装し、`resolve_layer_selection`はテストコード以外から呼ばれていない。trunk-sweep §6が指摘した「呼び手ゼロ」は解消されていない(死んだコードとして残存)。ただし`normal-map.tsv`のどの行もこの識別子を`理由`に引用していないため、本監査のverdict不一致としては数えていない(次レーン向けのメモ)
- **id504/505の`理由`列(既定割当K)が事実と食い違っていた**が、write-setの制約(verdict列のみ)により修正できなかった。理由列の修正権限を持つレーンへの申し送りが要る
- **迷った点**: id1192を`採用予定`にするか`不採用`にするか。「タイムライン形式/ビデオモニタリング/最適化メディア/作業フォルダ」はDaVinci Resolve固有の込み入ったプロジェクト設定で、Motoliiの設計方針(裁定113等の最小化路線)からは`不採用`(scope=out-of-domain寄り)の可能性もある。判断材料が本レーンの範囲(実装有無の照合)を超えるため、安全側の`採用予定`(=「まだ判定していない実装課題」ではなく「まだやっていない」という既存の意味)に倒し、要否判断はsupervisorに委ねた
- **`GOALS.md`の状態欄はnormal-map.tsvより古い**。normal-map側は230行が地道に`採用済`へ倒れ続けているのに対し、GOALS.mdは複数の必須項目(M6/M7/M9/M11/M18)がC-1〜C-4着地前の文言のまま放置されていた。**2つの正典が同じ実装に対して違う進捗を語っている状態**そのものが、裁定192の「結線待ち」だけでは足りない別の穴(=**進捗の写し忘れ**)であることが今回の副産物として見えた

## 索引

なし(下記README追記で対応)
