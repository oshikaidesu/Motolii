# P3 — シェイプ/マスク/エフェクトで1カット作り、続きを別の日に開く

対象読者: 手順書は最終的に利用者が読む製品成果物。ここでは「新規プロジェクトを開く」から
「別の日に開いて続きから作業する/別マシンで開く/人に渡す」までを、名前の無い操作もすべて
独立した1手順として書く(README の粒度規約)。

判定は実装の `file:line` に対してのみ行う。4値: `書ける` / `【穴】入口が無い` / `【穴】意味が無い` / `【未確認】`。

---

## 前半 — シェイプ/マスク/エフェクトで1カット作る

### A. 新規プロジェクトを開く

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 1 | アプリを起動する | OS | `next/shell/motolii-shell/src/main.rs:104-135` `main()` が `Shell::boot` を呼ぶ | 書ける |
| 2 | 起動直後に既定の空プロジェクト(640x360・30fps・300フレーム)が開いていることを確かめる | Stage/Settings | `next/shell/motolii-shell/src/lib.rs:1103-1112` `default_document()` が `Intent::SetComposition{width:640,height:360,fps:30,duration_frames:300}` を適用 | 書ける |
| 3 | 「スタート画面」や「開くプロジェクトを選ぶ」画面が出ないことを確かめる(いきなり編集画面) | 起動直後 | `grep -rn "StartScreen\|WelcomeScreen" next/shell next/ui next/core` = 0件(実在するのはテストの死んだ参照のみ、`next/core/motolii-testkit/src/lib.rs:23,254`)。=そういう仕様として確かめられる(画面自体が無いことを確認できる) | 書ける |
| 4 | Cmd+N で改めて新規プロジェクトを作る | どこでも | `next/ui/motolii-keymap/src/defaults.rs:277-283` Cmd+N → `VerbId::NewProjectRequested`、`next/shell/motolii-shell/src/lib.rs:1864-1877` `reset_document()` | 書ける |
| 5 | 未保存の変更がある状態で Cmd+N を押すと、上書きしてよいか確認される | どこでも | `next/shell/motolii-shell/src/lib.rs:505-517` `NewProjectRequested`(dirty-guarded) | 書ける |
| 6 | コンポジションの解像度・fps・尺を Settings パネルで変える | Settings | `next/ui/motolii-settings-pane/src/sections.rs:100-134` `CompField::{Width,Height,Fps,DurationFrames}` | 書ける |
| 7 | 値を Enter で確定する | Settings | 同上(Enter 確定) | 書ける |
| 8 | この変更が Undo 1回で戻ることを確かめる | どこでも | `Document::apply_all` が1回の edit 刻みへ書く(裁定48) | 書ける |

### B. シェイプを描く

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 9 | Browser パネルを開く | Browser | パネル常設(4 section 常設方針、裁定 UI 手触り方向) | 書ける |
| 10 | Create タブへ切り替える | Browser | `next/ui/motolii-browser-pane/src/model.rs:341-364` `enum CreateKind` | 書ける |
| 11 | Rectangle カードをダブルクリックして層を作る | Browser | `model.rs:494-556` `CREATE_PREVIEW`、L511 `creates: Some(CreateKind::Rectangle)` | 書ける |
| 12 | 層が Timeline に1行増え、選択された状態になることを確かめる | Timeline | `AddLayer`+`SetMeta`+`SetAttrs` が同一 `apply_all`(`shell/motolii-shell/src/lib.rs:1548` `create_from_card`) | 書ける |
| 13 | Stage に矩形が実際に表示されることを確かめる | Stage | `next/shell/motolii-shell/src/create.rs:58-108` が Rectangle に `Intent::SetShapes` と既定 Fill を同じ `apply_all` で書く。`next/shell/motolii-shell/src/render.rs:331-358` が Stage の `PreviewSnapshot` に shape 本体を集め、`next/shell/motolii-shell/src/stage_presenter.rs:807-816` → `next/engine/motolii-engine/src/render.rs:369-394` → `next/engine/motolii-engine/src/texture.rs:82-90` が GPU 経路へ渡してラスタライズする。回帰柵は `next/shell/motolii-shell/src/render.rs:619-658` | 書ける |
| 14 | Stage 上でシェイプツール(ペン/矩形/楕円)に持ち替えて手描きする | Stage | `next/ui/motolii-stage-pane/src/shape_tool.rs:31-59` の工具語彙/Message、`next/ui/motolii-stage-pane/src/shape_tool.rs:127-283` の toolbar・座標変換・ドラッグ/ペン確定、`next/shell/motolii-shell/src/shape_ops.rs:27-99` の `AddLayer`+`SetShapes` 一括書き込み。`shape_tool` focused test と shell 型検査が緑 | 書ける |
| 15 | 矩形の幅・高さ・角丸半径を Inspector の数値で変える | Inspector | `next/ui/motolii-inspector-pane/src/projection.rs:258-310` が矩形/楕円の `ShapeNode` を `ShapeSectionProjection` へ投影し、`next/ui/motolii-inspector-pane/src/shape.rs:37-119` が値を検証して `Intent::SetShapes` へ一括確定、`next/ui/motolii-inspector-pane/src/shape.rs:123-192` が SHAPE section を描く。`shape_inspector_changes_geometry` が角丸追加と幅変更を検収 | 書ける |
| 16 | パスの頂点を Stage 上でドラッグして形を変える | Stage | `next/ui/motolii-stage-pane/src/path_edit.rs:52-120` が選択中の flat Bezier leaf の頂点を投影し、`next/ui/motolii-stage-pane/src/path_edit.rs:307-358` が hit-test/drag を layer-local 座標へ戻す。`next/shell/motolii-shell/src/path_ops.rs:32-110` が `edit::move_vertex` → `Intent::SetShapes` で確定し、`path_vertex_drag_changes_shape` が実値を検収 | 書ける |
| 17 | パスを閉じる/開く | Stage/Inspector | `next/ui/motolii-stage-pane/src/path_edit.rs:145-190` の toolbar が先頭 contour の状態に応じて Close/Open を表示し、`next/shell/motolii-shell/src/path_ops.rs:45-54,96-108` が `edit::{close_path,open_path}` を `SetShapes` へ写す。`path_close_and_open_are_one_document_edit_each` が往復を検収 | 書ける |
| 18 | 角を丸める(RoundedCorners)・トリムパス・繰り返し(Repeater)などの modifier を足す | Browser/Inspector | `next/ui/motolii-browser-pane/src/model/tabs.rs:409-475` が7種の `ShapeOpKind` カードを宣言し、`preview_view.rs:260`→`Message::ApplyOpFromCard`→`next/shell/motolii-shell/src/shape_operator.rs:36-106` が既定 `OpKind` を組んで `Intent::SetShapes` へ1段積む。カード宣言は `tabs/tests.rs:316-339`、実書き戻しは `shape_operator.rs:118-160` が検収 | 書ける |
| 19 | 星形・多角形のシェイプを作る | Browser | 【穴】意味が無い — `CreateKind` には Star/Polygon 相当の variant 自体が無い(4種類+Text のみ、`model.rs:341-364`) | 【穴】意味が無い |

