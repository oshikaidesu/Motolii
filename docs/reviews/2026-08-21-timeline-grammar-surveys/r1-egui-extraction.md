# egui版 Timeline 操作文法 — コードからの意味抽出(R1)

裁定144-a の実施。対象:
`crates/motolii-ui/src/timeline_editor/mod.rs`(8,566行)+ `audio_seat.rs` / `import_seat.rs` / `waveform_band.rs` / `playback.rs`、
および同梱の周辺ジェスチャ機構 `crates/motolii-ui/src/timeline_move_gesture.rs` / `timeline_trim_gesture.rs`。

パスはすべて `crates/motolii-ui/src/` からの相対。行番号はこのworktree(`session-handoff-day2-e1fbd3`)時点。

**backend注記(先に読む)**: この実装が話す相手は `motolii_doc::DocumentWriter`(旧D2、`GestureId` + `apply_command` + 明示的 `undo()/redo()` スタック)であり、`next/` の rerun-store Document ではない。次の3点は**この抽出のUI文法には含めない**(next/裁定2「undoはeditタイムラインの時間移動」・裁定47/48・裁定118(b)「同一at刻みでのbatch」と非互換な旧機構だから):
- `GestureId` を握って `apply_command` を呼ぶ経路そのもの
- `writer.undo()/redo()` の明示スタック(`undo_len()`/`redo_len()`)
- ドラッグ中に**都度絶対値で出し直す**(毎フレーム `prepare_set_clip_start` 等を呼び直す)実装

台帳へ持ち越すのは「**1操作=1確定単位**」という粒度の意味(=1ドラッグ=1 undo 相当)だけで、実現機構(GestureId)は next 側の「同一 `at` へのbatch→検証失敗ならdrop_time_range」に置き換わる前提。

---

## 定数(px・秒の一次資料)

| 定数 | 値 | 意味 | 出典 |
|---|---|---|---|
| `RAIL_W` | 196.0px | 左レール(名前列)幅 | mod.rs:128 |
| `ROW_H` / `ROW_H_LARGE` | 24.0 / 34.0px | object行の高さ(小/大) | mod.rs:130,133 |
| `PROP_H` / `PROP_H_LARGE` | 20.0 / 26.0px | パラメータ行の高さ(小/大) | mod.rs:131,134 |
| `HEAD_H` | 34.0px | ヘッダ(transport)帯高 | mod.rs:135 |
| `RULER_H` | 36.0px | ルーラ帯高(ループ+locator+目盛を含む) | mod.rs:136 |
| `LOOP_H` | 10.0px | ループ帯(ルーラ最上段) | mod.rs:202 |
| `LOCATOR_H` | 13.0px | locator帯(ループ帯の下) | mod.rs:204 |
| `MIN_SPAN` | 0.25秒 | 最大ズーム(1秒を4分割まで) | mod.rs:140 |
| `NAV_H` | 14.0px | 下端ナビゲータ帯高 | mod.rs:142 |
| `SCROLLBAR_W` | 6.0px | 縦スクロールバー幅 | mod.rs:144 |
| `LOOP_GRAB` | 8.0px | ループ帯の端を掴む許容 | mod.rs:208 |
| `EDGE_PAN` | 28.0px | 端パンが働き始める縁からの距離 | mod.rs:210 |
| `EDGE_PAN_RATE` | 0.8 | 端パンの速度係数 | mod.rs:212 |
| `TRIM_EDGE` | 8.0px | bar端のトリム掴み幅(見た目は7px、掴みは8px) | mod.rs:956 |
| `SNAP_PX` | 7.0px | 吸着の画面距離しきい値 | mod.rs:5412 |
| キー菱形の掴み的 | 12×12px(中心からのRect) | ダイヤ本体は8×8(半径4px)、当たりはそれより大きい | mod.rs:4680-4681 |
| fold三角(▸/▾) / params(◇/◆) / M・S・L | 16×16px各 | 全部 `Rect::from_center_size(_, Vec2::splat(16.0))` | mod.rs:4434-4448,4506-4519,4526-4534 |

