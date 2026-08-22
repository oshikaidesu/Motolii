# ペルソナ「歌詞動画/MV 制作者」で最短の欠落を洗い出す

利用者の危機感: 台帳の行を freq や依存で上から潰す運転では製品を貫く1本の線が見えない。ワークフローは直列なので、最初に壊れる所が最短の答え。この文書は歌詞動画/MV制作という第一ペルソナ(`next/shell/motolii-shell/src/fixture.rs` の15層 fixture が実在の証拠)の10工程を、**実コードに対する到達可能性判定**として1歩ずつ検分する。

## 対象

`next/` ワークスペース(2026-08-22 時点、`main` へ fast-forward 済み — commit `8e48917a` 以降)。判定は grep で示せる実在識別子のみを根拠にする。推測は書かない。

## 工程表(10段)

| # | 工程 | 判定 | 根拠 |
|---|------|------|------|
| 1 | プロジェクトを作る/開く/保存する | **通る** | `next/shell/motolii-shell/src/menu.rs` File メニュー: `Message::NewProjectRequested`(Cmd+N)/`OpenRequested`/`SaveAsRequested`(Cmd+Shift+S)/`SaveACopyRequested`。`lib.rs:1458-1479` に実ハンドラ(`self.confirm_then_pick_open()`→`self.perform_open(path)`、`self.dialogs.pick_save_path()`→`self.perform_save_as(path)`)があり rfd 経由の実ファイルダイアログに繋がる。 |
| 2 | 曲(音声)と映像素材を取り込む | **通る** | File メニュー `Import Media…`(`Message::ImportMediaRequested` → `self.dialogs.pick_import_paths()` → `Message::AdmitPaths`、`lib.rs:1477`)。OS drop も同経路。音声 mime 判定は `lib.rs:2073`(`"wav" \| "mp3" \| "aac" \| "flac" \| "ogg" \| "m4a" => "audio/{ext}"`)、ファイルダイアログのフィルタも `file_dialogs.rs:150` に実在。Browser pane の `Category::Audio` 分類は `motolii-browser-pane/src/model.rs`。 |
| 3 | 曲を配置し、波形を見ながらサビの位置を掴む | **詰まる(重度・迂回は苦しい)** | 意味(`motolii-media::waveform_peaks`)・pane 内 fetch 計画(`motolii-timeline-pane/src/write.rs` の `PaneState::plan_waveforms`/`Message::WaveformFetched`/`WaveformFetchFailed`)・描画(`canvas.rs:259-273`、`pane.waveforms.get(&row.id)` があれば波形を描く)は**実装済み**。だが **shell 側の結線が無い** — `next/shell/motolii-shell/src/lib.rs` 全体を grep しても `plan_waveforms`/`with_waveforms`/`WaveformFetched` は **0件**。shell は `pane.waveforms` を一度も埋めない(常に空 `HashMap`)ので `canvas.rs` の描画分岐は実行時には常に不発。マーカー機構自体(`Intent::SetMarkers`)は store にあるが、UI から「サビの位置に印を置く」操作(マーカー追加 verb)も `motolii-verbs/src/registry.rs` に無い。迂回は「音声を再生しながら耳で掴んで概算する」のみ(§9 参照、cpal 再生自体は動く)。 |
| 4 | 映像を配置し、リズムに合わせて切る(Split・トリム) | **半分詰まる(トリムは通る/Splitは詰まるが迂回可能)** | **トリム = 通る**: `clip_gesture.rs` の `BarPart::EdgeIn`/`EdgeOut` ドラッグが `write.rs:continue_drag` で `timing.start`/`timing.duration`/`timing.source_in` を書き換える(実装済み・採用済み)。**Split = 詰まる**: 純関数 `split_plan`/`split_selected_plan` は `motolii-timeline-pane/src/split.rs` に実装済みだが、モジュール冒頭 doc が明記する通り「`Message::SplitAtPlayhead` は宣言のみで `write::Message`/shell へ未統合」。決定的な一次資料は `motolii-menubar/src/context.rs:55` の表: `\| Split \| **存在しない** — ...宣言のみで write::Message/shell へ未統合... \| — \| 見送り(次波の統合を待つ) \|`。メニュー・keymap・右クリックのどこにも Split の入口が無い。迂回可能: 同じ素材を複数レイヤーとして重ねて import し、各レイヤーを EdgeIn/EdgeOut トリムで任意の source 区間へ絞り、Body ドラッグで開始位置をずらせば「1本の素材から複数カットを組む」ことは手数をかければ到達できる(ただし Split 1操作の代わりに import+trim×N が要る)。 |
| 5 | 歌詞テキストを載せる(テキストレイヤー作成→文字入力→フォント/サイズ/色) | **致命的に詰まる(迂回不可)** | store 側の型 `LayerSource::Text`(`next/core/motolii-store/src/lib.rs:216-219`)・組版データ `TextDocument`/`TextDocumentStyle`(`next/core/motolii-store/src/text.rs`)・Inspector の TEXT section(`next/ui/motolii-inspector-pane/src/text.rs`、Font/Size/Line Height/Tracking/Justify の編集 UI)は実装済み。**だが text layer を作る UI 入口が一切存在しない**: Layer メニューは `New Layer`(`Message::AddLayer`)1本のみで、そのハンドラ(`lib.rs:1409-1446`)は常に `LayerSource::Solid`(単色矩形)を作る。`motolii-verbs/src/registry.rs` の動詞一覧(edit.*/layer.*/window.*/help.*/layer_row.*/keyframe.*)に text 系動詞は無い。リポジトリ全体を `LayerSource::Text` で grep しても、生成箇所はテスト・fixture(`fixture.rs`)・store/engine の内部処理だけで、shell/UI 側の生成箇所は **0件**。ステージ上のダブルクリックでテキストを打つような「テキストツール」も存在しない(`grep -i "text.*tool\|TypeTool\|AddTextLayer"` は全て0件)。**さらに** TEXT section 自体にも文字内容(`document.content: ContentTrack`)を編集する `text_input` が無い(Font/Size/LineHeight/Tracking/Justify の5項目のみ)ことと、塗り色(`fc`)編集 UI も無い(`text.rs:248-250` のコメントが自認: 「塗り色・線色は実在するが Value::Color 用の editor がまだ無い」)ことを、コード自身が明記している。**結果: 現在のビルドでは歌詞(あるいはどんな文字列)を一切画面に置けない** — 迂回経路が存在しない(Shape レイヤーはベクタパスであり文字を持たない、Media import は既存ファイルの取り込みであり動的な文字レイヤーの代替にならない)。 |
| 6 | 歌詞をタイミングに合わせて出し入れ(不透明度キーフレーム・イン/アウト) | **通る(text layer が存在すれば)** | Inspector Transform 行の Key click(`motolii-inspector-pane/src/transform.rs:321` 「Key click の意味」— クリックで `Intent::SetTrack` を発行)は Opacity 行にも適用される(`lib.rs:420` `selection.transform.iter().filter(\|r\| r.label == "Opacity")`)。fixture の「タイトルロゴ」「メインボーカル映像」レイヤーが同じ機構(`Intent::SetTrack` + `property::OPACITY`)でフェード in/out を実演済み(`fixture.rs:242-323`)。ただし前提として text layer 自体が作れないため(§5)、歌詞という対象に対しては到達不能。 |
| 7 | 文字を動かす(位置/スケールのキーフレーム・イージング) | **通る(text layer が存在すれば)** | `PropertyId::POSITION`/`SCALE`(`motolii-store/src/lib.rs:115-125`)は `Intent::SetTrack` に乗る。イージングは Edit メニューの `Interpolation: Hold/Linear/Easy Ease/Easy Ease In/Easy Ease Out`(`menu.rs` 内 `Message::Timeline(timeline_pane::Message::SetKeyInterp(...))`)。fixture の「サビ歌詞」レイヤーが `Interp::Bezier` 付き position キーで実演済み(`fixture.rs:274-297`)。§6 と同じ理由で text layer 前提の到達不能あり。 |
| 8 | トランジション/エフェクト(グリッチ的な物・ブレンドモード) | **多くが詰まる** | **ブレンドモード = 通る**: `BlendMode` enum(17種、`motolii-store/src/attrs.rs:24-42`)は全レイヤーが既定で持つフィールドであり、Inspector の `Message::CycleBlendMode`(`attrs.rs:124`、shell側ハンドラ `lib.rs:2114`)で巡回できる(「追加」操作が要らないので通る)。**エフェクトの追加 = 詰まる**: engine が対応する plugin_id は `"motolii.glow"` の1本のみ(`motolii-engine/src/lib.rs:1235-1258`)で、グリッチ的なエフェクトは存在しない。さらに EFFECTS section(`motolii-inspector-pane/src/effects.rs`)の Message は `RemoveEffect`/`MoveEffectUp`/`MoveEffectDown`/`ToggleEffectBypass` のみで **`AddEffect` に相当する Message が無い**(`motolii-inspector-pane/src/lib.rs` を grep しても存在しない)— 既に effect を持つレイヤーの管理はできても、新規にレイヤーへ Glow を付ける UI 入口が無い(fixture は `Intent::SetEffects` を直接叩いている)。**トランジション = 完全に不在**: リポジトリ全体で「Transition」という編集概念(クロスフェード等)を実装したコードは無い(test 名の英単語ヒット1件を除き0件)。fixture の「グリッチトランジション」レイヤーは名前が示す意匠だけで、実体は単色矩形(`LayerSource::Solid`)に親子関係を1本付けただけ(`fixture.rs:394-410`)— グリッチ効果そのものは実装されていない。 |
| 9 | プレビューで確認(再生・スクラブ・音付き) | **通る** | `Message::TogglePlayback`(Space キー、`lib.rs:5665`)・`Message::ScrubTo`/`timeline_pane::Message::ScrubTo`(`lib.rs:1256,1305`)。音声再生は `next/engine/motolii-audio`(cpal ベース、`transport.rs` が `motolii_audio::PlaybackSession`/`AudioProgram::from_view` を使って実デバイス出力する — `debug_start_playback_with_session` はテスト用フェイクセッションで実 cpal 経路とは別)。 |
| 10 | 書き出す(範囲・形式・進捗) | **通る** | `motolii-export-pane::Message`: `RangeSelect(ExportRange)`(全体/作業範囲)・`QualitySelect(ExportQuality)`(Normal=CRF18/Lossless=qp0)・`PickOutputPath`→`OutputPathChosen`・`Export`→`CancelExport`(`lib.rs:101-124`)。進捗は `ExportProgress`/`progress_fraction`/`format_progress`(`lib.rs:262-310`、`frames_done`/`frames_total` を分数表示)。コンテナ/コーデックは MP4/H.264 の1択のみ(`CONTAINER_CODEC_LABEL`)だが、選択肢を発明していないだけで機能としては通る。 |