### C. 塗りと線を付ける

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 20 | 塗りの色をカラーピッカーでつまむ | Inspector | 【穴】意味が無い — `grep -rn "color_picker" next/ui next/shell` = 0件。色ピッカー部品自体が `next/` に存在しない(iced 標準・サードパーティとも) | 【穴】意味が無い |
| 21 | 塗りの色を16進テキストで打つ | Inspector | 【穴】入口が無い — `Fill`/`Brush::Solid` は store に実装済み(`engine/motolii-vector/src/lib.rs:316-463`)だが SHAPE section が無く読み書き口ゼロ(手順15と同根) | 【穴】入口が無い |
| 22 | グラデーションを付ける | Inspector | 【穴】入口が無い — `Brush::Gradient`/`Gradient`/`GradientStop` は実装済みだが同上 | 【穴】入口が無い |
| 23 | 線幅・線の形(角/丸)・破線を設定する | Inspector | 【穴】入口が無い — `Stroke`/`Dash`/`LineCap`/`LineJoin` 実装済み・呼び手ゼロ | 【穴】入口が無い |

### D. マスクを切る

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 24 | Browser の Effects タブへ切り替える | Browser | `model.rs:438-489` `EFFECTS_PREVIEW` | 書ける |
| 25 | マスクを掛けたいレイヤーを1つ選ぶ | Timeline/Stage | `Session` の選択が唯一の正本(裁定46/107) | 書ける |
| 26 | Mask カードをダブルクリックしてマスクを追加する | Browser | `model.rs:475-489` `id:"mask"` カード → `next/ui/motolii-browser-pane/src/lib.rs:1579-1589` `Message::AddMaskFromCard` → `shell/motolii-shell/src/lib.rs:1410-1414`,`1692-1721` `add_mask_to_selected_layer` が `Intent::AddMask{layer,mask,shape}` を適用(既定の矩形パス込みで1回の apply_all) | 書ける |
| 27 | 選択済みレイヤーが2つ以上/0の状態でマスクを追加しようとすると、理由つきで断られる | Browser | `shell/motolii-shell/src/lib.rs:1692-1696` — 「マスクを追加するレイヤーを1つ選んでください」を status へ出す | 書ける |
| 28 | 追加直後、既定の矩形マスクが Stage に反映されることを確かめる | Stage | `Intent::AddMask` が shape を同時に書くため `resolved_masks` はエラーにならない(`core/motolii-store/src/view.rs:671-676` のエラー分岐を回避) | 書ける |
| 29 | マスクの形をドラッグで変える(頂点編集) | Stage | 【穴】入口が無い — マスクの頂点も B-16 と同じ `motolii-vector` の edit 関数群止まりで呼び手ゼロ | 【穴】入口が無い |
| 30 | マスクのモード(Add/Subtract/Intersect/Lighten/Darken/Difference)を巡回で切り替える | Inspector | `ui/motolii-inspector-pane/src/mask.rs:37-44` `next_mask_mode`、`lib.rs:1720`/`shell/motolii-shell/src/lib.rs:2376` `CycleMaskMode` | 書ける |
| 31 | マスクを反転(invert)する | Inspector | `ui/motolii-inspector-pane/src/mask.rs` の `toggle_inspector_mask_inverted`(`lib.rs:154` import) | 書ける |
| 32 | マスクの不透明度を数値で変える | Inspector | `TransformField::MaskOpacity(MaskId)`(`transform.rs:63`) | 書ける |
| 33 | マスクの不透明度にキーフレームを打つ | Inspector | `KeyRow::MaskOpacity(MaskId)`(`transform.rs` 197 台) | 書ける |
| 34 | マスクの膨張(Expand)を数値で変える | Inspector | 【穴】入口が無い — `TransformField` に `MaskExpansion` 相当が無い(`Position/Scale/Rotation/Opacity/Anchor/MaskOpacity/EffectParam/...` のみ、`transform.rs:47-70`)。store 側の `PropertyId::mask_expansion`(`core/motolii-store/src/document.rs:89`)は書けても UI に口が無い | 【穴】入口が無い |
| 35 | Expand を UI 経由で書けたとしても、実際にマスクが膨らんで見えることを確かめる | Stage | 【穴】意味が無い — `ResolvedMask` 構造体自身が `expansion` フィールドを持たず `resolved_masks` も読んでいない(`core/motolii-store/src/mask.rs` 冒頭「未完(次のレーンへ)」) | 【穴】意味が無い |
| 36 | マスクを削除する | Inspector | 【未確認】— `grep -rn "RemoveMask\|DeleteMask" next/shell next/ui` = 0件。`RemoveEffect`(`inspector-pane/src/lib.rs:265`)と同型の `RemoveMask` メッセージは見当たらない。実装が本当に無いのか名前が違うだけか、grep だけでは断定できない(ビルドしない縛りのため実機確認できず) | 【未確認】 |

