# egui Timeline 機能台帳 — 普通のタイムラインと照らして iced 側の漏れを機械的に潰す

作成日: 2026-08-19

状態: **台帳**(決定を含まない。読んで並べただけ。判断・優先順位付けは別文書の仕事)

対象: `crates/motolii-ui/src/timeline_editor/`(`mod.rs` 8,186行 + `audio_seat.rs` 286 + `import_seat.rs` 129 + `waveform_band.rs` 458 = 計9,059行)、`crates/motolii-ui/src/timeline_rows.rs`(405行)、`crates/motolii-ui/src/blitz_shell/pane.rs`・`app.rs`・`intent.rs` の Timeline 結線部分、`crates/motolii-doc/src/command/variant.rs`(D2 command 一覧)。比較対象は `crates/motolii-shell-iced/src/timeline/`(`mod.rs` 23 / `pane.rs` 441 / `canvas.rs` 947 / `semantics.rs` 743 / `waveform.rs` 208 = 計2,362行)+ `crates/motolii-shell-iced/src/shortcuts.rs`(189行)。

背景: [icedホスト移行裁定](2026-08-18-iced-host-migration-decision.md)でホストを egui → iced へ絞め殺し方式(M-0〜M-5)で移す方針が決まっている。移植の漏れ検出がこれまで supervisor の目視に依存していたため、egui 側が「何をしていたか」を利用者の1操作単位で先に台帳化し、iced 側の有無と機械的に突き合わせる。

## 0. 読み方

- 行の単位は**利用者から見た1操作**(「clip の端を掴んで trim」「行を畳む」)。関数単位では切っていない
- 「状態の持ち主」列は `session`(Project session が持つ、Document には入らない)/ `Document`(D2 が持つ、Undo/redo と保存の対象)/ `session→Document`(session 上の draft を経て確定時に Document へ書く)の3通り
- 「iced 側の現状」列は **有 / 部分 / 無** の3値。判定はコードを読んで行った(推測していない)。部分は何が欠けているかを同じセルに1行で書く
- oracle 列の `mod.rs` テストは `crates/motolii-ui/src/timeline_editor/mod.rs` の `#[cfg(test)] mod tests`(5570–8186行、78件)を指す。**このレーンではテスト名だけを確認し、本文(アサーションの詳細)までは読んでいない行が大半**(§6「読めなかった/未確認」に明記)。`timeline_rows.rs` / `audio_seat.rs` / `waveform_band.rs` のテストは本文まで読んだ。iced 側の oracle は `crates/motolii-shell-iced/tests/drive_timeline.rs`(875行、25件)を指し、名前一覧は §6 に列挙した(本文までは読んでいない)

## 1. 能力台帳

### 1.1 再生・playhead・ループ

| # | 能力(1操作) | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 1 | 再生 / 一時停止 | `Space` キー(mod.rs:3573,3586) / transport 帯のボタン(3418) | session(`playing`, `playhead`, `audio`) | なし | soundtrack があれば実device再生・無ければ壁時計。開けなければ**明示的に**壁時計へ落ち理由を status に出す(audio_seat.rs) | `wall_clock_fallback_advances_and_stops_at_the_end`, `playback_keeps_the_playhead_centred_and_stops_scrolling_at_both_ends`, `a_long_frame_does_not_teleport_the_playhead` | **無**(再生機構自体が無い。`shortcuts.rs` が「Q0: 機構が無いものは近道として書かない」として明記) |
| 2 | 先頭へ戻す | transport 帯のボタン(mod.rs:3414) | session(`playhead=0`) | なし | — | — | **無**(ボタン自体が無い。transport帯は表示専用と `canvas.rs` に明記) |
| 3 | ループ ON/OFF | `L` キー(3574,3595) / surface メニュー "Loop to selection"/"Clear loop" | session(`loop_region.on/start/end`) | なし | 帯は引いたまま効きだけ切れる、引き直さずに戻せる | `playback_wraps_when_it_reaches_the_loop_end_not_when_it_starts_outside` | **無** |
| 4 | ループ帯をドラッグ(新規/移動/端伸縮) | ルーラ上端10pxの `loop_hit` の `drag_started/dragged/drag_stopped`(3706–3760) | session(`hold=Loop`, `loop_region`, `view`) | なし | 端判定は`LOOP_GRAB`=8pxで甘め(外すと区間が消える)。反対側の端は掴んだ瞬間の値で固定、追い越しても畳まれない | `a_loop_drag_reads_the_same_in_either_direction`, `the_ends_of_a_loop_are_grabbable_and_a_near_miss_is_not_a_new_region`, `dragging_an_end_past_the_other_keeps_the_region_anchored`, `dragging_to_the_edge_scrolls_and_the_middle_does_not` | **無** |
| 5 | ルーラでplayheadをスクラブ | `scrub.is_pointer_button_down_on()`(3953–3954) | session(`playhead`, `playing=false`) | なし(**Document は触らない**) | 掴んだら再生停止、フレームに乗る、端まで運べば窓が追随(edge pan) | (暗黙。playback系テスト群が前提) | **有**(`ScrubStarted`→`PointerMoved`→`PointerReleased`→`UiIntent::SetPlayhead`)。**egui は毎フレーム session を直接書くのに対し iced は release一回のintentで確定**(§4-③) |
| 6 | playheadを1コマ送る(←/→、Shiftで10コマ) | 矢印キー、text focus中は0(5064–5075) | session(`playhead`) | なし | 左右同時押しも0(値を発明しない) | `the_arrow_keys_resolve_to_one_frame_and_shift_makes_it_ten`, `a_focused_text_field_keeps_the_arrow_keys_to_itself`, `stepping_frames_moves_only_the_playhead_and_lands_on_a_frame`, `stepping_frames_stops_at_both_ends_of_the_composition` | **有**(`PlayheadStepped`→`UiIntent::StepPlayhead`) |
| 7 | ロケータをクリックしてジャンプ | pin の `r.clicked()`(3862) | session(`playhead`) | なし | 押したら跳ぶ(ロケータの本体は「そこへ行くこと」) | (暗黙) | **無**(ロケータ機構自体が無い) |
| 8 | ナビゲータ帯(下端)でパン/ズーム | `navigator()` 内の drag(3045–3135)。中央=pan、両端6px=片側固定ズーム | session(`view`, `hold=Nav`) | なし | 窓は composition の外に出ない | `dragging_to_the_edge_scrolls_and_the_middle_does_not`, `the_bands_are_nailed_to_time_not_to_the_window` | **部分**:iced にも ARRANGEMENT 帯があり、押す/引きずると `OverviewSeek` でその時刻へ view を中心寄せできる(`canvas.rs:170-171,200-212`)。**端を掴んでの片側固定ズーム(span変更)は無い** — 中央送りだけ |

