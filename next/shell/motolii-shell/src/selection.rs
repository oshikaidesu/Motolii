

use motolii_store::{
    Intent, LayerAttrs, LayerAttrsPatch, LayerId, LayerSource,
};

use crate::{
    clipboard, Shell,
};

impl Shell {
    /// **選択の正本の単一の書き手(C-2、軸台帳 A08)。** `Session::selected_layers`
    /// (複数集合)が正本、`Session::selection`(単一 focus)はその**導出キャッシュ**
    /// — この関数の外で `session.selection`/`session.selected_layers` を直接
    /// 書き換えないこと(この不変条件が唯一の保証点)。
    ///
    /// **なぜ2フィールドのまま1本化するか**(3本立てだった内訳: `selection`/
    /// `selected_layers`/`selected_keys` — 後者はレイヤーでなく打点の選択で
    /// 別軸なので畳まない): Inspector 側の ~40 箇所は今も `session.selection`
    /// (単一)を直接読んでいる(`inspector_ops.rs` 等、write-set 外 — C-2 の
    /// write-set は `selection.rs` のみ)。この 40 箇所を一度に `selected_layers`
    /// 読みへ移行すると、移行の途中で「両者が食い違う」瞬間を必ず作ってしまう
    /// (裁定に反する、RETURN 参照)。代わりに **書き手を1箇所へ絞る**ことで
    /// 「2フィールドだが必ず一致する」不変条件を作り、`selection` を安全な
    /// 読み取り専用キャッシュへ格下げする——先例(4製品)通り「単一選択は
    /// N=1 の複数選択」として扱い、`[only] => Some(*only), _ => None` の
    /// 導出規則だけを唯一の合流点に固定する。
    pub(crate) fn set_selected_layers(&mut self, ids: Vec<LayerId>) {
        self.session.selection = match ids.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        self.session.selected_layers = ids;
    }

    /// 単一 layer を選ぶ。[`Self::set_selected_layers`] の1要素版
    /// (Select All(複数選択)から普通のクリックへ戻る時に古い複数選択が
    /// 居座る事故を防ぐ、という元の意図はそのまま — 単一集合を書くだけで
    /// 自動的に満たされる)。
    pub(crate) fn select_single(&mut self, layer: LayerId) {
        self.set_selected_layers(vec![layer]);
    }

    /// Stage 矩形選択/クリック選択の適用(第6波、`Message::Marquee` 腕)。
    /// `stage::marquee::apply_selection` の結果を [`Self::set_selected_layers`]
    /// へそのまま渡すだけ(gizmo は単一選択にしか出ないので、複数選択では
    /// 自動的に隠れる)。
    pub(crate) fn apply_stage_selection(&mut self, ids: Vec<LayerId>) {
        self.set_selected_layers(ids);
    }

    // ---- layer クリップボード(普通地図 消化第1波 U1、正典 §4) ----