## 最初に致命的に詰まる所

**工程5「歌詞テキストを載せる」— テキストレイヤーを作成する UI 入口が存在しない。**

- store/engine/Inspector は文字を描画・編集する意味とレンダリング経路(cosmic-text → swash → motolii-vector、裁定190)を持っているのに、**「レイヤーを追加」する唯一の verb(`Message::AddLayer`)が常に単色矩形を生成する**(`next/shell/motolii-shell/src/lib.rs:1409-1446`)。
- メニュー(`Layer` メニューは New Layer/Group/Ungroup/Freeze/Unfreeze の5項目のみ)・keymap(`motolii-keymap/src/defaults.rs` に text 系無し)・右クリック(`motolii-menubar/src/context.rs`)・パネル常設コントロール(rail glyph/inspector swatch)のどこにも text layer 作成の入口が無い。
- 仮に text layer が何らかの方法(fixture 相当のコード)で存在しても、TEXT section には文字列そのもの(`content`)を打ち込む欄も、塗り色を選ぶ欄も無い(`next/ui/motolii-inspector-pane/src/text.rs:241-280` — Font/Size/Line Height/Tracking/Justify の5項目のみ)。
- 工程3(波形)・工程4(Split)は「意味はあるが結線されていない/未統合」という同種の欠落だが、**耳で聞く/複数レイヤーで代替するという迂回路が(苦しくとも)存在する**。工程5は迂回路そのものが無い — Shape(ベクタ)は文字を持たず、Media import は静的ファイルの取り込みであって動的なテキスト編集の代替にならない。歌詞動画から歌詞そのものを除くと製品の目的が成立しない。