### 1.2 選択

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 9 | 行(左レール)をクリックして選択(素=置換、Cmd=足し引き、Shift=範囲) | rail の `r.clicked()`(4116) | session(`selected`) | なし | Shift範囲は**見えている object 行の上で**数える | `shift_click_selects_the_range_between_two_rows`, `rows_and_keys_share_one_selection_rule` | **有**(`RowPicked`→`UiIntent::SelectLayer`) |
| 10 | clip bar本体をクリックして選択 | bar の `r.clicked()`(4505)。ドラッグ開始とは同じ response から分岐 | session(`selected`) | なし | 同上 | 同上 | **有**(`BarGrabbed{zone:Body}`→`press_selection_intents`)。**egui は常に選び直すが、iced は既に選択済みなら plain click で何もしない**(複数選択を保ったままドラッグへ入るための意図的差異。`pane.rs:419-441` に明記) |
| 11 | キーフレーム(菱形)をクリックして選択 | 菱形の `r.clicked()`(4601)→`select_key` | session(`selected_keys`) | なし | 行と同じ3規則。Deleteの対象がここで決まる | `rows_and_keys_share_one_selection_rule`, `a_range_of_keys_follows_what_is_on_screen` | **無**(キー選択UI自体が無い) |
| 12 | 矩形(marquee)選択 | 空面の `surface_bg.drag_started/dragged/drag_stopped`(4050,4055,4665) | session(`hold=Marquee`, `selected`, `selected_keys`) | なし | 行に掛かるだけでは選ばない(時間方向もbarと重なる必要)。掃いた中のキーも拾う | (専用テスト名は確認できず。§6) | **無** |
| 13 | 何も無い所をクリックして選択解除 | `surface_bg.clicked()`(4044) | session(`selected`,`selected_keys` clear) | なし | — | (暗黙) | **有**(`EmptyPressed`→`UiIntent::ClearSelection`) |
| 14 | 全選択(Cmd+A) | (5007,5077–5082) | session(`selected`=見えているobject行) | なし | 閉じたGroupの中は見えていない=選ばれない | `command_a_selects_every_visible_object_row`(drive_timeline.rs) | **有**(`SelectAllPressed`)。**iced は egui より丁寧**: 既に選択済みの行へは additive intent を出さない filter があり、`SelectLayer{additive:true}` がトグルである副作用で「もう一度押すと外れる」バグを構造的に避けている(`pane.rs:247-263` に明記) |