    /// Copy。**Document は触らない**(capture のみ)ので undo に一切乗らない。
    pub(crate) fn copy_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("コピーする layer が選ばれていない".to_owned());
            return;
        };
        match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => self.clipboard.set(snapshot),
            Err(error) => self.status = Some(format!("layer をコピーできない: {error}")),
        }
    }

    /// Paste。**元時刻のまま**(playhead ペーストは今回作らない)。
    /// `LayerSnapshot::instantiate` が組む intent 列を1回の `apply_all` で書くので
    /// 1操作 = 1 undo。配置後は増えた方を選ぶ(正典 §4)。
    pub(crate) fn paste_layer(&mut self) {
        let Some(snapshot) = self.clipboard.get().cloned() else {
            self.status = Some("クリップボードが空".to_owned());
            return;
        };
        let new_id = LayerId(self.next_layer_id());
        match self.doc.apply_all(snapshot.instantiate(new_id)) {
            Ok(()) => self.select_single(new_id),
            Err(error) => self.status = Some(format!("layer を貼り付けられない: {error}")),
        }
    }

    /// Cut = Copy + 削除。**削除は `Intent::RemoveLayer` 1回だけ**(capture 自体は
    /// Document を触らないので、apply 1回 = 1 undo)。locked な layer は
    /// `Intent::RemoveLayer` の `check_not_locked` が理由つきで拒む(M13) —
    /// 拒否された時はクリップボードも書き換えない(コピーだけ成立してしまう
    /// 中途半端を作らない)。
    pub(crate) fn cut_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("切り取る layer が選ばれていない".to_owned());
            return;
        };
        let snapshot = match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.status = Some(format!("layer をコピーできない: {error}"));
                return;
            }
        };
        match self.doc.apply(Intent::RemoveLayer(layer)) {
            Ok(()) => {
                self.clipboard.set(snapshot);
                if self.session.selection == Some(layer) {
                    self.session.selection = None;
                }
                self.session.selected_layers.retain(|&id| id != layer);
            }
            Err(error) => self.status = Some(format!("layer を切り取れない: {error}")),
        }
    }

    /// Duplicate(Cmd+D)。**クリップボードを経由しないその場複製** — capture と
    /// instantiate は clipboard.rs の同じ形を使い回すが、`self.clipboard` へは
    /// 一切触らない(Copy の中身を上書きしない)。**軸台帳 A08 id434 の穴を
    /// 閉じる**: 旧実装は `session.selection`(単数)しか読まなかったため
    /// 何枚選んでいても1枚しか複製できなかった。`selected_layers` 全員を
    /// 束ねて1回の `apply_all`(= 1 undo)で複製する — clipboard(単一
    /// `LayerSnapshot`)を経由しないのでクリップボード側の構造制約(Copy/Paste
    /// が単数のみ、id429/430)を継承しない。複製後は増えた方を全部選ぶ
    /// (正典 §4 の「増えた方を選ぶ」を複数形へ素直に延長)。
    pub(crate) fn duplicate_layer(&mut self) {
        if self.session.selected_layers.is_empty() {
            self.status = Some("複製する layer が選ばれていない".to_owned());
            return;
        }
        // 新規 id は `next_layer_id()`(= 現在の max+1)を基点に連番で予約する
        // — `apply_all` 前は view が更新されないため、1件ずつ都度問い合わせると
        // 全員が同じ id を取り合う。max+1 起点の連番は互いに重複せず、かつ
        // 既存の全 layer id より必ず大きい(`next_layer_id` の doc 参照)。
        let base = self.next_layer_id();
        let mut intents = Vec::new();
        let mut new_ids = Vec::with_capacity(self.session.selected_layers.len());
        {
            let view = self.doc.view();
            for (offset, &layer) in self.session.selected_layers.iter().enumerate() {
                match clipboard::LayerSnapshot::capture(&view, layer) {
                    Ok(snapshot) => {
                        let new_id = LayerId(base + offset as u64);
                        intents.extend(snapshot.instantiate(new_id));
                        new_ids.push(new_id);
                    }
                    Err(error) => {
                        self.status = Some(format!("layer を複製できない: {error}"));
                        return;
                    }
                }
            }
        }
        match self.doc.apply_all(intents) {
            Ok(()) => self.set_selected_layers(new_ids),
            Err(error) => self.status = Some(format!("layer を複製できない: {error}")),
        }
    }

    /// Delete(専用動詞、軸台帳 A08 id431 の穴を閉じる)。**Cut の副作用としてのみ
    /// 存在していた**削除を独立させる — Cut はクリップボード(単一
    /// `LayerSnapshot`)を経由するので単数のみのままだが(id429/430、
    /// clipboard.rs は write-set 外・未着手)、Delete はクリップボードを
    /// 一切経由しないので `selected_layers` を丸ごと `Intent::RemoveLayer`
    /// へ束ねて1回の `apply_all`(= 1 undo)で消せる(先例: AE も Delete と
    /// Cut は別動詞で、Delete のほうが複数選択に素直に効く)。locked な
    /// layer が混じっていれば `check_not_locked` がバッチ全体を理由つきで
    /// 拒む(M13、`group_selected_layers` と同じ姿勢)。
    pub(crate) fn delete_selected_layers(&mut self) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        let intents = self
            .session
            .selected_layers
            .clone()
            .into_iter()
            .map(Intent::RemoveLayer);
        match self.doc.apply_all(intents) {
            Ok(()) => self.set_selected_layers(Vec::new()),
            Err(error) => self.status = Some(format!("layer を削除できない: {error}")),
        }
    }

    /// Select All(正典 §4「Cmd+A 正: 見えている行だけ」)。fold はまだ shell に
    /// 無いので、今は present な全 layer が「見えている」全部(`clipboard::select_all`
    /// doc 参照)。複数選択に入るので単一 focus(`selection`)は持たない。
    pub(crate) fn select_all_layers(&mut self) {
        let visible = self.doc.view().layers();
        self.session.selected_layers = clipboard::select_all(&visible);
        self.session.selection = None;
    }

    /// Deselect All(正典: 空白クリックと同義のキーボード入口)。単一 focus・
    /// 複数選択の両方を解除する。
    pub(crate) fn deselect_all_layers(&mut self) {
        self.session.selection = None;
        self.session.selected_layers.clear();
    }

    // ---- G1 グループ化動詞(裁定174) ----

    /// ⌘G。`selected_layers` は `select_single`/`select_all_layers` が常に
    /// `selection` と同期させているので(単一選択でも `[layer]` が入っている)、
    /// ここ1本を選択の正本として読めばよい。空選択は no-op(status も出さない
    /// — 動詞が意味を持たない状態なので「拒否」ではない)。成功したら
    /// Group 自身を選ぶ(AE 同型、裁定174 選択規則)。
    pub(crate) fn group_selected_layers(&mut self) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        match self.doc.group_layers(&self.session.selected_layers.clone()) {
            Ok(Some(group)) => self.select_single(group),
            Ok(None) => {}
            // 拒否は必ず出す(M13: 無反応ゼロ) — locked な layer が選択に
            // 混じっていた場合、`Document::group_layers` の `Intent::SetAttrs`
            // 柵がバッチ全体を `Err` にする。
            Err(error) => self.status = Some(format!("layer をグループ化できない: {error}")),
        }
    }

    /// ⌘⇧G。選択に含まれる `LayerSource::Group` layer だけを解除する(Group
    /// でない選択は `Document::ungroup_layers` が黙って飛ばす)。解除後は
    /// 旧子らを選ぶ(裁定174 選択規則) — 1層だけなら `select_single` と同型、
    /// 複数なら Select All と同型(単一 focus は持たない)。
    pub(crate) fn ungroup_selected_layers(&mut self) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        match self.doc.ungroup_layers(&self.session.selected_layers.clone()) {
            Ok(children) if children.is_empty() => {}
            Ok(children) => self.set_selected_layers(children),
            Err(error) => self.status = Some(format!("グループを解除できない: {error}")),
        }
    }

    /// Freeze/Unfreeze(裁定119 の意図動詞、MB-2 で UI 初露出)。選択に含まれる
    /// `LayerSource::Group` layer だけを対象にする(Group でない選択は
    /// `ungroup_selected_layers` と同じく黙って飛ばす — store 側の
    /// `freeze_attrs_batch` は非 Group を `Err` にするため、ここで先に絞る)。
    /// 1 `apply_all` = 1 undo(Q2)。選択は動かさない(層構造が変わらない)。
    /// 凍結ゲートの拒否(locked な Group 等)は既存 status 経路で理由つきで出す
    /// (M13: 無反応ゼロ)。
    pub(crate) fn set_selected_groups_frozen(&mut self, frozen: bool) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        let groups: Vec<LayerId> = {
            let view = self.doc.view();
            self.session
                .selected_layers
                .iter()
                .copied()
                .filter(|&layer| {
                    view.meta(layer)
                        .ok()
                        .flatten()
                        .is_some_and(|meta| meta.source == LayerSource::Group)
                })
                .collect()
        };
        if groups.is_empty() {
            return;
        }
        let intents = groups.into_iter().map(|group| {
            if frozen {
                Intent::Freeze { group }
            } else {
                Intent::Unfreeze { group }
            }
        });
        if let Err(error) = self.doc.apply_all(intents) {
            let verb = if frozen { "凍結できない" } else { "解凍できない" };
            self.status = Some(format!("グループを{verb}: {error}"));
        }
    }

    // ---- 選択集合一括の M/S/L(軸台帳 A08: Lock selected layers id874・
    // hidden/solo の同型行の穴を閉じる) ----
    //
    // 行ごとのクリック(`inspector_ops.rs::toggle_layer_{hidden,solo,lock}`、
    // write-set 外なので不触)は残したまま**併存**させる(裁定195: 単一入口を
    // 作らない・S6)。目標値の計算は `motolii-timeline-pane::rows::
    // bulk_toggle_target`(B52 で書かれ呼び手ゼロだった純関数 — 台帳
    // 「呼び手は統合後の write.rs」の想定と違う場所からだが、公開関数を
    // 別 crate から呼ぶだけなので write-set(`next/ui/motolii-timeline-pane/`
    // は不触)を侵さない)をそのまま使う。「1人でも false が混ざっていたら
    // true へ揃える、全員 true なら false へ」という一括トグルの意味は
    // そちらの doc が正本。

    /// 選択集合の hidden/solo/locked を bulk_toggle_target の目標値へ揃えて
    /// 1回の `apply_all`(= 1 undo)で書く共通実装。空選択は no-op。
    fn toggle_selected_layers_attr(
        &mut self,
        read: impl Fn(&LayerAttrs) -> bool,
        make_patch: impl Fn(bool) -> LayerAttrsPatch,
        label: &str,
    ) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        let states: Vec<bool> = {
            let view = self.doc.view();
            self.session
                .selected_layers
                .iter()
                .map(|&layer| read(&view.attrs(layer).ok().flatten().unwrap_or_default()))
                .collect()
        };
        let target = crate::timeline_pane::rows::bulk_toggle_target(&states);
        let selected = self.session.selected_layers.clone();
        let intents = selected.into_iter().map(|layer| Intent::SetAttrs {
            layer,
            patch: make_patch(target),
        });
        if let Err(error) = self.doc.apply_all(intents) {
            self.status = Some(format!("{label} を書けない: {error}"));
        }
    }

    /// M glyph 一括版。軸台帳 A08「Timeline rail Hidden トグル」— 押した1行
    /// だけしか隠せなかった穴を閉じる。
    pub(crate) fn hide_selected_layers(&mut self) {
        self.toggle_selected_layers_attr(
            |attrs| attrs.hidden,
            |target| LayerAttrsPatch {
                hidden: Some(target),
                ..Default::default()
            },
            "hidden",
        );
    }

    /// S glyph 一括版。軸台帳 A08 id314「Timeline rail Solo トグル」の穴を
    /// 閉じる。
    pub(crate) fn solo_selected_layers(&mut self) {
        self.toggle_selected_layers_attr(
            |attrs| attrs.solo,
            |target| LayerAttrsPatch {
                solo: Some(target),
                ..Default::default()
            },
            "solo",
        );
    }

    /// L glyph 一括版。軸台帳 A08 id874「Lock selected layers」に対応する
    /// 動詞がゼロだった穴を閉じる。
    pub(crate) fn lock_selected_layers(&mut self) {
        self.toggle_selected_layers_attr(
            |attrs| attrs.locked,
            |target| LayerAttrsPatch {
                locked: Some(target),
                ..Default::default()
            },
            "locked",
        );
    }
}

