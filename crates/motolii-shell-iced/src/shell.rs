//! モデルと `update` — この殻の唯一の可変状態。
//!
//! 中身は `ShellGateway` 1つだけである。座席も transcript も journal も
//! ゲートウェイの中に在り、この型からは**読みしか出せない**。
//! 「journal を通らずに製品状態へ着く道」を新しい殻でも作らない、という
//! 2026-08-18 の構造の強制をそのまま引き継いでいる
//! (柵: `tests/intent_gateway_fence.rs`)。
//!
//! ## 判断は移していない
//!
//! 未保存 guard の3択は `motolii_ui::blitz_shell::decide_unsaved` を**そのまま**
//! 呼ぶ。Export を始めてよいかは `ShellGateway::can_start_export`。どちらも
//! egui shell が見ているのと同じ関数で、host を替えても答えが変わらない
//! (=「意味は不変のまま iced へ移る」)。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use motolii_ui::blitz_shell::{
    decide_unsaved, IntentEvent, ShellGateway, ShellPrompts, ShellTranscript, StatusEvent,
    UiIntent, UiItemFlag, UnsavedDecision,
};

use crate::inspector_model::{project_inspector, InspectorSeat};
use crate::inspector_pane::InspectorEvent;
use crate::message::Message;
use crate::widgets_stub::ScrubEvent;

/// `update` 1件のあとで窓に残る唯一の要求。
///
/// iced では「窓を閉じる」も副作用なので、殻は決めるだけで実行しない
/// (実行は host = `main.rs` の `iced::exit()`)。テストは窓を持たないまま
/// 同じ判断を読める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Outcome {
    /// このまま窓は開いている。
    #[default]
    Stay,
    /// 窓を閉じてよい(未保存の始末は済んでいる)。
    Close,
}

/// iced ホストの shell 状態。
pub struct Shell {
    /// 製品状態へ触れる唯一の口。**この crate は他の道を持たない。**
    gateway: ShellGateway,
    /// 人に訊く口。窓は `NativePrompts`、テスト・CLI 駆動は `ScriptedPrompts`。
    /// **egui shell と同じ trait の同じ実装**である(`crate::prompts` の注記)。
    prompts: Box<dyn ShellPrompts>,
    /// Inspector の live 投影に使う plugin catalog。座席の writer が使うのと同じ
    /// `reference_catalog` を1度だけ組んで持ち回る(毎フレーム組み直さない —
    /// egui 版 `InspectorPanel` と同じ判断)。組めなかったら理由を持ち、
    /// Inspector が空面としてその理由を出す(黙らない)。
    catalog: Result<Arc<motolii_plugin::PluginCatalog>, String>,
}

impl Shell {
    /// 座席なしで始める(スタート画面)。
    pub fn new(prompts: impl ShellPrompts + 'static) -> Self {
        Self {
            gateway: ShellGateway::new(ShellTranscript::default()),
            prompts: Box::new(prompts),
            catalog: motolii_plugin::reference::reference_catalog()
                .map(Arc::new)
                .map_err(|error| format!("plugin catalog を作れない: {error}")),
        }
    }

