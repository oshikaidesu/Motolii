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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use motolii_audio::PcmCache;
use motolii_doc::{Document, LayerId};
use motolii_ui::blitz_shell::{
    decide_unsaved, IntentEvent, Resume, ShellGateway, ShellPrompts, ShellTranscript, StatusEvent,
    UiIntent, UiItemFlag, UnsavedDecision,
};

use crate::browser::{BrowserCard, BrowserPane};
use crate::inspector_model::{project_inspector, InspectorSeat};
use crate::inspector_pane::InspectorEvent;
use crate::message::Message;
use crate::timeline::{TimelineCtx, TimelinePane, WaveformBandState, WaveformSeat};
use crate::widgets::ScrubEvent;

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
    /// Browser pane の view 状態(rail・選択・受け皿表示)と登録 folder の走査。
    /// Document の写しは**持たない**(読みは都度 `gateway` の snapshot を渡す)。
    browser: BrowserPane,
    /// Timeline pane の view 状態(zoom / pan / scroll / 進行中ジェスチャ)。
    /// Document には入らない Project session の棚(M-3)。
    timeline: TimelinePane,
    /// soundtrack 波形の生成座席と、その最新の答え・decode 済み PCM の控え。
    waveform: WaveformSeat,
    waveform_state: WaveformBandState,
    pcm_caches: HashMap<(String, u32), Arc<PcmCache>>,
    /// 直前の [`Message::PlaybackTick`] の時刻。**製品状態ではない** —
    /// `dt` を計るためだけの host 側の刻み(egui の `ctx.input(|i| i.stable_dt)`
    /// に相当する、iced にはフレームワークが持ってこない値を自分で測る)。
    /// `TogglePlayPressed` のたびに `None` へ戻す — pause で刻みの購読が外れる
    /// あいだの実時間を、次の再生開始時にまとめて足さないため
    /// (2026-08-19 iced 再生機構移植レーン)。
    last_playback_tick: Option<std::time::Instant>,
}

impl Shell {
    /// 座席なしで始める(スタート画面)。Browser の登録 folder は既定
    /// (repo の starter media)。
    pub fn new(prompts: impl ShellPrompts + 'static) -> Self {
        Self::with_browser(
            prompts,
            BrowserPane::default_shell(),
            ShellGateway::new(ShellTranscript::default()),
        )
    }

    /// 登録 folder を差し替えて始める(運転席テスト用。窓の経路は同じ)。
    pub fn with_browser_root(prompts: impl ShellPrompts + 'static, root: PathBuf) -> Self {
        Self::with_browser(
            prompts,
            BrowserPane::with_root(root),
            ShellGateway::new(ShellTranscript::default()),
        )
    }

    /// 座席ごと始める — `--project` で窓より先に開いた座席か、引数なし起動の
    /// [`resume::decide_resume`](crate::resume::decide_resume) が返す3択(`Resume`)を
    /// 受ける。判断はここに無く、`motolii_ui::blitz_shell::ShellGateway::resumed` が
    /// 済ませている(egui 版 `BlitzShellApp::with_seat` と同じ分担)。
    ///
    /// `last_project` を渡した経路(窓を開く `main.rs`)だけが、次に座り直した
    /// project を覚える — 渡さなければ(`None`)何も書かない。テストと replay が
    /// 利用者の「続きが開く」設定を踏まないための境目は egui 版と同じ。
    pub fn resumed(
        prompts: impl ShellPrompts + 'static,
        resume: Resume,
        last_project: Option<PathBuf>,
    ) -> Self {
        let gateway =
            ShellGateway::resumed(ShellTranscript::default(), resume).remembering(last_project);
        Self::with_browser(prompts, BrowserPane::default_shell(), gateway)
    }

