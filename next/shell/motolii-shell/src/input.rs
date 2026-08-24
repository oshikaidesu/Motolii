//! responsibility: wire
//!
//! OS/icedの入力をdomain Messageへ翻訳するWIRE層。ズームの意味計算は
//! `motolii-stage-pane::zoom`へ委譲し、ここはShellの状態へ接続する。

use crate::{export_pane, inspector_pane, stage, timeline, timeline_pane, Message, Shell};

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
/// 実測)ので、typing の Esc-cancel に使いたい場合に届かなくなる)。Escape/
/// NudgeKeyframe/ResetToRenderCamera はその方針どおり `status` を無視する
/// (Escape は text_input 編集中でも Esc-cancel として意味を持つ望ましい
/// 二重発火、Alt+Arrow/Shift+F は text_input 内で衝突する意味を持たない)。
/// **Backspace/Delete(キー優先の文脈付き削除)と playhead ナビゲーション動詞束
/// (U2)は逆** — 前者は `status != Captured` をこの関数内で直接見て
/// (2026-08-23 修正、下記コメント参照)、後者は `resolve_navigation_key` へ
/// `status == Captured` を渡す。どちらも text_input が既にそのキーを
/// 消費していれば Document 側を一切発火しない(正典 §5「テキスト入力中は
/// 横取りしない」・M20)。Home/End/裸の j/k/i/o は text_input 内でも
/// カーソル移動・文字入力として意味を持つので、Escape 系と同じ「常に拾う」
/// は採らない。
pub fn inspector_pointer_event(
    event: iced::Event,
    status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(
            Message::Inspector(inspector_pane::Message::PointerMoved(position)),
        ),
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
        // 選択削除の要求(正典 §3)。**Mac の「Delete」キーラベルは
        // `Named::Backspace` として届く**(`iced_core::keyboard::key` の doc
        // コメント実測 — 主部の物理キーは Backspace、`Named::Delete` は
        // `Fn+Delete`/外付けキーボードの forward-delete)。両方拾う。
        //
        // **`status` を見る**(Cmd+Z 等の編集ショートカット腕と同じ形 —
        // `resolve_navigation_key` の doc 参照)。**旧実装は `captured` を
        // 一切見ていなかった**が、これは事情でなく漏れだった: fork
        // `core/src/text/input.rs:181`(`apply` 内 `editor::Update::Action`
        // 腕)が `editor.perform(action); shell.capture_event();` を
        // **`Edit::Backspace` を含む全 `Action::Edit` に対して無条件に**
        // 呼ぶ(Cmd+Z の `Update::Custom`/Copy/Paste と同じ
        // `shell.capture_event()` 経路 — 実測: 分岐先で action 種別による
        // 除外は無い)。つまり text_input にフォーカスがある間、Backspace は
        // 既に iced 側で `Status::Captured` になっている。旧 doc の
        // 「`delete_selected_keys` は選択が空なら no-op」という正当化は、
        // **選択中のキーフレームがある場合には成立しない**(AX-5 実測 —
        // Timeline でキー選択したまま rename の text_input で Backspace を
        // 押すと文字削除とキー削除が同時に起きる)。M20「TextInput 中は
        // テキスト優先」に従い、captured の間は Document 側を発火しない
        // (判断が割れたら厳しい側 — 消えるより消えない方が安全)。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace),
            ..
        })
        | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete),
            ..
        }) if status != iced::event::Status::Captured => Some(Message::DeleteSelectionRequested),
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
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(
                -step,
            )))
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
            modifiers,
            ..
        }) if modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(
                step,
            )))
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
/// `DeselectAllLayers`/`NewProjectRequested`/`SaveRequested`/`SaveAsRequested`/
/// `QuitRequested`)だけを正本として残す。
pub fn resolve_navigation_key(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
    captured: bool,
) -> Option<Message> {
    if captured {
        return None;
    }
    if let Some(message) = resolve_easing_shortcut(key, modifiers) {
        return Some(message);
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
        Key::Character(c)
            if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("z") =>
        {
            Some(Message::Undo)
        }
        Key::Character(c)
            if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("z") =>
        {
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
        Key::Character(c)
            if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("a") =>
        {
            Some(Message::SelectAllLayers)
        }
        Key::Character(c)
            if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("a") =>
        {
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
        Key::Character(c)
            if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("n") =>
        {
            Some(Message::NewProjectRequested)
        }
        Key::Character(c)
            if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("s") =>
        {
            Some(Message::SaveAsRequested)
        }
        // Cmd+S = 平の上書き保存(裁定150: 4製品とも「一度保存したらパスを聞かない」)。
        // Shift 付きの Save As と同じ振り分けの形で、`!modifiers.shift()` を明示する。
        Key::Character(c)
            if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("s") =>
        {
            Some(Message::SaveRequested)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("q") => {
            Some(Message::QuitRequested)
        }
        // ---- Split(レイヤー分割、B39/GOALS M6・E-1 結線) ----
        // Cmd+K(map 267 は Command+B だが、GOALS.md M6 の既定表記は Cmd+K —
        // `split.rs` モジュール doc「`write::Message` への統合」参照。bare
        // `k` は上で ShuttleStop に取られている(`!modifiers.command()` 付き)ので
        // 衝突しない)。`timeline_pane::Message::SplitAtPlayhead` は shell の
        // `Message::Timeline` match(`lib.rs`)に専用腕を持たず、`other` 経路
        // (`PaneState::update`)がそのまま拾う — `ToggleFold`/`ToggleLoop` と
        // 同じ「shell 先取りなしで pane 側が完結する」形(5例外に数えない、
        // `write.rs::split_at_playhead` doc 参照)。
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("k") => {
            Some(Message::Timeline(timeline_pane::Message::SplitAtPlayhead))
        }
        // ---- G1 グループ化動詞(裁定174)。Undo/Redo・SelectAll/DeselectAll と
        // 同じ Shift 振り分けの形(既定割当は仮、上の注記どおり)。
        Key::Character(c)
            if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("g") =>
        {
            Some(Message::GroupLayers)
        }
        Key::Character(c)
            if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("g") =>
        {
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
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("m") => Some(
            Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead),
        ),
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
        Key::Named(Named::ArrowUp)
            if modifiers.command() && modifiers.alt() && !modifiers.shift() =>
        {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::Forward,
            )))
        }
        Key::Named(Named::ArrowDown)
            if modifiers.command() && modifiers.alt() && !modifiers.shift() =>
        {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::Backward,
            )))
        }
        Key::Named(Named::ArrowUp)
            if modifiers.command() && modifiers.alt() && modifiers.shift() =>
        {
            Some(Message::Timeline(timeline_pane::Message::RestackLayer(
                timeline::StackDirection::ToFront,
            )))
        }
        Key::Named(Named::ArrowDown)
            if modifiers.command() && modifiers.alt() && modifiers.shift() =>
        {
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
        Key::Character(c) if modifiers.command() && (c == "=" || c == "+") => Some(Message::ZoomIn),
        Key::Character(c) if modifiers.command() && c == "-" => Some(Message::ZoomOut),
        Key::Character(c) if modifiers.command() && c == "9" => Some(Message::ZoomToFit),
        _ => None,
    }
}