### 1.3 clip編集(move / trim / split / duplicate / delete / group / reorder)

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 15 | clip本体をドラッグして時間移動(複数選択・Groupは子ごと・自身のキーごと追従) | bar の `drag_started(Body)/dragged/drag_stopped`(4510–4552) | session(`hold`)→Document | `prepare_set_clip_start` (+ 追従キー分の `SetPositionKeyTime`/`SetTransformParamKeyTime`) | **塊のまま動く**: 誰か1人でも端を越えたら全員そこで止まる。Groupは子clipをまとめて動かす。キーは重なり回避のため出す順が要る | `dragging_a_clip_moves_it_in_the_document_and_undo_puts_it_back`, `dragging_a_group_moves_every_child_by_the_same_delta`, `moving_a_clip_carries_its_position_keys`, `moving_a_clip_carries_scale_keys_too_not_just_position`, `moving_a_group_carries_the_groups_own_keys_too`, `moving_one_of_several_selected_clips_moves_them_all`, `selecting_a_group_and_its_child_moves_the_child_once` | **有**(`BarGrabbed{Body}`→preview→`PointerReleased`→`UiIntent::MoveClips`。経路は egui と同じ `begin_selected_clips_move`/`drag_to`/`release` を共有engine越しに呼ぶ)。**ただしロック中のUI挙動が違う — 危険候補#1、§4-①参照** |
| 16 | clip端をドラッグしてtrim(In/Out) | bar端の `drag_started(Edge)/dragged/drag_stopped` | session(`hold`)→Document | `prepare_trim_clip_in` / `prepare_trim_clip_out` | Groupと細いbar(24px未満)は端を持たない。端の判定幅は egui=6px(`TRIM_EDGE`相当)、iced=8px(`semantics.rs::TRIM_EDGE`) — 実装差はあるが機能差ではない | `a_group_bar_has_no_trim_edges`, `a_thin_bar_keeps_a_body_to_grab`, `trim_preview_is_clamped_to_a_frame_and_the_composition`(iced) | **有**(`BarGrabbed{Edge}`→`UiIntent::TrimClip`) |
| 17 | ドラッグ中に候補へ吸着(clip端・キー・playhead・ループ端・0・終端)、Altで解除 | 同上ドラッグ中 | session(`snap`) | なし(位置計算のみ) | 間合いは画面距離7px、フレーム丸めより吸着が優先 | `dragging_snaps_to_nearby_edges_and_keys_but_not_to_itself`, `snapping_uses_screen_distance`(iced) | **部分**: `semantics.rs::snap_candidates` はclip端・playhead・0・終端のみ。**コード自身が「ループ帯とキー行はこの pane にまだ無いので候補に出ない」と明記**。Alt解除トグルも無い |
| 18 | ドラッグ中にEscでキャンセル | `Escape`(5002,5012) | session(`hold` clear) | 書いていた分だけ`writer.undo()` | 何も書いていなければundo呼ばない(無関係な編集を巻き込まない) | `escape_during_a_drag_restores_the_original_start` | **有**(`GestureCancelled`)。**意味論が違う**: egui は毎フレームDocumentへ書いてからEscでundoする「live-commit」、iced はrelease まで一切書かない「preview-then-commit」で、Escはpreviewを捨てるだけ(`pane.rs`冒頭のモジュールdocが明記) |
| 19 | 選択レイヤーを複製 | `Cmd+D`(5003,5022) / row menu "Duplicate" | Document | `prepare_duplicate_track_item` | 1複製=1Undo。複数なら増えたほうを選ぶ | `duplicating_a_group_copies_its_children_with_fresh_ids`, `duplicating_two_selected_layers_makes_two_and_selects_them` | **無**(shortcuts.rsに「担当外」と明記) |
| 20 | 選択を削除(Delete/Backspace、Groupは中身ごと) | `Delete`/`Backspace`(5004,5029) | Document | `prepare_remove_track_item` | キー選択があればキーが先。ドラッグ中は効かない | `deleting_a_group_takes_its_children_and_one_undo_puts_them_back`, `deleting_two_selected_layers_is_one_undo` | **有**(`DeletePressed`→`UiIntent::DeleteSelection`)。iced はキー選択UIが無いので常に層削除(egui の「キー優先」分岐は該当なし) |
| 21 | playheadで分割 | `Cmd+K`(5006,5039) / row menu "Split at playhead" | Document | `prepare_split_clip` | Groupは切れない(断りであって失敗ではない)。端で切れない場合も黙って飛ばす | `splitting_at_the_playhead_makes_two_clips` | **無**(`UiIntent`自体が存在しない。`shortcuts.rs`が「D2には在るがiced側のUiIntentに無い」と明記) |
| 22 | 選択をひとつのGroupにまとめる | `Cmd+G`(5005,5034) / row menu "Group" | Document | `prepare_add_group` + `prepare_reparent_clip`×N(1 gesture) | 親が揃っていない選択は断る。位置はいちばん上のものの場所。まとめたら中を開いて見せる | `grouping_two_layers_puts_them_under_one_new_group`, `grouping_refuses_a_selection_that_spans_parents` | **無** |
| 23 | 行をドラッグして並べ替え/開いたGroupへ出し入れ | rail の `r.drag_started/dragged/drag_stopped`(4120,4123、境界判定は`drop_target`) | session(`hold=Reorder`,`drop`)→Document | `prepare_reparent_clip(new_start: None)` | 落とし先は「境界」(行と行のあいだ)で決まる。自分自身の中へは落とせない。同じ親内で下へ動かすとindexを1つ引く | `dropping_a_row_above_another_reorders_the_document`, `dropping_a_row_at_the_end_lands_after_the_last_one`, `dropping_a_row_into_an_open_group_reparents_it`, `a_group_cannot_be_dropped_inside_itself` | **無**(iced の `BarGrabbed` は時間移動のみ。rail 側にドラッグ入口が無い) |

### 1.4 キーフレーム編集

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 24 | playheadへキーを打つ | row menu "Add key at playhead ▸"(param別) / Inspectorの◇ | Document | `prepare_add_position_key` / `prepare_add_transform_param_key` | 既に同時刻にキーがあれば選ぶだけ(書かない) | `adding_a_key_at_the_playhead_puts_one_key_and_selects_it` | **部分**: Timeline発の入口(右クリックメニュー)は無いが、**Inspector経由の入口(◇ボタン)は`UiIntent::KeyParamAtPlayhead`として存在**(shell.rs:332)。Timeline上でキーの有無を見ながら打つことはできない |
| 25 | キーをドラッグして時刻を変える(全パラメータ対応) | 菱形の `drag_started/dragged/drag_stopped`(4606–4636) | session(`hold=KeyTime`)→Document | `prepare_set_position_key_time` / `prepare_set_transform_param_key_time` | 押した場所(数px動く前)を起点に採る。**Position縛りは意図的に外してある**(D2にSetTransformParamKeyTimeが入った時点で理由が消えた) | `dragging_a_position_key_changes_only_that_key`, `a_scale_key_can_be_dragged_like_a_position_one` | **無** |
| 26 | キーの補間(イージング)を変える | key menu "Easing ▸"(Hold/Linear/Ease in-out) | Document | `prepare_set_position_key_interp` | **入口があるのはPositionだけ**(他はD2に無いので"席") | (暗黙) | **無** |
| 27 | キーを削除(選択がキーのとき優先) | `Delete`/`Backspace`(selected_keys非空時) / key menu "Delete key" | Document | `prepare_remove_position_key` / `prepare_remove_transform_param_key` | 1回=1Undo | `delete_removes_the_selected_keys_and_leaves_the_layer` | **無** |
| 28 | キー行の開閉(◇/◆) | rail_glyphクリック(4258–4267) | session(`fold.params_open`) | なし | 子の開閉と独立(2軸) | `param_rows_and_child_rows_open_independently`(timeline_rows.rs) | **無**(iced は常に `TimelineFoldState::default()` を使う=全閉じ固定。トグル自体が無い) |

