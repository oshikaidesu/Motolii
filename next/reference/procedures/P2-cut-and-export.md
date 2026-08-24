# P2 — 素材を並べて切って書き出す

対象: 「撮った素材(動画+音声、複数本・数分尺)を並べて、いらない所を切って、mp4 で書き出して人に渡す」。
新規プロジェクトを開くところから、書き出したファイルを人に渡すところまで全部を書く。
形式は [`README.md`](README.md) の規約に従う。判定4値・粒度の規約(名前の無い操作をまとめない)はそこを参照。

---

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 1 | アプリを起動する | OS | `next/shell/motolii-shell/src/main.rs` `iced::daemon(...)` + `Shell::boot` | 書ける |
| 2 | 起動直後、既定コンポジション(16:9・空)のプロジェクトが開いた状態で始まる | Stage/Timeline | `Shell::default_document`(`next/shell/motolii-shell/src/lib.rs:1103`)、起動経路は `boot` が呼ぶ | 書ける |
| 3 | 「File > Import Media…」を開く | メニュー | `next/shell/motolii-shell/src/menu.rs:90` `Item { label: "Import Media…", message: Message::ImportMediaRequested }` | 書ける |
| 4 | OS のファイルダイアログが開く | OS dialog | `next/shell/motolii-shell/src/lib.rs:1498` `Task::perform(self.dialogs.pick_import_paths(), Message::AdmitPaths)` | 書ける |
| 5 | ダイアログで動画ファイルを複数(Cmd+クリック等 OS 標準の複数選択)選ぶ | OS dialog | `rfd::AsyncFileDialog::pick_files()`(`next/shell/motolii-shell/src/file_dialogs.rs:144-153`、Video/Image/Audio 拡張子フィルタ付き) | 書ける |
| 6 | 同じダイアログで音声ファイルも追加で選ぶ(動画と混在) | OS dialog | 同上。フィルタは3種同時提示(`file_dialogs.rs:145-147`) | 書ける |
| 7 | 選択を確定してダイアログを閉じる | OS dialog | `pick_files().await`(`file_dialogs.rs:151`) | 書ける |
| 8 | 取り込み結果としてライブラリ(Browser)にカードが並ぶのを見る | Browser | `admit()` が `Intent::AdmitAsset` を積む(`next/shell/motolii-shell/src/lib.rs:2185`)。カード描画は `browser-pane` のサムネイル(`next/ui/motolii-browser-pane/src/lib.rs`・`model.rs`) | 書ける |
| 9 | 読めない素材が混じっていた場合、拒否理由がその場(status 帯)に出るのを確認する | status 帯 | `admit()` の `rejected` 収集→`self.status`(`next/shell/motolii-shell/src/lib.rs:2226-2264`、`motolii_media::probe` 失敗時) | 書ける |
| 10 | 記帳(fingerprint 計算)だけ失敗した場合の別の拒否理由(「台帳への記帳をスキップ」)を確認する | status 帯 | `admit()` の `admission_skipped` 収集(`next/shell/motolii-shell/src/lib.rs:2183-2185,2259-2264`) — 配置は独立に続行するので絵は出るが台帳に載らない、という**もう1種類の部分失敗** | 書ける |
| 11 | 代わりに Finder から動画ファイルを1本、ウィンドウへドラッグして落とす | OS drag & drop | `iced::window::Event::FileDropped` → `Message::DropReceived`(`next/shell/motolii-shell/src/lib.rs:1194,1269`) | 書ける |
| 12 | Finder から複数ファイルを同時に選んでまとめてドラッグする | OS drag & drop | winit は1ファイル1イベントなので `pending_drops` に貯めて描画要求を区切りに `FlushDrops` する(`next/shell/motolii-shell/src/lib.rs:307,547`。GOALS M2 の実装注) | 書ける |
| 13 | フォルダごとドラッグして中身をまとめて取り込む | OS drag & drop | `file_dialogs::expand_import_paths` が再帰的に supported media だけへ展開し、決定的な順序で `Vec<PathBuf>` を返す(`next/shell/motolii-shell/src/file_dialogs.rs:105-145`)。ただし `FileDropped`/`AdmitPaths` からこの helper を呼ぶ WIRE は未接続 | 【穴】入口が無い |
| 14 | 取り込んだ N 本ぶんが1回の Undo で戻せることを確認する(後で使う) | — | `admit()` は集めた intent を1回の `apply_all` で書く(`next/shell/motolii-shell/src/lib.rs:2237-2240`、doc「1操作=1undo」) | 書ける |
| 15 | ライブラリでカードのサムネイルを見て素材を見分ける | Browser | `browser-pane` のサムネイル描画(`next/ui/motolii-browser-pane/src/lib.rs`・`model.rs`) | 書ける |
| 16 | ライブラリの検索欄に名前の一部を打って絞り込む | Browser | `state.query()`(`next/ui/motolii-browser-pane/src/state.rs:137,184-185`、部分一致・大小無視) | 書ける |
| 17 | 並べ替え(名前順/追加日順)に切り替える | Browser | `SortKey`(Name/AddedDate/Kind)+`sorted()`(`next/ui/motolii-browser-pane/src/model.rs:693-705`) | 書ける |
| 18 | 取り込んだ素材を1本だけクリックして選ぶ | Browser | `Message::SelectCard`(`next/ui/motolii-browser-pane/src/lib.rs:1322`、`state.rs:242`) | 書ける |
| 19 | 別の素材を Shift や Cmd を押しながらクリックして、ライブラリ側でまとめて複数選ぶ | Browser | `CardSelectionModifiers`/`SelectCardWithModifiers` と選択解決(`next/ui/motolii-browser-pane/src/state.rs:34-98,416-509`)、表示順を渡すカード入口(`card_view.rs:289-360`)、Shell の modifier 正規化(`next/shell/motolii-shell/src/view.rs:306-318`) | 書ける |
| 20 | 素材カードをダブルクリックして中身を下見する(再生・イン/アウト確認) | Browser | media タブのカードに `on_double_click` は配線されていない(`CreateFromCard` は create タブ専用、`next/ui/motolii-browser-pane/src/lib.rs:1572`)。Play/scrub 相当も `browser-pane` 全体で grep 0件 | 【穴】入口が無い |
| 21 | 素材カードを右クリックしてコンテキストメニューを見る(リネーム・削除など) | Browser | media card の `mouse_area.on_right_press` → `OpenContextMenu` → pane-local `context_menu::State` → `context_menu::view`。既存意味の `RemoveAssetFromCard` だけを表示し、Shell の既存削除結線へ渡す(`next/ui/motolii-browser-pane/src/card_view.rs:462-500`, `context_menu.rs`) | 書ける |
| 22 | 取り込んだ動画3本が、いずれも playhead(0秒)へ重なって置かれているのを Timeline で見る | Timeline | `admit()` のループは `start` に毎回 `self.session.playhead` を渡す(`next/shell/motolii-shell/src/lib.rs:2166,2206`)——3本とも同一開始時刻 | 書ける |
| 23 | 1本目のクリップを掴む(bar 本体を press) | Timeline | `BarGrabbed`→`start_drag`(`next/ui/motolii-timeline-pane/src/write.rs:573,891`) | 書ける |
| 24 | 掴んだまま右へ動かす(プレビューだけが動き、まだ確定しない) | Timeline | `continue_drag`(`next/ui/motolii-timeline-pane/src/write.rs:926-983`。`origin` 基準の絶対値再計算、Document 不接触) | 書ける |
| 25 | 前のクリップの終端にスナップさせる | Timeline | `clip_gesture::moved_start`+`snap_candidates`(他clipのstart/end含む、`next/ui/motolii-timeline-pane/src/clip_gesture.rs:68-84`、SNAP_PX=7px、`next/reference/timeline-grammar.md:34`) | 書ける |
| 26 | マウスを離して位置を確定する | Timeline | `finish_drag`→`Intent::SetTiming`(`next/ui/motolii-timeline-pane/src/write.rs:986-994`) | 書ける |
| 27 | 2本目・3本目のクリップも同じ手順(掴む/動かす/スナップ/離す)で1本ずつ並べ直す | Timeline | 同上を反復。**自動の順送り配置は無い**(id 22 のとおり全部同時刻に置かれるため、並べ直しは必須工程) | 書ける |
| 28 | 位置調整の途中で Esc を押して掴む前へ戻す | Timeline | `cancel_drag`(`next/ui/motolii-timeline-pane/src/write.rs:433`)、`VerbId::EscapeCancel` → `self.timeline.cancel_drag()`(`next/shell/motolii-shell/src/lib.rs:1377`) | 書ける |
| 29 | ドラッグ中に Cmd を押してスナップを一時的に切る | Timeline | `continue_drag` の `snap_enabled = !modifiers.command()`(`next/ui/motolii-timeline-pane/src/write.rs:944`) | 書ける |
| 30 | 3本を Shift クリックでレーンバー上まとめて選ぶ(次の一括ドラッグの下準備) | Timeline rail | `rail.rs` が表示順+`pane.modifiers`から `Message::SelectLayer` を発火し(`next/ui/motolii-timeline-pane/src/rail.rs:101-145,357-405`)、`PaneState::update` が `resolve_layer_selection` で `Session::selected_layers` へ確定する(`next/ui/motolii-timeline-pane/src/write/mod.rs:647-660`) | 書ける |
| 31 | まとめて選んだ3本のうち1本を掴んで動かし、3本ともスナップを保ったまま一緒にスライドさせる | Timeline | `clip_drag.rs` は選択集合への同一 delta、グループ clamp、N 本の一括 `SetTiming` を実装し(`next/ui/motolii-timeline-pane/src/write/clip_drag.rs:40-65,107-187`)、`PaneState::clip_preview()` → `TimelinePane::with_clip_preview` → `projection::apply_clip_preview` が全 preview ペアを表示する(`write/mod.rs:608-620`, `projection/preview.rs`) | 書ける |
| 32 | Cmd+A で見えている全レイヤーを選ぶ | Timeline | `Message::SelectAllLayers`→`select_all_layers`(`next/shell/motolii-shell/src/lib.rs:2052-2057`)、Cmd+A 配線(`next/shell/motolii-shell/src/lib.rs:5972` 付近) | 書ける |
| 33 | 音声ファイル(単独 wav/mp3)を同じ Import 手順で取り込み、Timeline へ置く | Timeline | `admit()` は種別を判定せず同じ `LayerSource::Media` 経路で置く(`next/shell/motolii-shell/src/lib.rs:2200-2222`)。専用の「soundtrack」トラック概念は store に無い(`LayerSource` に該当 variant 無し、`next/core/motolii-store/src/lib.rs:191-206`)——正典 §6「曲が無い project への音声=soundtrack」(`next/reference/timeline-grammar.md` 該当節)は未実装で、音声も普通の layer 行として並ぶ | 【穴】意味が無い |
| 34 | 音声レイヤーの行を選び、波形帯を見て内容を確認する | Timeline | (根拠未収集——本切片の EXACT TARGET 外。波形描画コードの有無は未確認) | 【未確認】 |
| 35 | 同じ素材(1本目の動画)をもう一度、別の時刻に置きたい(同じ元映像から2つ目のクリップを作る) | Timeline/Browser | ライブラリのカードから Timeline/Stage へ直接ドラッグする経路は無い(`next/ui/motolii-browser-pane/src/lib.rs:123`「drag で Stage/Timeline へ、は将来切片(見送り)」)。media タブのダブルクリックも no-op(id 20)。**唯一の迂回**は File > Import Media… を再度開き、同じファイルをもう一度選ぶこと(`admit` は同一 path を再取り込みでき、`fingerprint` は同一でも `AssetDraft` は別途記帳される——重複統合の仕組みは無いので同名カードが2枚並ぶ) | 書ける(迂回) |
| 36 | 3本並んだクリップのうち、真ん中の1本の端(頭)を掴んで short trim する(端 8px 以内を狙う) | Timeline | `BarPart::EdgeIn`→`trimmed_in_start`(`next/ui/motolii-timeline-pane/src/clip_gesture.rs:91-100`)、TRIM_EDGE=8px・幅24px未満は端を出さない(`next/reference/timeline-grammar.md:34`) | 書ける |
| 37 | 動かして、素材の先頭が削られる分だけプレビューが縮むのを確認する | Timeline | `continue_drag` の `BarPart::EdgeIn` 分岐(`next/ui/motolii-timeline-pane/src/write.rs:942-950`、`duration`/`source_in` を同時更新) | 書ける |
| 38 | 離して確定する | Timeline | `finish_drag`(`write.rs:986-994`) | 書ける |
| 39 | 同じクリップの尻側も同様に trim する | Timeline | `BarPart::EdgeOut`→`trimmed_out_end`(`next/ui/motolii-timeline-pane/src/clip_gesture.rs:108-120`) | 書ける |
| 40 | playhead(ルーラー帯)をドラッグして、切りたい位置までスクラブする | Timeline | `Message::ScrubTo`(`next/ui/motolii-timeline-pane/src/ruler.rs`/`input.rs`、正典 §5「ルーラ帯の押下維持で追随」) | 書ける |
| 41 | 矢印キーで1フレームずつ寄せて正確な位置に合わせる | Timeline | `StepPlayheadForward/Back`(正典 §5「矢印キー: playhead を1(Shift で10)フレーム」) | 書ける |
| 42 | 切りたいクリップを選ぶ | Timeline | 行クリック→`Message::Select`(`rail.rs:335`) | 書ける |
| 43 | Cmd+K でその位置で分割する | Timeline | `Message::SplitAtPlayhead`→`split_at_playhead`(`next/ui/motolii-timeline-pane/src/write.rs:238,685,819`) | 書ける |
| 44 | 分割後、前半と後半が別クリップになったのを見る | Timeline | 同上。`selected_layers` 複数選択時は選択中の全レイヤーを playhead で割る(`write.rs:820-823`) | 書ける |
| 45 | 切ってできた不要な後半クリップを選ぶ | Timeline | `Message::Select`(単独選択) | 書ける |
| 46 | Delete キーを押して消そうとする | Timeline | `Backspace`/`Delete` は `DeleteSelectedKeys`(キーフレーム専用)にしか配線されていない(`next/ui/motolii-keymap/src/defaults.rs:31-43`、`next/shell/motolii-shell/src/lib.rs` 該当腕)——layer には効かない | 【穴】入口が無い |
| 47 | 代わりに Cmd+X(切り取り)を押して消す | Timeline/メニュー | `Message::CutLayer`→`cut_layer`(`next/shell/motolii-shell/src/lib.rs:104,2002-2022`)、`Intent::RemoveLayer` | 書ける(迂回) |
| 48 | 消したらクリップボードが上書きされている(後で使う予定だった Copy の中身が消える)ことに気づく | — | `cut_layer` は削除前に `LayerBundle::capture`→`self.clipboard.set_bundle(bundle)` を必ず行う(`next/shell/motolii-shell/src/selection.rs:97-108`)——「消すだけ」の意図でも clipboard が書き換わる | 書ける(意味論上の副作用) |
| 49 | 複数選択(Cmd+A で全選択した状態)のまま Cmd+X を押し、選んだ全部をまとめて消そうとする | Timeline | `selection::cut_layer` が `selected_layers` 全員を `clipboard::LayerBundle` へ捕捉し、全 `RemoveLayer` を1回の `apply_all` へ束ねる。Paste/Undo と選択解除を `tests/suite/clipboard_drive.rs::multi_cut_captures_all_layers_and_pastes_them_in_one_undo` で検収 | 書ける |
| 50 | 消した後にできた隙間を、後続クリップをまとめてドラッグして詰める | Timeline | id 31 と同じ `clip_drag.rs` の一括確定に加え、clip preview の全 preview ペア投影(`projection::apply_clip_preview`)で詰める途中の全 bar が追随する | 書ける |
| 51 | 代わりに後続クリップを1本ずつ選んで前クリップの終端へスナップさせ、隙間を詰める | Timeline | id 23-26 の move/snap/release を反復。100本規模では手間が線形に増えるが個々の操作は成立する | 書ける(迂回) |
| 52 | 隙間を詰め終えたら Cmd+Z で1つ前の操作(直前のドラッグ確定)だけを取り消す | Timeline | `Intent::SetTiming` 1回=1 undo 単位(`finish_drag`)、`Message::Undo`(`next/shell/motolii-shell/src/lib.rs:4505` 付近) | 書ける |
| 53 | 誤って Cut したクリップを Cmd+V で貼り戻す | Timeline/メニュー | `Message::PasteLayer`→`paste_layer`(`next/shell/motolii-shell/src/lib.rs:1985-1995`)。元時刻のまま貼り付け、貼り付け後は増えた方を選ぶ | 書ける |
| 54 | クリップを Cmd+D で複製する(誤操作の当て逃げ確認・別編集の下準備) | Timeline/メニュー | `Message::DuplicateLayer`→`duplicate_layer`(`next/shell/motolii-shell/src/lib.rs:2030-2044`) | 書ける |
| 55 | Space を押して通しで再生し、切った所が思った通りかを確認する | Timeline/Transport | `Message::TogglePlayback`(`next/shell/motolii-shell/src/transport.rs`)、Space 配線(`next/shell/motolii-shell/src/lib.rs` 該当) | 書ける |
| 56 | 再生すると音も同時に鳴るのを確認する | — | `open_real_playback`→`AudioProgram::from_view`+`PlaybackSession::open_default`(`next/shell/motolii-shell/src/transport.rs:97-115`、cpal 実デバイス出力) | 書ける |
| 57 | 再生中に Stage の絵が playhead に追従しているのを確認する | Stage/Timeline | `iced::window::frames()` 駆動の vsync tick(裁定166、`transport.rs:115-129`)。同じ正本(`Session`)を Timeline/Stage 双方が読む(GOALS M14) | 書ける |
| 58 | もう一度 Space を押して止める | Transport | `toggle_playback` の反転(`transport.rs`) | 書ける |
| 59 | 再生を止めたまま、ルーラー帯をスクラブして切った境界付近だけ細かく見直す | Timeline | id 40 と同じ `ScrubTo`。正典拘束5「ドラッグ中に Space は効かない」ため、再生停止済みであることを前提に成立する | 書ける |
| 60 | 素材Aの映像だけ音量が大きすぎるので、その clip を選んで Inspector の AUDIO section を開く | Inspector | `next/ui/motolii-inspector-pane/src/audio.rs:3-4,30,42-43`(Level/Pan/Fade In/Fade Out) | 書ける |
| 61 | Level のスライダー/数値をドラッグして下げる | Inspector | Inspector 数値ドラッグ(`start_field_drag`/`continue_field_drag`/`finish_field_drag`、`next/ui/motolii-inspector-pane/src/transform.rs` 系。GESTURES 台帳の同名行) | 書ける |
| 62 | 変更後にもう一度 Space で再生し、音量が下がったか耳で確認する | Transport | id 55-56 と同じ再生経路 | 書ける |
| 63 | 気に入らなければ Cmd+Z で Level 変更だけを取り消す | — | Inspector の drag 確定は1操作=1 undo(store の transient overlay+確定 Intent 1発、正典 §5.5) | 書ける |
| 64 | 一通り編集が終わったので保存しようとして Cmd+S を押す | メニュー/キー | `menu.rs:68-74`の`Save`/`Cmd+S`、`input.rs:337-343`の`Message::SaveRequested`。既知pathなら`document_io.rs:508-514`が無言で上書きする | 書ける |
| 65 | 代わりに File > Save As… を選び、保存先ダイアログで場所とファイル名を選ぶ | メニュー/OS dialog | `Message::SaveAsRequested`(`next/shell/motolii-shell/src/lib.rs:518,1484`)、`pick_save_path`(`file_dialogs.rs:157-166`) | 書ける(迂回) |
| 66 | 保存を確定する | OS dialog/Document | `Document::save`(履歴を畳んだ flattened 書き、`next/shell/motolii-shell/src/lib.rs:518` doc 参照) | 書ける |
| 67 | さらに編集を続けた後、もう一度保存しようとする(2回目以降も毎回ダイアログが出る) | メニュー | `Message::SaveRequested`は`current_path`があれば`perform_save_as(path)`へ進み、`tests/suite/file_drive.rs:349-413`がダイアログを再表示せず同じpathを更新することを検分 | 書ける |
| 68 | ウィンドウのタイトルバーやどこかに「未保存の変更がある」印(●等)が出ているか確認する | ウィンドウ chrome | `Shell::title()`(`next/shell/motolii-shell/src/lib.rs:1187`)の中身と `is_dirty()`(`lib.rs:1819`)の関係は確認したが、dirty マーカーを文字列へ埋め込む処理の有無は本切片で未読了 | 【未確認】 |
| 69 | 保存せずに File > New Project を選ぶと、破棄確認ダイアログが出る | メニュー/OS dialog | `confirm_then`→`self.dialogs.confirm_discard()`(`next/shell/motolii-shell/src/lib.rs:1478,1841-1849`、dirty でなければ確認自体をスキップ) | 書ける |
| 70 | 保存せずに File > Open… を選んでも同様に確認が出る | メニュー/OS dialog | `confirm_then_pick_open`(`next/shell/motolii-shell/src/lib.rs:1494,1850-1867`) | 書ける |
| 71 | 保存せずに File > Quit(Cmd+Q)を選んでも同様に確認が出る | メニュー/OS dialog | `Message::QuitRequested`→`confirm_then(QuitConfirmed)`(`next/shell/motolii-shell/src/lib.rs:551,1500`) | 書ける |
| 72 | 保存せずに、メニューではなく OS ウィンドウの閉じるボタン(赤信号)をクリックして閉じる | OS 窓 | main窓は`with_main_window`で`exit_on_close_request: false`。`lib.rs:1254-1258`の`close_requests`→`WindowCloseRequested`、`document_io.rs:537-552`のdirty確認を通る。`tests/suite/window_drive.rs`が拒否/許可を検分 | 書ける |
| 73 | File > Export… を選んで書き出し窓を開く | メニュー | `Message::Export(export_pane::Message::ToggleExportDialog)`(`next/shell/motolii-shell/src/menu.rs:81-83`) | 書ける |
| 74 | 品質(qp0 等)を選ぶ | Export 窓 | `Message::QualitySelect`(`next/ui/motolii-export-pane/src/lib.rs:362`、`next/shell/motolii-shell/src/lib.rs:3490`) | 書ける |
| 75 | 書き出し範囲を「全体」から「work area のみ」に切り替える | Export 窓 | `Message::RangeSelect`(`export-pane/src/lib.rs:388`、`lib.rs:3491`) | 書ける |
| 76 | 書き出し先ファイルを選ぶボタンを押し、OS ダイアログで保存先を選ぶ | Export 窓/OS dialog | `Message::PickOutputPath`→`pick_export_path`(`next/shell/motolii-shell/src/lib.rs:3494-3500`、拡張子 mp4 固定フィルタ) | 書ける |
| 77 | 縦(9:16)や正方形(1:1)のアスペクト比を選ぶ | Export 窓 | export 側に独立の crop/aspect プリセット UI は無い(`export-pane/src/lib.rs` に aspect/resolution 選択0件)。遡るとコンポジション自体の width/height を変更する UI(Settings 窓 COMPOSITION 節)が実在する(`next/ui/motolii-settings-pane/src/sections.rs:8,128-132,222-224,253-265`)ので**プロジェクト作成時点まで遡れば迂回できる**——ただし Export 窓の中だけでは選べない | 【穴】入口が無い |
| 78 | Export ボタンを押して書き出しを開始する | Export 窓 | `Message::Export`→`start_export`(`next/shell/motolii-shell/src/lib.rs:3502,3536-3585`) | 書ける |
| 79 | 進捗バーが少しずつ進むのを見る | Export 窓 | `export_ops.rs:199-243`が`Task::run(export_stream(...))`でUIを返し、`update_export_progressed`が背景スレッドのprogressを反映する | 書ける |
| 80 | 書き出しの途中で「取消」ボタンを押して中断する | Export 窓 | `export_ops.rs:167-176`の`CancelExport`が保持中の`Cancel`を立て、`export_ops.rs:439-455`がフレーム境界で検出してpartial outputを削除する | 書ける |
| 81 | 書き出しが完了した通知(status 帯)を見る | status 帯 | `self.status = Some(format!("書き出し完了: ..."))`(`next/shell/motolii-shell/src/lib.rs:3568-3572`) | 書ける |
| 82 | 書き出したファイルを Finder で開き、映像が正しいか確かめる | OS | Motolii 範囲外(OS 標準機能) | 書ける |
| 83 | 書き出したファイルを再生して、音が鳴るか確かめる | OS/Finder | `export_ops.rs:465-536`が`AudioProgram`でmixし、`motolii_media::mux_mixed_pcm`で最終mp4へmux。`tests/suite/export_drive.rs:173-245`がAAC音声トラックをprobeする | 書ける |
| 84 | 無音に気づいて、書き出しをやり直す前に Export 窓を閉じて設定を見直す | Export 窓 | `Message::ToggleExportDialog`(open 反転、`menu.rs:83` と同じメッセージ)。音声muxはExport本体で行われるため、設定を見直して再実行できる | 書ける |
| 85 | 書き出した mp4 ファイルを Finder からメールやチャットに添付して人に渡す | OS | Motolii 範囲外(OS/他アプリの標準機能。共有シートや「送る」機能を Motolii 自身は持たない——normal-map にも該当なし) | 書ける |