use iced::Task;

/* motolii-component
id = "timeline.easing_shortcuts"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["resolve_easing_shortcut", "resolve_navigation_key"]
meaning = ["SetKeyInterp"]
evaluation = ["EASY_EASE", "EASY_EASE_IN", "EASY_EASE_OUT"]
render = ["resolve_navigation_key"]
observable = ["easing_shortcuts_select_the_three_bezier_presets"]
*/

fn resolve_easing_shortcut(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Message> {
    use iced::keyboard::key::Named;
    use iced::keyboard::Key;
    let Key::Named(Named::F9) = key.as_ref() else {
        return None;
    };
    let interp = if modifiers.command() && modifiers.shift() {
        timeline_pane::EASY_EASE_OUT
    } else if modifiers.shift() {
        timeline_pane::EASY_EASE_IN
    } else if !modifiers.command() {
        timeline_pane::EASY_EASE
    } else {
        return None;
    };
    Some(Message::Timeline(timeline_pane::Message::SetKeyInterp(interp)))
}

impl Shell {
    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_input(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::KeyboardModifiersChanged(modifiers) => self.keyboard_modifiers = modifiers,
            Message::EscapePressed => {
                if !self.timeline.cancel_drag()
                    && !self.timeline.cancel_key_drag()
                    && !self.timeline.cancel_loop_drag()
                    && !self.timeline.cancel_rename()
                    && !self.timeline.cancel_marker_rename()
                    && !self.timeline.cancel_frame_input()
                    && !self.timeline.cancel_graph_interaction()
                    && !self.cancel_gizmo_drag()
                {
                    self.cancel_inspector_interaction();
                }
            }
            Message::ZoomIn => self.zoom_in(),
            Message::ZoomOut => self.zoom_out(),
            Message::ZoomToFit => self.zoom_to_fit(),
            other => return Err(other),
        }
        Ok(task)
    }
}