### E. エフェクトを足す

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 37 | 対象レイヤーを1つ選ぶ | Timeline/Stage | (D-25 と同じ選択機構) | 書ける |
| 38 | Effects タブの Glow カードをダブルクリックして適用する | Browser | `model.rs:468-476` `applies_to_selection: Some(SelectionAction::ApplyEffect("motolii.glow"))` → `shell/motolii-shell/src/lib.rs:1415-1416` `apply_effect_to_selected_layer` | 書ける |
| 39 | Glow が実際に画に効くことを Stage で確かめる | Stage | `EffectPass::Glow`(`engine/motolii-compositor/src/effects/mod.rs:24-38`)が唯一の実装 effect | 書ける |
| 40 | Glow 以外のエフェクト(Echo Bloom / Sine 等)を試す | Browser | 【穴】意味が無い — `EFFECTS_PREVIEW` の Echo Bloom(`model.rs:442-450`)・Sine(`460-467`)は `applies_to_selection: None` の飾りカード(モック)。実装済み effect 種は Glow 1つのみ | 【穴】意味が無い |
| 41 | エフェクトのパラメータ(強さ等)を数値で変える | Inspector | `TransformField::EffectParam(EffectId, GlowParam)`(`transform.rs:64-67`) | 書ける |
| 42 | パラメータにキーフレームを打つ | Inspector | `KeyRow::EffectParam(EffectId, GlowParam)`(`transform.rs` 199台) | 書ける |
| 43 | エフェクトを一時的に無効化する(bypass) | Inspector | `KeyRow::EffectEnabled(EffectId)`(`transform.rs` 200台)、`crate::effects::toggle_inspector_effect_bypass` | 書ける |
| 44 | エフェクトを削除する | Inspector | `ui/motolii-inspector-pane/src/effects.rs:363` "Remove" ボタン → `Message::RemoveEffect(id)`(`lib.rs:265`)→ `shell/motolii-shell/src/lib.rs:2399` | 書ける |
| 45 | 削除を Undo で戻し、パラメータも一緒に戻ることを確かめる | Inspector | `effects_section.rs:308` のテストコメントが同保証を明記 | 書ける |
| 46 | 同じレイヤーに2つ目の違う種類のエフェクトを重ねる(例: Glow の上にもう1種) | Inspector | 【穴】意味が無い — 実装 effect 種が Glow 1つしか無いため「違う種類を重ねる」自体が現状再現不能(手順40と同根) | 【穴】意味が無い |

