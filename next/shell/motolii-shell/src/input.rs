


use crate::{
    export_pane, inspector_pane, stage, timeline,
    timeline_pane, Message, Shell,
};

impl Shell {
    /// map 1441「Zoom In」(A10 id1441、`resolve_navigation_key` の Cmd+=/+ 腕
    /// から呼ばれる)。**API 分析の根拠**(裁定199):
    /// [`stage::zoom::zoom_step`] のシグネチャは
    /// `fn zoom_step(current: ObservationCamera, direction: ZoomStepDirection)
    /// -> ObservationCamera`(pure、bounds 不要 — `ZOOM_LADDER` 上の隣接段への
    /// 純関数ジャンプ)。`current` には `self.observation`(`None` = 「render
    /// camera を通して見る」既定、`update_stage` の `stage::Message::Observe`
    /// と同じ書き先)を渡す — `None` の間は `ObservationCamera::default()`
    /// (`zoom=1.0`)を起点にするのが正しい(既定ズームは1.0のラダー中央、
    /// `zoom.rs` 冒頭 doc「1.0を中心に2のべき倍」)。結果は必ず `Some` へ積む
    /// (`ZoomIn`/`ZoomOut` を押した時点で利用者は明示的に自由視点へ入る —
    /// `ResetToRenderCamera`(Shift+F)が `None` へ戻す経路として既に在る)。
    pub(crate) fn zoom_in(&mut self) {
        let current = self.observation.unwrap_or_default();
        self.observation = Some(stage::zoom::zoom_step(
            current,
            stage::zoom::ZoomStepDirection::In,
        ));
    }

    /// map 1442「Zoom Out」(A10 id1442)。[`Self::zoom_in`] の doc と同じ根拠、
    /// `ZoomStepDirection::Out` を渡すだけ。
    pub(crate) fn zoom_out(&mut self) {
        let current = self.observation.unwrap_or_default();
        self.observation = Some(stage::zoom::zoom_step(
            current,
            stage::zoom::ZoomStepDirection::Out,
        ));
    }

    /// map 1491/1492「Zoom to fit」(A10 id1491)。**API 分析の根拠**:
    /// [`stage::zoom::named_zoom_level`] は `NamedZoomLevel::Fit` の枝を
    /// `bounds`/`comp` に触れる前に早期 return する
    /// (`if level == NamedZoomLevel::Fit { return Some(ObservationCamera::
    /// default()); }`、`zoom.rs` 実測)——つまり Fit は常に
    /// `ObservationCamera::default()` で、bounds に依存しない。`Shell` は
    /// Stage pane の描画時 bounds を保持していない(`iced::Rectangle` は
    /// `stage_presenter.rs`/`view.rs` の draw 呼び出し引数としてしか出て
    /// 来ない、`grep -n "Rectangle" src/*.rs` で `Shell` フィールドに無い
    /// ことを確認済み)ので、ダミーの bounds/comp を渡して関数を呼ぶより、
    /// 実際に返る値(`ObservationCamera::default()`)を直接代入する方が
    /// 「渡した引数が意味を持つふりをしない」ぶん正直——関数の**契約**
    /// (Fit の戻り値)はそのまま使っている。id1490「Zoom to 100%」
    /// (`NamedZoomLevel::ActualSize`)は `actual_size_zoom` 経由で本物の
    /// bounds が要るため、この波では結線しない(RETURN 参照)。
    pub(crate) fn zoom_to_fit(&mut self) {
        self.observation = Some(motolii_engine::ObservationCamera::default());
    }
}


