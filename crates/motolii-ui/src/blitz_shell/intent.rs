//! **原因のログ**と、その唯一の実行口(単一ゲートウェイ)。
//!
//! [2026-08-18裁定](../../../../docs/reviews/2026-08-18-log-and-structure-enforcement.md)
//! (「ログと構造の強制が組めれば egui は iced になれる」)の第1弾。iced が
//! フレームワークとして強制する2つの規律を、toolkit を替えずに**この module の
//! 形**で強制する:
//!
//! 1. **ログ** … 利用者が起こした shell 層の状態変化は全て [`UiIntent`] 1件になり、
//!    **行動の前に** [`IntentJournal`] へ記録される。落ちた行動も「何をしようと
//!    したか」が残る。`ShellTranscript`(結果のログ)と対になる
//! 2. **構造** … その intent を実行できるのは [`ShellGateway::dispatch`] だけで、
//!    製品状態(`ProjectSeat` = Document の唯一の writer / 実行中の書き出し)は
//!    この module の外から掴めない。journal を通らずに同じ副作用へ着く道が
//!    shell から消える(フェンス: `tests/shell_intent_gateway_fence.rs`)
//!
//! そして [`ShellGateway::replay`] が、記録した intent 列を**窓も egui も無しで**
//! 再実行する。iced の time-travel に相当する検証をこれで自前に持つ
//! (審判は `drive_tests.rs`)。
//!
//! ## ここに無いもの
//!
//! - **訊くこと**。dialog(`ShellPrompts`)は intent の外に居て、**決まった答え**
//!   (`NewProject { path }` の path)だけが intent の中へ入る。だから replay は
//!   dialog を二度と開かない
//! - **view の操作**。camera・panel の並びは本レーンのスコープ外(将来枠)。
//!   Timeline の zoom / pan / scroll も view の状態なので intent にしない
//!   (再現は shell 側の Message 列 replay が持つ)
//! - **egui pane の中の編集の intent 化**。Timeline の編集 intent
//!   (`SelectLayer` 〜 `StepPlayhead`)は 2026-08-18 M-3 で入り、iced shell は
//!   これだけを通る。egui pane は当面 `project_mut` の穴(下記)で D2 へ直行する

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::drive::ShellTranscript;
use crate::export_seat::{can_start_export, ExportFinish, ExportRequest, ExportRun};
use crate::timeline_editor::{ItemFlag, TimelineEditor};
use crate::timeline_rows::ParamRef;

/// intent が運ぶ µs を秒へ。逆(秒 → µs)は shell 側([`seconds_to_us`])が持つ。
fn us_to_seconds(us: i64) -> f32 {
    (us as f64 / 1_000_000.0) as f32
}

/// 秒 → µs(intent へ入れる側)。shell(iced / egui)が dispatch の前に呼ぶ。
/// 丸めは 1µs で、フレーム境界(実行側の `seconds_to_time`)より2桁以上細かい。
pub fn seconds_to_us(seconds: f32) -> i64 {
    (f64::from(seconds) * 1_000_000.0).round() as i64
}

// ---------------------------------------------------------------------------
// ログ: 型付き intent と、その台帳
// ---------------------------------------------------------------------------

/// 利用者が起こした shell 層の状態変化1件。**製品(`motolii-doc`)の型は
/// 漏らしてよいが、egui / kittest の型は入れない**(U0a の境界規律)。
///
/// dialog の答えは値として中に入っている(`NewProject { path }`)ので、
/// この列だけで replay が成立する — 訊き直す口はどこにも要らない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiIntent {
    /// 新しい project を作って、そこへ座り直す(Cmd+N / スタート画面の New)。
    NewProject { path: PathBuf },
    /// 既にある project へ座り直す(Cmd+O / スタート画面の Open / `--project` 起動)。
    OpenProject { path: PathBuf },
    /// いまの writer snapshot を project ファイルへ書き戻す(Cmd+S)。
    SaveProject,
    /// 書き出しを始める(status 帯の Export)。`output` は dialog が決めた保存先。
    BeginExport { output: PathBuf },
    /// 実行中の書き出しを止める(status 帯の Cancel)。
    CancelExport,
    /// 素材を取り込んで playhead へ置く。**OS ドロップと Browser のカードの
    /// ダブルクリックが合流する1点**で、入口が増えても intent は1種類のまま。
    AdmitPaths { paths: Vec<PathBuf> },
    // ---- ここから wave E の第1弾(2026-08-18 M-4b Inspector) ----
    //
    // Inspector が起こす編集は全部この列になる。**実体は既存の `TimelineEditor`
    // 操作 API へ1対1で落ちるだけ**で、新しい編集経路・新しい Document command は
    // ここに無い(D2 journal / Undo の粒度はエディタ側の決定のまま)。
    // wire は layer identity を u64 で運ぶ(2026-08-10「wire carries layer
    // identity only」。`motolii_doc::LayerId` は `#[serde(transparent)]` なので
    // JSON 上は同じ数値 — M-3 の Timeline 選択と M-4b の Inspector 選択が
    // 同じ intent 1本に合流できる)。
    //
    // `SelectLayer` は Stage / Timeline のクリックと Inspector が誰を映すかの
    // 両方の原因になる、選択の唯一の intent(2026-08-18 統合第2弾で union)。
    // `additive` は Timeline の Cmd 併用(足す / 外すのトグル)、それ以外の
    // 呼び出し元(Inspector / Stage クリック)は `false` を渡す。
    /// 選択を1つに置き換える(または `additive` で足す/外す)。
    /// Document には入らない session 状態だが、Inspector が誰を映すかの原因なので
    /// journal に載る。
    SelectLayer { layer: u64, additive: bool },
    /// M / S を1手反転する。押下状態の正本は Document(`ItemEnvelope`)で、
    /// intent は反転要求だけを運ぶ(F-03 枝A)。
    ToggleItemFlag { layer: u64, flag: UiItemFlag },
    /// Inspector の数値を掴んだ。ここから `EndParamEdit` までが1 gesture = 1 Undo
    /// (エディタの `begin_param_edit` と同じ意味)。
    BeginParamEdit { layer: u64, param: UiEditParam },
    /// 成分1本を書く(`set_param_component`)。開いている gesture があれば
    /// そこへ入り、無ければ1回ぶんの gesture で書く。
    SetParamComponent {
        layer: u64,
        param: UiEditParam,
        component: usize,
        value: f64,
    },
    /// 掴みを離した(`end_param_edit`)。
    EndParamEdit,
    /// ◇: playhead へキーを打つ(`key_param_at_playhead`)。`components` は
    /// 画面に出ている成分値で、**画面の数がそのままキーになる**。
    KeyParamAtPlayhead {
        layer: u64,
        param: UiEditParam,
        components: Vec<f64>,
    },
    /// 共有 FX の ON/OFF(`set_effect_enabled`)。相手は `EffectDefinition`。
    SetEffectEnabled { definition_id: u64, enabled: bool },

    // ---- Timeline の編集(2026-08-18 iced ホスト移行 M-3 で intent 化) ----
    //
    // 時刻は **マイクロ秒の整数**で運ぶ(`*_us`)。秒の f32 を入れると `Eq` と
    // JSONL の決定性が壊れる。フレーム境界への吸着は intent の後ろ
    // (`commit_drag` の `seconds_to_time`)がやるので、µs 量子化は意味に届かない。
    //
    // ドラッグ系(Move / Trim)は **release の1件だけ**が intent になる。
    // ドラッグ中の絵は shell 側の preview で、Document は release まで触らない —
    // 途中で Esc したら intent が1つも残らず、復元がそのまま成立する
    // (`timeline_move_gesture.rs` / `timeline_trim_gesture.rs` の transient
    // lifecycle と同じ判断)。egui pane の live-commit とはここが違う。
    /// Timeline: 何も無い所のクリック = 選択を空にする。
    ClearSelection,
    /// Timeline: 選択中の clip 群を move gesture 1回ぶん動かす(release で1件)。
    /// 実行は `begin_selected_clips_move` → `drag_to` → `release` —
    /// egui のドラッグと同じ経路・同じクランプ(端は塊のまま止まる)・同じキー追従。
    MoveClips {
        grabbed: motolii_doc::LayerId,
        grab_at_us: i64,
        drop_at_us: i64,
    },
    /// Timeline: clip の端の trim(release で1件)。D2 の
    /// `prepare_trim_clip_in` / `prepare_trim_clip_out` が最終判断を持つ。
    TrimClip {
        layer: motolii_doc::LayerId,
        edge: crate::timeline_editor::TrimEdge,
        at_us: i64,
    },
    /// Timeline: 選択中のものを消す(キー選択が先、無ければ層。Group は中身ごと)。
    DeleteSelection,
    /// Undo(Cmd+Z / 帯のボタン)。1 gesture = 1 Undo 単位で戻る。
    Undo,
    /// Redo(Shift+Cmd+Z / 帯のボタン)。
    Redo,
    /// playhead を置く(ルーラのスクラブの確定)。フレーム境界へ乗る。
    SetPlayhead { at_us: i64 },
    /// playhead を矢印キーでコマ送りする(±1 / Shift ±10)。
    StepPlayhead { frames: i32 },

    // ---- Timeline のキー編集(2026-08-19 M-6 キー編集レーン)。名前は
    // 構造操作レーン(Group/Rename/Lock/畳み開閉)と同時に走る発注の凍結名
    // そのまま。KeyframeId は運ばない — `at_us` / `from_us` で名指し、
    // 実行の直前にゲートウェイが引き直す(wire は layer identity だけを運ぶ
    // 2026-08-10 の決定と同じ理由)。 ----
    /// Timeline: キー1本を消す(菱形選択中の Delete / Backspace)。
    RemoveParamKey {
        layer: u64,
        param: UiEditParam,
        at_us: i64,
    },
    /// Timeline: キー1本の時刻をドラッグで動かす(release で1件)。
    MoveParamKey {
        layer: u64,
        param: UiEditParam,
        from_us: i64,
        to_us: i64,
    },
    /// Timeline: キー1本の補間を右クリックで指定する。**D2 の口は Position だけ**
    /// (`prepare_set_position_key_interp`)— 他は断られる(帯が理由を言う)。
    SetParamKeyInterp {
        layer: u64,
        param: UiEditParam,
        at_us: i64,
        interp: UiKeyInterp,
    },
    // 将来枠(足す時はフェンスの禁止リストも一緒に伸ばす):
    // - view: camera 操作・panel の開閉/並び
    // - wave E 残り: audio gain(エディタに操作 API が立ってから)
}