ドラッグ開始しきい値そのもの(何px動いたらドラッグ扱いか)は **egui既定に依存し、このファイルでは上書きしていない**。ただし押した瞬間の座標(`press_origin`)を意図的に使う箇所が複数あり(下記M-1/K-1参照)、「eguiの report が来る時点では既に数px動いている」ことをコード自身が明記している(mod.rs:4766-4770 のコメント)。

---

## 操作一覧(30件)

### M-1. クリップ move(単一/複数選択の本体ドラッグ)
- **起動条件**: bar本体(`BarPart::Body` — 端8px以外)を左ボタンでdrag_started。Groupのbarは常にBody扱い(端を持たない、mod.rs:1010-1013)。ロック中は拒否してstatusへ理由(mod.rs:4764-4766)。押した時点で未選択なら単独選択に差し替えてから移動集合を作る(mod.rs:4779-4783)。
- **ドラッグ中の意味**: `commit_drag_snapped`→`commit_drag`。掴んだ瞬間の値を基準に**絶対値で出し直す**(delta方式ではない、原文コメント「掴んだ瞬間の値を持ったままにする」mod.rs:1004)。複数選択は`begin_move_many`が選択集合の"root"(祖先が選択に含まれないもの)だけを動かす対象にし、子孫のclipをLayerIdで畳んで重複を消す(mod.rs:1037-1042,2469-2485)。**Group移動時は子のPositionキーも同じdeltaで追従**(Position/Anchor/Scale/Rotation/Opacityの5paramすべて、mod.rs:1073-1080)。**塊制約**: 選択集合のうち誰か1人でも0秒/終端を越えるなら全員がそこで止まる(clamp、mod.rs:2153-2163)。閾値: なし(押した瞬間からGrabを持つ)。スナップ: `snap_candidates`(0秒・終端・playhead・ループ両端・他clipのstart/end・全キー時刻)に`SNAP_PX=7px`の画面距離で吸着、Altで無効化(mod.rs:3824,3397-3448)。
- **確定/キャンセル**: マウスアップ(`drag_stopped`)で`hold=None`(値は既にdrag中に書き込み済み、確定という別ステップはない)。Esc押下中は`cancel_drag`が`undo_base`まで`writer.undo()`を1回呼び戻す(掴んだだけで未移動ならundoしない、mod.rs:2284-2308)。
- **フィードバック**: カーソル`Grabbing`(mod.rs:867)。ドラッグ中のbarはACCENT色に変わる(mod.rs:4655)。選択bar枠は2pxのSELECTED枠(mod.rs:4667-4673)。掴んでいる間ポインタ近くにタイムコードのミニラベルを出す(mod.rs:5228-5253)。
- **エッジケース**: 空groupの移動は"nothing to move (empty group)"でreturn(mod.rs:2146-2149)。キー追従は同時刻衝突を避けるため移動方向で順序をソートしてから出す(delta>=0なら遅いキーから、mod.rs:2185-2192)。親子同時選択は子が二重にカウントされない(`selection_roots`、mod.rs:2469-2485、テストmod.rs:6472-6500で「0.3sだけ動く(0.6sではない)」を確認)。
- **出典**: mod.rs:981-991(Grab::Move型), 1037-1080(begin_move_many), 2134-2283(commit_drag), 4764-4820(bar drag_started/dragged), テストmod.rs:6430-6471,6472-6500

### M-2. トリム左端(TrimIn / 入り点)
- **起動条件**: bar左端`TRIM_EDGE=8px`以内(`classify_bar_edge`)。Group bar・幅`TRIM_EDGE*3=24px`未満のbarは端を差し出さない=常にBody(mod.rs:970-980)。判定は**押した瞬間の座標**(`press_origin`)で行う——egui方式では drag_started の報告時点で既にポインタが数px動いているため、報告時点の座標で判定すると「右端を掴んで左へ引いた」ケースがBodyに誤分類される、という明記されたバグ修正(mod.rs:4766-4770)。
- **ドラッグ中の意味**: `Grab::TrimIn{layer}`→`commit_drag`→`writer.prepare_trim_clip_in`。絶対時刻指定(delta方式ではない)。スナップはM-1と同じ`snapped()`を通る。
- **確定/キャンセル**: M-1と同じ(drag_stoppedで終了、Escでcancel_drag)。
- **フィードバック**: カーソル`ResizeHorizontal`(mod.rs:865)。hover時またはselected時のみ端バンドを白半透明でハイライト、掴んでいる端は`SELECTED`色(mod.rs:4674-4699)。
- **エッジケース**: ロック中はdrag_startedで拒否(mod.rs:4764-4766)。
- **出典**: mod.rs:934-980(BarPart/classify_bar_edge), 943-956(TrimEdge enum), 2222-2229(commit_drag内のTrimIn), 4681-4700(端の視覚化)