### F. キーフレームで動かす

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 47 | 再生ヘッド(playhead)を先頭(0フレーム)へ移動する | Timeline | `timeline_pane::Message::JumpPlayheadToStart`(shell の `Message::Timeline` match、`session.playhead = 0`) | 書ける |
| 48 | Position の値を Inspector で変える | Inspector | `commit_inspector_field`(`transform.rs:748`)→ `Intent::SetTrack` | 書ける |
| 49 | Position 行のダイヤモンド(◇)をクリックして、その時刻に打点する | Inspector | `KeyRow::Position`+`toggled_key_track`(`transform.rs:330-402`、Static→insert key)、クリック Message は `lib.rs:247` `KeyPressed(KeyRow)` | 書ける |
| 50 | ダイヤモンドが実菱形(◆)になった=打点されたことを確かめる | Inspector | `KeyCellState::AtKey`(`transform.rs:276-285`) | 書ける |
| 51 | 次の時刻(例: +30フレーム)へ移動する | Timeline | `Message::StepPlayhead(delta)` → `nav::step_playhead`(`ui/motolii-timeline-pane/src/nav.rs:19-21`) | 書ける |
| 52 | 特定のフレーム番号を数字で直接タイプして移動する | Timeline | 【穴】入口が無い — Timecode 表示は読み取り専用(`ui/motolii-timeline-pane/src/transport.rs:61-76,148,188`)。フレーム番号のテキスト入力欄そのものが無い(`Message::ScrubTo(i64)` はルーラーのクリック/ドラッグ駆動のみ) | 【穴】入口が無い |
| 53 | Position の値を再度変えて、2つ目のキーフレームを打つ | Inspector | 手順48-49と同じ経路 | 書ける |
| 54 | 2つのキーの間をスクラブして中間の動きを見る | Timeline/Stage | `ScrubTo`(`write.rs:42`)、evaluate は store の補間規則 | 書ける |
| 55 | 「次のキーフレームへジャンプ」する | Timeline | 【穴】入口が無い — 実装されているのは**クリップの編集点**へのジャンプ(`Message::JumpToNextClipEdit`/`JumpToPreviousClipEdit`、`write.rs:122,125`、`keys2::clip_edit_points`)のみで、property のキーフレーム位置への専用ジャンプは無い(`grep -n "NextKeyframe\|PreviousKeyframe" write.rs` = 0件)。台帳には `Next Keyframe`(id 504)/`Previous Keyframe`(id 505)の行があり両方「採用済」表示だが、実コードは前記のクリップ編集点ジャンプしか無い — **台帳の verdict(採用の意思決定)と実装の有無が一致しない例** | 【穴】入口が無い |
| 56 | キーフレームをダイヤモンドではなく、タイムライン上のマーカーとして見て、ドラッグで動かす | Timeline | `Message::KeyGrabbed{key,at_frame,retime}`/`KeyDragMoved`/`KeyDragReleased`(`write.rs:157-165`)、`start_key_drag`/`continue_key_drag`/`finish_key_drag`(`write.rs:620-631`) | 書ける |
| 57 | Esc でドラッグ中のキー移動を取り消す | Timeline | `Message::KeyDragCancelled`(`write.rs:165`)、`cancel_key_drag` | 書ける |
| 58 | キーフレームをキーボード(矢印キー)でフレーム単位に動かす | Timeline | `Message::NudgeKeyframe(i64)`(`write.rs:168`)→ `nudge_keyframe`(`write.rs:1614-1633`) | 書ける |
| 59 | 選択中のキーフレームを消す | Timeline | `Message::DeleteSelectedKeys`(`write.rs:63`)→ `delete_selected_keys`(`write.rs:1309`)、Delete/Backspace キー(`shell/motolii-shell/src/lib.rs:5781`) | 書ける |
| 60 | 消したキーフレームを Undo で戻す | Timeline | `apply_all` 経由の1 undo(裁定48と同型) | 書ける |
| 61 | 複数のキーフレームをまとめて選ぶ(タイムライン上でクリック/Shift+クリック/範囲) | Timeline | `write.rs:1266-1297`(単一選択・追加・Shift範囲選択の3分岐) | 書ける |

### G. イージングを付ける

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 62 | 動かしたいキーフレームを選ぶ | Timeline | 手順61と同じ選択機構 | 書ける |
| 63 | Edit メニューから「Interpolation: Hold」を選ぶ | メニュー | `menu.rs:128-158` → `Message::SetKeyInterp(Interp::Hold)` | 書ける |
| 64 | 「Interpolation: Linear」を選ぶ | メニュー | 同上、`Interp::Linear` | 書ける |
| 65 | 「Interpolation: Easy Ease」を選ぶ | メニュー | 同上、`timeline_pane::EASY_EASE`(`write.rs:285`、`Interp::Bezier`プリセット) | 書ける |
| 66 | 「Easy Ease In」/「Easy Ease Out」を選ぶ | メニュー | 同上、`write.rs:287,289` | 書ける |
| 67 | イージングの切替をキーボードショートカットで行う | どこでも | 【穴】意味が無い — メニュー項目にショートカット未設定(`menu.rs:128-158` の doc「shortcut は未実装」)。ショートカット自体が存在しない設計状態であり、入口が塞がっているのではなく機能そのものが無い | 【穴】意味が無い |
| 68 | ベジエカーブをグラフで見て、ハンドルをドラッグして調整する(Graph Editor) | 専用パネル | 【穴】意味が無い — Graph Editor 自体が存在しない(round2 §1 壁5)。台帳には対応行あり(id 519「Toggle between Graph Editor and layer bar modes」採用予定) | 【穴】意味が無い |
| 69 | x1/y1/x2/y2 を数値で直接入力してベジエを調整する | Inspector | 【穴】意味が無い — 数値入力 UI 無し(5固定プリセットのみ、`write.rs:285-289` 不変) | 【穴】意味が無い |