/// intent が運ぶ M / S。Timeline 側の `ItemFlag` と一対一(Lock は Timeline の
/// 行だけの操作なので wire に載せない — 載せるときは Inspector 側の UI と一緒に)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiItemFlag {
    Mute,
    Solo,
}

impl UiItemFlag {
    fn to_item_flag(self) -> ItemFlag {
        match self {
            Self::Mute => ItemFlag::Mute,
            Self::Solo => ItemFlag::Solo,
        }
    }
}

/// intent が運ぶ編集対象 param。エディタの `ParamRef` と同じ語彙のうち、
/// Inspector が行を出す4つだけ(Anchor は Inspector に行が無いので wire にも
/// 載せない — 行を出すときに一緒に増やす)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiEditParam {
    Position,
    Scale,
    Rotation,
    Opacity,
}

impl UiEditParam {
    fn to_param_ref(self) -> ParamRef {
        match self {
            Self::Position => ParamRef::Position,
            Self::Scale => ParamRef::Scale,
            Self::Rotation => ParamRef::Rotation,
            Self::Opacity => ParamRef::Opacity,
        }
    }
}

/// intent が運ぶキーの補間。egui 版の右クリックメニュー
/// (`Hold` / `Linear` / `Ease in-out`)と同じ3択(2026-08-19 M-6)。
/// `motolii_eval::Interp` の `Bezier { .. }` は数値を4つ持つので wire に
/// そのまま出さない — `Smooth` は egui 版と同じ既定パラメータへ写す。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiKeyInterp {
    Hold,
    Linear,
    Smooth,
}

impl UiKeyInterp {
    fn to_interp(self) -> motolii_eval::Interp {
        match self {
            Self::Hold => motolii_eval::Interp::Hold,
            Self::Linear => motolii_eval::Interp::Linear,
            // egui 版 `key_menu` の "Ease in-out" と同じ既定値。
            Self::Smooth => motolii_eval::Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
        }
    }
}

/// journal の1行。`seq` は 1 始まりの通し番号で、`--intent-log` の JSONL
/// (`{"seq":n,"intent":{"kind":"…",…}}`)がそのまま名乗る番号でもある。
/// `--status-log`(`{"seq":n,"text":"…"}`)と同型で、並べて読める。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentEvent {
    pub seq: u64,
    pub intent: UiIntent,
}

/// 原因のログを溜める唯一の場所。`ShellTranscript` と同じ形(`Arc<Mutex<_>>` の
/// 共有)で、runner が `--intent-log` へ流す側もここを読む。
#[derive(Clone, Default)]
pub struct IntentJournal {
    entries: Arc<Mutex<Vec<IntentEvent>>>,
}

impl IntentJournal {
    /// 1件記録して、付けた `seq` を返す。**呼ぶのは行動の前**。
    fn record(&self, intent: &UiIntent) -> u64 {
        let mut entries = self.lock();
        let seq = entries.len() as u64 + 1;
        entries.push(IntentEvent {
            seq,
            intent: intent.clone(),
        });
        seq
    }

    /// 記録の全部(順のまま)。
    pub fn entries(&self) -> Vec<IntentEvent> {
        self.lock().clone()
    }

    /// intent だけを順に。replay へそのまま渡せる形。
    pub fn intents(&self) -> Vec<UiIntent> {
        self.lock().iter().map(|event| event.intent.clone()).collect()
    }

    /// 既に `count` 行を見た側が、その後に増えた分だけ受け取る(`--intent-log` 用)。
    pub fn since(&self, count: usize) -> Vec<IntentEvent> {
        let mut entries = self.entries();
        if count >= entries.len() {
            return Vec::new();
        }
        entries.drain(..count);
        entries
    }