### 1.5 行の構造(fold / mute-solo-lock / rename / color / 表示設定)

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 29 | Group子の開閉(▸/▾) | rail_glyphクリック(4186–4195) | session(`fold.children_open`) | なし | 畳んでも子の開閉状態は保持、開き直すと復元 | `group_closed_hides_children_but_keeps_the_group_row`, `group_open_puts_children_directly_after_at_depth_plus_one`, `reopening_a_group_restores_each_child_fold_state`(timeline_rows.rs) | **無**(同上、常に全閉じ) |
| 30 | Mute切り替え | rail button "M" / row menu / Inspector M | Document | `prepare_set_item_visible`(反転) | 押下状態の正本はDocument。Timeline行とInspectorが**同じ入口**(`toggle_item_flag`)を共有 | `muting_a_layer_writes_through_to_the_document`, `the_three_flags_write_through_and_read_back` | **有**(`FlagPressed{Mute}`→`UiIntent::ToggleItemFlag`) |
| 31 | Solo切り替え | rail button "S" 他同上 | Document | `prepare_set_item_solo` | 同上 | 同上 | **有**(`FlagPressed{Solo}`) |
| 32 | Lock切り替え | rail button "L" / row menu | Document | `prepare_set_item_lock` | 親から継承したロックは自分では外せない(inherited、薄く表示のみ) | `locking_a_group_locks_everything_inside_it` | **無**。`UiItemFlag`にLockが無く、`intent.rs`に「Lockは Timeline の行だけの操作なので wire に載せない」と明記 |
| 33 | ロック中は掴めない(move/trim/reorder/rename共通の拒否) | 各drag_started直前に`self.is_locked()`を読む | Document(読むだけ) | なし(拒否のみ) | 断ったら理由をstatus/rejectionsへ | `a_locked_layer_keeps_its_place_and_its_keys`, `a_locked_layer_cannot_be_renamed` | **部分 — 危険候補#1。§4-①参照**。Document書き込みは共有engine(`begin_selected_clips_move`/`begin_trim`)がロックを見て拒否するため実害(保存されたDocumentが壊れること)は無い。**しかしiced の `hit_test`/`mouse_interaction`はロックを一切見ないため、掴んだ瞬間のカーソル・ドラッグpreviewは非ロック時と同じに動き、releaseで無言のまま元の位置へ戻る** |
| 34 | レイヤー名を変更(ロック中は不可) | row menu "Rename…" / `Enter`(単一選択時、5051–5058) | session(`renaming` draft)→Document | `prepare_set_layer_name` | 空名は拒否。確定するまでDocumentは触らない。同名打ち直しは失敗ではない | `renaming_a_layer_only_moves_the_ledger_entry`, `an_empty_name_is_refused_and_keeps_the_editor_open`, `a_locked_layer_cannot_be_renamed` | **無** |
| 35 | 行の色を設定/id由来へ戻す | row menu "Colour ▸"(8色スウォッチ+"Default") | Document | `prepare_set_item_color` | 選択が複数なら全部に付く。選ばれていなければidから導出した色(複製で変わる) | `a_layer_colour_is_derived_until_it_is_chosen` | **無** |
| 36 | 色表示のon/off | surface menu "Layer colours on/off" | session(`colors_on`) | なし(**Documentは触らない**) | オフ=白/灰統一(好みの表示切替、Workspace profileが本来の持ち主) | `turning_colours_off_does_not_touch_the_document` | **無** |
| 37 | 行の高さ切り替え(Small/Large) | surface menu "Row height ▸" | session(`large_rows`) | なし | 意味は変わらない、見やすさだけ | (暗黙) | **無** |
| 38 | Fit to composition(viewを全体に合わせる) | surface menu "Fit to composition" | session(`view`) | なし | — | (暗黙) | **無**(専用コマンドは無い。初期viewはcompへclampはされる) |

### 1.6 ロケータ(マーカー)

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 39 | playheadへロケータをタップで打つ | `M`キー(no Cmd, 5044–5049) | Document | `prepare_add_locator` | **再生中でも止めずに打てる**。同一フレームへの連打は1つに畳む | `tapping_m_while_stopped_drops_a_named_locator_without_entering_rename`, `tapping_m_while_playing_adds_a_locator_and_keeps_playing`, `double_tapping_the_same_frame_folds_into_one_locator` | **無**(`shortcuts.rs`に「D2には在るがUiIntentが無い」と明記) |
| 40 | 右クリックでその時刻にロケータを追加 | surface menu "Add locator here" | Document | `prepare_add_locator` | 右クリックした瞬間の時刻(`context_time`)を使う。置いた直後から編集できる(renameへ入る) | `a_locator_is_placed_at_the_playhead_and_empty_text_removes_it` | **無** |
| 41 | ロケータをドラッグして時刻を変える | pin の `drag_started/dragged/drag_stopped`(3867–3890) | session(`hold=Locator`)→Document | `prepare_set_locator_time` | 掴んだ瞬間に1つgestureを採る(毎フレーム開き直さない=1ドラッグ1Undo) | `dragging_a_locator_is_one_undo_step` | **無** |
| 42 | ロケータのメモ文を編集(空なら削除) | inline TextEdit + `lost_focus`/`Enter`/`Escape`(3917–3939) | session(`editing_locator` draft)→Document | `prepare_set_locator_text` / `prepare_remove_locator` | 空なら消す。確定するまでDocumentは触らない | `a_locator_is_placed_at_the_playhead_and_empty_text_removes_it` | **無** |
| 43 | ロケータを削除(右クリックメニュー) | locator `context_menu` "Remove locator"(3891–3899) | Document | `prepare_remove_locator` | — | (暗黙) | **無** |

### 1.7 Undo / Redo・素材取り込み・波形帯・状態表示