### H. 見て直す

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 70 | Space キーで再生する | どこでも | `Message::TogglePlayback`(`shell/motolii-shell/src/lib.rs:5671`) | 書ける |
| 71 | 再生中に Stage の絵が更新され続けることを確かめる | Stage | `PlaybackTick` によるスクラブ/自動再生(round2 §1 行12) | 書ける |
| 72 | 音が鳴ることを確かめる | どこでも | 【未確認】— GOALS M8(音声クロック→playhead)は本調査時点で GOALS.md に明記された条件だが、このターンはビルド禁止のため実機で音が鳴るか確認できない。KNOWN.md に「audio-clock-master は移植元として名指し可」とあるのみで next/ 内の結線有無を私は未検証 | 【未確認】 |
| 73 | 動きが意図と違うので Undo で1つ戻す | どこでも | Cmd+Z → `VerbId::Undo`(`defaults.rs:211-217`)、`menu.rs:97` | 書ける |
| 74 | Redo でやり直す | どこでも | Cmd+Shift+Z → `VerbId::Redo`(`defaults.rs:218-224`)、`menu.rs:98` | 書ける |
| 75 | 値を直して同じ時刻に打ち直す | Inspector | 手順48-50の反復 | 書ける |
| 76 | 深く遡って何回も Undo しても壊れない/落ちないことを確かめる | どこでも | 【未確認】— KNOWN.md「D2 Undo が壊れない・深さで落ちない=済(R0)」はあるが GC 方針は空席と併記されており、深い遡及での挙動は実機でないと確認できない | 【未確認】 |

---

## 後半 — 続きを別の日に開く

### I. 保存する

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 77 | ただの上書き保存(Cmd+S 相当)をしようとする | どこでも | `menu.rs:68-74`の`Save`/`Cmd+S`、`input.rs:337-343`の`Message::SaveRequested`、`document_io.rs:508-514`の既知path上書き | 書ける |
| 78 | Cmd+Shift+S で Save As を実行する | どこでも | `defaults.rs:284-289` Cmd+Shift+S → `VerbId::SaveAsRequested`、`menu.rs:58-67` | 書ける |
| 79 | OS のファイル保存ダイアログでファイル名と保存先を選ぶ | OS ダイアログ | `shell/motolii-shell/src/file_dialogs.rs:66-72` `FileDialogs` トレイト、本番実装は `rfd`(`lib.rs:991`) | 書ける |
| 80 | 保存が成功し、以後この path が「現在のプロジェクト」になったことを確かめる | どこでも | `perform_save_as`(`lib.rs:1878-1892`)が `current_path`/`saved_revision` を更新 | 書ける |
| 81 | 保存成功を示す目に見える合図(トースト・チェックマーク等)を探す | どこでも | `document_io.rs:105-119`の成功分岐が`status = "保存しました: …"`を書き、sidecarも更新する | 書ける |
| 82 | 別名で複製を保存する(Save a Copy) | メニュー | `menu.rs:58-67`、`perform_save_a_copy`(`lib.rs:1894-1900`)。**現在開いているプロジェクトの身分(current_path)は変わらない** | 書ける |
| 83 | 保存に失敗した時(権限・ディスク満杯等)、理由がその場で分かる | どこでも | `perform_save_as` の `Err(error) => self.status = Some(format!("保存できない: {error}"))` | 書ける |
| 84 | 自動保存の頻度・保持世代数を Settings で設定する | Settings | `core/motolii-store/src/persist.rs` の `auto_save`(AE 型、`<project隣>/<name> auto-save/` へ世代保存)。頻度・世代数は Settings で編集可(`sections::Message::AutoSaveToggle`/`AutoSaveFieldInput`、`shell/motolii-shell/src/lib.rs:3100-3106`) | 書ける |
| 84b | 自動保存が「今この瞬間に走った」ことを示す視覚合図に気づく | どこでも | `document_io.rs:207-229`の`run_auto_save`成功分岐が`status = "自動保存しました: …"`を書く | 書ける |
| 85 | 一度も明示保存していない新規プロジェクトで自動保存が発動しないことを確かめる | どこでも | `auto_save` の `project_path` が `None` なら即 `Ok(None)`(persist.rs、doc「未保存の新規project」節) | 書ける |

### J. 閉じる

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 86 | Cmd+Q でアプリを終了する | どこでも | `Message::QuitRequested`(`lib.rs:1500`)、`menu.rs:73` | 書ける |
| 87 | 未保存の変更がある状態で Cmd+Q すると、保存するか確認される | どこでも | `confirm_then(Message::QuitConfirmed)` → `confirm_discard_future`(`lib.rs:1831-1837`)が `is_dirty()` の時のみ `dialogs.confirm_discard()` を呼ぶ | 書ける |
| 88 | ウィンドウの閉じるボタン(赤い×)をクリックして閉じる | OS ウィンドウ | `lib.rs:1079-1083`で`exit_on_close_request:false`、`lib.rs:1254-1258`で`close_requests`を`WindowCloseRequested`へ翻訳 | 書ける |
| 89 | ×ボタンで閉じる時も、未保存なら確認が出ることを期待する | OS ウィンドウ | `document_io.rs:537-552`が`WindowCloseRequested`をdirty確認へ通し、falseでは窓を維持。`tests/suite/window_drive.rs`が拒否側を検分 | 書ける |
| 90 | ×ボタンで未保存のまま閉じてしまい、変更が失われたことに後で気づく | 次回起動時 | `document_io.rs:545-548`は確認結果trueの時だけ`iced::exit()`へ進む。破棄を選んだ場合の結果として扱い、確認なしの損失とは区別する | 書ける |

