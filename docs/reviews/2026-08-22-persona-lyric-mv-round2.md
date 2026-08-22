# ペルソナ「歌詞動画/MV 制作者」第2周 — 壁の向こうを掘る

## 前提と方法

第1周(`docs/reviews/2026-08-22-persona-lyric-mv.md`、commit `49e38610`/`8569d905`)は工程5「歌詞テキストを載せる」の**テキストレイヤー作成入口が無い**で致命的に止まった。その後の裁定205(`next/DECISIONS.md` #205)は「追加する意図の家は Browser ただ1つ」— create タブに Text カードを生やし Inspector に本文入力欄と色エディタを繋ぐ、と処置を決めた。

**発注書はこの処置が着地した前提で先を掘る指示だったが、実測すると裁定205はまだ実装されていない**(下記「逸脱」参照)。本調査は実コード(`main` fast-forward後 `fcd72f30`)を grep して確認できた事実だけを根拠にし、「未着地」という実測結果自体を最初の壁として記録したうえで、**round1 が「text layer 前提で通る」と留保していた工程(6・7)およびそれより先(量産・直し)を、店に架空の text layer が既に居るという最小の仮定のもとで**掘り下げた。仮定を要する箇所は全て明示する。

対象コミット: `fcd72f30`(`git merge main` 後、`docs: 裁定205(...)`)。

## 壁の順序リスト

以下、工程を辿る順に番号を振る。round1 の #1〜#4(プロジェクト/取り込み/波形/トリム)は結果に変化が無いため再掲は最小限にとどめ、round1 が「未検証」で止めた先(#5以降)に焦点を当てる。

### 壁1 — テキストレイヤー作成の入口(round1から不変・実測で再確認)

- **何が無いか**: `Message::AddLayer` は依然として常に `LayerSource::Solid` を作る(`next/shell/motolii-shell/src/lib.rs:1409-1446`)。Browser create タブの `CreateKind` enum は `Rectangle`/`Ellipse`/`Solid`/`Null` の4種のみで `Text` が無い(`next/ui/motolii-browser-pane/src/model.rs:341-349`)。`create_from_card`(`lib.rs:1548-1559`)の match も同じ4種のみ。
- **迂回可否**: 不可(round1と同じ — Shape はベクタパス、Media import は静的ファイル)。
- **根拠**: 上記 grep 実測。裁定205自体はこの欠落の**処置方針**を記録した文書であって実装ではない(逸脱節参照)。

### 壁2 — 文字列内容(歌詞そのもの)を打つ欄が無い

- **何が無いか**: TEXT section の `TextField` enum は `FontFamily`/`Size`/`LineHeight`/`Tracking` の4種のみで(`next/ui/motolii-inspector-pane/src/text.rs:36-46`)、`Content` に相当する腕が無い。`TextDocument::content`(`ContentTrack` 型、`next/core/motolii-store/src/text.rs:436-467` 付近)を書き込む `Intent::SetTextDocument` 呼び出しは repo 全体で `text.rs` 自身の `apply_text_document_edit`(既定値生成のみ)を除き **UI 側に無い**。
- **迂回可否**: 不可。壁1が塞がっていること自体もこの壁を実測不能にしているが、`text.rs` の TEXT section 実装そのものにも文字入力欄が構造的に無い(FontFamily 欄はフォント名を打つ欄であって本文欄ではない)。
- **根拠**: `text.rs:36-46`(enum 定義)・`text.rs:251-280`(`text_section` が組む行は Font/Size/LineHeight/Tracking/Justify のみで content 行が無い)。

### 壁3 — 塗り色(fc)エディタは書けているが未結線

- **何が無いか**: `next/ui/motolii-inspector-pane/src/color.rs`(689行、commit `a25108fd`)が `TextDocumentStyle::fill`/`stroke_color` へ RGBA 編集を書く `color_row`/`commit_text_style_color` を実装済みだが、**crate の `Message` enum にも `text_section` の Fill/Stroke 行にも一切繋がっていない**——同ファイルの doc が自認: 「この module は自己完結 — `crate::Message`/`crate::view` を一切変えない...結線には...`Message` enum へ2腕...を足し、`text_section` の Fill/Stroke 行を `color_row` へ差し替える必要があり、どちらも今回の write-set 外」(`color.rs:53-59`)。`text.rs:248-250` も「塗り色・線色は実在するが `Value::Color` 用の editor がまだ無い」と旧来通りのコメントを残している(この一文は `color.rs` の存在によって半分古い — editor コードは在るが繋がっていない、が正確)。
- **迂回可否**: 無し。歌詞の色を変える手段がUIに一切無い(既定値 `[0,0,0,1]` 黒固定、`text.rs:88`)。
- **根拠**: `color.rs:53-59`(自己申告の未結線)、`text.rs:248-250`。

### 壁4 — Timeline レーンリストに縦スクロールが無い(量の根本壁・round2の新規発見)

- **何が無いか**: `TimelinePane::view`(`next/ui/motolii-timeline-pane/src/lib.rs:368-375`)は `content_height()`(`lib.rs:344-348` — `ruler_height + row_height*rows.len() + param_row_height*property_rows.len()`)を**固定高さ**として `row![rail, field].height(Length::Fixed(height))` を返す。`next/ui/motolii-timeline-pane/src/` 全体・`next/shell/motolii-shell/src/lib.rs` 全体を grep しても `scrollable`/`Scrollable` は**0件**(他の pane — browser-pane/inspector-pane/settings-pane — には実在するので、意図的な省略ではなく単に未実装)。shell 側の埋め込み(`lib.rs:4034-4048`)も `pane_grid::Content::new(content)` に直接渡すだけで scrollable でラップしない。行高は `row_height: 20`(`next/ui/motolii-tokens-rs/tokens/dimensions.json:4`)固定。マウスホイール/垂直スクロールのハンドラも `next/ui/motolii-timeline-pane/src/*.rs` に無い(横方向の zoom/scroll も `nav.rs:56-60` のコメントが「入った時の」と将来形で書いており未着手)。
- **迂回可否**: 部分的。`Message::GroupLayers`(Cmd+G、round1確認済み)でレイヤーをグループ化し `ToggleFold`(`write.rs:634`、`session.timeline_fold`)で畳めば `rows()`(`projection.rs:78`、`children_open = !session.timeline_fold.is_folded(id)` で子行を除外)が縮み、一覧性は多少戻る。だが個々のレイヤー(例: 47行目の歌詞)を実際に選択・編集する時は展開が要り、展開すれば同じ「画面高さを超えた分は見えない・触れない」問題に戻る——**恒久的な迂回にならない**。
- **根拠**: `lib.rs:344-348,368-375`(TimelinePane)、`lib.rs:4034-4048`(shell 埋め込み)、`dimensions.json:4`、`projection.rs:78,183`(fold反映)、`write.rs:634`(ToggleFold)、grep count 0 for `scrollable` in timeline-pane と shell。

### 壁5 — 後から一括で色/フォント/文字内容を変える経路が無い(量と直しの核心)

- **何が無いか**: TEXT section の書き込み経路(`inspector_pane::Message::TextFieldSubmit`/`CycleTextJustify` のハンドラ)はどちらも `self.session.selection`(**単一 `Option<LayerId>`**)だけを渡す(`next/shell/motolii-shell/src/lib.rs:2227-2244`)。複数選択の集合 `self.session.selected_layers: Vec<LayerId>`(`next/ui/motolii-shell-state/src/lib.rs:44`)は同ファイルのdocコメントが自認する通り「`inspector_pane` の行 UI 自体はまだこちらを読まない」(`lib.rs:38-43`)。つまり50〜100枚の歌詞レイヤーを Shift/Cmd で複数選択しても、フォントサイズや色を一度に変える操作は存在しない。
- **迂回可否**: 有り(ただし線形コスト)。1枚ずつ選び直して同じ編集をN回繰り返す——歌詞が80行あれば80回の個別操作・80回分のUndo境界。
- **根拠**: `lib.rs:2227-2244`(TextFieldSubmit/CycleTextJustify が `self.session.selection` を渡す)、`motolii-shell-state/src/lib.rs:37-44`(selection と selected_layers の身分差、doc内自認)。
- **対比(通る側)**: キーフレーム(`Session::selected_keys: Vec<KeySelector>`)側は複数選択・複数選択に対する一括ドラッグ/削除/反転/コピーが実装済み(`write.rs:1026,1048-1052,1148-1243,1309-1476` — `origins = session.selected_keys.clone()` を起点にした一括移動)。**静的フィールド(フォント/色/内容)の一括編集と、動的トラック(位置/不透明度のキーフレーム)の一括編集は別実装で、前者だけが欠けている**——この非対称は正確に記録する価値がある。

### 壁6 — 複製(Duplicate)は単一レイヤーのみ

- **何が無いか**: `duplicate_layer`(`lib.rs:1800-1817`)は `self.session.selection`(単一)のみを読み、複製後は新規1枚を選択する。`selected_layers` を対象にした一括複製は無い。「テンプレ1枚を作って歌詞だけ差し替える」量産パターンにおいて、複製そのものは50〜100回リピートする必要がある。
- **迂回可否**: 有り(線形回数のCmd+D+リネーム+content書き換えの繰り返し。壁2が塞がっているため content 書き換え自体は現状不可能だが、壁2が仮に開通しても複製はN回になる)。
- **根拠**: `lib.rs:1800-1817`。

### 壁7 — 波形が見えない(round1から不変、再確認)

- **何が無いか**: `plan_waveforms`/`WaveformFetched`/`with_waveforms` は `next/shell/motolii-shell/src/lib.rs` 全体で **0件**(round1と同じ grep 結果を再実測・変化なし)。意味・pane側計画・描画は実装済みだが shell 結線が無い。
- **迂回可否**: 有り(耳で概算)。
- **根拠**: `grep -c "plan_waveforms\|WaveformFetched" next/shell/motolii-shell/src/lib.rs` → `0`。

### 壁8 — Split はロジック完成・入口は依然ゼロ(round1からの前進を正確に記録)

- **何が無いか**: `write::Message::SplitAtPlayhead` は**もはや宣言のみではない**——`write.rs:685`(`self.split_at_playhead(doc, session)`)と `write.rs:811-837`(`split_at_playhead` 本体)が実装済みで、`session.selected_layers`(複数選択)が非空ならそれを、空なら `session.selection` を対象に `split::split_selected_plan` を呼び1回の `apply_all` で確定する——**複数選択にも対応済み**。しかし shell/menu/keymap のどこからも `SplitAtPlayhead` を送る経路が無い(`next/shell/motolii-shell/src/lib.rs`・`menu.rs`・`next/ui/motolii-keymap/src/defaults.rs` を grep して0件)。`next/ui/motolii-menubar/src/context.rs:55` のコメント「宣言のみで write::Message/shell へ未統合」は**半分stale**——write::Message 側は統合済み、shell 側だけ未統合という、より狭い残り穴になっている。
- **迂回可否**: round1と同じ(素材を複数レイヤーへ import してトリムで代替)。
- **根拠**: `write.rs:685,809-837`(実装)、`context.rs:55`(stale化したコメント)、shell/menu/keymap 側 grep 0件。

### 壁9 — マスクで新規に抜く経路が無い

- **何が無いか**: `next/ui/motolii-inspector-pane/src/mask.rs` は既存マスクの mode 巡回(`MaskMode::Add → Subtract → ...`)のみを扱い、新規マスク(パス)を追加する Message が無い(`AddMask`/`CreateMask` 相当を `next/ui/`・`next/shell/` 全体で grep して0件)。fixture 相当の直接 `Intent::SetMasks` 呼び出し以外に手段が無い。
- **迂回可否**: 無し。
- **根拠**: grep 0件(`AddMask|CreateMask` 全体)。round1 の別ペルソナ調査(P3, commit `fa681dd3`)が同じ穴を独立に検出済み。

### 壁10 — トラックマット(下のレイヤーで抜く重ね)のUI入口が無い

- **何が無いか**: `Matte`/`MatteMode` 型(`next/core/motolii-store/src/attrs.rs`)はレンダリング側(engine)に結線済み(commit `5e799836`)だが、`next/ui/`・`next/shell/` を通して `Matte` を grep すると **0件**——設定する UI が存在しない。
- **迂回可否**: 無し。利用可能な重ね手段は `BlendMode` の巡回(`CycleBlendMode`、round1確認済み・通る)のみ。
- **根拠**: `grep -rln Matte next/ui/ next/shell/` → 0件。

### 壁11 — 文字を1文字ずつ動かすアニメーター(TextRange)が完全に不在

- **何が無いか**: `TextRange`/`TextRangeSelector`(AEのText Animator相当、`next/core/motolii-store/src/text.rs:259-373`)は store 型として存在するが、`next/ui/`・`next/shell/` に `TextRange`/`TextAnimator` の参照が**0件**。
- **迂回可否**: 無し(部分的代替として、レイヤー全体を1塊として position/opacity キーフレーム+イージングでフェード/スライドさせることは壁1・2が開けば round1 #6/#7 の経路で可能——ただし「1文字ずつ」の粒度は出せない)。
- **根拠**: `grep -rln "TextRange\|TextAnimator" next/ui/ next/shell/` → 0件。

### 壁12 — エフェクト新規追加/トランジション(round1から不変)

- 内容は round1 と同じ(`AddEffect` 相当のMessageが無い、トランジション概念が実装として存在しない)。再確認のみで詳細は省略(round1参照)。

## 量と直しの観点で見た摩擦の一覧

| 観点 | 実測 |
|---|---|
| **50〜100枚を作る** | 壁1・2が塞がっているため作成そのものが不可能。仮に開通しても、複製は1枚ずつ(壁6)・タイムラインの縦スクロールが無い(壁4)ため、40行目を過ぎたあたりで「作れるが見えない/選べない」状態になる。 |
| **一括変更(色・フォント)** | 静的フィールド(TextDocumentStyle)は単一選択のみ書き込み可能(壁5)。80行の歌詞のフォントサイズを直す指示があれば80回の個別操作になる。 |
| **一括変更(タイミング/位置)** | ここは**通る**——`selected_keys` の複数選択+一括ドラッグ/削除/反転/コピー(`write.rs`)が実装済み。位置・不透明度のキーフレーム調整は複数レイヤー・複数キーをまたいで一度に動かせる。「静的値は不可・動的トラックは可」という非対称が実態。 |
| **命名** | Rename(Enter、`RenameSelectedLayer`)は1枚ずつ・inline text_input。バッチリネームは無い。100行に自動連番("Lyric 001".."Lyric 100")を振る手段も無い(手作業でN回)。 |
| **選択** | Select All(Cmd+A)・Shift/Cmd複数選択（`selected_layers`）は実装済みで「選ぶ」こと自体は量に耐える。「選んだ後に何をするか」で壁5にぶつかる。 |
| **一覧性(俯瞰して直す)** | 壁4(縦スクロール無し)により、画面に入らない分は物理的に触れない。Group+Fold は一覧性の部分的な緩和(折り畳んだ1行に要約)だが、個別編集時は展開して同じ壁に戻る。 |
| **通しの確認と手直し** | 再生(`TogglePlayback`)・スクラブ(`ScrubTo`)自体は通る(round1確認済み、変化なし)。ただし壁4のせいで「再生しながら47行目の歌詞の位置を直す」ときにその行が画面外なら、まずタイムラインで辿り着く手段が無い。 |
| **書き出しと作り直し** | Export自体は通る(round1確認済み、変化なし)。修正→再書き出しのループを阻む要素は export 機構自体には無い——阻むのは上流(壁1〜11)がまだ塞がっていること。 |

## この1本が通るために必要な最小実装(順序つき)

「歌詞が画面に出る」→「量に耐える」→「タイミングが合う」→「仕上げ」の順で並べる。1〜3は round1 の必須セットの再掲(裁定205はまだこの3つを実装していない)、4〜7が本調査(round2)の核心である「量と直し」に直結する必須セット、8以降は仕上げ。

1. **Text create card を Browser create タブへ足す**(裁定205 の処置そのものをまず実装する — `CreateKind::Text` を `model.rs:341-349` へ追加し、`create_from_card`(`lib.rs:1548-1559`)に `LayerSource::Text` 腕を足す)。
2. **TEXT section に `TextField::Content` を足す**(`text.rs:36-46` の enum へ1腕追加、`ContentTrack` への text_input を配線 — 裁定92 の「丸ごと差し替え `Intent::SetTextDocument`」の型にそのまま乗る、round1 の指摘のまま)。
3. **`color.rs` を結線する**(コードは書き上がっている——crate `Message` へ `ColorChannelInput`/`ColorChannelSubmit` の2腕を足し、`text_section` の Fill/Stroke 行を `color_row` に差し替えるだけ。`color.rs:53-59` が手順を自ら書いている)。
4. **Timeline レーンリストに縦スクロールを足す**(`TimelinePane::view` の固定高さ `row![rail, field].height(Length::Fixed(height))` を `scrollable` でラップする、または `pane_grid::Content` 側でラップする——量産ワークフローの根本ボトルネック、これが無いと1〜3が開通しても40行を超えたところで詰まる)。
5. **TEXT section の書き込み経路を `selected_layers`(複数選択)にも対応させる**(`lib.rs:2227-2244` の `commit_text_field`/`cycle_text_justify` 呼び出しを、`selection` 単体でなく `selected_layers` が非空ならその全員へ適用するループに変える——`split_at_playhead`(`write.rs:811-837`)が既にこの「複数選択があればそちらを優先」の形をキーフレーム外の操作で実演済みなので、同型を踏襲できる)。
6. **Duplicate を複数選択対応にする**(`duplicate_layer`、`lib.rs:1800-1817` を `selected_layers` ループへ拡張)。
7. **`SplitAtPlayhead` を menu/keymap へ配線する**(ロジックは`write.rs`に完成済み——`context.rs:55`の「次波の統合」を実行するだけ、新しい意味は不要)。
8. **波形の shell 結線**(round1と同じ内容、`plan_waveforms`/`Task::perform`/`WaveformFetched`)。
9. **マスク新規作成のUI入口**(`AddMask`相当のMessage)。
10. **トラックマットのUI入口**(既存 engine 結線に対する設定UI)。
11. **`TextRange`(1文字ずつのアニメーター)のUI**。
12. **`AddEffect`相当のMessage+グリッチ系エフェクトの実装**(round1と同じ)。

## 逸脱

- **発注書の前提「裁定205でその処置が決まり実装が走行中」は実測と食い違う**。裁定205(`next/DECISIONS.md` #195行、内容番号205)は方針決定の文書であり、`main` fast-forward後の `fcd72f30` 時点で `CreateKind::Text`・TEXT section の content 欄・color.rs の結線のいずれも**着地していない**(壁1〜3として詳述)。関連する `git branch -a` 走査でも "text"/"browser"/"create" を含む名前のブランチはすべて2026-08-05〜08-19の古いものだけで、裁定205後の作業ブランチは見当たらなかった(ただし全ブランチ・全 worktree を網羅的に照合したわけではなく、他の並列レーンが未push/未commitで進行中の可能性は排除できない)。本調査はこの実測結果に基づき、round1の到達可能性判定をそのまま引き継いで壁1〜3を再確認する形で進めた。
- **Split(`SplitAtPlayhead`)は round1 時点から前進していた**——round1・`context.rs:55` が「宣言のみ」と評した状態は、現在は「`write::Message` 側は複数選択対応で完成、shell/menu/keymap の配線だけが残る」まで縮んだ。`context.rs:55` のコメント自体は更新されておらず、コード実態より一段古い記述になっている(壁8参照)。
- **静的フィールドと動的トラックの一括編集能力の非対称**(壁5)は、どのペルソナ調査文書にもまだ明記されていなかった実装上の分岐点で、本調査で新たに特定した。「Session::selected_layers の Inspector 側 unread」は `motolii-shell-state/src/lib.rs` 自身のdocコメントが既に自認していた既知の穴だったが、これまでの調査はそれを歌詞動画の量産ワークフローの具体的な障害として結び付けていなかった。
- **Timeline縦スクロール不在(壁4)は今回が初出の発見**。round1はテキストレイヤー作成不能で早期に停止していたため、量産段階(50〜100層)特有のこの壁は検出されなかった。scrollable widgetが他のpane(browser/inspector/settings)には実在することから、意図的な設計判断ではなく単純な未実装と判断した(根拠: 使用パターンの一貫性)。
- 本調査は「壁の向こうを掘る」という発注意図に沿い、round1が「text layer前提で通る」と留保した工程(6・7=キーフレームによるフェード/移動)についてはround1の判定をそのまま引き継ぎ再検証を省略した(round1のfixture実演による実測が十分に具体的だったため)。
