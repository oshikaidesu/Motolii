

use motolii_store::{
    Intent, LayerId, LayerSource,
};

use crate::{
    clipboard, Shell,
};

impl Shell {
    /// 単一 layer を選ぶ(既存の `Session::selection` に加え、`selected_layers` も
    /// 単一集合へ揃える — Select All(複数選択)から普通のクリックへ戻る時に
    /// 古い複数選択が居座る事故を防ぐ)。
    pub(crate) fn select_single(&mut self, layer: LayerId) {
        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];
    }

    /// Stage 矩形選択/クリック選択の適用(第6波、`Message::Marquee` 腕)。
    /// `stage::marquee::apply_selection` の結果を `Session` へ写すだけ —
    /// `select_single`/`select_all_layers`/`deselect_all_layers` と同じ
    /// 「単一なら `selection` も揃える・そうでなければ `None`」の規約
    /// (`select_all_layers`/`deselect_all_layers` doc 参照 — gizmo は単一選択
    /// にしか出ないので、複数選択では自動的に隠れる)。
    pub(crate) fn apply_stage_selection(&mut self, ids: Vec<LayerId>) {
        self.session.selection = match ids.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        self.session.selected_layers = ids;
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
    /// 一切触らない(Copy の中身を上書きしない)。1 `apply_all` = 1 undo。
    /// 複製後は増えた方を選ぶ(正典 §4)。
    pub(crate) fn duplicate_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("複製する layer が選ばれていない".to_owned());
            return;
        };
        let snapshot = match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.status = Some(format!("layer を複製できない: {error}"));
                return;
            }
        };
        let new_id = LayerId(self.next_layer_id());
        match self.doc.apply_all(snapshot.instantiate(new_id)) {
            Ok(()) => self.select_single(new_id),
            Err(error) => self.status = Some(format!("layer を複製できない: {error}")),
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
            Ok(children) if children.len() == 1 => self.select_single(children[0]),
            Ok(children) => {
                self.session.selected_layers = children;
                self.session.selection = None;
            }
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

}