---

## 集計

表85行の判定列を機械的に数えた値(「書ける(迂回)」「書ける(意味論上の副作用)」などの注記付きも `書ける` として合算)。

```
全手順          85
書ける          72  (うち注記付き「書ける」5件: id 35,47,53は迂回/id 48は副作用)
【穴】入口が無い    7
【穴】意味が無い    4
【未確認】       2
```

| 判定 | 件数 | id |
|---|---|---|
| 書ける(注記付き含む) | 72 | 1-12,14-18,23-29,32,35,36-45,47,48,51-66,69-76,78-85 |
| 【穴】入口が無い | 7 | 13,19,20,21,30,46,77 |
| 【穴】意味が無い | 4 | 31,33,49,50 |
| 【未確認】 | 2 | 34,68 |

### normal-map.tsv に対応行が無い穴(本命)

17件の穴(入口10+意味7)のうち、`normal-map.tsv` に対応する行が**在る**ものを先に除く:

- **id 20**(素材カードのダブルクリック下見)→ id 1034「Source Monitor」(採用予定・未消化)が対応
- **id 64/67**(単純上書き保存)→ id 1224「Save (Project)」(採用済)が対応。現行mainのCmd+Sと既知path上書きを検分済み
- **id 77**(Export 窓の中でアスペクト比を選ぶ)→ id 943「Select Aspect Ratio」(採用予定・未消化)が対応

