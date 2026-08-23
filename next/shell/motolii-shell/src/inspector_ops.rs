
use iced::Task;

use motolii_store::{
    Intent, LayerAttrsPatch, LayerId, LayerTiming, Speed, TextStyleId,
};

use crate::inspector_pane::{FieldDraft, TransformField};
use crate::{
    chrome, inspector_pane,
    timeline_pane, Message, Shell,
};

impl Shell {
    /// [`Message::Inspector`] の唯一の分配口。腕ごとの意味は
    /// `inspector_pane::Message` 側の doc を参照。書き込み本体は
    /// `motolii-inspector-pane` crate 側の自由関数([`inspector_pane::
    /// commit_inspector_field`] 等)が持ち、ここは `self.doc`/`self.session`/
    /// 下書きフィールドをそのまま貸す glue だけ(裁定160 切片8、
    /// `update_settings` と同じ形)。
    pub(crate) fn update_inspector(&mut self, message: inspector_pane::Message) -> Task<Message> {
        match message {
            inspector_pane::Message::FieldInput(field, text) => {
                self.inspector_field_draft = Some(FieldDraft { field, text });
                Task::none()
            }
            inspector_pane::Message::FieldSubmit(field) => {
                self.commit_inspector_field(field);
                Task::none()
            }
            inspector_pane::Message::NameInput(text) => {
                self.inspector_name_draft = Some(text);
                Task::none()
            }
            inspector_pane::Message::NameSubmit => {
                self.commit_inspector_name();
                Task::none()
            }
            inspector_pane::Message::ToggleHidden => {
                self.toggle_inspector_hidden();
                Task::none()
            }
            inspector_pane::Message::CycleBlendMode => {
                self.cycle_inspector_blend_mode();
                Task::none()
            }
            inspector_pane::Message::ValuePressed(field) => {
                self.start_field_drag(field);
                Task::none()
            }
            // **D-1 修正**: window 全体購読は `Message::Inspector` を1本しか
            // 発行しない(`inspector_pointer_event`、write-set 外の
            // `input.rs`)ので、Transform drag(`inspector_drag`)と text-style
            // drag(`inspector_text_style_drag`)の両方をここから追う——
            // press は排他(`ValuePressed`/`TextStyleValuePressed` のどちらか
            // 一方だけが `Some` を立てる)なので、両方呼んでも無害(片方は
            // 必ず `None` の no-op)。
            inspector_pane::Message::PointerMoved(point) => {
                self.continue_field_drag(point);
                self.continue_text_style_field_drag(point);
                Task::none()
            }
            inspector_pane::Message::PointerReleased => {
                let task = self.finish_field_drag();
                self.finish_text_style_field_drag();
                task
            }
            inspector_pane::Message::SpeedInput(text) => {
                self.inspector_speed_draft = Some(text);
                Task::none()
            }
            inspector_pane::Message::SpeedSubmit => {
                self.commit_inspector_speed();
                Task::none()
            }
            inspector_pane::Message::ResetSpeed => {
                self.reset_inspector_speed();
                Task::none()
            }
            inspector_pane::Message::KeyPressed(row) => {
                self.toggle_inspector_key(row);
                Task::none()
            }
            // MASK section(B02 第1切片、裁定184)。書き込み本体は pane の
            // 自由関数(`ToggleHidden`/`CycleBlendMode` と同じ即時操作の形)—
            // ここは `Err` を status 帯へ渡す glue だけ(M13)。
            inspector_pane::Message::CycleMaskMode(mask) => {
                if let Err(error) = inspector_pane::cycle_inspector_mask_mode(
                    &mut self.doc,
                    self.session.selection,
                    mask,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ToggleMaskInverted(mask) => {
                if let Err(error) = inspector_pane::toggle_inspector_mask_inverted(
                    &mut self.doc,
                    self.session.selection,
                    mask,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // EFFECTS section(B38 第3切片、裁定184 型別 section 第2号)。
            // 書き込み本体は pane の自由関数(MASK section と同じ即時操作の形)—
            // ここは `Err` を status 帯へ渡す glue だけ(M13)。
            inspector_pane::Message::RemoveEffect(effect) => {
                if let Err(error) = inspector_pane::remove_inspector_effect(
                    &mut self.doc,
                    self.session.selection,
                    effect,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::MoveEffectUp(effect) => {
                if let Err(error) = inspector_pane::move_inspector_effect_up(
                    &mut self.doc,
                    self.session.selection,
                    effect,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::MoveEffectDown(effect) => {
                if let Err(error) = inspector_pane::move_inspector_effect_down(
                    &mut self.doc,
                    self.session.selection,
                    effect,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ToggleEffectBypass(effect) => {
                // `effect.{id}.enabled` は playhead 時刻での値編集(裁定213/214、
                // `commit_inspector_field` と同じ「comp が無ければ何もしない」柵)。
                let Ok(Some(composition)) = self.doc.view().composition() else {
                    return Task::none();
                };
                if let Err(error) = inspector_pane::toggle_inspector_effect_bypass(
                    &mut self.doc,
                    self.session.selection,
                    effect,
                    self.session.playhead,
                    composition.fps,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // ラベル色チップ(B03、ident 帯)— 同上の glue のみ。
            inspector_pane::Message::CycleLabelColor => {
                if let Err(error) = inspector_pane::cycle_inspector_label_color(
                    &mut self.doc,
                    self.session.selection,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // TEXT section(B46 第1切片、裁定184)。書き込み本体は pane の
            // 自由関数(MASK 腕と同じ「ここは Err を status 帯へ渡す glue
            // だけ」の形、M13)。
            inspector_pane::Message::TextFieldInput(field, text) => {
                self.inspector_text_field_draft =
                    Some(inspector_pane::TextFieldDraft { field, text });
                Task::none()
            }
            // **D-1 修正(2026-08-23)**: Size/Line Height/Tracking は
            // `text_field_track_target` で「track 書き口(A-1b)へ渡すべき
            // field か」を判定する——`view.rs`(shell、write-set 外)を触らずに
            // track 化するための橋(`inspector_pane::text` モジュール冒頭 doc
            // 参照)。Content/FontFamily は従来どおり静的値の口
            // (`commit_text_field`)のまま。
            //
            // track 側は下書きの型が違う(`TextFieldDraft` ではなく
            // `TextStyleTrackDraft`)ので、ここで1回だけ包み替える——
            // `commit_text_style_track_field` 自身が `Option::take()` で
            // 消費するので、包み替えた一時 `Option` を渡すだけで良い
            // (`self.inspector_text_field_draft` 側は下で必ず空にする)。
            inspector_pane::Message::TextFieldSubmit(field) => {
                if let Some(style_field) = inspector_pane::text_field_track_target(field) {
                    let Some(taken) = self.inspector_text_field_draft.take() else {
                        return Task::none();
                    };
                    let Ok(Some(composition)) = self.doc.view().composition() else {
                        return Task::none();
                    };
                    let mut style_draft = Some(inspector_pane::TextStyleTrackDraft {
                        style: TextStyleId(0),
                        field: style_field,
                        text: taken.text,
                    });
                    if let Err(error) = inspector_pane::commit_text_style_track_field(
                        &mut self.doc,
                        &mut style_draft,
                        self.session.selection,
                        self.session.playhead,
                        composition.fps,
                        TextStyleId(0),
                        style_field,
                    ) {
                        self.status = Some(error);
                    }
                } else if let Err(error) = inspector_pane::commit_text_field(
                    &mut self.doc,
                    &mut self.inspector_text_field_draft,
                    self.session.selection,
                    field,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // Key 列 click(D-1 結線)。`ToggleHidden`/`KeyPressed` と同じ即時
            // 操作の形——書き込み本体は `toggle_text_style_key`
            // (`crate::transform::toggled_key_track` を再利用した A-1b の
            // 純関数)。この section は既定行(`TextStyleId(0)`、裁定98)しか
            // 編集しないスコープなので id は固定で渡す(`text.rs` doc 参照)。
            inspector_pane::Message::TextStyleKeyPressed(field) => {
                let Ok(Some(composition)) = self.doc.view().composition() else {
                    return Task::none();
                };
                if let Err(error) = inspector_pane::toggle_text_style_key(
                    &mut self.doc,
                    self.session.selection,
                    self.session.playhead,
                    composition.fps,
                    TextStyleId(0),
                    field,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // drag ハンドル press(D-1 結線)。`ValuePressed` と同じ「click か
            // drag かはまだ未確定」の形——move/release は
            // `PointerMoved`/`PointerReleased`(下記、window 全体購読を共有)
            // が `self.inspector_text_style_drag` を追う。
            inspector_pane::Message::TextStyleValuePressed(field) => {
                self.start_text_style_field_drag(field);
                Task::none()
            }
            inspector_pane::Message::CycleTextJustify => {
                if let Err(error) =
                    inspector_pane::cycle_text_justify(&mut self.doc, self.session.selection)
                {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ResetLineHeightAuto => {
                if let Err(error) =
                    inspector_pane::reset_text_line_height(&mut self.doc, self.session.selection)
                {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ResetTracking => {
                if let Err(error) =
                    inspector_pane::reset_text_tracking(&mut self.doc, self.session.selection)
                {
                    self.status = Some(error);
                }
                Task::none()
            }
            // MATTE(2026-08-22 発注「レイヤーを指す」文法 第1号)。書き込み
            // 本体は pane の自由関数(MASK/EFFECTS 腕と同じ「ここは Err を
            // status 帯へ渡す glue だけ」の形、M13)。
            inspector_pane::Message::PickMatteSource(source) => {
                if let Err(error) = inspector_pane::set_inspector_matte_source(
                    &mut self.doc,
                    self.session.selection,
                    source,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::CycleMatteMode => {
                if let Err(error) = inspector_pane::cycle_inspector_matte_mode(
                    &mut self.doc,
                    self.session.selection,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ClearMatte => {
                if let Err(error) =
                    inspector_pane::clear_inspector_matte(&mut self.doc, self.session.selection)
                {
                    self.status = Some(error);
                }
                Task::none()
            }
            // LINK(2026-08-22 発注「レイヤーを指す」文法 第3号)。同上の glue
            // のみ — 書き込み本体は `inspector_pane::commit_inspector_link`/
            // `clear_inspector_link`。
            inspector_pane::Message::PickLinkSource(target, candidate) => {
                if let Err(error) = inspector_pane::commit_inspector_link(
                    &mut self.doc,
                    self.session.selection,
                    target,
                    candidate,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            inspector_pane::Message::ClearLink(target) => {
                if let Err(error) = inspector_pane::clear_inspector_link(
                    &mut self.doc,
                    self.session.selection,
                    target,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // フォント pick_list(2026-08-22 追い発注「フォントが選べる・
            // 選ばなくても落ちない」)。`TextFieldSubmit` と同じ「ここは Err を
            // status 帯へ渡す glue だけ」の形(M13) — 書き込み本体は
            // `inspector_pane::commit_text_font_pick`。
            inspector_pane::Message::PickFont(family) => {
                if let Err(error) = inspector_pane::commit_text_font_pick(
                    &mut self.doc,
                    self.session.selection,
                    &family,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // 色エディタ(`inspector_pane::color`、2026-08-22 発注「歌詞が
            // 入れられる道を通す」で結線)。`TextFieldInput`/`TextFieldSubmit`
            // と同じ「打鍵は下書きだけ・Enter で1回の Intent」の形 —
            // 書き込み本体は pane の自由関数
            // ([`inspector_pane::color::commit_text_style_color`])、ここは
            // `Err` を status 帯へ渡す glue だけ(M13)。
            inspector_pane::Message::Color(inspector_pane::color::Message::ChannelInput(
                target,
                channel,
                text,
            )) => {
                self.inspector_color_field_draft =
                    Some(inspector_pane::color::ColorFieldDraft { target, channel, text });
                Task::none()
            }
            inspector_pane::Message::Color(inspector_pane::color::Message::ChannelSubmit(
                target,
                channel,
            )) => {
                if let Err(error) = inspector_pane::color::commit_text_style_color(
                    &mut self.doc,
                    &mut self.inspector_color_field_draft,
                    self.session.selection,
                    target,
                    channel,
                ) {
                    self.status = Some(error);
                }
                Task::none()
            }
            // 裁定217 連続量 drag 化(E-5、write-set 外だが `color::Message` の
            // 網羅性のために必須 — 最小の1腕): 実際には `lib.rs`
            // (`Message::Inspector` 腕)がこの variant を先取りして
            // `Shell::start_value_drag` へ渡すため、ここへは来ない
            // (`lib.rs` 側のコメント参照)。
            inspector_pane::Message::Color(inspector_pane::color::Message::ChannelDragPressed(..)) => {
                Task::none()
            }
        }
    }

    /// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
    /// (1 gesture = 1 undo)。書き込み本体は [`inspector_pane::
    /// commit_inspector_field`](自由関数、`&mut self.doc`/`&mut self.
    /// inspector_field_draft`/`self.session.selection` をそのまま貸す)——
    /// ここは `Err` を status 帯へ渡す glue だけ(M13)。
    pub(crate) fn commit_inspector_field(&mut self, field: TransformField) {
        // comp が無い(= layer も選択も無い)なら下書きを捨てるだけ —
        // 旧実装(選択なしで draft を消費して no-op)と同じ安全側。
        let Ok(Some(composition)) = self.doc.view().composition() else {
            self.inspector_field_draft = None;
            return;
        };
        if let Err(error) = inspector_pane::commit_inspector_field(
            &mut self.doc,
            &mut self.inspector_field_draft,
            self.session.selection,
            self.session.playhead,
            composition.fps,
            field,
        ) {
            self.status = Some(error);
        }
    }

    /// Attrs の Name 欄 — 下書きを確定して1回の `Intent::SetAttrs` を出す。
    /// [`commit_inspector_field`](上記)と同じ glue の形。
    pub(crate) fn commit_inspector_name(&mut self) {
        if let Err(error) = inspector_pane::commit_inspector_name(
            &mut self.doc,
            &mut self.inspector_name_draft,
            self.session.selection,
        ) {
            self.status = Some(error);
        }
    }

    /// Key セル click(K1)— 即1回の `Intent::SetTrack` を出す(下書きを経由
    /// しない、[`toggle_inspector_hidden`] と同じ即時操作の形 — 1 click = 1
    /// undo)。3状態の意味と新 track の組み立ては純関数
    /// [`inspector_pane::toggled_key_track`] が持ち、ここは playhead・track・
    /// 現在の評価値を貸して `Err` を status 帯へ渡す glue だけ(M13)。
    /// 選択なし・comp なしは黙って無視(`commit_inspector_field` と同じ柵)。
    pub(crate) fn toggle_inspector_key(&mut self, row: inspector_pane::KeyRow) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return;
        };
        let Ok(property) = inspector_pane::key_row_property_id(row) else {
            return; // 標準 property なので起こらない — 安全側で無視。
        };
        let Some(t) = self.time_at_playhead() else {
            return;
        };
        let store = self.doc.view();
        let Ok(track) = store.track(layer, &property) else {
            return;
        };
        // 初キー化(track 無し)の値の正本: 解決済みの現在値(スロット参照も
        // ここで track へ戻る — `SetTrack` がスロットを普通の track に置き換える
        // 既存の意味論)。値が読めなければ行の既定値。
        let current_value = match store.value_at(layer, &property, t) {
            Ok(Some(value)) => value,
            _ => inspector_pane::key_row_default_value(row),
        };
        let new_track = match inspector_pane::toggled_key_track(
            track.as_ref(),
            self.session.playhead,
            composition.fps,
            current_value,
        ) {
            Ok(new_track) => new_track,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        drop(store);
        if let Err(error) = self.doc.apply(Intent::SetTrack {
            layer,
            property,
            track: new_track,
        }) {
            self.status = Some(format!("キーを書けない: {error}"));
        }
    }

    /// Attrs の Hidden トグル — 即 `Intent::SetAttrs` を1回出す(下書きを経由しない)。
    /// **`motolii-inspector-pane` crate へは移設していない**: 対象を
    /// `Session::selection` から引くだけで、書き込み自体は `LaneBarToggleMute`
    /// とも共有する cross-cutting な [`toggle_layer_hidden`] へ委譲する —
    /// Inspector 固有の write ロジックを1行も持たないため(RETURN の
    /// write-set 外 finding 参照)。
    pub(crate) fn toggle_inspector_hidden(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        self.toggle_layer_hidden(layer);
    }

    /// Attrs の Blend 巡回ボタン — 即 `Intent::SetAttrs` を1回出す(下書きを経由
    /// しない、[`toggle_inspector_hidden`] と同じ即時操作の形)。**lane bar には
    /// 無い**(発注書 EXACT TARGET — 対象は Inspector の選択レイヤのみ)ので
    /// `toggle_layer_hidden` のような cross-cutting な共有関数へは切り出さない。
    /// 対応 mode の一覧([`inspector_pane::SUPPORTED_BLEND_MODES`])は Inspector
    /// 側が持つ(発注書「決定済み事項」)。
    pub(crate) fn cycle_inspector_blend_mode(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .blend_mode;
        let patch = LayerAttrsPatch {
            blend_mode: Some(inspector_pane::next_blend_mode(current)),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("blend_mode を書けない: {error}"));
        }
    }

    /// Speed 欄(ATTRS、SP1 第一波、supervisor 決定1-7)— 下書きを確定して
    /// 1回の `Intent::SetTiming` を出す(1 gesture = 1 undo)。**`LayerTiming`
    /// の組み立て・duration 再計算はここで行う**(`inspector_pane` crate は
    /// `motolii-timeline-pane::clip_gesture` へ依存できないため — `inspector_pane`
    /// crate doc 参照)。数値として読めない・0以下は `Err` の理由文を status
    /// 帯へ渡す(`commit_inspector_field` と同じ glue の形、M13)。
    pub(crate) fn commit_inspector_speed(&mut self) {
        let Some(text) = self.inspector_speed_draft.take() else {
            return;
        };
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(percent) = chrome::parse_number(&text) else {
            self.status = Some(format!("数値として読めない: {text}"));
            return;
        };
        let Some((new_num, new_den)) = inspector_pane::percent_to_speed_ratio(percent) else {
            self.status = Some(format!("speed は正の値のみ: {text}"));
            return;
        };
        self.apply_speed(layer, new_num, new_den);
    }

    /// Speed 行の Reset ボタン — 下書きを経由せず即 100%(`Speed::NORMAL`)へ。
    /// [`commit_inspector_speed`] と同じ [`apply_speed`] を呼ぶので、既に100%
    /// なら no-op(Undo を積まない、決定7)は自動的に成り立つ。
    pub(crate) fn reset_inspector_speed(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        self.apply_speed(layer, Speed::NORMAL.num(), Speed::NORMAL.den());
    }

    /// [`commit_inspector_speed`]/[`reset_inspector_speed`] 共通の書き口。
    /// **start・source_in は不変**(決定4「source 窓が保存される」)、duration
    /// だけ [`timeline_pane::clip_gesture::retimed_duration`](第二波
    /// Shift+端drag と共有する純関数、δ 採択理由)で再計算する。**ロック
    /// layer は `Document::apply` の `check_not_locked` がそのまま拒む**
    /// (M13、move/trim と同じ理由文の型 — ここで重複判定しない)。**現在値と
    /// 同じ speed なら `Intent` を出さない**(決定7 — reset の no-op と、
    /// 打鍵で同値を submit した場合の両方をこの1箇所で満たす)。
    pub(crate) fn apply_speed(&mut self, layer: LayerId, new_num: i64, new_den: i64) {
        let Ok(new_speed) = Speed::try_new(new_num, new_den) else {
            self.status = Some("speed の分母は正でなければならない".to_owned());
            return;
        };
        let Ok(Some(meta)) = self.doc.view().meta(layer) else {
            return; // 素材が無い layer(起こらないはず) — 安全側で無視。
        };
        let old_timing = meta.timing;
        if old_timing.speed == new_speed {
            return; // 決定7: 同値は Undo を積まない。
        }
        let new_duration = timeline_pane::clip_gesture::retimed_duration(
            old_timing.duration,
            (old_timing.speed.num(), old_timing.speed.den()),
            (new_speed.num(), new_speed.den()),
        );
        let new_timing = LayerTiming {
            duration: new_duration,
            speed: new_speed,
            ..old_timing
        };
        if let Err(error) = self.doc.apply(Intent::SetTiming { layer, timing: new_timing }) {
            self.status = Some(format!("speed を書けない: {error}"));
        }
    }

    // ---- Timeline レーンバー(裁定147・第2波T1) ----
    //
    // 3つとも同じ形: 現在値を読んで反転した `LayerAttrsPatch` を1回出す
    // (`toggle_inspector_hidden` と同じ即時操作)。**対象は明示の `layer`**
    // (`Session::selection` ではない) — レーンバーは選択と無関係にどの行の
    // M/S/L も直接叩ける(裁定147「M/S/L もレーンバーで直接設定できる」)。
    //
    // 拒否の経路(M13)は書き口(`Document::write`)が既に持つ:
    // `hidden`/`solo` は locked な行への書き込みを理由つき `Err` で拒む
    // (`motolii_store::document::check`)。`locked` 自身は「解除/再ロックだけ
    // 常に通す」規則があるので `toggle_layer_lock` だけは常に成功する。

    /// M glyph。`LayerAttrs.hidden` をトグルする(`inspector_pane::Message::
    /// ToggleHidden` と同じ書き口 — 対象の layer が違うだけ)。
    pub(crate) fn toggle_layer_hidden(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .hidden;
        let patch = LayerAttrsPatch {
            hidden: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("hidden を書けない: {error}"));
        }
    }

    /// S glyph。`LayerAttrs.solo` をトグルする。
    pub(crate) fn toggle_layer_solo(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .solo;
        let patch = LayerAttrsPatch {
            solo: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("solo を書けない: {error}"));
        }
    }

    /// L glyph。`LayerAttrs.locked` をトグルする。locked 自身への書き込みは
    /// `Document::write` が locked な行でも常に通す(先に触れなくなる詰みを
    /// 作らない規則、`motolii_store::attrs::LayerAttrs::locked` の doc 参照)。
    pub(crate) fn toggle_layer_lock(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .locked;
        let patch = LayerAttrsPatch {
            locked: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("locked を書けない: {error}"));
        }
    }

    /// 値セルの press — click か drag かはまだ未確定
    /// (`inspector_pane::FieldDragState::origin_x` が `None` のまま)。選択
    /// なし・comp なし・対応する field が投影に無い、のいずれも黙って無視。
    /// playhead(frame)と fps は press 時点の物を渡す — キー持ち track の
    /// 確定(キー upsert、`inspector_pane::edited_value_track`)の宛先になる。
    pub(crate) fn start_field_drag(&mut self, field: TransformField) {
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return; // comp が無ければ投影も無い — drag は始まらない。
        };
        let projection = self.inspector_selection();
        inspector_pane::start_field_drag(
            &mut self.inspector_drag,
            self.session.selection,
            projection.as_ref(),
            field,
            self.session.playhead,
            composition.fps,
        );
    }

    /// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op。
    /// **1px = 感度表の刻み**(`inspector_pane::dragged_value`)。
    pub(crate) fn continue_field_drag(&mut self, point: iced::Point) {
        let fine = self.keyboard_modifiers.shift();
        inspector_pane::continue_field_drag(&mut self.doc, &mut self.inspector_drag, point, fine);
    }

    /// 左クリック release(window 全体から — `mouse_area` 自身の `on_release` は
    /// bounds を出た drag を捉えられないので使わない)。**drag が実際に動いて
    /// いたら確定**: 最後の transient 値そのものを1回の本編集 `Intent` として
    /// `apply` してから `clear_transient`(1 gesture = 1 undo、overlay を残さない)。
    /// 動いていなければ click として type 編集へ切り替える
    /// (`inspector_pane::finish_field_drag` が `Ok(Some(field))` で知らせる)。
    pub(crate) fn finish_field_drag(&mut self) -> Task<Message> {
        match inspector_pane::finish_field_drag(&mut self.doc, &mut self.inspector_drag) {
            Ok(Some(field)) => self.enter_field_editing(field),
            Ok(None) => Task::none(),
            Err(error) => {
                self.status = Some(error);
                Task::none()
            }
        }
    }

    /// TEXT section の Size/Line Height/Tracking の値セル press(D-1 結線)。
    /// `start_field_drag` と同格 — `inspector_drag`(`FieldDragState`)とは
    /// **別状態**(`self.inspector_text_style_drag`、A-1b の
    /// `TextStyleDragState`)。この section は既定行(`TextStyleId(0)`)しか
    /// 編集しないスコープなので id は固定で渡す。
    pub(crate) fn start_text_style_field_drag(&mut self, field: inspector_pane::TextStyleField) {
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return; // comp が無ければ投影も無い — drag は始まらない。
        };
        inspector_pane::start_text_style_drag(
            &mut self.inspector_text_style_drag,
            &self.doc,
            self.session.selection,
            TextStyleId(0),
            field,
            self.session.playhead,
            composition.fps,
        );
    }

    /// window 全体の cursor 移動 — text-style drag 版(`continue_field_drag`
    /// と同格)。`self.inspector_text_style_drag` が `None` なら no-op
    /// (`inspector_pane::continue_text_style_drag` 自身の早期 return)。
    pub(crate) fn continue_text_style_field_drag(&mut self, point: iced::Point) {
        let fine = self.keyboard_modifiers.shift();
        inspector_pane::continue_text_style_drag(
            &mut self.doc,
            &mut self.inspector_text_style_drag,
            point,
            fine,
        );
    }

    /// release — text-style drag 版(`finish_field_drag` と同格)。動いて
    /// いなければ [`inspector_pane::finish_text_style_drag`] 自身が no-op
    /// (`TransformField` 版と違い、text-style は値セルが常時 text_input の
    /// ままなので「click→type 編集への切替」は要らない——`text.rs` の
    /// `size_row`/`line_height_row`/`tracking_row` doc 参照)。
    pub(crate) fn finish_text_style_field_drag(&mut self) {
        if let Err(error) =
            inspector_pane::finish_text_style_drag(&mut self.doc, &mut self.inspector_text_style_drag)
        {
            self.status = Some(error);
        }
    }

    /// click(ドラッグせず release)→ type 編集。下書きを立て、text_input へ
    /// フォーカスを戻す(値セルは編集していない間は `mouse_area` + 静止
    /// `text` なので、click 直後にはまだ text_input が木に無く自動フォーカス
    /// されない — 明示的な focus task が要る)。**focus task の構築自体は
    /// Document を触らない**ので、値の計算(`inspector_pane::drag_origin`/
    /// `format_number`/`field_decimals`)以外はここへ移設していない
    /// (`inspector_pane` crate doc 参照)。
    pub(crate) fn enter_field_editing(&mut self, field: TransformField) -> Task<Message> {
        let Some(selection) = self.inspector_selection() else {
            return Task::none();
        };
        let Some((value, _)) = inspector_pane::drag_origin(&selection, field) else {
            return Task::none();
        };
        self.inspector_field_draft = Some(FieldDraft {
            field,
            text: inspector_pane::format_number(value, inspector_pane::field_decimals(field)),
        });
        iced::widget::operation::focus(inspector_pane::field_input_id(field))
    }

    /// Esc — 進行中の drag があれば復元、無ければ typing 下書き(値セル/名前欄)
    /// を破棄する(hint 行「Esc to cancel」を両方について正直にする)。
    ///
    /// drag/field_draft の分岐は [`inspector_pane::cancel_field_interaction`]
    /// (自由関数)——`true` を返したらここで終わる(元実装の早期 return を
    /// 保つ)。名前欄・Settings 下書きの破棄は Inspector pane の write-set 外
    /// (`settings_pane` の下書きも同じ Esc で破棄する、hint 文言との整合)
    /// なのでここに残した。
    pub(crate) fn cancel_inspector_interaction(&mut self) {
        if inspector_pane::cancel_field_interaction(
            &mut self.doc,
            &mut self.inspector_drag,
            &mut self.inspector_field_draft,
        ) {
            return;
        }
        self.inspector_name_draft = None;
        self.inspector_speed_draft = None;
        self.inspector_text_field_draft = None;
        // Settings パネルの下書きも同じ Esc で破棄する(hint 文言との整合)。
        self.background_draft = None;
        self.ui_scale_draft = None;
    }

}

