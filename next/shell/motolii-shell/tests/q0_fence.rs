//! Q0 横断柵(タスク#15、`../GOALS.md` M12・順序7)。
//!
//! Q0(`../../docs/ui-quality-bar.md`)は「見えて押せそう/掴めそうな UI 要素は
//! 必ず実機能へ接続されていること」。このファイルは Shell の `view()` を機械的に
//! 歩いて、その主張を widget 単位で検査する。**パネルが増えても、この2ファイル
//! を触らずに自動で効く**(`view()` が返す木を歩くだけで、pane 別の知識を
//! 一切埋め込んでいない)。
//!
//! ## 仕組み
//!
//! iced の `widget::Operation` は widget 自身が `operate()` で明示的に登録した
//! 物しか見えない(`iced_core::widget::Operation` の `container`/`focusable`/
//! `scrollable`/`text_input`/`text`/`custom` の6種、upstream 0.14.0 実測)。
//! `iced_test::Selector` はその登録済み候補(`Candidate`)を辿るだけなので、
//! **`view()` にある widget が全部見えるわけではない**(下の「限界」参照)。
//!
//! `Simulator::find` は1件だけ返す(`Selector::find()` → `One` 戦略)。
//! `Simulator` に `find_all` は無い(iced_test 0.14.0 の公開APIを実際に読んで
//! 確認 — `test/src/simulator.rs` に `find` はあっても `find_all` は無い)。
//! 全件を得るため、[`collect_targets`] は「まだ拾っていない候補」を選ぶ
//! stateful な closure selector で `find` を **`Err` が返るまで繰り返す**。
//! `Selector::select` は DFS 順で呼ばれ、1回の `find` は最初の未回収候補で
//! 止まる(`One::is_done`)ので、これは実質的な `find_all` の代替になる。
//!
//! 各候補について: 中心点へ press→release を送り、**どちらかの event が
//! `Captured`(=何らかの widget がその場で反応した)なのに Message が1件も
//! 出ない**なら Q0 違反として記録する。「Ignored のまま何も出ない」は違反に
//! しない — それは単なる非対話要素(ラベル・余白・純粋な layout container)で
//! あって、Q0 が名指す「押せそうな物」ではない。
//!
//! この判定ルールが実際に違反を捕まえることは
//! [`the_fence_catches_a_captured_but_silent_widget`] が示す
//! (わざと「capture するが publish しない」widget を作って柵にかける)。
//!
//! ## 検出できる違反の例
//!
//! - `button(...)` の `on_press` を外し忘れて「見えて押せそうなのに押しても
//!   何も起きない」状態になった場合(disabled 表現も無いなら二重に Q0 違反)
//! - text_input・checkbox(ラベル付き)・radio・pick_list・scrollable など、
//!   `operate()` を実装している標準 widget 全般で、capture はするが Message
//!   経路が壊れている(繋ぎ忘れ・条件分岐の食い違い)場合
//!
//! ## 検出できない(構造的な)限界 — 正直に書く
//!
//! - **`canvas::Program` は `operate()` を一切実装しない**(upstream
//!   `widget/src/canvas.rs` に `fn operate` が無いことを実測)。Timeline pane
//!   は丸ごと canvas 1枚なので、bar・ruler・playhead はこの柵に**一切映らない**。
//!   Timeline の Q0 は `tests/iced_test_spike.rs` の座標直撃テストのような
//!   pane 別の手当てが要る — この柵はそれを代替しない
//! - **`slider` も `operate()` を実装しない**(実測)。transport の scrub
//!   slider も同様に不可視
//! - `checkbox`/`radio`/`toggler` はラベルが無いと `Candidate` を1つも
//!   出さない物がある(`checkbox` はラベル無しだと無登録 — upstream 実測)
//! - 「Captured なのに無反応」を違反の定義にしている。**iced の標準 widget は
//!   capture と publish が同じ条件(`on_press.is_some()` 等)に紐付いている
//!   ことが多く**、「見た目は普通なのに端から何もしない」死に chrome は
//!   iced のスタイル系が `on_press` 無しを自動で disabled 見た目にするため
//!   起きにくい(旧 egui 実装の `Settings`/`Export` 死にtext のような形の
//!   違反は、iced の button ではむしろ作りにくい)。この柵が主に守るのは
//!   「capture 側だけ実装して publish を繋ぎ忘れる」という、**カスタム
//!   widget や条件分岐のある wiring** で起きるバグの型
//! - Captured すら**しない**読みにくい chrome(hover の色だけ変わって
//!   click には何も反応しない要素)は、この柵の判定ルールでは捕まえられない
//!   — iced の event 消費モデル自体の限界であって、実装で埋められない
//! - dedup は `Target` の構造的等価(bounds/内容)。**同じ内容・同じ座標に
//!   重なる別 widget が万一あれば片方だけ数える**(現行の pane 構成では
//!   実測 0 件 — 下の `no_pane_leaves_a_captured_click_silent` が通っている
//!   ことがその実測)
//! - 1回の scan は Shell の**1状態**しか見ない。[`no_pane_leaves_a_captured_click_silent`]
//!   は代表的な3状態(空・layer1枚・layer1枚→Undo)を個別に回すだけで、
//!   全状態空間を尽くしてはいない

use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{layout, mouse, renderer, Layout, Shell as WidgetShell, Widget};
use iced_test::selector::{Candidate, Target};

use motolii_shell::{Message, Shell};

// ---------------------------------------------------------------------------
// 列挙 — `find_all` が無いので `find` を尽きるまで繰り返す
// ---------------------------------------------------------------------------