| # | 能力 | 入口 | 状態の持ち主 | D2 command | 意味関数/不変量 | oracle | iced側 |
|---|---|---|---|---|---|---|---|
| 44 | Undo | `Cmd+Z`(5000,5014) / status帯ボタン | Document(`writer.undo`) | — | 1ドラッグ=1Undo単位。ボタンとキーは同じ入口(`undo_gesture`) | (複数テストが前提とする共通基盤) | **有**(`UndoPressed`→`UiIntent::Undo`) |
| 45 | Redo | `Shift+Cmd+Z`(5001,5017) / status帯ボタン | Document(`writer.redo`) | — | 同上 | 同上 | **有**(`RedoPressed`→`UiIntent::Redo`) |
| 46 | OSドロップ/Browserダブルクリックで取り込み・playheadへ配置(音声で曲未設定ならsoundtrack化) | `dropped_files` / Browserカードダブルクリック → `UiIntent::AdmitPaths`(1点合流) | Document | `prepare_admit_asset`+`apply_prepared_asset_admission`+`prepare_place_asset_clip`(1 gesture、`import_seat.rs`) | 音声で曲が未設定ならsoundtrackになる(offset0/gain1.0)。1本ずつ処理し理由付きでskipできる | `a_project_without_a_soundtrack_stays_on_the_wall_clock`(audio_seat.rs)。専用取り込みテストは`import_seat.rs`本文に無く、`blitz_shell`側の結合テストが持つ(未読) | **有**(両ホスト共有の `import_and_place` を`AdmitPaths`経由でそのまま呼ぶ。Timeline固有実装ではなくshell共通) |
| 47 | soundtrack波形帯の表示(decode/縮約/段mip) | 自動(soundtrack有無で高さ0↔44px) | session(`WaveformSeat`: id/pending/peaks/failure) | なし(読み取りのみ) | 別threadでdecode、フレームは止めない。O(見えているpx)。同じファイルは二度decodeしない(再生キャッシュと共有) | `every_level_keeps_the_overall_peak`, `the_chosen_level_keeps_at_most_a_few_buckets_per_column`, `a_document_without_a_soundtrack_never_starts_a_build`(waveform_band.rs) | **有**。`WaveformPeaks`/`WaveformWindow`はegui版の`pub`型をそのまま共用(移植ではなく共用)。相違は「いつpollするか」だけ — egui:描画関数内で毎フレーム、iced:`Shell::update`(Message経由)+`window::frames()`購読 |
| 48 | 拒否理由のstatus表示・控え | 自動(各種拒否時に`reject()`が呼ばれる) | session(`status`,`rejections`) | なし | **同じ理由の連続は言わない**(latch防止)。`take_rejections()`で引き取られたら空になる | `a_refused_edit_is_handed_back_not_only_painted`, `a_writer_rejection_is_handed_back_too`, `the_same_refusal_in_a_row_is_said_once` | **無 — 危険候補#2、§4-②参照**。`crates/motolii-shell-iced/src/**/*.rs` に `take_rejections` の呼び出しは1件も無い(grep 0件)。`shell.rs` の Timeline メッセージ処理(192–206行)は `self.gateway.dispatch(intent)` の戻り値(成功/失敗bool)を毎回 `let _ =` で捨てている(他のintentも同型パターン) |