    fn with_browser(
        prompts: impl ShellPrompts + 'static,
        browser: BrowserPane,
        gateway: ShellGateway,
    ) -> Self {
        let mut shell = Self {
            gateway,
            prompts: Box::new(prompts),
            catalog: motolii_plugin::reference::reference_catalog()
                .map(Arc::new)
                .map_err(|error| format!("plugin catalog を作れない: {error}")),
            browser,
            timeline: TimelinePane::default(),
            waveform: WaveformSeat::default(),
            waveform_state: WaveformBandState::Absent,
            pcm_caches: HashMap::new(),
            last_playback_tick: None,
        };
        // 走査時に作れなかったサムネイルの理由は、最初から帯経路に居る(F-09)。
        shell.drain_browser_notices();
        shell
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
                if self.gateway.dispatch(UiIntent::NewProject { path }) {
                    // 座り直したら Timeline の view も最初から(別 project の
                    // zoom / ジェスチャを引き継がない)。
                    self.timeline.reset();
                }
                self.poll_waveform();
            }
            Message::OpenProjectPressed => {
                if !self.clear_unsaved_or_stay() {
                    return Outcome::Stay;
                }
                let Some(path) = self.prompts.open_project_path() else {
                    return Outcome::Stay;
                };
                if self.gateway.dispatch(UiIntent::OpenProject { path }) {
                    self.timeline.reset();
                }
                self.poll_waveform();
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
                // 落ちた音声が曲になっていたら、波形の生成がここから始まる。
                self.poll_waveform();
            }
            Message::Timeline(timeline_message) => {
                // 座席が無ければ Timeline は立っていない(絵も出ていない)。
                let Some(ctx) = self.timeline_ctx() else {
                    return Outcome::Stay;
                };
                // 1) この Message で起こす intent を pane に訊く(pane は書かない)。
                let intents = self.timeline.plan(&timeline_message, &ctx);
                // 2) dispatch はここ1箇所(journal → 実行。egui 版と同じ背骨)。
                for intent in intents {
                    let _ = self.gateway.dispatch(intent);
                }
                // 3) intent が効いた後の座席を見て、pane が自分の状態を進める。
                if let Some(ctx) = self.timeline_ctx() {
                    self.timeline.note(&timeline_message, &ctx);
                }
            }
            Message::WaveformPolled => {
                // **intent ではない** — decode thread からの返事を受けるだけ。
                self.poll_waveform();
            }
            Message::StagePolled => {
                // **intent ではない** — この Message 自体は状態を変えない。
                // Stage 島の `frame_seat` は shader widget の thread_local
                // (`stage_island::Embed`)が持ち、`prepare` が毎回 request/poll
                // する。ここが在る理由は redraw の刻みだけ:
                // `main.rs::Host::subscription` の `window::frames()` 購読が
                // この Message を刻み続けて `update → view` を回すことで、
                // 非同期に完成した評価済みフレームを次の `prepare` が拾える
                // (`ExportPolled` / `WaveformPolled` と同じ型の解決)。
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
            Message::StageReported(lines) => {
                // **intent ではない** — Stage 島の報告(初期化・texture import・描画の
                // 失敗)を帯 / `--status-log` へ写すだけ。journal には載せない
                // (replay は Stage の故障まで再現しない)。egui shell の pane が
                // `transcript.report(...)` している行と同じ層である。
                for line in lines {
                    self.gateway.transcript().report(line);
                }
            }
            Message::LayerSelected(layer) => {
                let _ = self
                    .gateway
                    .dispatch(UiIntent::SelectLayer { layer, additive: false });
            }
            Message::TogglePlayPressed => {
                // **intent ではない**(module doc 参照)。`ShellGateway` の
                // dispatch を通らない専用口(`poll_export` と同じ型)。
                self.gateway.toggle_playing();
                // 次の tick の dt を 0 から始める — pause のあいだの実時間を
                // 次の再生開始時にまとめて足さない(`last_playback_tick` の doc)。
                self.last_playback_tick = None;
            }
            Message::ToggleLoopPressed => {
                self.gateway.toggle_loop();
            }
            Message::ToStartPressed => {
                // playhead を特定の値(0)へ置く操作なので、scrub 確定と同じ
                // intent を通す(`SetPlayhead` — こちらは replay で再現できる)。
                let _ = self.gateway.dispatch(UiIntent::SetPlayhead { at_us: 0 });
            }
            Message::PlaybackTick => {
                // **intent ではない** — 時間経過そのものは操作ではない
                // (zoom/pan と同じ scope、module doc 参照)。dt は host が測る
                // (egui の `stable_dt` に相当する値をここで作る)。
                let now = std::time::Instant::now();
                let dt = self
                    .last_playback_tick
                    .take()
                    .map(|last| (now - last).as_secs_f32())
                    .unwrap_or(0.0);
                self.last_playback_tick = Some(now);
                self.gateway.tick_playback(dt);
            }
            Message::Inspector(event) => self.apply_inspector(event),
            Message::BrowserRailChosen(rail) => {
                // pane 内の表示切替。Document に触れないので intent にならない
                // (view 系 intent は `blitz_shell/intent.rs` の将来枠)。
                self.browser.set_rail(rail);
            }
            Message::BrowserCardClicked(id) => {
                // 単クリック = 選択だけ(Q1)。報酬は selection tray と枠の強調。
                self.browser.select(&id);
            }
            Message::BrowserCardActivated(id) => {
                // 置いたカードは選ばれてもいる(egui 版と同じ)。
                self.browser.select(&id);
                match self.browser_place_path(&id) {
                    Some(path) => {
                        // **OS ドロップと同じ1本の合流点。** 成立も失敗も帯が言う
                        // (`admit_dropped_paths` の一言がそのまま出る)。
                        let _ = self.gateway.dispatch(UiIntent::AdmitPaths { paths: vec![path] });
                    }
                    None => {
                        // 拒否の報酬(Q3): 黙って無視しない。存在しない path を
                        // intent にもしない(egui 版 `place_request` と同じ判断)。
                        self.gateway.transcript().report(format!(
                            "browser: cannot place {id} — the file is missing or was moved"
                        ));
                    }
                }
            }
            Message::BrowserDropHover(hovering) => {
                // panel 内の受け皿表示だけ。取り込みは `FilesDropped` のまま。
                self.browser.set_drop_hover(hovering);
            }
        }
        // Document が進んでいれば、新しく載った image asset の縮小実体を用意する
        // (既に在る分は fs metadata を見るだけ)。作れなかった理由は帯経路へ。
        self.sync_browser();
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