## 迂回可能な詰まりの一覧

| 工程 | 詰まりの内容 | 迂回 |
|---|---|---|
| 3(波形) | shell が `plan_waveforms`/`WaveformFetched` を一度も呼ばない(未結線) | 音声を再生しながら耳で概算する(cpal 再生経路は生きている)。精度・速度は大きく劣る。 |
| 4(Split) | `SplitAtPlayhead` 相当の Message が無い(宣言のみ) | 同じ素材を複数レイヤーとして import し、各レイヤーを EdgeIn/EdgeOut でトリム+Body ドラッグで配置(手数増だが到達可能)。 |
| 8(エフェクト追加) | `AddEffect` 相当の Message が無い | 無し(fixture 同等の直接 Intent 操作は一般利用者には不可能)。実質、この項目は Glow を含め**新規適用が誰にもできない**ため迂回不可に近い。 |
| 8(トランジション) | 概念自体が実装されていない | 無し。 |

## この1本が通るために必要な最小の実装(順序つき)

歌詞動画という「製品を貫く1本の線」を通す最短経路として、致命点(工程5)から着手し、直列の次の壁を順に倒す:

1. **テキストレイヤー作成の UI 入口を1つ作る**(Layer メニューに `New Text Layer` を追加 → `Intent::AddLayer` + `Intent::SetMeta{source: LayerSource::Text, ...}` + `Intent::SetTextDocument` で既定の空文書を積む、`Message::AddLayer` ハンドラ `lib.rs:1409` と同型の1操作=1 undo で)。これが無いと工程5以降(6・7も含む)が全滅する。
2. **TEXT section に文字列編集欄を足す**(`text.rs` の `TextField` enum へ `Content` を追加し、`text_input` で `document.content`〈`ContentTrack`〉を編集できるようにする — 裁定92のスタイル表既定行と同じ「丸ごと差し替え `Intent::SetTextDocument`」の型にそのまま乗る)。文字が打てなければ歌詞にならない。
3. **TEXT section に塗り色(fc)の editor を足す**(`text.rs` 冒頭コメントが自認する既知の穴。`Value::Color` editor は Effect 束の仕事とされているので、そこと同時に倒すのが筋が良い)。
4. **Split(`Message::SplitAtPlayhead`)を `write::Message`/shell へ統合する**(`split.rs` の純関数は完成済み。`context.rs:55` が指す「次波の統合手順」を実行するだけ — 新しい意味は要らない)。工程4のリズム編集を1操作に戻す。
5. **波形の shell 結線**(`plan_waveforms` の呼び出し、`Task::perform(motolii_media::waveform_peaks(path, buckets), ...)`、`Message::WaveformFetched`/`WaveformFetchFailed` の shell 側ハンドラ、`with_waveforms` での pane への反映)。`waveform_view.rs`/`write.rs` の doc が既に手順を書いている。
6. **`AddEffect` 相当の Message を EFFECTS section に足す**(既存 Glow を新規レイヤーへ付けられるようにする — 最低限これが無いと fixture 以外の誰も Glow を使えない)。
7. **グリッチ系エフェクトの実装**(工程8の「グリッチ的な物」自体の内容。優先度は上記6点より低い — まず「既存の Glow を誰でも付けられる」ことが先)。