/// `Shell::subscription` が使う、Inspector drag-to-scrub 用の window 全体の
/// 事象フィルタ。**翻訳だけ**(`subscription()` 冒頭の規律どおり、判断は持たない)
/// — 実際に drag 中かどうかの判断・Shift の要否は `Shell::update` 側
/// (`inspector_drag`/`keyboard_modifiers` の状態)。
///
/// `iced::event::listen_with` を選んだ理由: `status` を見ずに常に拾える
/// (`iced::keyboard::listen()`(Ignored 限定)だと、typing 中の text_input は
/// Escape を自分で `shell.capture_event()` する(`iced_widget::text_input`
/// 実測)ので、typing の Esc-cancel に使いたい場合に届かなくなる)。既存の
/// Escape/Backspace/Delete/NudgeKeyframe/ResetToRenderCamera はその方針どおり
/// `status` を無視する。**playhead ナビゲーション動詞束(U2)だけは逆**
/// — `resolve_navigation_key` へ `status == Captured` を渡し、text_input が
/// 既にそのキーを消費していれば一切出さない(正典 §5「テキスト入力中は
/// 横取りしない」。Home/End/裸の j/k/i/o は text_input 内でもカーソル移動・
/// 文字入力として意味を持つので、Escape 系と同じ「常に拾う」は採らない)。
pub fn inspector_pointer_event(
    event: iced::Event,
    status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::Inspector(inspector_pane::Message::PointerMoved(position)))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::Inspector(inspector_pane::Message::PointerReleased))
        }
        iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::KeyboardModifiersChanged(modifiers))
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        }) => Some(Message::EscapePressed),
        // Timeline のキー削除(正典 §3)。**Mac の「Delete」キーラベルは
        // `Named::Backspace` として届く**(`iced_core::keyboard::key` の doc
        // コメント実測 — 主部の物理キーは Backspace、`Named::Delete` は
        // `Fn+Delete`/外付けキーボードの forward-delete)。両方拾う —
        // `Shell::delete_selected_keys` は選択が空なら no-op なので、text
        // 編集中に Backspace で文字を消す操作とは(選択キーが無い限り)衝突
        // しない。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace),
            ..
        })
        | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete),
            ..
        }) => Some(Message::Timeline(timeline_pane::Message::DeleteSelectedKeys)),
        // NudgeKeyframe(正典 §8.1)。**既定割当は仮**(拘束6・裁定146の隣接注記
        // どおり、キーの皮は keymap 層が無い今だけ直結) — アクション名
        // (`timeline_pane::Message::NudgeKeyframe`)だけを正本として残す。
        // Alt+←/→=1フレーム、Alt+Shift+←/→=10フレーム。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
            modifiers,
            ..
        }) if modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(-step)))
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
            modifiers,
            ..
        }) if modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(step)))
        }
        // ResetToRenderCamera(裁定157・EXACT TARGET 1「カメラへ戻るは1アクション」)。
        // **既定割当は仮**(NudgeKeyframe と同じ「keymap 層が無い今だけ直結」の
        // 注記どおり) — アクション名(`stage::Message::ResetToRenderCamera`)だけを
        // 正本として残す。Shift+F。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.shift()
                && matches!(key.as_ref(), iced::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("f")) =>
        {
            Some(Message::Stage(stage::Message::ResetToRenderCamera))
        }
        // playhead ナビゲーション動詞束(U2、正典 §5・§8.1)。**この分岐だけ
        // `status` を見る**(上の doc 参照)— キーそのものの解決は
        // `resolve_navigation_key` へ委譲する(試験(`tests/suite/nav_drive.rs`)
        // が `iced::Event`/`Status` を毎回組み立てずにその関数を直接叩ける
        // ようにするための分割。既存の Alt+Arrow(NudgeKeyframe)/Shift+F
        // (ResetToRenderCamera)は上の枝で先に確定しているのでここには落ちて
        // 来ない)。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            resolve_navigation_key(&key, modifiers, status == iced::event::Status::Captured)
        }
        _ => None,
    }
}