    /// Browser の読み(rail・選択・受け皿表示・サムネイル)。
    pub fn browser(&self) -> &BrowserPane {
        &self.browser
    }

    /// いまの rail の card 一覧(Document snapshot を写した投影)。
    pub fn browser_cards(&self) -> Vec<BrowserCard> {
        let seat = self.gateway.project();
        let snapshot = seat.map(|seat| seat.snapshot());
        let root = self.browser_project_root();
        self.browser
            .cards(snapshot.as_deref(), root.as_deref())
    }

    /// card 1枚が指す実ファイル(配置要求の解決)。
    fn browser_place_path(&self, id: &str) -> Option<PathBuf> {
        let snapshot = self.gateway.project().map(|seat| seat.snapshot());
        let root = self.browser_project_root();
        self.browser
            .place_path(id, snapshot.as_deref(), root.as_deref())
    }

    /// project root = document path の親(CLI export と同じ規約)。
    fn browser_project_root(&self) -> Option<PathBuf> {
        self.gateway
            .project()
            .and_then(|seat| seat.path().parent().map(Path::to_path_buf))
    }

    /// Document の進みに Browser のサムネイルを追いつかせ、理由を帯へ写す。
    fn sync_browser(&mut self) {
        if let Some(seat) = self.gateway.project() {
            let snapshot = seat.snapshot();
            let revision = seat.editor().revision();
            let root = seat.path().parent().map(Path::to_path_buf);
            self.browser
                .ensure_asset_thumbnails(&snapshot, root.as_deref(), revision);
        }
        self.drain_browser_notices();
    }