### M-3. トリム右端(TrimOut / 出し点)
- M-2と対称(`prepare_trim_clip_out`)。**trim familyは実装されていない** — ripple/roll/slip/slideに相当する挙動・型は存在しない。TrimIn/TrimOutはそれぞれ単独clipのstart/endのみを動かし、隣接clipや後続clip群への連動は一切ない。既決(AE型自由配置・trim family不採用)と**一致**。
- **出典**: mod.rs:2230-2237

### M-4. キー(菱形)ドラッグ — 時刻移動
- **起動条件**: 菱形中心から12×12pxの矩形(`Rect::from_center_size(c, Vec2::splat(12.0))`)、bar端と同じ寸法思想。**Positionに限らずAnchor/Scale/Rotation/Opacityの全paramで掴める**(旧仕様はPosition限定だったが「D2にSetTransformParamKeyTimeが入った時点で理由は消えた」と明記、mod.rs:4805-4808)。ロック中は拒否。押した瞬間の座標を起点にする(bar端と同じ理由)。
- **ドラッグ中の意味**: `Grab::KeyTime{layer,param,key,grab_at,original}`。絶対値で出し直し、0秒〜終端でclamp(mod.rs:2192-2199)。Position以外は専用入口ではなく`prepare_set_transform_param_key_time`を通す共通口(`key_time_command`、mod.rs:2104-2120)。スナップ・端パンはM-1と同じ`Surface`/`snapped()`を共有。
- **確定/キャンセル**: M-1と同じ。
- **フィードバック**: 選択中/dragged/hovered時にACCENT、それ以外KEY_IDLE。菱形本体は8×8(半径4px)、白1pxストローク(mod.rs:4859-4874)。カーソルGrab(ロック中NotAllowed)。
- **エッジケース**: **時刻を動かせる補間の"入口"はPositionのみ**という制約は別物として残る——`set_key_interp`(イージング変更)はPosition以外だと"interp is Position-only in D2"で拒否(mod.rs:2971-2975)。**時刻移動自体は5param全部で可能**という非対称に注意(移動口は統合済み、イージング口は未統合)。
- **出典**: mod.rs:4790-4880, 2104-2120, テストmod.rs:5989-6028(dragging_a_position_key_changes_only_that_key)

### M-5. 行の並べ替え(左レール上下ドラッグ)
- **起動条件**: 左レール(`rail`、RAIL_W幅)のどこを押してもその行を掴む。1回のクリックで選択も兼ねる(未選択なら選び直してから開始)。
- **ドラッグ中の意味**: **境界(boundary)判定** — `boundary_at`が行の中心y(`top+h*0.5`)を境に「その行の上」を返す。落とし先は**行ではなく行と行のあいだ**(`drop_target`)。開いたGroupの先頭子行の上へ落とせば「Groupの中の先頭」になる。自分自身の子孫の中へは落とせない(`is_descendant`チェック、mod.rs:791-794)。同じ親内で下へ動かす場合は`index`を1引く(外した後の位置で数えるため)。**Documentは離した瞬間にしか書かない** — ドラッグ中は境界線を絵で見せるだけ(mod.rs:1077-1080コメント、4979-5009)。
- **確定/キャンセル**: `reorder_released`(drag_stopped)で`commit_reorder`が1回だけ`prepare_reparent_clip(layer, to.parent, to.index, None)`(`new_start=None`=時刻は変えない、mod.rs:2975-2988)。Escでの取り消しは**このGrabにも効く**(`hold_cursor`のGrab::Reorder分岐がGrabbingを返す=Item扱いなので、Escでcancel_dragの対象——ただしReorderはドラッグ中Document未変更なのでundo対象は無い)。
- **フィードバック**: カーソルGrab(触れているとき)/Grabbing(掴んでいるとき)。落とし先にACCENT色2px線(mod.rs:4993-5000)。
- **エッジケース**: **Groupを自分の子孫の中へは落とせない**(木が壊れるため、drop_target内でNone)。末尾境界は「最後の行の次」を指す特別枠。テストで確認: `dropping_a_row_above_another_reorders_the_document`(時刻は変わらない)、`dropping_a_row_into_an_open_group_reparents_it`、`a_group_cannot_be_dropped_inside_itself`(境界1〜3は拒否、境界0と末尾は許可)。
- **出典**: mod.rs:395-401(DropTarget), 724-741(boundary_at/boundary_y), 777-799(drop_target), 981-991(Grab::Reorder), 4753-4763(pick/reorder_started/released捕捉), 4979-5009(反映), 2975-2988(commit_reorder), テストmod.rs:6536-6631