/// `element` の中で `widget::Operation` に登録されている候補を重複なく全部集める。
/// 6種(Container/Focusable/Scrollable/TextInput/Text/Custom)を区別せず集める
/// — button/row/column/container はどれも自分を `Container` として登録するので
/// (upstream 実測)、種別を絞ると button を取りこぼす。
fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();

    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        // 暴走防止。現行 pane 構成なら数十件で尽きる。増えるのは pane が
        // 増えた時の自然な成長であって、上げてよい — ここは無限ループだけを防ぐ。
        assert!(
            found.len() <= 5_000,
            "widget candidate が5000件を超えた — dedup が効いていない疑い"
        );
    }

    found
}

/// 1点へ press→release を送り、(捕まえた event があったか, 出た Message)を返す。
fn click_at(element: iced::Element<'_, Message>, point: iced::Point) -> (bool, Vec<Message>) {
    let mut ui = iced_test::simulator(element);
    ui.point_at(point);
    let statuses = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);
    let captured = statuses
        .iter()
        .any(|status| matches!(status, iced::event::Status::Captured));
    (captured, ui.into_messages().collect())
}

/// 1状態を Q0 で検査する。違反があれば理由つきで返す(空 = 合格)。
fn scan_state(build: impl Fn() -> Shell, label: &str) -> Vec<String> {
    let seed = build();
    let targets = collect_targets(seed.view());
    drop(seed);

    let mut violations = Vec::new();
    for target in &targets {
        let Some(bounds) = target.visible_bounds() else {
            continue;
        };
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            continue;
        }
        let point = iced::Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );

        let probe = build();
        let (captured, messages) = click_at(probe.view(), point);

        if captured && messages.is_empty() {
            violations.push(format!(
                "{label}: {point:?} 付近({target:?})が click に反応した(event captured)のに \
                 Message が出ない — 触れそうで触れない(Q0)"
            ));
        }
    }
    violations
}

fn fresh() -> Shell {
    Shell::new().0
}

fn with_one_layer() -> Shell {
    let mut shell = fresh();
    let _ = shell.update(Message::AddLayer);
    shell
}

fn with_one_layer_then_undo() -> Shell {
    let mut shell = with_one_layer();
    let _ = shell.update(Message::Undo);
    shell
}

/// **本命**。3状態(空 / layer1枚で Undo が有効 / Undo 済みで Redo が有効)を
/// 横断して、「見た目は反応したのに何も起きない」widget が無いことを見る。
/// 状態を分けているのは、Undo/Redo が文脈disabled(=on_press無し=captureしない)
/// になる時とならない時の両方を通すため — disabled一辺倒だと「on_pressが
/// 繋がっているのに条件を間違えている」バグを見逃す。
#[test]
fn no_pane_leaves_a_captured_click_silent() {
    let mut violations = Vec::new();
    violations.extend(scan_state(fresh, "fresh(空 project)"));
    violations.extend(scan_state(with_one_layer, "layer 1枚(Undoが有効なはず)"));
    violations.extend(scan_state(
        with_one_layer_then_undo,
        "layer追加→Undo(Redoが有効なはず)",
    ));

    assert!(
        violations.is_empty(),
        "Q0違反 {}件:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 柵の自己検定 — 「captured なのに無反応」を実際に検出できることを示す
// ---------------------------------------------------------------------------

/// わざと「capture するが publish しない」widget。`operate()` で自分を
/// `Container` として登録する(button と同じ手口)ので [`collect_targets`] から
/// 見える。押されると `shell.capture_event()` だけ呼び、Message は絶対に出さない
/// — Q0 違反(触れそうで触れない)の最小標本。
struct SilentTrap {
    size: iced::Size,
}

impl Widget<Message, iced::Theme, iced::Renderer> for SilentTrap {
    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size::new(
            iced::Length::Fixed(self.size.width),
            iced::Length::Fixed(self.size.height),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            iced::Length::Fixed(self.size.width),
            iced::Length::Fixed(self.size.height),
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut WidgetShell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
            event
        {
            if cursor.is_over(layout.bounds()) {
                // わざと publish しない — これが柵の検出対象。
                shell.capture_event();
            }
        }
    }
}

impl<'a> From<SilentTrap> for iced::Element<'a, Message> {
    fn from(trap: SilentTrap) -> Self {
        iced::Element::new(trap)
    }
}

/// [`collect_targets`] が `SilentTrap` を候補として見つけ、かつ
/// `scan_state` と同じ判定ルールがそれを違反として拾うことを直接示す
/// — `no_pane_leaves_a_captured_click_silent` が「たまたま何も見つからない
/// だけの空振りテスト」でないことの証拠。
#[test]
fn the_fence_catches_a_captured_but_silent_widget() {
    let build_element = || -> iced::Element<'_, Message> {
        SilentTrap {
            size: iced::Size::new(40.0, 40.0),
        }
        .into()
    };

    let targets = collect_targets(build_element());
    assert_eq!(
        targets.len(),
        1,
        "SilentTrap 自体が候補として見つかっていない(この検定の前提が崩れている): {targets:?}"
    );
    let bounds = targets[0]
        .visible_bounds()
        .expect("SilentTrap の bounds が見える");
    let point = iced::Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );

    let (captured, messages) = click_at(build_element(), point);
    assert!(
        captured,
        "SilentTrap 自体が capture していない(前提が崩れている)"
    );
    assert!(
        messages.is_empty(),
        "SilentTrap のはずが Message を出している(前提が崩れている): {messages:?}"
    );

    // ここが本題 — scan_state と同じ判定ルールを手で適用する。
    let is_violation = captured && messages.is_empty();
    assert!(
        is_violation,
        "SilentTrap が Q0違反(captured かつ無反応)として判定されない — 柵の判定ルールが機能していない"
    );
}
