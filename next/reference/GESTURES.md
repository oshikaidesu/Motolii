# ジェスチャ台帳(裁定216)

裁定216(2026-08-23・利用者裁定)の成果物。各意図に**「マウスだけで完遂できる路」と
「キーボードだけで完遂できる路」の両方**が要る、という要求を裁定212(`Intent` の枝に
入口が在るかの機械判定)と**同じ形**で台帳化する — あちらが「入口が在るか」なら、
こちらは**「それぞれの手に入口が在るか」**。

## 0. 読み方

- **判定**列は4値: **両方あり** / **マウスのみ** / **キーボードのみ** / **どちらも不完全**。
  「完遂路」とは、その意図を**最初から最後まで、その入力手段だけで**達成できることを言う
  — 途中で他方が要るなら不完全(例: click→type 編集はマウスで型入力欄まで辿り着く必要が
  あるので、type 部分だけがキーボードでも「キーボードだけの完遂路」にはならない)
- **なぜこの作法か**列は先例の出典を書く。**倣った記録が見つからない物は「出典なし」と
  正直に書く**(捏造しない)。件数は §6 に集計する
- **カーソル予告**は文法 §5.5 の5状態(`next/reference/timeline-grammar.md`):
  Grab(掴める)/ Grabbing(掴んでいる)/ ResizeHorizontal(端)/ Crosshair(空白面)/
  NotAllowed(禁止・ロック中)。iced 側の型名は `mouse::Interaction::{Grab,Grabbing,
  ResizingHorizontally,Crosshair,NotAllowed}`(`grep -rn "Interaction::" next/ui` で実装確認 —
  実際に出現するのは5状態全部+既定矢印+`Pointer`(クリック可能な小物、ボタン相当)+
  `Move`/`ResizingVertically`/`ResizingDiagonally{Up,Down}`(gizmo のスケールハンドル —
  文法5状態の対象外の連続 resize 系、正当な拡張)
- この台帳は**棚卸しの網羅を主張しない**(発注書の注意どおり)。再現・追検分用に、
  使った grep をそのまま残す:
  `grep -rn "on_press(" next/ui next/shell`(38件・`tests/` 除く)/
  `grep -rn "mouse_area(" next/ui next/shell`(6件)/
  `grep -rn "Interaction::" next/ui next/shell`(62件)/
  `grep -rn "on_drag(" next/ui next/shell`(2件)/
  `grep -rn "on_double_click(" next/ui next/shell`(2件)/
  `grep -rln "pick_list(" next/ui next/shell`(3 crate: inspector-pane の
  link.rs/matte.rs/text.rs)

## 1. 主要ジェスチャ

| ジェスチャ | 仕える意図 | なぜこの作法か(先例と出典) | マウスだけの完遂路 | キーボードだけの完遂路 | カーソル予告 | 判定 |
|---|---|---|---|---|---|---|
| Inspector 数値ドラッグ(`start_field_drag`/`continue_field_drag`/`finish_field_drag`、`transform.rs`) | Position/Scale/Rotation/Opacity 等の値編集 | AE の値セル drag-to-scrub(慣習形)。Shift=1/10 精度は AE 同型(裁定216 本文が名指し) | ○(press→drag→release で1確定、Shift=微調整も掴んだまま) | **なし** — text_input へは `finish_field_drag` の `Ok(Some(field))`(=drag が動かず release=click)を経由してのみ入る。Tab 等でフィールドへフォーカス移動する経路が無い(`grep -rn "Tab" next/shell next/ui/motolii-keymap` 空振り) | ResizeHorizontal(値セル全域、`chrome.rs:382`) | **マウスのみ** |
| 同・click→type 編集(`field_input_id`+`enter_field_editing`) | 同上、正確な数値入力 | 同上(AE も click で type 欄に切替) | ○(click→type→Enter) | 不完全(値を打つのはキーボードだが、そのフィールドへ入るのに click が要る) | ResizeHorizontal→(切替後は通常の text カーソル) | **マウスのみ**(フィールド到達がマウス依存) |
| ラベル色チップ 巡回(`chrome.rs::label_color_chip`、`Message::CycleLabelColor`) | layer のラベル色を変える | 巡回ボタン文法(`next_blend_mode` と同型、BL2 に前例あり) | ○(click) | なし(`button` はキーボード到達性を持たない — §2 参照。対応 VerbId 無し) | 既定(Pointer 相当、ボタンとして hover するが明示 Interaction 配線なし) | **マウスのみ** |
| Inspector M(Hidden)トグル(`chrome.rs::mute_glyph`、`Message::ToggleHidden`) | layer を隠す | S6(可視性原理・意見6): 複数入口の一つ。Timeline rail にも同じ意味の入口がある(下記) | ○(click) | なし | 既定 | **マウスのみ** |
| Timeline rail M/S/L トグル(`rail.rs`、`Message::ToggleMute/Solo/Lock`) | 同上+Solo/Lock | 正典 §1.5「M/S/L もレーンバーで設定できる(メニュー経由だけにしない)」 | ○(click) | なし | 既定 | **マウスのみ** |
| Timeline 行選択(`rail.rs`、行全体を包む `mouse_area.on_press`) | layer 選択 | — | ○(click) | 不完全(`SelectAllLayers`/`DeselectAllLayers` はあるが**特定1行を選ぶ**キーボード動詞が無い。正典 §8.1 の `FocusRowPrev/Next` は**未実装** — `grep -rn "FocusRow" next/ui next/shell` 空振り) | 既定(行 hover は特別な予告なし) | **マウスのみ** |
| Timeline clip move(`clip_gesture.rs::moved_start`、`Hit::Bar`→`BarPart::Body`) | clip の時刻位置を変える | 正典 §2。スナップ対象=0/終端/playhead/他clip端(画面距離 SNAP_PX=7px) | ○(press→drag→release、Cmd で一時スナップ解除) | 不完全 — 正典 §8.1 に `MoveClipInToPlayhead`/`Out`([ / ])が「正へ採用」とあるが**未実装**(`grep -rn "MoveClipIn" next/ui next/shell` 空振り。VerbId::ALL に無い) | Grab(body、`input.rs`) | **マウスのみ** |
| Timeline clip trim(`clip_gesture.rs::trimmed_in_start/trimmed_out_end`、`BarPart::EdgeIn/EdgeOut`) | clip の In/Out を変える | 正典 §2。TRIM_EDGE=8px、幅24px未満は端を出さない(誤trim防止) | ○ | 不完全 — 正典 §8.1 `TrimInToPlayhead`/`TrimOutToPlayhead`(Alt+[ / ])は「AE+Unity+Unreal の三社一致」で採用されているが**未実装** | ResizeHorizontal | **マウスのみ** |
| Playhead scrub(ルーラー/空白面ドラッグ、`ruler.rs`/`input.rs`、`Message::ScrubTo`) | playhead を動かす | 正典 §1.5「ルーラ帯=時間の目印とスクラブ」 | ○(スナップ無し、画面距離のみ) | ○(`StepPlayheadForward/Back(+Fast)`・`JumpPlayheadToStart/End`・`JumpToWorkAreaStart/End` の組み合わせで任意フレームへ到達可能。**フレーム単位の連打なので実務効率は別問題**) | Crosshair(空白面、正典 §5.5 の代表例) | **両方あり** |
| JumpMeaningPointPrev/Next(`nav.rs::nearest_meaning_point`、J/K) | playhead をキー/マーカー/clip端の**厳密な**時刻へ合わせる | AE 公式(`]`/`[` と同型)。Unity/Cavalry の次/前キージャンプも同義に畳んだ(正典 §8.1) | 理論上は可能(ズームを最大まで上げてドラッグ)だが scrub にスナップが無い(上記)ため**実務上は不正確** — 正典自身が「スナップ対象にキー時刻を含めない」(裁定151 T2 KNOWN)と明記 | ○(押すだけで厳密一致) | Crosshair | **両方あり**(ただし精度の非対称を備考に残す) |
| Timeline 矩形選択(marquee) | 複数 clip/key の一括選択 | — | **未実装**(Timeline 側に marquee は無い。`grep -n "Marquee\|marquee" next/ui/motolii-timeline-pane/src` 空振り — marquee は Stage のみ) | なし | — | **どちらも不完全**(意図自体の入口が Timeline に無い。Stage には在る、下記) |
| Timeline キー(菱形)クリック選択(`key_rows.rs`、単独/Cmd トグル/Shift 範囲) | キーフレーム選択 | 正典 §3・§4「クリック=単独/Cmd=トグル/Shift=範囲」 | ○ | 不完全(`SelectAllKeysOfProperty` は property 単位の全選択のみ実装済み — **クリックで代替する専用動詞であってキーボード動詞ではない**。個々のキーを1つ選ぶキーボード動詞は無い) | Pointer(菱形上) | **マウスのみ** |
| Timeline キー(菱形)時刻ドラッグ(`key_gesture.rs::dragged_group_frame`) | キーフレームの時刻を動かす | 正典 §3・§8.1「LottieFiles と同型」(複数選択は相対間隔維持) | ○ | ○ — **`NudgeKeyframe`(Alt+←/→、Shift で±10)が正典自身「§3 時刻ドラッグのキーボード等価」と明記する designated 対**(`nav.rs`/`shell/lib.rs:5776` 実装確認) | Grabbing | **両方あり** |
| 選択キーの削除(Backspace/Delete、`shell/lib.rs:5768`→`DeleteSelectedKeys`) | 選択キーフレームを消す | — | 不完全(**削除そのものを実行するボタン・右クリックメニューが無い** — `grep -n "Button::Right" next/ui/motolii-timeline-pane/src` は3ファイルにヒットするがどれもドラッグ取消の腕であって「Delete」項目を持つ context menu ではない) | 不完全(選択自体はマウスのクリック/Shift/Cmdでしか出来ない — 特定1キーを選ぶキーボード動詞が無い、上記行と同根) | — | **どちらも不完全**(選択=マウス必須・削除実行=キーボード必須という真のハイブリッド、単独入力手段で完結する意図ではない) |
| レイヤー削除(実質 `CutLayer`、Cmd+X) | 選択レイヤーを合成から消す | — | **なし** — 削除専用ボタン・右クリックメニュー項目・drag-to-trash のいずれも無い(`grep -n "Delete\|Remove\|Clear" next/shell/src/menu.rs` 空振り) | ○(Cmd+X。ただし意味は「切り取り」でクリップボードへも積む — 純粋な「削除」語彙は無い) | — | **キーボードのみ**(かつ意味が Cut に間借りしている) |
| NudgeKeyframe(Alt+←/→、既出) | 上記キードラッグのキーボード等価 | 正典 §8.1 | ○(ドラッグが上位互換) | ○ | — | **両方あり** |
| Loop帯(作業範囲)body drag / edge drag(`ruler.rs`、`work_area.rs::dragged_area`) | 作業範囲(In/Out)を設定 | LOOP_GRAB=8px(同距離ならOut優先)。正典 §1 | ○ | 不完全 — `SetWorkAreaOut`(playhead位置へOut設定)は実装済みだが**`SetWorkAreaIn`(bare `b`)は未転写**(`ui/motolii-keymap/src/verb.rs` 冒頭コメントが明記:「bare `b`(Mark In/SetWorkAreaIn)は同じ理由で未転写」) | ResizeHorizontal(端)/Grab(本体) | **マウスのみ**(キーボードは片翼のみ) |
| Transport 再生/停止ボタン(`transport.rs::transport_button`、`Message::TogglePlayback`) | 再生トグル | — | ○(click) | ○(Space、`shell/lib.rs:6001`) | Pointer相当 | **両方あり** |
| マーカー/ロケータ ドラッグ(`markers.rs::MarkerDrag::dragged`) | マーカーの時刻を動かす | — | ○ | なし(番号マーカー Shift+0〜9 はジャンプ専用で移動ではない、正典 §8.2) | Grabbing | **マウスのみ** |
| メニュー項目(`shortcut: Some(...)`、`menu.rs`) | Undo/Redo/Copy/Paste/Cut/Duplicate/Group/Ungroup 等 | S6(可視性原理): メニューと shortcut の2本立て(`menu.rs` モジュール doc) | ○(メニューを開いてクリック — ただしメニュー自体の開閉・項目移動に矢印キー等の対応は無い、§2 参照) | ○(実装済みキー、例 Cmd+Z/Cmd+G) | 既定 | **両方あり**(ただしメニュー内はマウス駆動、キー直叩きが実質の「キーボード路」) |
| メニュー項目(`shortcut: None`、例: New Layer/Freeze/Unfreeze 等) | 同種の意図 | 「飾り shortcut 禁止」(未実装のキーを書かない規律、`menu.rs`) | ○(メニュークリック) | なし | 既定 | **マウスのみ** |
| Stage gizmo move(`gizmo.rs::move_value`) | position を変える | AE のハンドル drag(意味の手本、裁定124) | ○ | なし(Inspector の PositionX/Y へ type すれば代替できるが、そのフィールド到達自体がマウス必須 — 上記 Inspector 行と同根) | Grabbing | **マウスのみ** |
| Stage gizmo scale(`gizmo.rs::scale_value`、Shift=比率固定) | scale を変える(不動点=anchor) | AE map 680「Modify Scale constrained to aspect ratio」 | ○ | なし(同上) | Grabbing/ResizeHorizontal系(角・辺で分岐) | **マウスのみ** |
| Stage gizmo rotate(`gizmo.rs::rotation_value`、Shift=15°スナップ) | rotation を変える | AE map 679「Modify Rotation in 15° increments」 | ○ | なし | Crosshair(回転ハンドル) | **マウスのみ** |
| Stage gizmo anchor drag(pan-behind 型) | anchor を変える(見た目不動、position 補償) | AE の pan-behind 挙動 | ○ | なし | Crosshair | **マウスのみ** |
| Stage 矩形選択(`marquee.rs::apply_selection`) | 複数レイヤーの一括選択 | Figma/AE の comp panel 選択慣習形 | ○ | なし(`SelectAllLayers`/`DeselectAllLayers` はあるが範囲選択の代替にはならない) | Crosshair(`marquee.rs:523`) | **マウスのみ** |
| Stage ズーム(wheel、`zoom_at_screen_point`) | 観測カメラのズーム | Ableton/Blender 系の wheel-zoom-at-cursor 慣習 | ○ | **なし** — `zoom_step`/`NamedZoomLevel`(Zoom In/Out・Fit・100%、map 1441/1442/1490/1491)は**pure関数として実装済みだが呼び手が無い**(`grep -rn "zoom_step\|NamedZoomLevel" next/shell/src` 空振り = 意味はあるが入口ゼロ、裁定212 と同種の欠落だが Intent 系ではなく pane-local メッセージ層なので既存 check.sh の Intent 到達可能性検査には映らない) | — | **マウスのみ**(かつ意味はあるが対応する入口が実装側に無い、要フォロー) |
| Stage パン(Middle-drag、`lib.rs:599/615`) | 観測カメラの pan | Blender の Shift+MMB と同型(モジュール doc 名指し) | ○ | なし | — | **マウスのみ** |
| Browser カード ダブルクリック(`Message::CreateFromCard`) | 素材からレイヤーを作る | ダブルクリック=開く/生成の慣習(多くのファイルブラウザ・AE Project panel 系) | ○ | なし(選択済みカードに対する Enter 等化なし) | 既定 | **マウスのみ** |
| Browser カード drag→Stage/Timeline | 同上の別入口 | — | **未実装**(`lib.rs:123`「drag で Stage/Timeline へ、は将来切片(見送り)」) | — | — | 対象外(意図の入口自体が無い、見送り済みなので欠陥台帳には計上しない) |
| pane_grid 境界ドラッグ(iced 標準、`PaneGrid::spacing`+`ResizeEvent`) | パネル比率を変える | — | ○(iced 提供のまま) | **なし**(iced 自体が持たない、§2 参照) | ResizeHorizontal/Vertical相当(iced既定) | **マウスのみ** |
| pane_grid 題帯ドラッグ(`pane_title_bar`、`Draggable`) | パネルの配置を入れ替える | S6「見えない帯はつかめない」— 旧8px匿名Spaceが pick面積ゼロで死んでいた実測を受けての置換(`shell/lib.rs:4439`) | ○ | なし | Grab(`content.rs::grid_interaction` — 追加配線なしで効く) | **マウスのみ** |
| Inspector LINK/MATTE/font `pick_list` 選択(`link.rs`/`matte.rs`/`text.rs`) | 参照先・書体を選ぶ | pick_list は「次/ に前例が無い」新規採用(`text.rs` doc — BL2 は blend/mask のような小さい巡回向けで、大きい一覧には不向きと判断) | ○(click で開く、Cmd+wheel で循環) | **なし** — iced 本体の `pick_list` は press 以外で `is_open` を開かない(実測、§2) | 既定 | **マウスのみ** |
| 効果 追加/削除/並べ替え/bypass(`effects.rs`、`Message::AddEffect/RemoveEffect/MoveEffectUp/Down/ToggleEffectBypass`) | エフェクトの管理 | — | ○(click) | なし | 既定 | **マウスのみ** |
| Mask/Matte/Blend の巡回ボタン(`CycleMaskMode`/`CycleMatteMode`/`CycleBlendMode`) | 列挙値を巡回 | 巡回ボタン文法(共通形) | ○(click) | なし | 既定 | **マウスのみ** |

## 2. iced が提供している物の一覧(裁定215「借りる」の確認)

`~/.cargo/git/checkouts/iced-*/…/widget/src/` を実測(rev `73e686ee05efd7d1b61cfea2647186b336d9ab9c`、`next/Cargo.toml` 固定先):

| widget/機構 | 提供する物 | キーボード完遂性 |
|---|---|---|
| `iced::widget::button` | `on_press`(mouse press + touch)。**フォーカス・キーボード活性化のコードが無い**(`grep -n "Key::Named\|is_focused\|focus" widget/src/button.rs` 空振り) | **無し** — Motolii の全 click-only ボタン(M/S/L・キー菱形・ラベル色チップ・巡回3種・効果の追加/削除/並べ替え/bypass 等、`on_press(` 38件の大半)がこの1点に還元される |
| `iced::widget::mouse_area` | press/release/move の生 event capture・`interaction()` でカーソル上書き | 無し(そもそもキーボード概念を持たない層) |
| `iced::widget::pick_list` | click で開閉、開いた状態で Cmd+wheel 循環(`widget/src/pick_list.rs:472-517`) | **無し** — Enter/矢印キーでの開閉・選択が無い(iced_aw::number_input の欠落と同型の穴。裁定216 が名指しした #299 は number_input 固有の bug だが、pick_list は最初から仕様として持たない) |
| `iced::widget::pane_grid` | split/resize(`ResizeEvent`)・`title_bar` を介した `Draggable`・pick 面積 hover で自動 `Interaction::Grab` | **無し**(`grep -n "Key\|keyboard" widget/src/pane_grid.rs` 空振り) — resize もパネル入替も keyboard 経路が存在しない |
| `iced::widget::operation::focusable`(`focus_next`/`focus_previous`) | Tab 相当のフォーカス移動を**アプリ側が呼べば**動く一般 operation(`examples/todos`/`toast`/`modal` で実演) | **提供はされているが Motolii は未配線**(`grep -rn "Tab" next/shell next/ui/motolii-keymap` 空振り)。これを配線すれば Inspector 数値欄・pick_list 等へのキーボード到達を作れる可能性があるが、要調査(pick_list 自体が focus 後も Enter で開かない、上記) |
| `iced::widget::text_input` | 文字入力・カーソル移動は標準対応。Tab ハンドリングは自前で持たない(`grep -n "Tab" widget/src/text_input.rs` 空振り) | 一度フォーカスが当たれば入力は完結するが、フォーカスを**当てる**手段が上記の理由で無い |
| `iced_aw::number_input` | ± ボタン付き数値欄 | **不採用**(裁定216 が実測: ドラッグ非対応・[#299](https://github.com/iced-rs/iced_aw/issues/299) でキーボード編集不可) |

**結論**: iced/iced_aw のどちらも「ボタン・ドロップダウン・パネル境界をキーボードだけで操作する」経路を無償で提供しない。裁定215 の意味で言えば、この領域は**上流不在が実際に確かめられた(b)**——Motolii が持つなら、Tab フォーカス配線+各 widget の Enter/矢印キー対応という**意見**を新たに持つ必要がある(コスト大、現状は未着手)。

## 3. マウスのみの一覧(埋めるべき穴・全体)

§1 の個別行に加え、**同型の穴が systemic**(`iced::widget::button` がキーボード到達性を
持たないため、対応する `VerbId` が無い click-only ボタンは全て同じ穴を持つ):

- Inspector: 数値ドラッグ・click→type 入口・ラベル色チップ・M(Hidden)・LINK/MATTE clear・
  効果 追加/削除/並べ替え/bypass・Mask/Matte/Blend 巡回・font pick_list
- Timeline: rail M/S/L・行選択・clip move/trim・マーカードラッグ・Loop帯・
  `shortcut: None` のメニュー項目
- Stage: gizmo move/scale/rotate/anchor・矩形選択・wheel ズーム・middle-drag パン
- Browser: カード ダブルクリック
- shell: pane_grid 境界ドラッグ・pane_grid 題帯ドラッグ

## 4. キーボードのみの一覧

- **レイヤー削除**(Cmd+X = CutLayer による代用。専用の Delete 動詞・マウス側の削除
  入口がどちらも無い)

見つかったのは実質この1件だけ。裁定151/146/§8.1 が「正へ採用」と書いた多くのキーボード
動詞(MoveClipInToPlayhead/TrimInToPlayhead/FocusRowPrev 等)は**未実装**なので、
「キーボードにしか無い」ではなく「まだどちらにも無い」(§1 のマウスのみ判定側に計上)。

## 5. どちらも不完全な一覧

- **Timeline 矩形選択**: 実装が無い(Stage にはある)。マウス・キーボードどちらの入口も
  意図そのものに存在しない
- **選択キーの削除**: 個々のキーを選ぶのはマウス(クリック/Shift/Cmd)必須・削除の実行は
  キーボード(Backspace/Delete)必須。どちらか一方だけでは完結しない、真のハイブリッド
  依存(削除ボタン/右クリックメニューが無いため)
- **作業範囲(Loop帯)の設定**: マウスは In/Out 両方を扱えるが、キーボードは
  `SetWorkAreaOut` のみ実装で `SetWorkAreaIn` が未転写(片翼)

## 6. 出典なしの件数

§1 の35行中、Browser drag→Stage/Timeline(「見送り」という既存決定があるため対象外・
出典の要否自体が無い)を除いた**34行**で数える。「なぜこの作法か」列に具体的な先例名
(AE/Figma/Blender/Ableton/S6/正典の節番号等)を書けたのは**26行**。
**出典なしと書いた行は8件**:

1. Timeline 行選択(`rail.rs` の `mouse_area.on_press`)
2. Timeline 矩形選択(実装が無いこと自体が理由 — 倣うべき自分の実装が無い)
3. 選択キーの削除(削除の実行そのものに先例語彙を引いていない)
4. レイヤー削除(Cmd+X 間借りの是非に先例を引いていない)
5. Transport 再生/停止ボタン(Space 割当自体は他のどの1製品を指すか明記していない)
6. マーカー/ロケータ ドラッグ
7. pane_grid 境界ドラッグ(iced 自身の resize 機構であって Motolii 側の意匠選択が無い)
8. 効果 追加/削除/並べ替え/bypass ボタン列

**捏造しない**ことを優先し、上記は正直に空欄(`—`)のまま残した。

## 7. pane 間で作法が割れている箇所

- **スナップの有無/切替**: Timeline の clip/key ドラッグは既定 ON+Cmd で一時解除という
  一貫した文法(正典 §8.2)を持つが、**Stage の gizmo move には対応する概念が無い**
  (他レイヤーの位置・comp境界へのスナップ/アライメントガイドが実装されていない —
  `grep -n "snap\|Snap" next/ui/motolii-stage-pane/src/gizmo.rs` は rotate の15°定数
  1件のみ)。時間軸(1次元)と空間(2次元)という領域の違いはあるが、「掴んで動かす」の
  作法として利用者が学習した Cmd トグルの直感は Stage には持ち込めない
- **Shift の意味**: Inspector 数値ドラッグ=1/10精度、Stage scale=比率固定、
  Stage rotate=15°スナップ、Timeline キー選択=範囲選択、Timeline Step/Nudge=歩幅×10。
  いずれも個別の先例(AE map 679/680・裁定216本文)が引けており、**ジェスチャの種類が
  違う(drag と click と key-repeat)ので同一キーの多義自体は正典 §5.5「場所で意味を
  変えない」の対象外**(あれはホイールの話) — 一意図多実装ではなく想定内の分岐と判断。
  ただし利用者が実機で触った時に混乱しないかは実測待ち(器具化していない)
- **M(Hidden)の入口が2箇所**(Inspector M glyph・Timeline rail M glyph): 見た目・
  widget 実装は別コード(2つの crate、別の glyph 関数)だが、**書き込み先の
  `toggle_layer_hidden` は shell に1つだけ共有されている**
  (`ui/motolii-timeline-pane/src/write.rs` 冒頭 doc「Inspector の
  `Message::InspectorToggleHidden` とも共有される Shell 側の汎用ヘルパー」)。
  これは意見6(可視性原理)が要求する複数入口そのものであり、**振る舞いが割れている
  実装ではない**(誤検出を避けるため明記)。一方 **Solo/Lock は Timeline rail にしか
  入口が無く**、Inspector 側には対応する glyph が無い(`grep -n "ToggleSolo\|ToggleLock"
  next/ui/motolii-inspector-pane/src` 空振り) — S6 の観点では Inspector 側が
  Solo/Lock の第二入口を欠いている
- **矩形選択の開始閾値**: 正典 §7 の未決6「矩形選択の開始閾値(明示3px かegui既定か)
  — 実装時に決めて追記(小)」がまだ書かれていない。Stage marquee は実装済みだが
  閾値の出典がコード上に見当たらない(`marquee.rs` に閾値定数の grep ヒットなし)。
  Timeline 側は marquee 自体が無いので比較すら出来ない
- **playhead の精度**: ルーラー scrub はスナップ無しの生ピクセル、
  JumpMeaningPointPrev/Next はフレーム厳密。同じ「playhead を動かす」意図の中で
  精度の質が違う入口が2本あり、後者だけがキーボード完備という非対称がある(§1 参照)

## 8. check.sh

`裁定212 の Intent 到達可能性検査と同じ形(情報表示のみ・fail させない)`の節を
足すことを検討したが、**足さなかった**。理由:

- 裁定212 が機械化できたのは、`Intent` が**閉じた単一の enum**で、判定が
  「`Intent::Variant` という実在識別子が呼び手コードに出現するか」という
  **構文一致**に還元できたからである(`enum` の枝という有限で名前の付いた集合)
- 本台帳の判定(両方あり/マウスのみ/キーボードのみ/どちらも不完全)は、
  「この `on_press` に対応する `VerbId` が意味的に同じ動作を指すか」という
  **意味対応**の判断を要る。`NudgeKeyframe` が「キー時刻ドラッグの designated
  キーボード等価である」ことはコード上のコメントには書いてあるが、機械的に
  「この gesture とこの verb は同一意図」と紐付ける閉じた語彙(裁定212 の
  `Intent` に相当する物)が UI 操作側には無い — `Message` enum は pane ごとに
  別々かつ非公開の粒度(`BarGrabbed`/`ScrubTo`/`NudgeKeyframe` 等)で、
  「意図」という上位概念に集約されていない
- Inspector 欄監査(裁定214、`check.sh` 該当節のコメント参照)で同じ理由により
  「機械化できるのは見出し数値と本文行数の一致(転記事故)だけ」と結論した先例と
  **同じ壁**にここでも当たる。台帳の**自己整合性**(表の行数と宣言件数が一致するか、
  判定列が4値の外に出ていないか)だけなら機械化できるが、それは「穴が実際に埋まって
  いるか」を検査しない ── 今のところそこまでの価値は薄いと判断し、見送った
  (足さない判断も正当という発注書の指示どおり)

## 9. 逸脱

- 発注書は「代表例」として mask 追加・drag&drop 等も棚卸し対象に挙げていたが、
  **mask のシェイプ頂点編集は未実装**(`grep -n "vertex\|Vertex\|path_point"
  next/ui/motolii-stage-pane next/ui/motolii-inspector-pane/src/mask.rs` 空振り)
  なので該当ジェスチャが存在せず、台帳に計上していない
- Settings pane・menubar の click-only ボタン群(`BackgroundPreset` 等)は
  §2/§3 の「systemic な穴」に一般化して回収し、個別行を割いていない
  (§1 の各行は代表例であって全 38 件の `on_press` を1行ずつ並べてはいない)