### M-6. Marquee(矩形)選択
- **起動条件**: 何もない面(`surface_bg`)の左ボタンdrag_started。**行より先にヒットテスト登録されるので、行の上ではこの矩形選択は発火しない**(コメント「行より先に登録する」mod.rs:4292)。
- **ドラッグ中の意味**: `Hold::Marquee{from,to}`をフレームごとに更新。矩形はDocumentを書かない(session状態のみ)。
- **確定/キャンセル**: drag_stoppedで矩形と各行の(y帯 × bar区間)の交差を判定——**行に掛かるだけでは選ばない、時間方向もbarと重なっている必要がある**(空の時間帯を囲んだだけで全選択されるのを防ぐ、mod.rs:4930-4935コメント)。同時にキー行も掃く(菱形のx座標が矩形内)。**キーが掃かれたときは行選択でキー選択をクリアしない**(swept_keysフラグ、mod.rs:4964-4971)。
- **フィードバック**: カーソルCrosshair。半透明ACCENT塗り+1px枠(mod.rs:5150-5162)。
- **エッジケース**: 何もない面の単クリック(dragなし)は選択を全クリア(mod.rs:4302-4306)。
- **出典**: mod.rs:233-247(Hold::Marquee), 4294-4310(drag_started/dragged), 4917-4972(finalize)

### M-7. 行クリック選択(単/Cmd/Shift)
- **起動条件**: 左レールまたはbar本体クリック(ドラッグでなくclicked)。
- **意味**: `select_click`共通関数——素クリック=単独選択、Cmd=足し引き(トグル)、Shift=**直前に触った行(anchor=selected.last())からここまで**を、画面に見えている順(`order`=object_layers、閉じたGroup内は含まれない)で数える。anchorは選択リストの末尾に残るので、続けてShiftを押しても基準は動かない。
- **確定**: クリック即座に反映、Undo対象ではない(selection自体はsession状態)。
- **エッジケース**: `order`に無いもの(畳まれて非表示)は範囲に入らない=「見えているとおりに採れる」。右クリックは未選択行なら先に単独選択してからメニューを開く(選択とメニュー対象の食い違いを防ぐ、mod.rs:4759-4763,4890-4892)。
- **出典**: mod.rs:888-921(select_click), 2440-2452(TimelineEditor::select), テストmod.rs:6501-6535

### M-8. キークリック選択(単/Cmd/Shift)
- M-7と同一規則を`select_key`で適用、並び順は`key_order`(行順→行内は時刻順)。Deleteの対象を決める。
- **出典**: mod.rs:2453-2468, 4864-4868

### M-9. 右クリックコンテキストメニュー(行/Group)
- **起動条件**: 行(rail)のsecondary_clicked。未選択なら先に単独選択。
- **内容**(mod.rs:2990-3117): Group(⌘G、複数選択時は"Group N layers")/ Duplicate(⌘D)/ Delete(⌫)/ Split at playhead(⌘K)/ Add key at playhead▸(5param submenu)/ Mute・Solo・Lock toggle(M/S/L)/ Show keys(◇)/ Expand children(▾、子持ちのみ)/ 席(未実装: Cut/Copy/Paste)/ Colour▸(8色パレット+Default)/ Rename…(⏎)/ 席(Reveal source)。
- **意味の原則**: メニュー内ではDocumentを触らない——行を回している最中に木が変わると位置がずれるため、`MenuAction`として貯めて行ループの外で1つだけ`run_menu`実行(mod.rs:560-570コメント)。
- **出典**: mod.rs:2990-3117, 3206-3298(run_menu)