    /// pane が返した理由を status 帯(= `--status-log`)へ写す。
    /// stderr に書かない(2026-08-18 外部診断 F-09 の流儀)。
    fn drain_browser_notices(&mut self) {
        for notice in self.browser.take_notices() {
            self.gateway.transcript().report(notice);
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

    /// composition の縦横比(幅/高さ)。Stage 島の document camera の横合わせに要る。
    ///
    /// 出所は座席の Document(正準では高さ1.0固定なので縦横比が寸法の全部 —
    /// egui shell の pane と同じ読み方)。座っていなければ `None`。
    pub fn composition_aspect(&self) -> Option<f32> {
        self.gateway.project().map(|seat| {
            let snapshot = seat.snapshot();
            let composition = &snapshot.composition;
            composition.aspect_num() as f32 / composition.aspect_den() as f32
        })
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

    // ---- Timeline pane の読み口(M-3)。canvas と view はここからしか読まない ----

    /// Timeline pane の view 状態(読み)。
    pub fn timeline_pane(&self) -> &TimelinePane {
        &self.timeline
    }

    /// pane へ流す immutable snapshot。座席が無ければ `None`。
    pub fn timeline_snapshot(&self) -> Option<Arc<Document>> {
        self.gateway.project().map(|seat| seat.snapshot())
    }

    /// いま選ばれている layer(選んだ順)。
    pub fn timeline_selection(&self) -> Vec<LayerId> {
        self.gateway
            .project()
            .map(|seat| seat.editor().selected_layers().to_vec())
            .unwrap_or_default()
    }

    /// playhead(秒)。座席が無ければ 0。
    pub fn timeline_playhead(&self) -> f32 {
        self.gateway
            .project()
            .map(|seat| seat.editor().playhead_seconds())
            .unwrap_or(0.0)
    }

    /// 再生中か(transport ボタンの絵)。座席が無ければ `false`。
    pub fn timeline_playing(&self) -> bool {
        self.gateway.is_playing()
    }

    /// loop が有効か。座席が無ければ `false`。
    pub fn timeline_loop_on(&self) -> bool {
        self.gateway.loop_on()
    }

    /// undo 台帳の深さ(帯の Undo ボタンの enabled)。
    pub fn undo_len(&self) -> usize {
        self.gateway
            .project()
            .map(|seat| seat.editor().undo_len())
            .unwrap_or(0)
    }

    /// redo 台帳の深さ(帯の Redo ボタンの enabled)。
    pub fn redo_len(&self) -> usize {
        self.gateway
            .project()
            .map(|seat| seat.editor().redo_len())
            .unwrap_or(0)
    }

    /// 波形帯のいまの答え(読み)。canvas が描く。
    pub fn waveform_state(&self) -> WaveformBandState {
        self.waveform_state.clone()
    }

    /// 波形の生成が走っているか(`window::frames()` を刻み続けるかの判断)。
    pub fn waveform_building(&self) -> bool {
        self.waveform.is_building()
    }

    /// plan / note へ渡す読み取り一式。座席が無ければ `None`。
    fn timeline_ctx(&self) -> Option<TimelineCtx> {
        let seat = self.gateway.project()?;
        Some(TimelineCtx {
            document: seat.snapshot(),
            selected: seat.editor().selected_layers().to_vec(),
            playhead: seat.editor().playhead_seconds(),
        })
    }

    /// 波形の生成座席を1歩進める。decode は別 thread — ここでは受け取るだけ。
    fn poll_waveform(&mut self) {
        let Some(seat) = self.gateway.project() else {
            self.waveform_state = WaveformBandState::Absent;
            return;
        };
        let document = seat.snapshot();
        let root = seat.editor().project_root().map(Path::to_path_buf);
        self.waveform_state =
            self.waveform
                .poll(&document, root.as_deref(), &mut self.pcm_caches);
    }
}

/// 帯が名乗る名前。egui shell の status 帯と同じ決め方。
fn project_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