1〜3が「歌詞を画面に出す」ための必須セット、4〜5が「MVらしい編集(リズム編集・サビ発見)」の必須セット、6〜7は仕上げ(装飾)に相当する。

## 逸脱

- `next/DECISIONS.md` 裁定190 は「Inspector TEXT section(現状 TEXT 束は grep 0件)」を gap として記録しているが、本調査時点(main マージ後)では TEXT section 自体は **実装済み**(`next/ui/motolii-inspector-pane/src/text.rs`)だった。裁定190 のこの1文は古い状態を指しており、現状と食い違う(TEXT section は存在するが、レイヤー作成入口と文字入力欄が無い、という**別の**欠落が残っている、というのが本調査の実測)。
- `next/reference/normal-map.tsv` の id 1249/1284(`Add Text (layer)`/`New text layer`)はいずれも「採用済」列に「`LayerSource::Text` / `Layer:text` component 実装済み」と記載されている。これは store 側の型が存在することの記述として事実だが、**UI からその型のインスタンスを作る手段が無い**ことは書かれていない。台帳の「採用済」表記と実際の到達可能性(UI 入口)の間にギャップがある — 台帳を設計根拠にせず実コードで確認する規律(review discipline 6点)が実際に機能した例として記録する。
- 本調査は工程を「到達可能か」の二値に近い形で判定したが、工程8(エフェクト/トランジション)は内部に複数の独立した欠落(ブレンドモード=通る/エフェクト追加=詰まる/エフェクト種類不足=詰まる/トランジション概念不在=詰まる)を含み、単純な「通る/詰まる」1マスに収まらない。工程表では最も厳しい判定(詰まる)を採用し、内訳を本文に展開した。