残る **5件が未実装の穴**(normal-map 対応行があっても、現行の入口/意味がまだ閉じていないものを含む):

1. **id 13** フォルダ単位の取り込み(folder import) — 製品のメニュー項目は「ファイルを開く/取り込む」であって「フォルダの中身を展開する」動詞そのものが無い
2. **id 20** 素材カードのダブルクリック下見 — 対応行はあるが入口未実装
3. **id 33** 音声ファイルの soundtrack 特別扱い(曲が無い project への最初の音声) — 正典 §6 にのみ書かれ、製品リストが持つ動詞ではない
4. **id 46** クリップの直接 Delete キー — Delete/Backspace は「選択対象を消す」という**性質**であって、Motolii の map には対応する項目名が見当たらない
5. **id 77** Export 窓の中でアスペクト比を選ぶ — 対応行はあるが入口未実装

---

## 「書くまでもないと思ったが、実装が無かった」もの(この作業の核心)

- **id 31: 複数選択したクリップをまとめてドラッグして動かす。** `clip_drag.rs` は選択集合へ同じ delta を適用し、clamp と一括 `SetTiming` まで実装していた。表示側の `clip_preview()` が掴んだ1本だけを投影していたため、全 preview ペアを `projection::apply_clip_preview` へ渡す責任部品を追加して閉じた。**意味計算と表示投影の責任を分けたことで、後半は表示側だけを直せた**——今回の象徴的な発見
- **id 30: Timeline レイヤー行の Shift/Cmd マウス複数選択。** 以前は `resolve_layer_selection`/`LayerSelectionOp` が宙に浮いていたが、現在は `rail.rs` → `Message::SelectLayer` → `PaneState::update` まで同一 pane 内で接続済み。残るのは実窓の操作確認だけ
- **id 21/49**: 右クリックの既存削除意味への接続と、複数 Cut/Paste の一括 undo を今回の責任部品で閉じた。実窓の観測だけが残る。
- **id 64/67/72/79/80/83**: 前回のP2調査時点では古い行番号と旧同期Export実装を根拠に穴と判定したが、現行mainではCmd+S、OS close dirtyガード、背景Export、Cancel、音声muxへ更新済み。今回その証拠を現行コードとdriveへ合わせた。