**合計48行。iced側「無」= 28件、「部分」= 4件(#8,17,24,33)、「有」= 16件。**

## 2. 非実装の席(egui にも無いプレースホルダ)

以下は egui 側のメニューに項目としては存在するが、`fn seat(ui, label)`(mod.rs:705–710)で描かれるだけの**押しても何も起きないプレースホルダ**である。iced 側の欠如と同列に数えると誤カウントになるため、台帳から分離する。

| 場所 | 項目 |
|---|---|
| row menu | Cut ⌘X / Copy ⌘C / Paste ⌘V / Reveal source |
| key menu | Copy key ⌘C / Set value… / Snap to playhead(Position以外のEasingも同様に"席") |
| surface menu | Paste ⌘V / New layer… / Zoom to loop |

また `mod.rs` 3805–3807, 4030–4037 に宣言されている `locator_clicked` / `locator_rename` / `rename_started` のようなローカル変数は**宣言・消費(3905–3912等)はあるが、行ループ内で `Some(...)` を代入する箇所が無い**(grepで確認、0件)。実際にロケータのリネームへ入る唯一の経路は「右クリック→Add locator here」直後の自動オープンであり、既存ロケータをクリックしてリネームへ入る経路は egui 版にも存在しない。モジュール冒頭のdoc(1行目付近)が「ダブルクリックは使わない」と明記しているのと整合するため、iced側の欠落としては数えない(egui側の死んだ変数として観察のみ記録)。

## 3. 接続(この Timeline が他の面と何をやり取りするか)

利用者の「そこから接続も導ける」を、実際に読んだ配線で具体化する。

1. **選択 → Inspector**: `BlitzPane::show_live_inspector`(pane.rs:336–362)が毎フレーム `editor.selected_layers()` を読み、選択がちょうど1件のときだけ Inspector にその layer を映す。Inspector が出した編集要求(`InspectorAction`)は `apply_inspector_action`(pane.rs:89–131)で `editor.begin_param_edit`/`set_param_component`/`end_param_edit`/`key_param_at_playhead`/`toggle_item_flag`/`set_effect_enabled` へ1対1で写る — **Inspector はDocumentを書かない、書くのは常にTimelineが抱える唯一の`DocumentWriter`**。iced側も `UiIntent::BeginParamEdit`/`SetParamComponent`/`EndParamEdit`/`KeyParamAtPlayhead` が同じ`TimelineEditor`APIへ1対1で落ちる(intent.rsで確認)。

2. **playhead → Stage**: `BlitzShellBehavior::pane_ui`(app.rs:79–83)が毎フレーム `pane.set_live_playhead(editor.playhead_seconds())` を呼び、Stageはその時刻の合成フレームを描く。**時刻の正本はTimeline(writerと同じ席)、Stageは読んで渡すだけ**。iced側のStage実装は本レーンの読む対象に入っておらず配線の詳細は未確認(§6)。

3. **Stage → Timeline選択**: `StagePane::show`が返す選択(entity path)は `stage_entity`(pane.rs:149–154)で `StageEntity::Layer(LayerId)` / `NotALayer` に写され、`PaneRequest::SelectLayer`→`editor.select_layer(layer)`(app.rs:88–91)としてTimelineの選択へ合流する。**Stage側では選択を持たない**(選択を持つのはエディタ1つだけ)。

4. **drop → Browser**: OSドロップ(`ctx.input(|i| i.raw.dropped_files)`)とBrowserカードのダブルクリック(`BrowserRequest::PlaceFile`)は両方とも `UiIntent::AdmitPaths` という**同一の合流点**に集約される(app.rs:423–435,652–655、intent.rs:1002で`editor_mut().import_dropped_media(paths)`を呼ぶ)。Browser自体はDocumentを1バイトも書かない(`browser_panel/mod.rs`のdocコメント)。

5. **audio → soundtrack**: `import_seat::import_and_place`が「音声で、まだ曲(`Document.soundtrack`)が無ければsoundtrackとして貼る」既定を持つ(CapCut/Ableton型)。以後の再生(能力#1)は`audio_seat::open_playback`が`document.soundtrack`の有無で実device再生/壁時計を切り替え、波形帯(能力#47)は同じsoundtrackのPCMを`pcm_caches`(`(content_hash, ordinal)`キー)で**再生と共有**し、同じファイルを二度decodeしない。

6. **M/S/Lock は Timeline行とInspectorの共有入口**: `toggle_item_flag`(mod.rs:1589)がTimeline行のボタンとInspectorのM/Sボタン両方から呼ばれる唯一の入口。**Lockだけは`UiEditParam`/`UiItemFlag`に載っておらずInspector側にも露出していない**(intent.rsのコメントが「Lockは Timeline の行だけの操作」と明記)。

7. **Undo/Redoは面をまたぐ**: `TimelineEditor`が単一の`DocumentWriter`を抱えるため(single writer)、Cmd+Zは出所(Timelineのドラッグ、Inspectorの数値編集、Browserからの取り込み)を問わず同じ1つのUndoスタックへ積まれる。

## 4. 暗黙の不変量(壊してはいけない約束)

コードを読んで分かった、iced移植で踏み外しやすい規約を優先順に並べる。

**① ロックは「D2/共有engineでは効くが、iced のUI層には存在しない」— 二層のうち下だけが生きている。**
`crates/motolii-doc/src/command/clip.rs`の`prepare_set_clip_start`/`prepare_trim_clip_in`/`prepare_trim_clip_out`を読んだが、**`envelope.lock`を検査する行は無い**(grep確認)。ロック拒否は`TimelineEditor::begin_selected_clips_move`/`begin_trim`(mod.rs:1499,1515)という**エディタAPI層**でのみ行われる。iced はこの同じAPIを`UiIntent::MoveClips`/`TrimClip`のdispatch経由(intent.rs:561–578)で呼ぶため、**Document保存の安全性(ロックされたclipが実際に動いてしまうこと)は無い**。しかし`crates/motolii-shell-iced/src/timeline/semantics.rs::hit_test`と`canvas.rs::mouse_interaction`はロック状態を一切読まない(`is_locked`/`effective_lock`相当の呼び出しがゼロ)。結果、iced側では**ロックされたclipを掴むとegui同様に滑らかにドラッグpreviewが動き、離した瞬間にだけ黙って元の位置へ戻る**(§1.5 #33)。

**② 拒否理由がiced側では発生源から先へ届いていない。**
`self.reject()`(mod.rs:1324)が積む`self.rejections`は`take_rejections()`(1339)で引き取られて初めて`--status-log`や帯へ出る設計だが、`crates/motolii-shell-iced/`配下に`take_rejections`の呼び出しは無い(grep 0件)。加えて`shell.rs`のTimelineメッセージ処理(192–206行)を含め、確認した`self.gateway.dispatch(intent)`の呼び出し箇所はことごとく戻り値(bool)を`let _ =`で捨てている(shell.rs:146–368付近で確認した限りほぼ全数)。**ロック拒否に限らず、iced側でDocumentへの書き込みが拒否された場合、利用者には現状「何も起きなかった(ように見える)」以上の情報が渡っていない可能性が高い**(§1.7 #48)。

**③ ドラッグの「確定境界」がegui/icedで違う設計。**
egui は毎`dragged`フレームごとに`commit_drag`/`commit_drag_snapped`(1880,1871)がDocumentへ書き、Escでは書いた分だけ`writer.undo()`する「live-commit」。iced は`pane.rs`冒頭のモジュールdocが明記する通り、release(`PointerReleased`)の1件だけがintentになり、ドラッグ中は`TimelineDrag`という**session内のpreview**だけが動く「preview-then-commit」。**Escの意味がDocumentレベルで違う**(egui=undo実行、iced=まだ何も書いていないのでpreview破棄のみ)。この差はTimeline側のSkia実装(`timeline_move_gesture`/`timeline_trim_gesture`、本レーンでは未読)が先に確立した設計とiced側が同じ判断を採ったことによる(pane.rsのdocに明記)。

**④ 単一writer: TimelineEditorがDocumentの唯一の書き手。**
`pub struct TimelineEditor { writer: DocumentWriter, document: Arc<Document>, ... }`(mod.rs:1134–1139)。他の面(Inspector/Stage/Browser)はこのエディタのAPIを経由してのみDocumentへ触れる。iced側も`UiIntent`のdispatch先は結局この同じ`TimelineEditor`(`ShellGateway`内に1つ)であり、二重writerは無い。

**⑤ 1 gesture = 1 GestureId = 1 Undo。**
`Hold`列挙(230–245行)のdocが明記する通り、掴んだ瞬間に一度だけ`writer.begin_gesture()`を呼び、離すまで同じ`GestureId`を使い回す。毎フレーム開き直すとフレーム数だけUndoが積まれる、という以前の不具合の修正としてこの規約がある。iced側もintent構築時(release一発でintentが確定する設計)によりこの粒度が自然に保たれている。

**⑥ メニューは行を回している最中にDocumentを触らない。**
`MenuAction`は行ループの間`out: &mut Option<MenuAction>`へ積むだけで、実際の適用(`run_menu`)はループを抜けてから1回だけ呼ばれる(2951–2952行のdoc「行を回し終えてから呼ぶ」)。木が変わると持っている位置(index等)が全部ずれるため。

**⑦ session ⇔ Document の境界は明確に線引きされている。**
zoom/pan/縦scroll/選択/hold/loop_region/playing/snap/large_rows/colors_on/renaming-draft/editing_locator-draft/status/rejections/fold は**すべてsession**(struct定義のコメントで明示、mod.rs:1137–1205)。レイヤー名・色・lock/mute/solo・位置・キー・ロケータ・soundtrack・compositionはDocument。ユーザーメモリの言う「この区別がreplayとundoの境界そのもの」はコード上も一貫している。

**⑧ 複数選択の移動は塊のまま止まる(端の共有クランプ)。**
`commit_drag`のMove腕(mod.rs:1892–1929付近)、iced版`clamped_move_delta`(semantics.rs:372–385)ともに、選択の誰か1人でも0または終端を越えるなら**全員そこで止まる**(置き去りにしない)。両実装で同じテスト意図(`a_selection_stops_together_at_the_left_edge`)が存在する。

**⑨ 波形帯のdecodeはフレームを止めない。**
`WaveformSeat::poll`はUI側から毎フレーム呼ばれるが、decodeと縮約は別threadに出し、届くまでは薄いplaceholderを返す。この構造は egui/iced で型ごと共用されている(`waveform_band.rs`の`pub`型をiced版がそのままuse)。

**⑩ ダブルクリックは使わない(設計上の意図的欠落)。**
mod.rs冒頭のモジュールdoc(21–23行)が「選択・並べ替え・跳ぶ が同じ場所に重なっている面では、2回目の押下が別の操作の途中と区別できない」と明記。これによりロケータの「既存ピンをダブルクリックしてrenameへ入る」経路が egui 版にも実装されていない(§2で観察した死んだ変数の理由)。iced側でこの欠落を「補う」実装をしないよう注意。

## 5. 危険候補(supervisorが見落としやすい順、上位5件)

1. **ロック中clipのドラッグが iced では無警告で動いて見え、release後に無言で消える**(§4-①、§1.5 #33)。D2/共有engineレベルでは安全だが、Q0(触れそうで触れない物は不合格)に対する**構造的な違反**であり、しかも「触れそうに見えて実は触れない」ではなく「**触れて動いたように見えたのに何も残らない**」という、より発見しづらい形。`canvas.rs`自身が「カーソルがtrimの形なのに掴むと移動する、が構造的に起きない」とhit_test統一の効能を謳っているだけに、ロック軸だけがこの統一から漏れているのは見落としやすい。
2. **拒否理由がiced側のどこにも表示されない**(§4-②、§1.7 #48)。`take_rejections`呼び出し0件、`dispatch`戻り値も`let _ =`で握りつぶし。Timelineに限らずInspector経由の拒否(BeginParamEdit等)も同型で影響を受ける可能性が高く、影響範囲がTimeline単体より広い。
3. **fold(行の畳み)状態がiced側に一切無い**(§1.4 #28, §1.5 #29)。`TimelineFoldState::default()`が固定で使われており、**Groupは常に閉じたまま**。ユーザーメモリ「zoom/pan/畳み/選択はsession」の3本柱のうち畳みだけがiced側に未着手であり、Cmd+Aの「見えている行だけ選ぶ」など他の機能の挙動("見えている"の定義)にも波及する。
4. **split/duplicate/group/locator/rename/color/keyframe編集がまるごと"無"** — 個別の欠落というより、**構造操作系(D2に既にあるがUiIntentに無い)とキー編集系が丸ごと1レーン分残っている**という規模の見落としやすさ。`shortcuts.rs`が意図的な先送りとして明記しているため「知らなかった」ことにはならないが、48行中28行(58%)が無というのは、supervisorが個別レビューで拾うには数が多すぎる。
5. **snap候補がclip端/playhead/0/終端のみで、キーとループ端が抜けている**(§1.3 #17)。egui版のsnap_candidatesは5種類(clip端・**キー**・playhead・**ループ端**・0・終端)を持つのに対し、iced版は`semantics.rs`が自ら「まだ無いので候補に出ない」と認めている3種類。将来キー編集やループUIをiced側に足したとき、**そのタイミングでsnap_candidatesを一緒に拡張し忘れる**のが典型的な漏れパターンになる(いま無いから見えないだけで、後から機能を足す側が「吸着も自動でついてくる」と誤解しやすい)。

## 6. 読めなかった/未確認

- `mod.rs`の`#[cfg(test)] mod tests`(5570–8186行、78件)は**テスト名(fn一覧)のみ確認**。本文(具体的なアサーション)まで読んだのは一部で、大半は名前からの推定に留まる。テスト名は fn一覧の付録(§7)に全数を残した
- `crates/motolii-shell-iced/tests/drive_timeline.rs`(875行、25件)も同様に**テスト名のみ**確認(下記に列挙)。本文は未読
- iced側の Stage pane 実装(`crates/motolii-shell-iced/src/` 内、Stageのplayhead/選択の受け渡し詳細)は本レーンの読む対象外だったため、§3-②③の記述は egui 側(pane.rs/app.rs)からの確認に基づく。iced側で同じ配線が成立しているかは未確認
- `crates/motolii-shell-iced/src/shell.rs`は全文を読んでいない(Timelineメッセージ処理まわり190–370行付近を中心に確認)。`let _ =`パターン以外の拒否表示経路が同ファイルの別の場所に存在する可能性は完全には排除できない
- `timeline_move_gesture.rs`/`timeline_trim_gesture.rs`(Skia側、pane.rsのdocコメントが「transient lifecycleをiced側と同じ判断で先に実証した」と触れている実装)は読む対象に入っておらず未読
- `crates/motolii-doc/src/command/apply.rs`・`clip.rs`はロック検査の有無を確認する目的でのみ部分的に読んだ(grep+該当関数2本)。他のprepare_*関数がロックを検査するかどうかは網羅していない
- `import_seat.rs`から呼ばれるCLI側の対応実装(`crates/motolii-cli/src/document_edit.rs:59-107`、import_seat.rsのdocコメントが参照)は未読
- マーキー選択(#12)専用のoracleテスト名は`grep`で見つけられなかった(`marquee`/`rectangle`等のキーワードで再検索すればmod.rs内に存在する可能性があるが、このレーンでは特定できなかった)

### iced側 drive_timeline.rs のテスト名一覧(本文未読)

`clicking_a_bar_selects_the_layer_through_the_gateway`, `dragging_a_body_moves_the_clip_and_lands_one_undo_unit`, `dragging_the_right_edge_trims_the_out_point`, `dragging_the_left_edge_trims_the_in_point`, `escape_during_a_drag_restores_without_a_trace`, `delete_removes_the_selection_and_undo_brings_it_back`, `a_multi_selection_moves_as_a_block_and_stops_at_zero`, `scrubbing_the_ruler_seeks_the_playhead`, `arrow_keys_step_the_playhead_by_frames`, `command_a_selects_every_visible_object_row`, `command_wheel_zooms_around_the_cursor`, `shift_wheel_pans_horizontally_inside_the_composition`, `a_timeline_session_replays_from_its_intent_log`, `the_same_message_sequence_reproduces_the_view_state`, `a_dropped_soundtrack_grows_a_waveform_band`。

## 7. 付録: 入口の生一覧

拾い漏れを防ぐため、`mod.rs`から機械的に抽出した生の一覧(表に畳む前の材料)。

- **`fn`一覧(mod.rs)**: 241件(struct/implメソッド・自由関数・入れ子fn・テストfn込み)。実装本体(1–5569行)に163件、テスト本体(5570–8186行)に78件
- **`egui::Key::`**: 17箇所(Space, L, Escape×2, Enter×2, Z(undo/redo), D(duplicate), Delete/Backspace, G(group), K(split), M(locator), ArrowLeft/ArrowRight, A(select all))
- **`.clicked()` / `.secondary_clicked()`**: 34箇所(rail_button内の`pressed()`経由分含む)
- **`.drag_started()` / `.dragged()` / `.drag_stopped()` / `.is_pointer_button_down_on()`**: 30箇所
- **`.hovered()` / `.context_menu(`**: 21箇所

主要な入口(egui `Response`/`InputState`ベース)の対応先モジュール別内訳:
- 行rail(選択・M/S/L・fold・rename・reorder): mod.rs 4110–4309
- clip bar(選択・move・trim): mod.rs 4374–4553
- keyframe diamond(選択・drag): mod.rs 4555–4660
- ループ帯: mod.rs 3706–3760
- ロケータpin: mod.rs 3815–3900
- ルーラscrub: mod.rs 3953–3954
- 空面(marquee・右クリック・選択解除): mod.rs 4040–4074
- 縦scrollbar: mod.rs 4809–4847
- ナビゲータ帯: mod.rs 3045–3135
- transport帯ボタン: mod.rs 3373–3418
- キーボードショートカット一括読み取り: mod.rs 4998–5009, 5044, 5052, 5064

---

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 追記 — 再計測(2026-08-19 夕、tip=e0f739fa)

キー編集・再生・構造操作・拒否理由テスト固定の着地後に48行を静的読解で再判定した。

**集計: 有 32 / 部分 2 / 無 14**(制定時: 有16 / 部分4 / 無28)。無→有14件、部分→有2件。

危険候補4件の現状:
1. ロック中 clip の嘘 → **解消**(canvas.rs:1119 で NotAllowed カーソル、pane.rs の plan/note が preview 自体を開始しない)
2. 拒否理由が届かない → **解消**。制定時の「crate 内に take_rejections 呼び出し0件」は
   **crate スコープの狭い観測からの誤った一般化**だった — 共有ゲートウェイ(intent.rs の
   edit/with_editor)内で report まで完結しており、iced は latest_report()/view.rs で描画する。
   drive_rejections.rs(11テスト)が実描画まで審判
3. fold 既定が全閉じ → **解消**(#28/#29 実装、描画は実 fold 状態を使う)
4. snap 候補にループ端・キーが無い → **未解消**(semantics.rs:489 のコメントごと現役)

**再計測が見つけた新しい不整合**: `pane.rs:394` の `SelectAllPressed` だけが
`TimelineFoldState::default()`(全閉じ)を固定で使い、実 fold 状態(canvas.rs:94)と食い違う。
開いた Group がある状態の Cmd+A で選び過ぎ/漏れの可能性(要実機)。

残る「無」14件: ループ帯drag / ロケータ機構全般(5件) / marquee選択 / split / 行reorder /
行色設定 / 色表示on-off / 行高さ切替 / Fit to composition。