### M-10. 右クリックコンテキストメニュー(キー)
- **内容**(mod.rs:3118-3164): Delete key(⌫)/ Easing▸(**Positionのみ**: Hold/Linear/Ease in-out、他paramは席)/ 席(Copy key/Set value…/Snap to playhead)。
- **出典**: mod.rs:3118-3164

### M-11. 右クリックコンテキストメニュー(何もない面)
- **起動条件**: `surface_bg.secondary_clicked()`。**押した瞬間の時刻を`context_time`として控える**(メニュー表示中にポインタが動いても位置がずれない、mod.rs:4318-4324)。
- **内容**(mod.rs:3165-3205): Fit to composition/ Loop to selection(L)/ Clear loop(loop_on時のみ)/ Add locator here/ Layer colours on/off/ Row height▸(Small/Large)/ 席(Paste/New layer/Zoom to loop)。
- **出典**: mod.rs:3165-3205, 4320-4324

### M-12. Group子の開閉(▸/▾三角)
- **起動条件**: 子を持つ行のみ表示。16×16pxヒット、インデント+2pxの位置。
- **意味**: `TimelineFoldState`(`fold.open_children`/`close_children`)——描画は`timeline_rows::rows()`が毎フレーム実Documentから作るので、この窓の見た目=製品の行モデルの挙動そのもの(モジュール冒頭コメント)。
- **出典**: mod.rs:4434-4448

### M-13. パラメータ行の開閉(◇/◆)
- **起動条件**: キーを持つ行のみ(`visible_params`が非空)。位置はrail右端-66pxの16×16pxヒット。子の開閉とは独立のトグル。
- **出典**: mod.rs:4506-4519

### M-14. M(mute)/S(solo)/L(lock)ボタン
- **起動条件**: rail右端-48px起点、18px間隔で3ボタン(16×16px each)。
- **意味**: 押下状態はDocumentから読む(ボタン側に状態を持たない)。`toggle_flag`が反転して1クリック=1書き込み。**継承ロック**は自分がlockしていなくても親から効いていれば薄い背景色(LOCK_INHERITED)で示すが、**自分では外せない**ため点灯はさせない(mod.rs:4531-4536コメント)。
- **エッジケース**: 親から継承されたlockがかかっている行への解除操作は`toggle_flag_gesture`で拒否・status表示(mod.rs:2384-2394)。
- **出典**: mod.rs:4520-4550, 2397-2416

### M-15. 名前のリネーム
- **起動条件**: 行メニューの"Rename…(⏎)"、または1つだけ選択中に Enter キー。ロック中は拒否。
- **意味**: その場(name_rect)がインライン `TextEdit` になる(別ウィンドウを出さない)。Enter確定/Escキャンセル(`lost_focus`+キー判定)。
- **確定/キャンセル**: 空名は拒否(status「name cannot be empty」)。同じ名前を打ち直した場合は書き込みなし=失敗ではない。
- **出典**: mod.rs:2728-2761, 4467-4491

### M-16. Locator追加(右クリックメニュー)
- **意味**: `add_locator`——既定名"Locator N"(連番)、置いた直後から即編集状態にする(`editing_locator`)。
- **出典**: mod.rs:2621-2642

### M-17. Locator追加(Mキータップ)
- **起動条件**: `M`キー(Cmdなし)、リネーム中/locator編集中でないとき。**再生中でも止めずに打てる**。
- **意味**: playhead位置にフレーム丸め後追加。**同一フレームへの連打は1つに畳む**(既に同フレームにlocatorがあれば何もせずundo台帳にも積まない、mod.rs:2650-2665コメント+実装)。名前入力には入らない(タップ用途、曲を聴きながら連打する想定)。
- **出典**: mod.rs:2643-2666, 5298-5304