    /// 溜まっている行数。
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// `ShellTranscript::lock` と同じ方針(毒されても台帳を失わない)。
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<IntentEvent>> {
        self.entries
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

// ---------------------------------------------------------------------------
// 構造: 製品状態へ触れる唯一の口
// ---------------------------------------------------------------------------

/// shell 層の製品状態と、そこへ触る唯一の口。
///
/// `project`(Document の唯一の writer を抱える座席)と `export`(実行中の
/// 書き出し)は**この型の private フィールド**で、外へ出るのは読み取りと
/// 「wave E で残した穴」(`project_mut`)だけである。状態を進めたい側は
/// [`dispatch`](Self::dispatch) に intent を渡すしかない。
pub struct ShellGateway {
    project: Option<ProjectSeat>,
    export: Option<ExportRun>,
    /// 結果のログ(status 帯 + `--status-log`)。窓と面が共有している台帳そのもの。
    transcript: ShellTranscript,
    /// 原因のログ。
    journal: IntentJournal,
    /// 「最後に開いていた project」の置き場(user 設定層)。座り直しが成立する
    /// たびにここへ1本だけ書き、**次の引数なし起動がそれを開く**(F-01)。
    ///
    /// `None` は**覚えない**。replay と単体テストは利用者の設定を踏んではならず、
    /// 覚え先を渡すのは窓を開く経路(`BlitzShellApp::with_seat`)だけである。
    last_project: Option<PathBuf>,
}

/// 引数なし起動の座席をどうするか。**判断は `resume_last_project` が済ませ**、
/// ゲートウェイはこの3択を受け取るだけである。
///
/// 変種の大きさは揃わない(座席は OS lock と writer を抱える)が、この値は
/// 起動時に1個だけ作って1回 move するだけなので box に包む意味が無い。
///
/// **公開している**理由: iced shell も同じ3択を通る(2026-08-19 M-2: `--project` /
/// 引数なし起動の「続きが開く」)。egui 版と同じ言い方(黙って fixture にも
/// 空の窓にも落ちない)を2つ目の型で複製させない。
#[allow(clippy::large_enum_variant)]
pub enum Resume {
    /// 覚えていた project が開けた。`--project` 起動と同じ座り方をする。
    Seated(ProjectSeat),
    /// 覚えていない(初回起動)。何も言わずスタート画面。
    Nothing,
    /// 覚えていたのに開けない。**理由を帯に言ってから**スタート画面。
    Explained(String),
}

/// 引数なし起動で座り直す先を決める。**黙って fixture にも空の窓にも落ちない。**
///
/// 覚えていない = 初回起動なので `Nothing`。覚えていたのに読めない・開けない
/// (消された・別プロセスが握っている・壊れている)は、理由と次の一手を持った
/// `Explained` にする — `--project` が開けなかった時に起動失敗として理由を出す
/// のと同じ原則を、自動 open の側でも守る。
///
/// **公開している**理由: iced shell の引数なし起動もこの判断を**そのまま**呼ぶ
/// (2026-08-19 M-2)。判断を2つ目に実装しない。
pub fn resume_last_project(store: Option<&Path>) -> Resume {
    let Some(store) = store else {
        return Resume::Nothing;
    };
    let remembered = match crate::last_project::load_last_project(store) {
        Ok(Some(path)) => path,
        Ok(None) => return Resume::Nothing,
        Err(error) => {
            return Resume::Explained(format!(
                "最後に開いていた project を思い出せない: {error} — \
                 Cmd+N で作るか Cmd+O で開く"
            ))
        }
    };
    match ProjectSeat::open(&remembered) {
        Ok(seat) => Resume::Seated(seat),
        // `ProjectSeat::open` の Err は既に path を名指ししている。
        Err(error) => Resume::Explained(format!("{error} — Cmd+N で作るか Cmd+O で開く")),
    }
}

impl ShellGateway {
    /// 座席なしで作る(スタート画面 / fixture 展示の起動)。
    pub fn new(transcript: ShellTranscript) -> Self {
        Self {
            project: None,
            export: None,
            transcript,
            journal: IntentJournal::default(),
            last_project: None,
        }
    }

    /// 座り直しを覚える先(user 設定層の path)を渡す。**渡さない限り何も書かない** —
    /// replay も単体テストも、利用者の「最後に開いていた project」を踏まない。
    ///
    /// **公開している**理由: iced shell も窓を開く経路(`main.rs`)だけがここへ
    /// `default_last_project_path()` を渡す(2026-08-19 M-2)。egui 版
    /// `BlitzShellApp::with_seat` と同じ境目を共用する。
    pub fn remembering(mut self, store: Option<PathBuf>) -> Self {
        self.last_project = store;
        self
    }

    /// 引数なし起動の座席。[`resume_last_project`] の3択を受けて、座るか・
    /// 理由を言うかを決める。**座る道は `--project` と同じ**([`Self::seated`])なので、
    /// 自動 open も journal の第1行に `OpenProject` を持つ。
    ///
    /// **公開している**理由: iced shell の `--project` 起動・引数なし起動が
    /// どちらもここを通る(2026-08-19 M-2。`motolii_shell_iced::resume::decide_resume`)。
    pub fn resumed(transcript: ShellTranscript, resume: Resume) -> Self {
        match resume {
            Resume::Seated(seat) => Self::seated(transcript, seat),
            Resume::Nothing => Self::new(transcript),
            Resume::Explained(reason) => {
                // 開けなかったことは**帯が言う**。起きなかった行動は journal に
                // 載せない(intent は起こした行動の記録であって、失敗の記録ではない)。
                transcript.report(reason);
                Self::new(transcript)
            }
        }
    }

    /// `--project` で**窓より先に**開いた座席を受け取って作る。
    ///
    /// 起動時の失敗を絵ではなく起動エラーにする決定(`runner.rs`)は変えないので、
    /// 開く操作そのものは runner が済ませている。ただし journal は
    /// 「どうやってこの座席に着いたか」を欠かしてはならないので、**同じ意味の
    /// `OpenProject` を第1行として記録する** — こうしておくと `--intent-log` は
    /// それ単体で replay 可能な列になる。
    pub(crate) fn seated(transcript: ShellTranscript, seat: ProjectSeat) -> Self {
        let journal = IntentJournal::default();
        journal.record(&UiIntent::OpenProject {
            path: seat.path().to_path_buf(),
        });
        Self {
            project: Some(seat),
            export: None,
            transcript,
            journal,
            last_project: None,
        }
    }

    /// **intent を記録してから実行する、唯一の口。**
    ///
    /// 返すのは「意図どおりに進んだか」で、進まなかった理由は必ず transcript に
    /// 言われている(黙って false を返さない)。呼び手はこの真偽で続きを決める
    /// (未保存 guard の「保存して続行」など)。
    pub fn dispatch(&mut self, intent: UiIntent) -> bool {
        // 行動の**前**に記録する。実行が panic しても・失敗しても、
        // 「何をしようとしたか」は残る。
        self.journal.record(&intent);
        self.perform(&intent)
    }

    /// 記録済みの intent 列を、新しい shell 状態へ再実行する(replay oracle)。
    ///
    /// 窓も egui も dialog も要らない — intent が答えを値として持っているため。
    /// 前提は「初期状態が同じであること」で、file system もその初期状態に含まれる
    /// (`NewProject` は既にあるファイルを踏まない)。
    pub fn replay(intents: &[UiIntent]) -> Self {
        let mut gateway = Self::new(ShellTranscript::default());
        for intent in intents {
            gateway.dispatch(intent.clone());
        }
        gateway
    }

    /// intent 1件の実体。**ここが shell 層の副作用の全部**である。
    fn perform(&mut self, intent: &UiIntent) -> bool {
        match intent {
            UiIntent::NewProject { path } => match create_project_file(path) {
                Ok(()) => self.reseat(path),
                Err(error) => {
                    self.transcript.report(error);
                    false
                }
            },
            UiIntent::OpenProject { path } => self.reseat(path),
            UiIntent::SaveProject => match self.project.as_mut() {
                Some(seat) => match seat.save() {
                    Ok(()) => {
                        self.transcript
                            .report(format!("saved {}", seat.path().display()));
                        true
                    }
                    Err(error) => {
                        self.transcript.report(error);
                        false
                    }
                },
                None => {
                    self.transcript
                        .report("open a project first — Cmd+N to create one, Cmd+O to open one");
                    false
                }
            },
            UiIntent::BeginExport { output } => {
                if !self.can_start_export() {
                    return false;
                }
                let seat = self
                    .project
                    .as_ref()
                    .expect("can_start_export checked the seat");
                self.transcript
                    .report(format!("exporting {}", output.display()));
                self.export = Some(ExportRun::start(ExportRequest {
                    document: seat.snapshot(),
                    // project root = document path の親（CLI export と同じ規約）。
                    project_root: seat.path().parent().map(Path::to_path_buf),
                    output_path: output.clone(),
                    // v0 は既定値のみ: composition 全長・通常品質。
                    frame_count: None,
                    qp0: false,
                }));
                true
            }
            UiIntent::CancelExport => match self.export.as_ref() {
                Some(run) => {
                    run.cancel();
                    true
                }
                None => false,
            },
            UiIntent::AdmitPaths { paths } => {
                let placed = self
                    .project
                    .as_ref()
                    .map(|seat| seat.editor().revision())
                    .unwrap_or(0);
                let report = admit_dropped_paths(self.project.as_mut(), paths);
                self.transcript.report(report);
                // 何も入らなかった(理由つき skip)も「意図は通った」ではない。
                self.project
                    .as_ref()
                    .is_some_and(|seat| seat.editor().revision() > placed)
            }
            // ---- wave E 第1弾: Inspector の編集。実体は既存エディタ操作1本ずつ ----
            // `SelectLayer` は Stage/Timeline のクリックと Inspector が誰を
            // 映すかの両方の原因(2026-08-18 統合第2弾で union)。session 状態
            // なので `edit`(accepted-no-op = true)を使う — `with_editor` の
            // revision 比較だと選択だけの操作は常に false になってしまう。
            UiIntent::SelectLayer { layer, additive } => self.edit(|editor| {
                let layer_id = motolii_doc::LayerId::from_raw(*layer);
                if *additive {
                    editor.add_to_selection(layer_id);
                } else {
                    editor.select_layer(layer_id);
                }
            }),
            UiIntent::ToggleItemFlag { layer, flag } => self.edit(|editor| {
                editor.toggle_item_flag(motolii_doc::LayerId::from_raw(*layer), flag.to_item_flag());
            }),
            UiIntent::BeginParamEdit { layer, param } => self.edit(|editor| {
                editor.begin_param_edit(motolii_doc::LayerId::from_raw(*layer), param.to_param_ref());
            }),
            UiIntent::SetParamComponent {
                layer,
                param,
                component,
                value,
            } => self.edit(|editor| {
                editor.set_param_component(
                    motolii_doc::LayerId::from_raw(*layer),
                    param.to_param_ref(),
                    *component,
                    *value,
                );
            }),
            UiIntent::EndParamEdit => self.edit(|editor| editor.end_param_edit()),
            UiIntent::KeyParamAtPlayhead {
                layer,
                param,
                components,
            } => self.edit(|editor| {
                editor.key_param_at_playhead(
                    motolii_doc::LayerId::from_raw(*layer),
                    param.to_param_ref(),
                    components,
                );
            }),
            UiIntent::SetEffectEnabled {
                definition_id,
                enabled,
            } => self.edit(|editor| {
                editor.set_effect_enabled(
                    motolii_doc::EffectDefinitionId::from_raw(*definition_id),
                    *enabled,
                );
            }),

            // ---- Timeline の編集。実体は全部 TimelineEditor の操作 API で、
            //      D2 command(1 gesture = 1 Undo)へ落ちる。断られた理由は
            //      editor の控え(`take_rejections`)から transcript へ写す。 ----
            UiIntent::ClearSelection => self.with_editor(|editor| {
                editor.clear_selection();
                true
            }),
            UiIntent::MoveClips {
                grabbed,
                grab_at_us,
                drop_at_us,
            } => self.with_editor(|editor| {
                let before = editor.revision();
                editor.begin_selected_clips_move(*grabbed, us_to_seconds(*grab_at_us));
                editor.drag_to(us_to_seconds(*drop_at_us));
                editor.release();
                editor.revision() > before
            }),
            UiIntent::TrimClip { layer, edge, at_us } => self.with_editor(|editor| {
                let before = editor.revision();
                editor.begin_trim(*layer, *edge);
                editor.drag_to(us_to_seconds(*at_us));
                editor.release();
                editor.revision() > before
            }),
            UiIntent::DeleteSelection => self.with_editor(|editor| {
                let before = editor.revision();
                editor.delete_selection_gesture();
                editor.revision() > before
            }),
            UiIntent::Undo => self.with_editor(|editor| {
                let before = editor.revision();
                editor.undo_gesture();
                editor.revision() != before
            }),
            UiIntent::Redo => self.with_editor(|editor| {
                let before = editor.revision();
                editor.redo_gesture();
                editor.revision() != before
            }),
            UiIntent::SetPlayhead { at_us } => self.with_editor(|editor| {
                editor.scrub_to_seconds(us_to_seconds(*at_us));
                true
            }),
            UiIntent::StepPlayhead { frames } => self.with_editor(|editor| {
                editor.step_playhead_frames(*frames);
                true
            }),

            // ---- Timeline のキー編集(M-6)。実体は timeline_editor に足した
            //      1本ずつの操作 API — MoveClips/TrimClip と同じ with_editor 経路。
            UiIntent::RemoveParamKey { layer, param, at_us } => self.with_editor(|editor| {
                let before = editor.revision();
                editor.remove_param_key_at(
                    motolii_doc::LayerId::from_raw(*layer),
                    param.to_param_ref(),
                    us_to_seconds(*at_us),
                );
                editor.revision() > before
            }),
            UiIntent::MoveParamKey {
                layer,
                param,
                from_us,
                to_us,
            } => self.with_editor(|editor| {
                let before = editor.revision();
                editor.move_param_key(
                    motolii_doc::LayerId::from_raw(*layer),
                    param.to_param_ref(),
                    us_to_seconds(*from_us),
                    us_to_seconds(*to_us),
                );
                editor.revision() > before
            }),
            UiIntent::SetParamKeyInterp {
                layer,
                param,
                at_us,
                interp,
            } => self.with_editor(|editor| {
                let before = editor.revision();
                editor.set_param_key_interp(
                    motolii_doc::LayerId::from_raw(*layer),
                    param.to_param_ref(),
                    us_to_seconds(*at_us),
                    interp.to_interp(),
                );
                editor.revision() > before
            }),
        }
    }

    /// 編集系 intent の共通の門(Inspector 側・`SelectLayer` 込み)。座席が無ければ
    /// **黙らずに**案内し、実行後はエディタが断った理由(`take_rejections`)を
    /// 全部帯へ写す。
    ///
    /// 返り値は「断られなかったか」。**accepted-no-op(同値への編集)は true** —
    /// 意図は通っていて、変化ゼロが正しい結果だからである(Q3 の扱いは表示側)。
    /// 状態は一切ここで持たない: 面は次の snapshot からだけ導出する(optimistic 禁止)。
    fn edit(&mut self, act: impl FnOnce(&mut TimelineEditor)) -> bool {
        let Some(seat) = self.project.as_mut() else {
            self.transcript
                .report("open a project first — Cmd+N to create one, Cmd+O to open one");
            return false;
        };
        act(seat.editor_mut());
        let rejections = seat.editor_mut().take_rejections();
        let ok = rejections.is_empty();
        for reason in rejections {
            self.transcript.report(reason);
        }
        ok
    }

    /// Timeline 系 intent の共通口: 座席が無ければ案内して `false`、在れば
    /// editor を1回貸して、**断られた理由を transcript へ写してから**答えを返す。
    ///
    /// egui shell の pane が `relay_editor_rejections` でやっているのと同じ写しで、
    /// intent 経由の編集でも「押しても無反応」を作らない(外部診断 F-07)。
    fn with_editor(
        &mut self,
        operate: impl FnOnce(&mut crate::timeline_editor::TimelineEditor) -> bool,
    ) -> bool {
        let Some(seat) = self.project.as_mut() else {
            self.transcript
                .report("open a project first — Cmd+N to create one, Cmd+O to open one");
            return false;
        };
        let ok = operate(seat.editor_mut());
        for rejection in seat.editor_mut().take_rejections() {
            self.transcript.report(rejection);
        }
        ok
    }

    /// 座席を差し替えて、成否を帯に言う。判断そのものは [`reseat_project`]。
    fn reseat(&mut self, path: &Path) -> bool {
        match reseat_project(&mut self.project, path) {
            Ok(()) => {
                self.transcript.report(format!("opened {}", path.display()));
                self.remember_last_project(path);
                true
            }
            Err(error) => {
                self.transcript.report(error);
                false
            }
        }
    }

    /// 次の引数なし起動が続きを開けるよう、いま座った project を覚える。
    ///
    /// New も Open も同じ `reseat` を通るので、覚える口はこの1つで足りる。
    /// **覚えられなくても編集は続く** — ただし黙らない(帯へ理由が出る)。
    fn remember_last_project(&self, path: &Path) {
        let Some(store) = self.last_project.as_deref() else {
            return;
        };
        if let Err(error) = crate::last_project::remember_last_project(store, path) {
            self.transcript.report(format!(
                "最後に開いた project を覚えられない({}): {error}",
                store.display()
            ));
        }
    }

    /// 実行中の書き出しの返事を受ける(**intent ではない** — 世界の側からの返事)。
    /// 完了/キャンセル/失敗は帯の一言になり、run は捨てる。
    ///
    /// **公開している**理由: iced shell も同じ書き出しを共用する(M-1)。刻み方だけが
    /// host で違い(egui は `request_repaint_after`、iced は `window::frames()` の購読)、
    /// 受け取る口はこの1つである。
    pub fn poll_export(&mut self) {
        let Some(run) = self.export.as_mut() else {
            return;
        };
        let Some(finish) = run.try_finish() else {
            return;
        };
        let report = match finish {
            ExportFinish::Done(report) => format!(
                "Exported {} ({}f)",
                run.output_path().display(),
                report.frames_written
            ),
            ExportFinish::Cancelled => {
                format!(
                    "export cancelled — {} was removed",
                    run.output_path().display()
                )
            }
            ExportFinish::Failed(reason) => format!("export failed: {reason}"),
        };
        self.transcript.report(report);
        self.export = None;
    }

    /// 座席(読み)。**書きは出さない** — 状態を進める道は [`dispatch`](Self::dispatch)
    /// だけ、という構造は公開しても変わらない(`ProjectSeat` は元から `pub`)。
    pub fn project(&self) -> Option<&ProjectSeat> {
        self.project.as_ref()
    }

    /// 座席(書き)。**egui pane にだけ残る穴**: あちらの Timeline / Inspector の
    /// 編集はまだ intent を通らず、`motolii-doc` 側の D2 journal だけが受けている。
    /// iced shell はこの穴を持たない(`pub(crate)` なので届かない) — あちらは
    /// Timeline 編集も `UiIntent` 経由である(2026-08-18 M-3)。
    /// 新しい shell 層の副作用をここから足さないこと(フェンスは呼び出しの
    /// 有無ではなく「禁止 API 名」で見ているので、増やせば黙って通ってしまう)。
    pub(crate) fn project_mut(&mut self) -> Option<&mut ProjectSeat> {
        self.project.as_mut()
    }

    /// 実行中の書き出し(読み)。帯がスピナーと経過秒に使う。
    pub fn export(&self) -> Option<&ExportRun> {
        self.export.as_ref()
    }

    /// live project が座っているか。
    pub fn is_seated(&self) -> bool {
        self.project.is_some()
    }

    /// Export ボタンの enabled と `BeginExport` の門。同じ関数を両方が見る。
    pub fn can_start_export(&self) -> bool {
        can_start_export(self.project.is_some(), self.export.is_some())
    }

    /// 原因のログ。
    pub fn journal(&self) -> &IntentJournal {
        &self.journal
    }

    /// 結果のログ。replay の審判が読む。
    pub fn transcript(&self) -> &ShellTranscript {
        &self.transcript
    }

    /// 座席の Document に立っている track item の総数(replay の審判用)。
    pub fn track_item_count(&self) -> usize {
        self.project
            .as_ref()
            .map(|seat| {
                seat.snapshot()
                    .tracks
                    .iter()
                    .map(|track| track.items.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    /// 座席の writer 世代(replay の審判用)。座席が無ければ 0。
    pub fn revision(&self) -> u64 {
        self.project
            .as_ref()
            .map(|seat| seat.editor().revision())
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// 製品状態そのもの(座席と、それを動かす関数)
// ---------------------------------------------------------------------------

/// live project の座席。開いた project の `ProjectSession`（OS 排他 lock）と、
/// Timeline エディタをひとまとめに持つ。**Document の唯一の writer はエディタの
/// 中に1つだけ座る**（single writer）。
///
/// pane 側へ出て行くのは `snapshot()` の immutable snapshot だけ。
/// 第二の Document 保持経路（cache / 読み直し）はここから先に作らない。
pub struct ProjectSeat {
    /// 開いている project のパス。**同じ project を開き直す時に lock を先に返す**
    /// 判断がこれを見る（`reseat_project`）。
    path: PathBuf,
    /// project identity の排他 lock。app が生きているあいだ握り続け
    /// （落とすと他プロセスが同じ project を開けてしまう）、保存（`save`）も
    /// この lock の中の `save_document` を通る（CLI `document_edit.rs` と同じ経路）。
    session: motolii_doc::ProjectSession,
    /// Timeline エディタ。編集は全部（キー入力も操作 API も）この中の writer を通る。
    editor: TimelineEditor,
    /// 最後に project ファイルへ書いた（開いた直後はファイルにある）内容の snapshot。
    /// dirty 判定はこれとの**内容比較**（`is_dirty`）。undo 台帳の深さや revision の
    /// 一致では判定しない — undo で保存内容へ戻れば clean、深さが偶然揃っても
    /// 内容が違えば dirty、が正しく出る。
    saved: Arc<motolii_doc::Document>,
    /// `is_dirty` の cache（判定した時の revision とその答え）。revision が
    /// 進んでいなければ Document 比較をやり直さない（status 帯が毎フレーム見るため）。
    dirty_cache: std::cell::Cell<Option<(u64, bool)>>,
}

impl ProjectSeat {
    /// 実プロジェクトを開いて座席を作る。経路は既存の `open_project_resolved`
    /// （= `ProjectSession::open` ＋ plugin 解決。`document_export.rs:18` /
    /// `timeline_widget_lab.rs:120` と同じ慣行）。
    ///
    /// 開けなければ `Err` — **fixture へ黙って fallback しない**。呼び手（`main.rs`）が
    /// 起動失敗として扱う。
    pub fn open(path: &Path) -> Result<Self, String> {
        let catalog = Arc::new(
            motolii_plugin::reference::reference_catalog()
                .map_err(|error| format!("plugin catalog を作れない: {error}"))?,
        );
        let opened = motolii_doc::open_project_resolved(
            path,
            &motolii_doc::ResourceLimits::production(),
            &catalog,
        )
        .map_err(|error| format!("project {} を開けない: {error}", path.display()))?;
        let writer = motolii_doc::DocumentWriter::new(opened.recovered.document, catalog).map_err(
            |error| {
                format!(
                    "project {} の DocumentWriter を作れない: {error}",
                    path.display()
                )
            },
        )?;
        let mut editor =
            TimelineEditor::new(writer).with_project_root(path.parent().map(Path::to_path_buf));
        seat_inspection_selection(&mut editor);
        // 開いた直後の snapshot がそのまま「ファイルにある内容」= clean の基準。
        let saved = Arc::clone(editor.document());
        Ok(Self {
            path: path.to_path_buf(),
            session: opened.session,
            // project root = document path の親(CLI export と同じ規約)。
            // soundtrack 再生の asset path 解決に使う。
            editor,
            saved,
            dirty_cache: std::cell::Cell::new(None),
        })
    }

    /// writer snapshot を project ファイルへ書き戻す（`UiIntent::SaveProject` の後ろ）。
    ///
    /// 経路は既存の `ProjectSession::save_document`（CLI `document_edit.rs` の
    /// `with_writer` が編集後に通すのと同じ）で、**新しい保存経路を作らない**。
    /// journal への常時追記はここではしない（明示保存のみ）。
    pub fn save(&mut self) -> Result<(), String> {
        let snapshot = Arc::clone(self.editor.document());
        self.session
            .save_document(&snapshot, &motolii_doc::SaveOptions::default())
            .map_err(|error| format!("save {} failed: {error}", self.path.display()))?;
        self.saved = snapshot;
        self.dirty_cache.set(Some((self.editor.revision(), false)));
        Ok(())
    }

    /// 未保存の編集があるか。**保存済み内容との比較**で答える。
    ///
    /// revision や undo 台帳の深さの一致は根拠にしない — undo で保存時の内容へ
    /// 戻れば clean に戻り、保存点より下へ undo してから別の編集で深さだけ揃っても
    /// dirty のまま、が正しく出る。比較は revision が進んだ時だけやり直す。
    pub fn is_dirty(&self) -> bool {
        let revision = self.editor.revision();
        if let Some((seen, dirty)) = self.dirty_cache.get() {
            if seen == revision {
                return dirty;
            }
        }
        let current = self.editor.document();
        let dirty = !Arc::ptr_eq(current, &self.saved) && **current != *self.saved;
        self.dirty_cache.set(Some((revision, dirty)));
        dirty
    }

    /// 開いている project のパス。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// pane へ流す immutable snapshot（エディタが writer から取った cached をそのまま配る）。
    pub fn snapshot(&self) -> Arc<motolii_doc::Document> {
        Arc::clone(self.editor.document())
    }

    /// Timeline エディタ（読み）。revision / snapshot の照合に使う。
    pub fn editor(&self) -> &TimelineEditor {
        &self.editor
    }

    /// Timeline エディタ（書き）。pane の描画と統合テストの操作 API がここを通る。
    /// writer 自体は外へ出さない — 編集は必ずエディタの操作として通る。
    pub fn editor_mut(&mut self) -> &mut TimelineEditor {
        &mut self.editor
    }
}

/// 検分用: `MOTOLII_SELECT_LAYER=<id>` があれば、開いた直後にその layer を選んでおく。
///
/// **製品の挙動ではない** — `--screenshot` と同じ「窓を触らずに絵を確かめる」ための口で、
/// `pane.rs` の `MOTOLII_SHELL_TRACE` と同じ扱いである。通る経路は素のクリックと同じ
/// `select_layer` なので、選択の意味も Undo 台帳も変えない。環境変数が無い・数として
/// 読めないときは**何もしない**（既定は従来どおり「選択なし」）。
fn seat_inspection_selection(editor: &mut TimelineEditor) {
    let Some(raw) = std::env::var_os("MOTOLII_SELECT_LAYER") else {
        return;
    };
    let Some(id) = raw.to_str().and_then(|text| text.trim().parse::<u64>().ok()) else {
        return;
    };
    editor.select_layer(motolii_doc::LayerId::from_raw(id));
}

/// 新規 project を1つ作る（空コンポ + V1 トラック1本）。
///
/// 意味は CLI の `new_document`（`crates/motolii-cli/src/document_debug.rs:12`）
/// そのもので、**作ったものはそのまま `ProjectSeat::open` で開ける**
/// （= `--project` 起動と同じ状態になる）。既にあるファイルは踏まない。
///
/// dialog はここに無い。呼び手（`UiIntent::NewProject`）が場所を決めてから通る。
pub fn create_project_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("project already exists: {}", path.display()));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut document = motolii_doc::Document::new_current();
    // 2026-08-17決定: 新規projectの出力解像度既定は1920x1080(CLI `new_document`と同値。
    // 以後の変更は`Command::SetCompositionResolution`だけが書く)。
    document
        .composition
        .set_resolution(Some((1920, 1080)))
        .map_err(|error| error.to_string())?;
    // 受け皿のトラックが無いと clip を1本も置けない（`prepare_place_asset_clip`）。
    let track = document
        .track_ids
        .allocate("V1")
        .map_err(|error| error.to_string())?;
    document.tracks.push(motolii_doc::Track {
        id: track,
        items: vec![],
    });
    let mut session =
        motolii_doc::ProjectSession::acquire(path, &motolii_doc::ResourceLimits::production())
            .map_err(|error| error.to_string())?;
    session
        .save_document(&document, &motolii_doc::SaveOptions::default())
        .map_err(|error| error.to_string())
    // session はここで落ちて lock を返す。開くのは呼び手（`reseat_project`）。
}

/// 座席を差し替える。**旧 session の lock をいつ返すかがここの全部である。**
///
/// - 別の project … 新しい席を**先に開いてから**旧席を落とす。開けなくても
///   いま編集している席を失わない
/// - 同じ project … 先に旧席を落として lock を返してから開き直す
///   （そうしないと自分の lock に自分がぶつかる）
///
/// 開けなければ `Err(理由)` で、app は落とさない。dialog はここに無い。
pub fn reseat_project(current: &mut Option<ProjectSeat>, path: &Path) -> Result<(), String> {
    if current
        .as_ref()
        .is_some_and(|seat| same_project(seat.path(), path))
    {
        // 旧 session を先に落として lock を返す。
        *current = None;
    }
    let seat = ProjectSeat::open(path)?;
    // ここで初めて旧席（別 project の場合）が落ちる。
    *current = Some(seat);
    Ok(())
}

/// 同じ project を指しているか。symlink や `./` の違いで別物に見えないよう、
/// 取れるなら canonical path で比べる。
fn same_project(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// OS から窓へ落ちてきた path 群を受ける。**返すのは一言の status。**
///
/// project が開いていないときは Document が無いので取り込めない。黙って
/// 捨てず、**先に project を作る／開く**と案内する（v0。ドロップから
/// 新規 project を作る意味はまだ決めていない）。
///
/// 落ちたものが clip になったか曲になったかは status がそのまま名指しで言う
/// （まだ曲が無い project へ落ちた音声は曲になる — 分岐は `import_seat`）。
pub fn admit_dropped_paths(seat: Option<&mut ProjectSeat>, paths: &[PathBuf]) -> String {
    let Some(seat) = seat else {
        return "open a project first — Cmd+N to create one, Cmd+O to open one".to_owned();
    };
    if paths.is_empty() {
        return "nothing dropped".to_owned();
    }
    seat.editor_mut().import_dropped_media(paths).summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// journal の JSONL は `--status-log` と同型で、答え(path)まで機械可読に運ぶ。
    /// これが崩れると `--intent-log` を読んだ側が replay を組めない。
    #[test]
    fn an_intent_event_round_trips_through_the_jsonl_shape() {
        let event = IntentEvent {
            seq: 3,
            intent: UiIntent::NewProject {
                path: PathBuf::from("/tmp/fresh.json"),
            },
        };
        let line = serde_json::to_string(&event).expect("intent は機械可読である");
        assert_eq!(
            line, r#"{"seq":3,"intent":{"kind":"new_project","path":"/tmp/fresh.json"}}"#,
            "--intent-log の1行の形"
        );
        let back: IntentEvent = serde_json::from_str(&line).expect("読み戻せる");
        assert_eq!(back, event, "答えごと往復する(dialog を訊き直さない)");

        let cancel = serde_json::to_string(&UiIntent::CancelExport).expect("unit variant");
        assert_eq!(cancel, r#"{"kind":"cancel_export"}"#);

        // wave E 第1弾(Inspector)の wire の形。layer は u64 で運ぶ
        // (2026-08-10「wire carries layer identity only」)。
        let toggled = serde_json::to_string(&UiIntent::ToggleItemFlag {
            layer: 7,
            flag: UiItemFlag::Mute,
        })
        .expect("編集 intent も機械可読である");
        assert_eq!(
            toggled,
            r#"{"kind":"toggle_item_flag","layer":7,"flag":"mute"}"#
        );
        let keyed = serde_json::to_string(&UiIntent::KeyParamAtPlayhead {
            layer: 7,
            param: UiEditParam::Position,
            components: vec![4.0, 2.0],
        })
        .expect("成分値ごと往復する");
        assert_eq!(
            keyed,
            r#"{"kind":"key_param_at_playhead","layer":7,"param":"position","components":[4.0,2.0]}"#
        );
        let back: UiIntent = serde_json::from_str(&keyed).expect("読み戻せる");
        assert_eq!(
            back,
            UiIntent::KeyParamAtPlayhead {
                layer: 7,
                param: UiEditParam::Position,
                components: vec![4.0, 2.0],
            }
        );
    }

    /// Timeline 系 intent の JSONL の形(M-3)。時刻は µs の整数 — f32 を入れて
    /// `Eq` と決定性を壊さない、が形として読める。
    #[test]
    fn a_timeline_intent_round_trips_through_the_jsonl_shape() {
        let mv = UiIntent::MoveClips {
            grabbed: motolii_doc::LayerId::from_raw(7),
            grab_at_us: 2_000_000,
            drop_at_us: 4_000_000,
        };
        let line = serde_json::to_string(&mv).expect("機械可読");
        assert_eq!(
            line,
            r#"{"kind":"move_clips","grabbed":7,"grab_at_us":2000000,"drop_at_us":4000000}"#
        );
        let back: UiIntent = serde_json::from_str(&line).expect("読み戻せる");
        assert_eq!(back, mv);

        let trim = UiIntent::TrimClip {
            layer: motolii_doc::LayerId::from_raw(3),
            edge: crate::timeline_editor::TrimEdge::Out,
            at_us: 10_000_000,
        };
        assert_eq!(
            serde_json::to_string(&trim).expect("機械可読"),
            r#"{"kind":"trim_clip","layer":3,"edge":"out","at_us":10000000}"#
        );
        assert_eq!(
            serde_json::to_string(&UiIntent::Undo).expect("unit variant"),
            r#"{"kind":"undo"}"#
        );
    }

    /// 座席なしの Timeline intent は案内だけ言って何も起こさない(黙らない)。
    #[test]
    fn a_timeline_intent_without_a_seat_says_what_to_do_first() {
        let mut gateway = ShellGateway::new(ShellTranscript::default());
        assert!(!gateway.dispatch(UiIntent::DeleteSelection));
        assert!(
            gateway
                .transcript()
                .latest()
                .is_some_and(|text| text.contains("Cmd+N")),
            "作る/開く口を名指しで案内する: {:?}",
            gateway.transcript().latest()
        );
    }

    /// **落ちた行動も記録される。** intent は行動の前に journal へ入るので、
    /// 「やろうとして失敗した」が消えない。
    #[test]
    fn a_failing_intent_is_recorded_before_it_is_attempted() {
        let dir = motolii_testkit::tmp_dir("intent_failed_new");
        let path = dir.join("fresh.json");
        let mut gateway = ShellGateway::new(ShellTranscript::default());

        assert!(
            gateway.dispatch(UiIntent::NewProject { path: path.clone() }),
            "1回目の New は通る"
        );
        assert!(
            !gateway.dispatch(UiIntent::NewProject { path: path.clone() }),
            "既にある project は踏まない(失敗する)"
        );

        let entries = gateway.journal().entries();
        assert_eq!(entries.len(), 2, "失敗した行動も台帳から消えない");
        assert_eq!(entries[0].seq, 1);
        assert_eq!(entries[1].seq, 2);
        assert_eq!(entries[1].intent, UiIntent::NewProject { path });
        assert!(
            gateway
                .transcript()
                .latest()
                .is_some_and(|text| text.contains("already exists")),
            "失敗の理由は帯に言われる: {:?}",
            gateway.transcript().latest()
        );
    }

    /// 座席なしの Save は Document を作らず、案内だけ言う(黙らない)。
    #[test]
    fn saving_without_a_seat_says_what_to_do_first() {
        let mut gateway = ShellGateway::new(ShellTranscript::default());
        assert!(!gateway.dispatch(UiIntent::SaveProject));
        assert!(
            gateway
                .transcript()
                .latest()
                .is_some_and(|text| text.contains("Cmd+N") && text.contains("Cmd+O")),
            "作る/開く口を名指しで案内する: {:?}",
            gateway.transcript().latest()
        );
    }

    // -----------------------------------------------------------------------
    // 引数なし起動で続きが開く(外部診断 F-01)
    // -----------------------------------------------------------------------

    /// **座り直した project は次の起動のために覚える。**
    ///
    /// New も Open も同じ `reseat` を通るので、覚える口も1つで足りる。
    /// 覚え先は外から渡された user 設定層の path だけで、`remembering` を
    /// 渡していないゲートウェイ(replay・大半のテスト)は何も書かない。
    #[test]
    fn a_seated_project_is_remembered_for_the_next_launch() {
        let dir = motolii_testkit::tmp_dir("intent_remember_last");
        let store = dir.join("last-project.json");
        let first = dir.join("first.json");
        let second = dir.join("second.json");
        create_project_file(&second).expect("create");

        let mut gateway = ShellGateway::new(ShellTranscript::default())
            .remembering(Some(store.clone()));
        assert!(gateway.dispatch(UiIntent::NewProject {
            path: first.clone()
        }));
        assert_eq!(
            crate::last_project::load_last_project(&store).expect("読める"),
            Some(first),
            "New で作った project が覚えられていない"
        );

        assert!(gateway.dispatch(UiIntent::OpenProject {
            path: second.clone()
        }));
        assert_eq!(
            crate::last_project::load_last_project(&store).expect("読める"),
            Some(second),
            "Open した project が覚えられていない"
        );
    }

    /// 覚え先を渡していないゲートウェイは**何も書かない**。replay と
    /// 単体テストが利用者の設定を踏まないための境目。
    #[test]
    fn a_gateway_without_a_store_remembers_nothing() {
        let dir = motolii_testkit::tmp_dir("intent_remember_none");
        let store = dir.join("last-project.json");
        let project = dir.join("fresh.json");

        let mut gateway = ShellGateway::new(ShellTranscript::default());
        assert!(gateway.dispatch(UiIntent::NewProject { path: project }));
        assert!(!store.exists(), "覚え先を渡していないのに書いている");
    }

    /// **引数なしの起動は、最後に開いていた project へそのまま座る。**
    /// 台本 P5「保存→再起動→続きがそのまま開く」の機械版。
    ///
    /// 座り方は `--project` 起動と同じ(`ShellGateway::seated`)なので、journal の
    /// 第1行も同じ `OpenProject` になる — `--intent-log` はそれ単体で replay できる。
    #[test]
    fn an_argument_less_launch_sits_back_down_in_the_last_project() {
        let dir = motolii_testkit::tmp_dir("intent_resume_seat");
        let store = dir.join("last-project.json");
        let project = dir.join("continue.json");
        create_project_file(&project).expect("create");
        crate::last_project::remember_last_project(&store, &project).expect("覚える");

        let transcript = ShellTranscript::default();
        let gateway = ShellGateway::resumed(transcript.clone(), resume_last_project(Some(&store)));

        assert_eq!(
            gateway.project().map(|seat| seat.path().to_path_buf()),
            Some(project.clone()),
            "続きが開いていない"
        );
        assert_eq!(
            gateway.journal().intents(),
            vec![UiIntent::OpenProject { path: project }],
            "自動 open も seq 1 の OpenProject として残る(--project 起動と同じ扱い)"
        );
    }

    /// **覚えている project が消えていたら、理由を言ってからスタート画面。**
    /// 黙って fixture にも空の窓にも落ちない(既存の起動失敗の原則と同じ)。
    #[test]
    fn a_last_project_that_vanished_says_why_and_leaves_the_start_screen() {
        let dir = motolii_testkit::tmp_dir("intent_resume_gone");
        let store = dir.join("last-project.json");
        let project = dir.join("deleted.json");
        create_project_file(&project).expect("create");
        crate::last_project::remember_last_project(&store, &project).expect("覚える");
        std::fs::remove_file(&project).expect("消す");

        let transcript = ShellTranscript::default();
        let gateway = ShellGateway::resumed(transcript.clone(), resume_last_project(Some(&store)));

        assert!(!gateway.is_seated(), "開けなかったのに座っている");
        let said = transcript.latest().unwrap_or_default();
        assert!(
            said.contains("deleted.json"),
            "開けなかった project を名指しで言う。実際: {said:?}"
        );
        assert!(
            said.contains("Cmd+N") || said.contains("New"),
            "次にどうすればよいかまで言う。実際: {said:?}"
        );
    }

    /// 初回起動(まだ何も覚えていない)は、**何も言わずに**スタート画面。
    /// 言うことが無いのに帯を出さない。
    #[test]
    fn a_first_ever_launch_is_the_start_screen_without_a_word() {
        let dir = motolii_testkit::tmp_dir("intent_resume_first");
        let store = dir.join("last-project.json");

        let transcript = ShellTranscript::default();
        let gateway = ShellGateway::resumed(transcript.clone(), resume_last_project(Some(&store)));

        assert!(!gateway.is_seated());
        assert!(gateway.journal().intents().is_empty(), "起きていない行動は記録しない");
        assert_eq!(transcript.latest(), None, "初回起動に帯は要らない");
    }

    /// `--project` 起動の座席も journal の第1行を持つ(replay 可能な列になる)。
    #[test]
    fn a_launch_seat_still_names_how_it_got_there() {
        let dir = motolii_testkit::tmp_dir("intent_launch_seat");
        let path = dir.join("launch.json");
        create_project_file(&path).expect("create");
        let seat = ProjectSeat::open(&path).expect("open");

        let gateway = ShellGateway::seated(ShellTranscript::default(), seat);
        assert_eq!(
            gateway.journal().intents(),
            vec![UiIntent::OpenProject { path }],
            "--project の座席も『どうやって着いたか』を名乗る"
        );
    }
}