    /// Message 1件を受ける。**ここが Message → `UiIntent` の唯一の写像**である。
    ///
    /// dialog が答えなければ何も起きない(intent も記録されない)。
    /// 「起こそうとした行動」だけが journal に載る、という規律は egui shell と同じ。
    pub fn update(&mut self, message: Message) -> Outcome {
        match message {
            Message::NewProjectPressed => {
                // 座席を差し替える = いまの編集を捨てる。未保存なら先に訊く。
                if !self.clear_unsaved_or_stay() {
                    return Outcome::Stay;
                }
                let Some(path) = self.prompts.new_project_path() else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::NewProject { path });
            }
            Message::OpenProjectPressed => {
                if !self.clear_unsaved_or_stay() {
                    return Outcome::Stay;
                }
                let Some(path) = self.prompts.open_project_path() else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::OpenProject { path });
            }
            Message::SavePressed => {
                // 訊くことは無い(保存先は座席が知っている)。座席が無い時に
                // 何を言うかもゲートウェイの側に既に在る。
                let _ = self.gateway.dispatch(UiIntent::SaveProject);
            }
            Message::ExportPressed => {
                // 判断(座席あり・実行中なし)はボタンの enabled と同じ関数を見る。
                if !self.gateway.can_start_export() {
                    return Outcome::Stay;
                }
                let Some(project) = self.project_path() else {
                    return Outcome::Stay;
                };
                // 訊いて断られたら何も記録しない(訊いただけの操作は intent ではない)。
                let Some(output) = self.prompts.export_path(&project) else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::BeginExport { output });
            }
            Message::CancelExportPressed => {
                let _ = self.gateway.dispatch(UiIntent::CancelExport);
            }
            Message::FilesDropped(paths) => {
                let _ = self.gateway.dispatch(UiIntent::AdmitPaths { paths });
            }
            Message::CloseRequested => {
                // 窓を閉じるのも「未保存のまま座席を捨てる」操作。
                return if self.clear_unsaved_or_stay() {
                    Outcome::Close
                } else {
                    Outcome::Stay
                };
            }
            Message::ExportPolled => {
                // **intent ではない** — 走っている thread からの返事を受けるだけ。
                self.gateway.poll_export();
            }
            Message::LayerSelected(layer) => {
                let _ = self.gateway.dispatch(UiIntent::SelectLayer { layer });
            }
            Message::Inspector(event) => self.apply_inspector(event),
        }
        Outcome::Stay
    }

    /// Inspector の1押しを intent へ写す。**判断はここに無い** — どの layer の
    /// 話か(いま映している選択)と、スクラブ事象の gesture 区切りを写すだけで、
    /// 「書けるか」「Undo の粒度」はゲートウェイの先のエディタが持つ。
    ///
    /// `KeyParamAtPlayhead` が運ぶ成分値は **accepted snapshot の投影そのもの**
    /// ([`Self::inspector`])から取る。面にも殻にも先行する局所値は無い
    /// (optimistic 禁止 — 2026-08-13 裁定)。
    fn apply_inspector(&mut self, event: InspectorEvent) {
        match event {
            InspectorEvent::SetEffectEnabled {
                definition_id,
                enabled,
            } => {
                let _ = self
                    .gateway
                    .dispatch(UiIntent::SetEffectEnabled {
                        definition_id,
                        enabled,
                    });
            }
            InspectorEvent::ToggleMute | InspectorEvent::ToggleSolo => {
                let Some(layer) = self.inspected_layer() else {
                    return;
                };
                let flag = if matches!(event, InspectorEvent::ToggleMute) {
                    UiItemFlag::Mute
                } else {
                    UiItemFlag::Solo
                };
                let _ = self.gateway.dispatch(UiIntent::ToggleItemFlag { layer, flag });
            }
            InspectorEvent::KeyPressed(param) => {
                let Some(layer) = self.inspected_layer() else {
                    return;
                };
                // 画面に出ている成分値がそのままキーになる(2026-08-13裁定)。
                let InspectorSeat::Ready(model) = self.inspector() else {
                    return;
                };
                let Some(row) = model.transform_row(param) else {
                    return;
                };
                if !row.editable {
                    return;
                }
                let _ = self.gateway.dispatch(UiIntent::KeyParamAtPlayhead {
                    layer,
                    param,
                    components: row.components.clone(),
                });
            }
            InspectorEvent::Scrub {
                param,
                component,
                event,
            } => {
                let Some(layer) = self.inspected_layer() else {
                    return;
                };
                match event {
                    ScrubEvent::Started => {
                        let _ = self.gateway.dispatch(UiIntent::BeginParamEdit { layer, param });
                    }
                    ScrubEvent::Changed(value) => {
                        let _ = self.gateway.dispatch(UiIntent::SetParamComponent {
                            layer,
                            param,
                            component,
                            value,
                        });
                    }
                    ScrubEvent::Committed(value) => {
                        let _ = self.gateway.dispatch(UiIntent::SetParamComponent {
                            layer,
                            param,
                            component,
                            value,
                        });
                        let _ = self.gateway.dispatch(UiIntent::EndParamEdit);
                    }
                    ScrubEvent::Cancelled => {
                        let _ = self.gateway.dispatch(UiIntent::EndParamEdit);
                    }
                }
            }
        }
    }

    /// Inspector が映している layer(選択がちょうど1つのとき)。
    fn inspected_layer(&self) -> Option<u64> {
        let seat = self.gateway.project()?;
        match seat.editor().selected_layers() {
            [one] => Some(one.get()),
            _ => None,
        }
    }

    /// 未保存のまま座席を捨てる操作(New / Open / 窓を閉じる)の前に挟む。
    /// 続行してよければ `true`。判断は `decide_unsaved`、訊き手は
    /// `prompts.unsaved_choice` に居る。「保存して続行」で保存に失敗したら
    /// **続行しない**(帯に理由が出る)。
    ///
    /// 「保存して続行」の保存も利用者が決めた行動なので、`UiIntent::SaveProject`
    /// として記録される — 記録を見た側は New / Open の直前に保存が入ったことを
    /// 読み取れるし、replay も同じ順で保存する。
    fn clear_unsaved_or_stay(&mut self) -> bool {
        let Some(seat) = self.gateway.project() else {
            return true;
        };
        let path = seat.path().to_path_buf();
        let dirty = seat.is_dirty();
        let prompts = &mut self.prompts;
        match decide_unsaved(dirty, || prompts.unsaved_choice(&path)) {
            UnsavedDecision::Proceed => true,
            UnsavedDecision::Stay => false,
            UnsavedDecision::SaveThenProceed => self.gateway.dispatch(UiIntent::SaveProject),
        }
    }

    /// live project が座っているか。スタート画面を出すかどうかがこれで決まる。
    pub fn is_seated(&self) -> bool {
        self.gateway.is_seated()
    }

    /// Inspector の座席 — **accepted snapshot から毎回導出**する(状態は持たない)。
    ///
    /// 選択・playhead の正本はエディタ(座席)に、値の正本は Document snapshot に
    /// 在り、ここはそれを1つの投影に写すだけ(Q5: 単一の真実)。
    pub fn inspector(&self) -> InspectorSeat {
        let Some(seat) = self.gateway.project() else {
            return InspectorSeat::NoSelection;
        };
        let catalog = match &self.catalog {
            Ok(catalog) => catalog,
            Err(reason) => return InspectorSeat::Unreadable(reason.clone()),
        };
        let editor = seat.editor();
        let Some(playhead) = editor.playhead_time() else {
            // fps の格子に載らない playhead はキーを打てない時刻。理由ごと出す。
            return InspectorSeat::Unreadable("the playhead is not a keyable time".to_owned());
        };
        project_inspector(
            &seat.snapshot(),
            catalog,
            editor.selected_layers(),
            playhead,
        )
    }

    /// エディタの status 1行(確定・断りの一言)。空なら席ごと出さない。
    pub fn editor_status(&self) -> Option<String> {
        self.gateway
            .project()
            .map(|seat| seat.editor().status().to_owned())
            .filter(|status| !status.is_empty())
    }

    /// 座席の Document snapshot(読み)。テストと投影の照合用で、
    /// **書く道はここから生えない**(snapshot は immutable)。
    pub fn document(&self) -> Option<std::sync::Arc<motolii_doc::Document>> {
        self.gateway.project().map(|seat| seat.snapshot())
    }

    /// 座っている project のパス。
    pub fn project_path(&self) -> Option<PathBuf> {
        self.gateway
            .project()
            .map(|seat| seat.path().to_path_buf())
    }

    /// 帯が名乗る project 名(パスが名前を持たなければパスそのもの)。
    pub fn project_name(&self) -> Option<String> {
        self.gateway.project().map(|seat| project_name(seat.path()))
    }

    /// 未保存の編集があるか。座席が無ければ「捨てる物が無い」= `false`。
    pub fn is_dirty(&self) -> bool {
        self.gateway.project().is_some_and(|seat| seat.is_dirty())
    }

    /// Export を始められるか。ボタンの enabled と `BeginExport` の門は同じ関数。
    pub fn can_start_export(&self) -> bool {
        self.gateway.can_start_export()
    }

    /// 書き出しが走っているか(帯が Exporting… と Cancel を出すかどうか)。
    pub fn export_running(&self) -> bool {
        self.gateway.export().is_some()
    }

    /// 走っている書き出しの経過秒。走っていなければ `None`。
    pub fn export_elapsed_seconds(&self) -> Option<u64> {
        self.gateway
            .export()
            .map(motolii_ui::export_seat::ExportRun::elapsed_seconds)
    }

    /// 既にキャンセルを頼んだか(Cancel の二度押しを止める)。
    pub fn export_cancel_requested(&self) -> bool {
        self.gateway
            .export()
            .is_some_and(motolii_ui::export_seat::ExportRun::cancel_requested)
    }

    /// status 帯が映す最新の一言。何も言われていなければ帯を出さない。
    pub fn latest_report(&self) -> Option<String> {
        self.gateway.transcript().latest()
    }

    /// 言われた全文(**結果**のログ)。replay の照合に使う。
    pub fn reports(&self) -> Vec<String> {
        self.gateway
            .transcript()
            .entries()
            .into_iter()
            .map(|event| event.text)
            .collect()
    }

    /// 原因のログ全文(順のまま)。`--intent-log` と replay がこれを読む。
    pub fn intents(&self) -> Vec<IntentEvent> {
        self.gateway.journal().entries()
    }

    /// journal に溜まっている行数。
    pub fn intent_count(&self) -> usize {
        self.gateway.journal().len()
    }

    /// 座席の writer 世代(replay の審判用)。座席が無ければ 0。
    pub fn revision(&self) -> u64 {
        self.gateway.revision()
    }

    /// 座席の Document に立っている track item の総数(replay の審判用)。
    pub fn track_item_count(&self) -> usize {
        self.gateway.track_item_count()
    }

    /// 既に `count` 行流した側が、その後に増えた分だけ受け取る
    /// ([`IntentLog`](crate::IntentLog) 用)。
    pub(crate) fn intents_since(&self, count: usize) -> Vec<IntentEvent> {
        self.gateway.journal().since(count)
    }

    /// 同じく結果のログの続き([`StatusLog`](crate::StatusLog) 用)。
    pub(crate) fn statuses_since(&self, count: usize) -> Vec<StatusEvent> {
        self.gateway.transcript().since(count)
    }

    /// transcript に溜まっている行数。
    pub(crate) fn status_count(&self) -> usize {
        self.gateway.transcript().len()
    }
}

/// 帯が名乗る名前。egui shell の status 帯と同じ決め方。
fn project_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