### M-18. Locatorドラッグ移動
- **起動条件**: pin(12×LOCATOR_H)のdrag_started。**掴んだ瞬間に1つGestureIdを採る**(毎フレーム開き直すと1フレームごとにUndoが積まれていた過去のバグの修正、mod.rs:4113-4117コメント)。
- **ドラッグ中**: スナップあり(`snapped`)、端パンあり(`edge_pan`)。
- **確定**: drag_stoppedでhold解除。
- **フィードバック**: カーソルGrab、hover/編集中はACCENT三角。
- **出典**: mod.rs:4113-4142

### M-19. Locatorクリック(ジャンプ)
- **意味**: クリックでplayheadをその時刻へジャンプ(「ロケータの本体はそこへ行くこと」mod.rs:4127コメント)。
- **出典**: mod.rs:4126-4128

### M-20. Locator削除
- **起動条件**: locator右クリックメニュー"Remove locator ⌫"。
- **出典**: mod.rs:4144-4151, 2688-2697

### M-21. ループ帯ドラッグ(新規/移動/両端リサイズ)
- **起動条件**: ルーラ最上段(`LOOP_H=10px`)のみがループの面。押した場所から`loop_grab_for`が種別決定: 端(`LOOP_GRAB=8px`許容、近い方を優先、同距離ならOut=伸ばす操作が多いため優先)→New/Move/In/Out。**引いたら即座にloop_region.on=trueになる**(引いてから別キーで入れる手順にしない、mod.rs:3963コメント)。
- **ドラッグ中の意味**: `LoopGrab::New{anchor}`(新規区間、左右どちらから引いても同じ結果)/`Move{grab_at,from}`(区間ごと平行移動、端で止まる)/`In{fixed}`・`Out{fixed}`(**反対側は掴んだ瞬間の値で固定**——毎フレーム`loop_region`から読み直すと追い越した瞬間に区間が畳まれ、戻しても復元しないため)。`loop_from_drag`が最短1フレームを保証しフレームへスナップ。
- **確定**: drag_stoppedでhold解除(値は逐次書き込み済み、Documentは触らない=session状態)。
- **フィードバック**: ResizeHorizontal(端)/Grabbing(中)/Crosshair(新規)。掴める端は白い縦バーで常時ヒント。
- **出典**: mod.rs:216-232(LoopRegion), 300-340(LoopGrab/loop_grab_for), 342-365(loop_from_drag), 925-931(loop_grab_cursor), 3960-4030(実装)

### M-22. ループON/OFFトグル(Lキー)
- **意味**: 帯を消さず効きだけ切る、**引き直さずに戻せる**(mod.rs:3826-3829,3847-3854)。Cmd+Lは対象外(GroupのCmd+GではなくCmdなしのLのみ)。
- **出典**: mod.rs:3828,3847-3854

### M-23. ルーラスクラブ(playhead)
- **起動条件**: `LOOP_H+LOCATOR_H`分だけ下からのルーラ帯(`ruler_track`)、click_and_drag。`is_pointer_button_down_on()`で押している間ずっと反応(drag_startedではなく押下維持)。
- **意味**: 掴んだら再生停止。playheadはフレームに丸めた時刻(seconds_to_time経由)。端パンあり。Documentは触らない。
- **出典**: mod.rs:4198-4222

### M-24. 波形帯(soundtrack)
- ルーラ直下、view/x換算を共有するのでMキーで打った印は見ていた波形の真上に落ちる(waveform_band.rs, mod.rs:3464-3579)。soundtrackが無ければ帯ごと出ない。**このバンド自体への直接ジェスチャ入力は無し**(表示のみ、下の行のscrub/選択とは独立)。
- **出典**: mod.rs:3464-3579, waveform_band.rs全体

### M-25. ナビゲータ帯(下端、パン/両端ズーム)
- **起動条件**: `NAV_H=14px`帯。掴んだ位置がknob左端/右端から6px以内なら`Left`/`Right`(ズーム)、それ以外は`Pan`。
- **ドラッグ中**: `Pan`=掴んだ時刻が窓中心に来るよう`TimelineView.start`を再計算。`Left`/`Right`=反対の端を固定してspanを`MIN_SPAN`未満にしない。
- **確定**: drag_stoppedでhold解除。Documentは触らない。
- **出典**: mod.rs:3299-3382