### K. 別の日に開き直して続きから作業する

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 91 | アプリを起動する | OS | 手順1と同じ | 書ける |
| 92 | 起動直後に「先週保存したプロジェクトが自動的に開いている」ことを期待する | 起動直後 | `document_io.rs:275-310`が保存済みpathをsidecarへ書き/読み、`lib.rs:1046-1060`がboot時に`LastProjectPathRead`を発行し、`document_io.rs:553-560`がDocumentを開く | 書ける |
| 93 | File メニューの「最近使ったファイル」一覧から選ぶ | メニュー | 【穴】意味が無い — `grep -i "Open Recent\|Recent Project\|Recent File" next/reference/normal-map.tsv` = 0件(そもそも台帳にも対応する製品コマンド行が無い)。実装側も `recent_files` 系の識別子ゼロ | 【穴】意味が無い |
| 94 | File → Open… でファイルダイアログを開き、先週保存したファイルを選ぶ | メニュー/OSダイアログ | `menu.rs:64` `Message::OpenRequested`、`perform_open`(`lib.rs:1902-1918`)、`Document::load`(`core/motolii-store/src/persist.rs:123`) | 書ける |
| 95 | 開いたプロジェクトが正しく復元されたこと(レイヤー・キーフレーム・マスク・エフェクト)を確かめる | Stage/Timeline/Inspector | `Document::save`/`load` の往復(裁定55/56、bezier・NTSC fps 込み) | 書ける |
| 96 | 前回どこを見ていたか(再生ヘッド位置・選択レイヤー)が復元されることを期待する | Timeline/Inspector | 【穴】意味が無い — `perform_open`(`lib.rs:1918`)は明示的に `self.session = Session::default()` を実行する。**選択も再生ヘッド位置も毎回ゼロへ戻る**(保存もされていない。`Session` は Document の外にあり persist.rs の対象外) | 【穴】意味が無い |
| 97 | 開いた直後から新しい編集を始め、Undo が今回のセッション分だけ効くことを確かめる(前回の Undo 履歴は保存時に畳まれている) | Timeline | `Document::save` は `flattened()` で履歴を畳んでから書く(persist.rs doc)。`Document::load` 直後は `mark_undo_floor` 済みで、それ以前へは戻せない(`perform_open` doc コメント) | 書ける |
| 98 | パネルのレイアウト(パネルの大きさ・並び)が前回のまま復元されることを期待する | 起動直後 | 【穴】意味が無い — 手順96と同根。`Session` すら保存対象外である以上、パネルレイアウトの永続化はさらに手前で存在しない(専用の永続化構造体が見つからない、`grep -rn "layout" next/ui/motolii-shell-state/src` 未実施だが `Session` が唯一の shell 状態構造体である以上、別経路は無い) | 【穴】意味が無い |

### L. 素材を別のフォルダへ動かしてしまった場合

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 99 | Finder で素材ファイルを別フォルダへ動かした後、プロジェクトを開く | OS/メニュー | 手順94と同じ open 経路 | 書ける |
| 100 | 素材が見つからないレイヤーの Stage 表示を確かめる(エラー表示か、空白か、古いサムネイルか) | Stage | 【未確認】— `Document::load`/`Asset` のデシリアライズは `path_absolute` の実在確認をしない(`core/motolii-store/src/asset.rs`・`persist.rs` に存在チェックのコード無し)。読み込み自体はエラーにならないため**その先の描画結果(空白か既定色か)は実機でないと分からない** | 【未確認】 |
| 101 | 「この素材が見つかりません」という理由つきの通知が出ることを期待する | どこでも | 【穴】意味が無い — `grep -rn "missing\|relink\|not found\|NotFound" --exclude-dir=target next/ui next/shell` はテスト名か無関係な doc comment のみ(素材/メディアに関する行は0件)。**「素材が欠けている」という状態そのものに名前も通知も無い**(台帳の `Find Missing Footage` はid 1393で回復コマンドとして存在するが、実装側には対応する識別子が無い) | 【穴】意味が無い |
| 102 | メニューやツールバーから「Find Missing Footage」(素材を探し直す)を実行する | メニュー | 【穴】意味が無い — 台帳には対応行(id 1393、`採用予定`)があるが、`next/ui` `next/shell` に該当コマンドの識別子は存在しない(手順101と同じ grep 結果) | 【穴】意味が無い |
| 103 | 素材を元の場所に手で戻して解決する(回避策) | OS | 書ける(OS のファイル操作。アプリ機能ではない。`path_absolute` が再び実在すれば次回描画時に解決される想定だが、実機未検証) | 書ける |
| 104 | Asset の参照パスを Inspector やダイアログで直接テキスト編集して繋ぎ直す | Inspector | 【穴】入口が無い — Asset の `path_absolute`/`path_project_relative`(`core/motolii-store/src/asset.rs:69-79`)を書き換える UI 入口が見当たらない(手順101の grep と同じ範囲で0件) | 【穴】入口が無い |