/// U2: playhead ナビゲーション動詞束のキー解決(正典 §5・§8.1)。**pure** —
/// `key`/`modifiers`/`captured`(他の widget が既にこのキーを消費したか、
/// `inspector_pointer_event` の `status == Captured` をそのまま渡す)だけを見て
/// `Message` を返す。`inspector_pointer_event` から分離してあるのは、試験が
/// `iced::Event`/`iced::event::Status` を毎回組み立てずにここを直接叩ける
/// ようにするため(`tests/suite/nav_drive.rs` の (e)/(f))。
///
/// - `captured` の間は何も返さない(正典 §5「テキスト入力中は横取りしない」)。
///   Home/End/裸の j/k/l/i/o/b/n/,/. は text_input 内でもカーソル移動・文字入力
///   として意味を持つキーなので、renaming/InspectorField 編集中に奪ってはいけない
/// - j/k/l/i/o/b/n/,/. は裸キー — `modifiers.command()` が立っていれば何も
///   返さない(前任レーンの実測教訓: Cmd+O 等の既存/将来ショートカットを
///   奪わない)。**第5波結線(B21+B18、supervisor 裁定・拘束6の仮置き)**:
///   J/K/L = シャトル(逆/停止/順)、旧 J/K(意味点ジャンプ)は `,`/`.` へ、
///   旧 L(ループトグル、正典 §5 既定)は Cmd+L へ移設。B/N = Set Work Area
///   In/Out、Shift+Home/End = 作業範囲の先頭/末尾へ
/// - ←/→ は Alt 修飾時は対象外(`NudgeKeyframe` が既に使っている、二重発火
///   防止)
/// - Cmd+Z/Cmd+Shift+Z(Undo/Redo)・Cmd+C/V/X/D(Copy/Paste/Cut/Duplicate)・
///   Cmd+A/Cmd+Shift+A(SelectAll/DeselectAll)は S0 段差 群0(κ 台帳 FINDING 1)
///   で追加した編集ショートカット腕 — 対応する `Message` は既に実装・テスト済み
///   だったが、この関数に腕が無かったため UI からは header の Undo/Redo ボタン
///   経由でしか届かなかった
/// - Cmd+N/Cmd+Shift+S/Cmd+Q(New Project/Save As/Quit)は MB-1(裁定176)で
///   足した File 束 — `menu.rs::file_items` と同じ4動詞・同じ割当(Save a
///   Copy はメニューのみ、shortcut 出典ゼロ)
///
/// **既定割当は仮**(拘束6・NudgeKeyframe と同じ「keymap 層が無い今だけ直結」
/// の注記どおり) — アクション名(`Message::StepPlayhead`/`JumpPlayheadToStart`/
/// `JumpPlayheadToEnd`/`JumpMeaningPoint`/`JumpClipEdge`/`Message::Undo`/`Redo`/
/// `CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/`SelectAllLayers`/
/// `DeselectAllLayers`/`NewProjectRequested`/`SaveAsRequested`/
/// `QuitRequested`)だけを正本として残す。
pub fn resolve_navigation_key(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
    captured: bool,
) -> Option<Message> {
    if captured {
        return None;
    }
    use iced::keyboard::key::Named;
    use iced::keyboard::Key;
    match key.as_ref() {
        // Step Forward/Back(素=1フレーム、Shift=10フレーム)。Alt 付きは
        // NudgeKeyframe の領分なのでここでは扱わない(上の枝が先に取る)。
        Key::Named(Named::ArrowLeft) if !modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::StepPlayhead(-step))
        }
        Key::Named(Named::ArrowRight) if !modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::StepPlayhead(step))
        }
        // 作業範囲の先頭/末尾へ(map 1064、B18 第5波結線 — supervisor 裁定の
        // 仮置き、拘束6): Shift+Home/End。素の Home/End(comp 先頭/末尾)より
        // 先に見る(修飾が厳しい方が先 — Undo/Redo の Shift 振り分けと同じ形)。
        Key::Named(Named::Home) if modifiers.shift() => Some(Message::JumpToWorkAreaStart),
        Key::Named(Named::End) if modifiers.shift() => Some(Message::JumpToWorkAreaEnd),
        Key::Named(Named::Home) => Some(Message::JumpPlayheadToStart),
        Key::Named(Named::End) => Some(Message::JumpPlayheadToEnd),
        // ---- JKL シャトル(B21 第5波結線、map 1097/1098・1100/1101・
        // 1125-1127・1135/1136 — supervisor 裁定済み・拘束6の仮置き)。J/K/L =
        // 逆/停止/順は Premiere/Resolve/AE 共通の慣習(S0 辞書式)。旧割当の
        // J/K(意味点ジャンプ)は `,`/`.` へ移設(下記)。Shift+J/L(map
        // 1056/1058 Fast Forward/Reverse)は「連打相当」(`shuttle.rs` doc —
        // keymap 層が同じ Forward/Reverse で解決する想定)なので Shift では
        // 分岐しない — 同じ Message が倍率を進める。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("j") => {
            Some(Message::Timeline(timeline_pane::Message::Shuttle(
                timeline_pane::ShuttleCommand::Reverse,
            )))
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("k") => {
            Some(Message::Timeline(timeline_pane::Message::Shuttle(
                timeline_pane::ShuttleCommand::Stop,
            )))
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("l") => {
            Some(Message::Timeline(timeline_pane::Message::Shuttle(
                timeline_pane::ShuttleCommand::Forward,
            )))
        }
        // JumpPrev/NextMeaningPoint。Shift 付きで選択レイヤー限定(§8.1)。
        // **旧 J/K から `,`/`.` へ移設**(B21 第5波結線 — JKL をシャトルへ
        // 譲る supervisor 裁定。`,`/`.` は Shift で `<`/`>` を運ぶ同じ物理キー
        // なので両方の字を受ける — layer_only の判定は modifiers 側)。
        Key::Character(c) if !modifiers.command() && (c == "," || c == "<") => {
            Some(Message::JumpMeaningPoint {
                direction: timeline::nav::JumpDirection::Prev,
                layer_only: modifiers.shift(),
            })
        }
        Key::Character(c) if !modifiers.command() && (c == "." || c == ">") => {
            Some(Message::JumpMeaningPoint {
                direction: timeline::nav::JumpDirection::Next,
                layer_only: modifiers.shift(),
            })
        }
        // JumpToClipIn/Out。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("i") => {
            Some(Message::JumpClipEdge(timeline::nav::ClipEdge::In))
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("o") => {
            Some(Message::JumpClipEdge(timeline::nav::ClipEdge::Out))
        }
        // Mark In/Out = 作業範囲の In/Out を playhead へ(map 725/726・296/297、
        // B18 第5波結線 — supervisor 裁定: B/N)。`!modifiers.command()` が
        // Cmd+N(New Project)/将来の Cmd+B を守る(Cmd 系は下の File 束が取る)。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("b") => {
            Some(Message::Timeline(timeline_pane::Message::SetWorkAreaIn))
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("n") => {
            Some(Message::Timeline(timeline_pane::Message::SetWorkAreaOut))
        }
        // ループトグル(map 1082/1083)。**旧割当 L(正典 §5 の既定)はシャトル
        // (上記)へ譲り、Cmd+L へ移設**(supervisor 裁定・拘束6の仮置き)。
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("l") => {
            Some(Message::Timeline(timeline_pane::Message::ToggleLoop))
        }
        // ---- 編集ショートカット(S0 段差 群0、κ 台帳 FINDING 1)。`Message::Undo`/
        // `Redo`/`CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/
        // `SelectAllLayers`/`DeselectAllLayers` は実装・テスト済みだったのに UI 入口
        // (キー)が1本も無かった(header の Undo/Redo ボタンのみ) — ここへ Cmd+文字 の
        // 腕を足して消化する。**既定割当は仮**(上の j/k/i/o と同じ「keymap 層が
        // 無い今だけ直結」の注記どおり、拘束6)。Shift の有無で Undo/Redo・
        // SelectAll/DeselectAll を振り分ける(NudgeKeyframe の歩幅振り分けと同じ形)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("z") => {
            Some(Message::Undo)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("z") => {
            Some(Message::Redo)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("c") => {
            Some(Message::CopyLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("v") => {
            Some(Message::PasteLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("x") => {
            Some(Message::CutLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("d") => {
            Some(Message::DuplicateLayer)
        }
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("a") => {
            Some(Message::SelectAllLayers)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("a") => {
            Some(Message::DeselectAllLayers)
        }
        // ---- File 束(MB-1、裁定176)。メニュー(`menu.rs::file_items`)と
        // 同じ4動詞・同じ割当 — S6 併存(発注書「メニューと shortcut を同切片
        // で併設する義務」、M-menu 調査の該当4行は着手前は入口ゼロだった)。
        // `!modifiers.shift()` は Undo/SelectAll と同じ「将来の Cmd+Shift+N 系
        // に予約を残す」防衛ガード(KNOWN.md の Cmd+O 教訓と同じ理由 —
        // 修飾キーを厳密にしないと後から足す動詞と衝突する)。Save a Copy は
        // normal-map の shortcut 出典がゼロなのでキーを発明しない
        // (`menu.rs::file_items` doc 参照)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("n") => {
            Some(Message::NewProjectRequested)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("s") => {
            Some(Message::SaveAsRequested)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("q") => {
            Some(Message::QuitRequested)
        }
        // ---- G1 グループ化動詞(裁定174)。Undo/Redo・SelectAll/DeselectAll と
        // 同じ Shift 振り分けの形(既定割当は仮、上の注記どおり)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("g") => {
            Some(Message::GroupLayers)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("g") => {
            Some(Message::UngroupLayers)
        }
        // Play/Pause(A2、正典 §2 拘束5)。`captured`(text_input 入力中)なら
        // 上の早期returnで既に弾かれている — typing 中の Space は普通の
        // スペース文字入力のまま(playback を奪わない)。ドラッグ中かどうかの
        // 判断(拘束5)はここでは持たない — `Shell::toggle_playback` 側
        // (`is_dragging()`)。
        Key::Named(Named::Space) => Some(Message::TogglePlayback),

        // ---- 第6波 shell 結線(2026-08-22 発注) ----
        // マーカー keymap(B19、`timeline::markers` 冒頭 doc 統合手順5
        // 「keymap M=AddAtPlayhead」)。`!modifiers.command()` は j/k/i/o と
        // 同じ「将来の Cmd+M に予約を残す」防衛ガード。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("m") => {
            Some(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead))
        }
        // rename 開始(正典 §6、`timeline::write` 冒頭 doc「Enter(単一選択)=
        // RenameBegin」)。実際の選択解決(`LayerId`)は `resolve_navigation_key`
        // が選択を知らないため `Shell::update` 側(`Message::RenameSelectedLayer`
        // 腕)が行う。
        Key::Named(Named::Enter) => Some(Message::RenameSelectedLayer),
        // Stage 合成順の並べ替え(B44、`timeline::write::Message::RestackLayer`
        // — 意味(`restack_layers`)は既に `PaneState::update` 側で完結して
        // いる、`Message::Timeline` の「other」経路がそのまま拾う。ここは
        // keymap の口を1本足すだけ)。Cmd+Alt+↑/↓ = 1歩前面/背面、+Shift =
        // 最前面/最背面(`StackDirection` の4 variant にそのまま対応)。
        Key::Named(Named::ArrowUp) if modifiers.command() && modifiers.alt() && !modifiers.shift() => {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::Forward,
            )))
        }
        Key::Named(Named::ArrowDown) if modifiers.command() && modifiers.alt() && !modifiers.shift() => {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::Backward,
            )))
        }
        Key::Named(Named::ArrowUp) if modifiers.command() && modifiers.alt() && modifiers.shift() => {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::ToFront,
            )))
        }
        Key::Named(Named::ArrowDown) if modifiers.command() && modifiers.alt() && modifiers.shift() => {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::ToBack,
            )))
        }
        // Export ダイアログ(B09、`export_pane` crate doc「shell 結線」・
        // map 529「Ctrl+E = Export」消化 — Mac 表記は Cmd+E)。
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("e") => {
            Some(Message::Export(export_pane::Message::ToggleExportDialog))
        }
        // ---- Stage 離散ズーム束(B24、A10 id1441/1442/1491 の結線 — 第7波)。
        // Cmd+=/Cmd+-(ブラウザ/多くのエディタ共通の zoom in/out 慣習、`=` は
        // US 配列で無 shift の「+」キー — `+` も一緒に拾って Shift 有無の
        // レイアウト差を吸収する)。Cmd+9 = Zoom to fit(Illustrator/Sketch の
        // 「Fit in Window」慣習に合わせる — 修飾digit は shift でグリフが
        // 変わる配列があるため Shift 系のキーは避けた、`zoom_to_fit` doc
        // 参照)。id1490(Zoom to 100%)はキーを発明しない(未結線、RETURN
        // 参照)。
        Key::Character(c) if modifiers.command() && (c == "=" || c == "+") => {
            Some(Message::ZoomIn)
        }
        Key::Character(c) if modifiers.command() && c == "-" => Some(Message::ZoomOut),
        Key::Character(c) if modifiers.command() && c == "9" => Some(Message::ZoomToFit),
        _ => None,
    }
}