### M-26. 縦スクロールバー(つまみドラッグ)
- **起動条件**: `content_h > rows_view.height()`のときのみ出現。track右端`SCROLLBAR_W=6px`。
- **意味**: knob高は`(track_h*ratio).max(24.0)`(最小24px)、`drag_delta().y`をper_px換算して`scroll_y`をclamp。
- **出典**: mod.rs:5076-5109

### M-27. ホイール/トラックパッド(縦スクロール・横パン・横ズーム・ピンチ)
- **割当**(AE/Premiere同型、モジュール冒頭コメントmod.rs:27-34): 素のホイール/二本指=縦スクロール、Shift+ホイール=横パン、Cmd+ホイール=横ズーム(**カーソル下の時刻が動かない**アンカーズーム)、ピンチ=横ズーム。二本指は**x/y同時に効く**(以前はxが少しでも動くとy方向を無視していたが「素直でない」として撤廃、mod.rs:5039-5042コメント)。
- **実装細部**: 生ホイール値(`raw_wheel`)を使用——`smooth_scroll_delta`はegui側で時間的に均された値で、指を止めても数フレーム流れ続けるため、パン/縦スクロールでは遅延として体感される(mod.rs:403-410コメント)。ズームだけは`smooth_scroll_delta`を使う(倍率は指数で効くため生値だと段が見える)。
- **出典**: mod.rs:409-421(raw_wheel), 5013-5064(適用)

### M-28. Transport(to-start / play-pause ボタン、Spaceキー)
- **起動条件**: ヘッダの2つの自前描画ボタン(18×18px)、click sense。
- **意味**: to-startはplayhead=0。play/pauseはトグル、終端で押したら頭から。Spaceキーは同じ効果だが**掴んでいる最中(`self.hold.is_some()`)は入り切りしない**——ドラッグ中に時間が流れると何が起きたか読めなくなるため(mod.rs:3840コメント)。
- **出典**: mod.rs:3616-3673, 3840-3846

### M-29. Undo/Redo(Cmd+Z / Cmd+Shift+Z)
- **意味**: 1ドラッグ=1GestureId=1Undo単位なので、掴んで動かした分がまとめて戻る。掴んでいる最中のEscとは別経路(掴んでいないときのEscは何もしない、Undoの代わりにしない)。
- **出典**: mod.rs:5252-5258, 5266-5273

### M-30. キーボードショートカット群(Duplicate/Delete/Group/Split/SelectAll/矢印/Enter)
- **Cmd+D**: 選択が空なら何もしない。複製後は増えたほうを選ぶ(mod.rs:2507-2548)。
- **Delete/Backspace**: **キー選択が先**(選ばれていれば層より先にキーを消す)。ドラッグ中は効かせない(掴んだものが消えるとgestureの行き先が無くなるため)。Groupは中身ごと1 undo単位(mod.rs:5290-5296)。
- **Cmd+G**: 選択を1つのGroupへ。**親が揃っていない選択は拒否**(別階層のものを1つに入れると位置が誰にも言えなくなるため、mod.rs:2762-2831コメント)。
- **Cmd+K**: playheadで選択中clipを分割。端に当たっているもの(clip外・端ちょうど)は黙ってスキップ(失敗ではない)。Groupは切れない(拒否であって失敗ではない)。
- **Cmd+A**: **見えている行だけ**を全選択(閉じたGroupの中は対象外)。
- **←/→**: playheadを1フレーム(Shiftで10フレーム)移動。**選択もclipも動かさない**——AE/Premiereと同型。リネーム中・locator編集中・他のtext fieldにフォーカスがある間は矢印を横取りしない(`egui_wants_keyboard_input`チェック)。
- **Enter**: 単一選択時のみリネーム開始(AEと同型)。
- **出典**: mod.rs:5259-5346

---

## 周辺: シンプル版ジェスチャ(blitz/shell側、timeline_editor外)

`timeline_move_gesture.rs`・`timeline_trim_gesture.rs`は`timeline_editor`とは別の、より単純なTransientライフサイクル型(`RationalTime`ベース、`pub(crate)`)。おそらく `timeline_blitz`/shell側のSkia描画パスから使われる別実装。