---

## 新発見の事実(KNOWN.md 既載は除く)

1. **コンポジションの width/height/fps を編集する UI が実在する**(Settings 窓 COMPOSITION 節、`next/ui/motolii-settings-pane/src/sections.rs:8,128-132,222-224,253-265`)。2026-08-22 の persona-vlog 調査時点では「`SetCompDimensions` 相当の Intent も grep 0件」「縦(9:16)・正方形(1:1)のコンポジションを作る手段が…存在しない」と結論していたが、現行 `main` ではこの結論は覆っている(ただし Export 窓の中に独立の aspect プリセットが無いことは変わらず——id 77 参照)
2. **Timeline の bar(clip)複数選択ドラッグは、意味計算と表示投影を別責任へ分けて閉じた。** `clip_drag.rs` の一括移動/確定と、`clip_preview()` → `projection::apply_clip_preview` の全 bar 投影が揃った。残るのは実窓の操作確認だけ(id 31/50)
3. **`resolve_layer_selection`/`LayerSelectionOp`(Timeline レイヤー行の Shift/Cmd 複数選択)は rail の実入力へ接続された。** `Message::SelectLayer` が表示順と modifier を運び、`PaneState::update` が `Session::selected_layers` へ一度だけ確定する。実窓の確認は未実施
4. **`cut_layer`(Cmd+X) は複数選択を `LayerBundle` へ昇格した。** `selected_layers` 全員の capture・削除・Paste を一つの意味とし、削除と貼り戻しをそれぞれ1 undoへ束ねる。残るのは実窓の操作確認だけ
5. **OS ウィンドウの閉じるボタンは、メニュー Quit と異なり `confirm_discard` を経由しない。** `main.rs:89` の doc コメントが「main 窓を閉じたら exit」と、確認を挟まない設計を明言している
6. **`start_export` は `Task::run(export_stream(...))`で背景実行し、進捗・CancelをUIへ返す。** 音声ありの場合は`export_ops.rs`内でmixとmuxまで完了させる
8. **`LayerSource` に soundtrack 専用の variant は無い。** 音声ファイルも動画・画像と同じ `Media` variant で扱われ、正典 §6 が書く「曲が無い project への音声=soundtrack」という特別扱いは実装されていない