### M. 別のマシンで開く(素材のパス・フォント)

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 105 | プロジェクトファイル(と、素材がプロジェクトの外にあるならその素材)を USB/クラウド同期で別マシンへコピーする | OS | 書ける(OS のファイル操作。アプリ機能ではない) | 書ける |
| 106 | 別マシンでアプリを起動し、File → Open… でプロジェクトファイルを開く | メニュー | 手順94と同じ | 書ける |
| 107 | 素材が project フォルダの直下にあった場合、`path_project_relative` のおかげで別マシンでも解決されることを期待する | Stage | 【未確認】— `AssetDraft::from_probed_source`(`core/motolii-store/src/asset.rs:126-152`)が `path_project_relative` を prefix 計算で埋めるのは確認できたが、**読込側がどちらの path を優先して解決を試みるか**(project-relative を実際に使う消費コードの有無)は今回の探索で特定できなかった。実機なしでは判定不能 | 【未確認】 |
| 108 | 素材が project フォルダの外にあった場合(絶対パスのみ記録)、別マシンでは解決できないことに気づく | Stage | 【穴】意味が無い — `path_absolute` はコピー元マシンの絶対パス文字列のまま(`canonicalize` 呼び出しは無し、`grep -n "canonicalize" next/core/motolii-store/src/*.rs next/engine/motolii-media/src/*.rs` = 0件)。解決できない状態への通知が無いのは手順101と同根 | 【穴】意味が無い |
| 109 | テキストレイヤーのフォントが別マシンに入っていないことに気づく | Inspector/Stage | 【穴】意味が無い — `find_family(family)`(`ui/motolii-font-catalog/src/lib.rs:132-133`)は見つからなければ `None` を返すだけで、これを消費して利用者へ通知する呼び手は `ui/motolii-inspector-pane/src/text.rs` のフォント選択 UI のみ(`171,329`)。**テキストの実描画経路(engine 側)がフォント未解決をどう扱うかは今回未追跡** — 少なくとも「見つからないフォントがある」という通知コマンドは0件(`grep -i "missing font" next/ui next/shell` 相当なし) | 【穴】意味が無い |
| 110 | 「Find Missing Fonts」を実行して代替フォントを選ぶ | メニュー | 【穴】意味が無い — 台帳には対応行(id 1273、`採用予定`)があるが実装側に識別子なし(手順109と同じ範囲) | 【穴】意味が無い |
| 111 | 手動でフォントファミリーを別のものに差し替える | Inspector | 書ける — `ui/motolii-inspector-pane/src/text.rs:171` フォント選択欄が `find_family` で候補を引く。差し替え自体はテキスト編集操作として通る(見つからない事への気づき方が無いだけで、差し替え操作自体はある) | 書ける |

### N. 誰かにプロジェクトごと渡す

| # | 利用者は何をするか | どこで | 実装の証拠 | 判定 |
|---|---|---|---|---|
| 112 | プロジェクトファイルと素材フォルダを手でまとめて相手に送る(zip 化・クラウド共有等) | OS | 書ける(OS のファイル操作。アプリ機能ではない) | 書ける |
| 113 | 「依存ファイルを収集する」機能(Collect Files 相当)でアプリに自動でまとめてもらう | メニュー | 【穴】意味が無い — 台帳には対応行(id 526「Collect Files…」、`採用予定`)があるが `grep -rln "collect_files\|CollectFiles\|package_for\|PackageProject" --exclude-dir=target next/` = 0件、実装ゼロ | 【穴】意味が無い |
| 114 | フォントも一緒に埋め込んで渡す | メニュー | 【穴】意味が無い — フォントは family 名参照のみで埋め込み経路が無い(手順109と同根)。台帳にも対応する「フォント埋め込み」行は見当たらない(埋め込みは Find Missing Fonts=回復コマンドとは別概念) | 【穴】意味が無い |
| 115 | 相手が別マシンで開き、素材/フォントが解決できないことに個別に気づいて手で直す | 相手側 | 手順108-111 の繰り返し(同じ穴を相手側でも踏む。個々の操作自体は開通するので手順としては書ける) | 書ける |
| 116 | 相手にプロジェクトの「意味の正本」を渡せたことを機械的に確認する手段(チェックサム・バージョン番号の一致確認等)を探す | どこでも | 【穴】意味が無い — そのような検証機能は見当たらない(`grep -rn "checksum\|integrity" next/core/motolii-store/src` 未実施だが、persist.rs の doc に類する記述は無し)。渡した後の整合性確認は利用者の目視のみ | 【穴】意味が無い |

---

## 末尾の集計

```
全手順 117
書ける              80
【穴】入口が無い     12
【穴】意味が無い     20
【未確認】            5
```

内訳(前半/後半):

- 前半(A〜H、手順1〜76): 76手順 — 書ける 53 / 入口が無い 11 / 意味が無い 9 / 未確認 3
- 後半(I〜N、手順77〜116+84b): 41手順 — 書ける 27 / 入口が無い 1 / 意味が無い 11 / 未確認 2

### 穴ごとの `normal-map.tsv` 対応行の有無