### G-1. TimelineMoveGesture(単一clip移動)
- `begin(layer, initial_pointer, initial_start, generation)`→`preview(pointer_time)`(delta加算のプレビュー、Documentへは書かない)→`release(pointer_time)`(同値ならNone=no-op、そうでなければ`TimelineMoveRequest{layer,new_start}`を1個返すだけ)。
- **multi-select・snap・edge-pan・group追従キーの概念が無い** — timeline_editorのM-1相当機能のうち、"何が動くか"の核だけを持つ縮小版。
- **出典**: timeline_move_gesture.rs:23-78, テスト同ファイル:80-129

### G-2. TimelineTrimGesture(単一clip・単一端トリム)
- `TimelineTrimEdge::Left/Right`のみ(In/Outと同義)。`preview`はdelta加算でstart/endどちらかだけを動かす。**ripple/roll/slip/slideに相当する型・分岐は存在しない**——timeline_editor側の既決(trim family不採用)と構造的に一致。
- **出典**: timeline_trim_gesture.rs:1-118

---

## 参考: OSドラッグ&ドロップによるメディア取り込み(import_seat.rs)

Timeline面へのファイルドロップは**ピクセル単位のジェスチャではなくOSイベント**だが、Timeline操作文法の一部として記録する。
- 1回のドロップ=1 GestureId=1 Undo単位(取り込みと配置を分けない)。
- **曲がまだ無いprojectへ音声を落とすとsoundtrackになる**(CapCut/Ableton同型の既定)。曲が既にあれば通常clipとして最初のtrack末尾(実際はplayhead位置、`import_media_at_playhead`)に置く。
- 置ける場所が無ければ(trackが空/playheadが終端以降)、台帳を触る前にエラーで拒否——素材だけ入ってclipが無い中途半端を作らない。
- **出典**: import_seat.rs:1-129, mod.rs:1399-1431

---

## 既決との照合まとめ

1. **trim familyは実装されていない(既決と一致)**: `TrimIn`/`TrimOut`は単独clipのstart/endのみを動かし、ripple(後続clip連鎖移動)・roll(隣接clip境界を保ったまま両clipの尺を再配分)・slip(clip内容だけ動かし尺は不変)・slideのいずれも型・分岐として存在しない。`timeline_trim_gesture.rs`の`TimelineTrimEdge::Left/Right`も同型。**既決「AE型自由配置・trim family不採用」を裏付ける一次資料。**
2. **move/trim/splitが別々のD2コマンド(旧Document層)**: 実装は`prepare_set_clip_start`(move)・`prepare_trim_clip_in/out`(trim)・`prepare_split_clip`(split)という**別々の`prepare_*`入口**を叩いている。next側の裁定51「move/trim/split/速度はすべて`LayerTiming`の上に乗り、intentは`SetTiming`1つ」とは**構造的に異なる**(この相違はUI操作文法そのものではなく、話している相手が旧`motolii-doc`のD2バックエンドだからで、next移植時はGrab種別ごとの意味論だけを引き継ぎ、出力コマンドはSetTiming1本へ収束させる必要がある)。
3. **ダブルクリックは明示的に不使用**(モジュール冒頭コメント: 「選択・並べ替え・跳ぶ が同じ場所に重なっている面では、2回目の押下が別の操作の途中と区別できない」)。AE(コンポジションを開く)・Premiere(edit点へトリム)のダブルクリック慣習とは意図的に非採用——既決ではなくこの実装内の設計原則だが、台帳に持ち越す価値のある否定形の決定。
4. **undo機構がGestureId+明示スタック**: next裁定2/47/48(undo=editタイムラインの時間移動)・裁定118(b)(同一at刻みへのbatch、失敗時drop_time_range)とは非互換。UI文法(「1ドラッグ=1確定単位」という粒度)は保存し、機構(`writer.undo()`呼び出し)は移植対象から除外すべき。
5. **キー時刻移動は5paramすべてで可能だが、イージング変更はPositionのみ**という非対称が実装に残っている(D2の`prepare_set_position_key_interp`しか存在しないため)。次期文法で解消するかどうかは未決の穴として明記すべき。