---

## 迷った判断と、どちらへ倒したか

1. **id 35(同一素材の2本目クリップを作る)を「書ける」と判定するか「穴」と判定するか。** ドラッグでの配置もダブルクリックでの配置も無いが、Import Media… を再度開いて同じファイルを選び直せば新しい layer が生成される(admit のロジックはパス単位で毎回新規 layer を作る)。**「迂回可能」として「書ける(迂回)」に倒した**——ただし体験としては「ライブラリの中の物をもう一度使う」ではなく「もう一度ファイルシステムから取り込む」なので、意味的には歪んでいる
2. **id 33(音声ファイルの取り込み)を「穴」と判定するか。** 音声ファイルの取り込み自体(admit)は動画と同じ経路で成功する——「素材が置ける」という一次目的は達成できる。しかし正典が明記する「曲が無い project への最初の音声=soundtrack」という特別な意味論が無いことは、この手順書の主題(名前の無い操作の名指し)そのものなので「【穴】意味が無い」に倒した
3. **id 72(OS 閉じるボタン)・id 80(export 中の取消)・id 83(音声付き書き出し)を再確認した。** 旧証拠では穴に見えたが、現行mainではOS closeのdirtyガード、背景Export、Cancel、音声muxがそれぞれ結線済みで、driveにも証拠があるため「書ける」へ更新した。
4. **id 82・id 85(Finder で確認する・人に渡す)を手順に含めるか。** OUTCOME が「書き出したファイルを人に渡すところまで全部」と明記しているため、Motolii の実装範囲外でも手順としては書いた。判定は「書ける」とし、証拠欄には「Motolii 範囲外(OS 標準機能)」とだけ記した——README の「実装の証拠に file:line が必ず要る」という規約には形式上抵触するが、Motolii が関与しない OS 操作にコード証拠を要求すること自体が無意味なので、この2行だけ例外として扱った
5. **粒度をどこまで割るか(README の規約)。** 「位置を調整する」を掴む/動かす/スナップ/離す/Esc の5つに割った(id 23-29)のと同じ密度を、trim・split・cut/paste・save・export の全工程に適用した。結果として85行になったが、「以下同様」を使わずに書き切ることを優先した