| 穴(手順#) | 対応行あり/なし | 該当 id・備考 |
|---|---|---|
| 13 シェイプが画に出ない | 解消済み(2026-08-24) | `SetShapes` → `PreviewSnapshot.shape_documents` → `render_resolved_to_texture_with_shapes` の GPU 経路を接続。`render.rs:619-658` の回帰柵で snapshot に shape 本体が残ることを確認 |
| 14 描画ツール入口なし | あり(間接) | Pen/Rectangle Tool は各社ツールパネルの標準項目、直接名の行は本調査では特定せず |
| 15 SHAPE 寸法編集入口なし | なし | `ShapeSectionProjection` と `commit_shape_field` を新しい `inspector.shape` component として実装(台帳の抽出粒度より粗い section 単位) |
| 16-18 SHAPE section の頂点/modifier入口 | なし | 16-17 の `stage.path_edit` と #18 の `browser.shape_operator_catalog`/`shell.shape_operator_writer` component で、flat Bezier の頂点移動・開閉と7種 modifier の入口/書き戻しを解消 |
| 19 Star/Polygon 作成 | あり | AE 等の Polystar 相当メニュー項目は台帳に存在するはず(未逐一確認、逸脱として明記) |
| 20 カラーピッカー部品皆無 | **なし** | 「色をつまむウィジェットが存在するか」は製品のメニュー/ショートカット/パネル一覧に現れない実装詳細。台帳の抽出規則が構造的に持てない |
| 34-35 mask expansion 未消費 | あり | id 197 相当(mask.x)は台帳に既載、2026-08-22裁定で「不採用→採用」に回収済み(KNOWN.md) |
| 36 マスク削除の有無 | 【未確認】のため判定保留 | — |
| 40, 46 Effect種が1つのみ | あり(間接) | 個々のエフェクト名(Glow等)は台帳に載るが「複数effectを重ねられるか」という**組合せ能力**は行を持たない |
| 52 フレーム番号を直接タイプ | あり | id 1074/1075「Go to Time…」 |
| 55 次のキーフレームへジャンプ | あり | id 504/505「Next/Previous Keyframe」(採用済表示だが実装は別物 = 手順55のノート参照) |
| 67 イージングのショートカット無し | なし | 「メニュー項目はあるがショートカットが割り当たっていない」という**割当の有無**は台帳の粒度(項目の存在)と別軸 |
| 68 Graph Editor 不在 | あり | id 519「Toggle between Graph Editor and layer bar modes」 |
| 77 プレーンな Save が無い | あり | id 1224「Save (Project)」。現行mainのCmd+S/既知path上書きで結線済み |
| 81 保存成功の可視合図が無い | **なし** | 「保存に成功した合図」という状態は製品メニューに現れない |
| 88-90 ×ボタンに閉じる確認が無い | **なし** | normal-mapに対応する動詞は無いが、現行mainは`close_requests`→dirty確認へ結線済み。window driveで拒否を検分し、許可腕は`WindowCloseConfirmed(true)`にある |
| 92 再起動で続きが開かない | **なし** | `grep -i "restore\|relaunch\|last session\|where you left" next/reference/normal-map.tsv` = 0件(ヒットは無関係な「フレーム最大化/復帰」のみ) |
| 93 最近使ったファイル一覧が無い | **なし** | `grep -i "Open Recent\|Recent Project\|Recent File"` = 0件 |
| 96, 98 セッション/レイアウト復元が無い | **なし** | 同上、「見ていた場所」「パネル配置」という状態自体に対応する行が無い |
| 100-104 素材欠落の通知・relink 入口が無い | あり(id 1393 Find Missing Footage)だが**状態自体は無い** | 「回復コマンド」の行はあるが「欠けている状態」には行が無い(前半調査 trunk と完全一致するパターン) |
| 108 別マシンで絶対パス解決不能 | **なし** | 「パス解決の失敗」という状態は製品メニューに現れない |
| 109-110 フォント未解決 | あり(id 1273 Find Missing Fonts)だが**状態自体は無い** | 100-104と同型 |
| 113 Collect Files 相当機能が無い | あり | id 526「Collect Files…」 |
| 114 フォント埋め込みが無い | **なし** | 該当行を発見できず |
| 116 整合性確認手段が無い | **なし** | 該当行を発見できず |

**「対応行が無い」穴は 6群**: 20、93、96/98、108、114、116。
純粋に**幹だけが要求していて葉に名前が無い物**として際立つのは:

1. **色ピッカー部品の不在**(#20)
2. **最近使ったファイル一覧が無い**(#93)
3. **前回のセッション状態(選択・再生ヘッド・パネルレイアウト)が保存も復元もされない**(#96/98)
4. **別マシンでの絶対パス解決不能という状態そのものに名前が無い**(#108)
5. **フォント埋め込みが無い**(#114)
6. **プロジェクト受け渡し後の整合性確認手段が無い**(#116)
7. **SHAPE sectionのcolor操作が未実装で、シェイプ固有の色・線の操作を一覧化できない**(#20-23)

残る穴は、前半のSHAPE/色表現と、後半の再利用・受け渡しに分かれている。