use iced::Task;
use crate::{stage, timeline_pane, Message};

impl Shell {
    /// Timeline rail の layer 行クリック(E-2、軸台帳 A08 隣接の穴)。
    /// **裸クリック=単独選択・Cmd=トグル(足し引き)・Shift=範囲**
    /// (`timeline_pane::rows::LayerSelectionOp` の3形そのまま)。解決自体は
    /// [`timeline_pane::rows::resolve_layer_selection`](純関数、`Session` を
    /// 書き換えない)へ委譲し、確定は必ず [`Self::set_selected_layers`]
    /// (C-2 の唯一の書き手)を経由する — この関数の外で `session.selection`/
    /// `selected_layers` を直接書き換えない。
    ///
    /// `order`(範囲の基準)は今 rail に見えている行(`timeline_pane::rows`、
    /// 畳まれて非表示の行は対象外 — `key_order` と同じ「見えているものだけ」
    /// の姿勢、`resolve_layer_selection` doc 参照)。`anchor` は `Session` では
    /// なく `Shell::layer_selection_anchor`(このレーンの write-set は
    /// `lib.rs`/`input.rs`/`timeline-pane::write.rs` のみ — `selection.rs` は
    /// `set_selected_layers` を呼ぶだけで書き換えないため、anchor はここに置く)。
    pub(crate) fn click_select_layer(&mut self, layer: LayerId) {
        let order: Vec<LayerId> =
            timeline_pane::rows(&self.doc.view(), &self.session).into_iter().map(|row| row.id).collect();
        let op = if self.keyboard_modifiers.command() {
            timeline_pane::rows::LayerSelectionOp::Toggle(layer)
        } else if self.keyboard_modifiers.shift() {
            timeline_pane::rows::LayerSelectionOp::Range(layer)
        } else {
            timeline_pane::rows::LayerSelectionOp::Single(layer)
        };
        let (selected, anchor) = timeline_pane::rows::resolve_layer_selection(
            &order,
            self.layer_selection_anchor,
            &self.session.selected_layers,
            op,
        );
        self.layer_selection_anchor = anchor;
        self.set_selected_layers(selected);
    }


    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_selection(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::Select(layer) => self.select_single(layer),
            Message::CopyLayer => self.copy_layer(),
            Message::PasteLayer => self.paste_layer(),
            Message::CutLayer => self.cut_layer(),
            Message::DuplicateLayer => self.duplicate_layer(),
            Message::BatchRenameSelectedLayers => self.batch_rename_selected_layers(),
            Message::SelectAllLayers => self.select_all_layers(),
            Message::DeselectAllLayers => self.deselect_all_layers(),
            Message::DeleteSelectedLayers => self.delete_selected_layers(),
            Message::HideSelectedLayers => self.hide_selected_layers(),
            Message::SoloSelectedLayers => self.solo_selected_layers(),
            Message::LockSelectedLayers => self.lock_selected_layers(),
            Message::GroupLayers => self.group_selected_layers(),
            Message::UngroupLayers => self.ungroup_selected_layers(),
            Message::FreezeGroups => self.set_selected_groups_frozen(true),
            Message::UnfreezeGroups => self.set_selected_groups_frozen(false),
            Message::Marquee(select) => {
                let next = stage::marquee::apply_selection(
                    &self.session.selected_layers,
                    &select.ids,
                    select.additive,
                );
                self.apply_stage_selection(next);
            }
            Message::RenameSelectedLayer => {
                if let Some(layer) = self.session.selection {
                    if let Some(reason) = self.timeline.update(
                        timeline_pane::Message::RenameBegin(layer),
                        &mut self.doc,
                        &mut self.session,
                        self.keyboard_modifiers,
                    ) {
                        self.status = Some(reason);
                    }
                }
            }
            other => return Err(other),
        }
        Ok(task)
    }

    /// Auto-number the selected layer names in Timeline row order. The plan
    /// and atomic write live in the dedicated batch-rename component.
    pub(crate) fn batch_rename_selected_layers(&mut self) {
        let selected = if self.session.selected_layers.is_empty() {
            self.session.selection.into_iter().collect::<Vec<_>>()
        } else {
            self.session.selected_layers.clone()
        };
        match crate::batch_rename::apply_selected(&mut self.doc, &selected) {
            Ok(0) if selected.is_empty() => {
                self.status = Some("一括改名する layer が選ばれていない".to_owned());
            }
            Ok(0) => {
                self.status = Some("選択中の layer 名は既に採番済み".to_owned());
            }
            Ok(changed) => {
                self.status = Some(format!("{changed} layer を一括採番しました"));
            }
            Err(error) => self.status = Some(error),
        }
    }
}
