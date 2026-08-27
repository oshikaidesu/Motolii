//! Inspector パネル。**構造の正本は `next/reference/mocks/inspector-semantics.html`**
//! (第4号、propertyRow 25px = 比率の分母、inspector-ratio-ledger 実測基準)。
//! 第一波(M01)の実線分だけを描く: selection summary / 列見出し / TRANSFORM /
//! APPEARANCE / FX STACK の行 / footer ヒント。mode tabs・extension tabs・notes は
//! Q0 スコープ外(I-ratio 台帳)。左 3px = 値型の色。◆=keyed。
//! 皮は Ableton の形文法(裁定267): 平坦・角丸ゼロ・1px 線・琥珀 on。
//!
//! **フッターが約束した3つは実際に効く**(欠陥 D1 / 2026-08-27 台帳):
//! 値セルは `InkLabel` ではなく `ScrubValue` — 横ドラッグで値が動き、
//! 押して離すだけならその場でタイプでき、Esc がどちらも取り消す。
//! **節の三角と FX の ON も効く**(欠陥 D2): 節は makepad の `FoldHeader`
//! (三角は `fold_button` という名前でなければ本体が見つけられない)、
//! ON は `CheckBoxFlat` の活性状態そのもの。開閉状態を自前で持たない —
//! makepad の機構を自前で作り直さない(2026-08-27 の教訓)。
//!
//! **ホストとの継ぎ目**: 値の確定は [`ScrubValueAction`] として widget action に
//! 出る。`prop`(例 `"position.x"`)が「何が動いたか」を運ぶだけで、どのレイヤーか
//! は言わない — 選択は `main.rs` の `session` が持っている真実だからで、
//! `Intent::SetAttrs` へ写すのはホストの仕事。ここは Document を持たない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let InspectorRule = SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.seam}

    // 値セル — 数値だけがこの型を貰う。空欄や色見本は素の InkLabel のまま:
    // 触れる物と触れない物を型で分ける(Q0「触れそうで触れない物は不合格」の
    // 裏返し — 触れる物は触れるように見える必要がある)。
    // 欄は Ableton の窪み語法(face.well)。窪みが「ここは値の欄だ」と言う
    mod.widgets.ScrubValueBase = #(ScrubValue::register_widget(vm))
    mod.widgets.ScrubValue = set_type_default() do mod.widgets.ScrubValueBase{
        width: 52
        height: Fill
        // 欄同士がくっつくと3つの値が1つの欄に見える。地(面)を 2px だけ見せて割る
        margin: Inset{top: 3.0 bottom: 3.0 right: 2.0}
        align: Align{y: 0.5}
        // 1px 動かした時に足す量。行ごとに上書きする(--hot で振れる値なのでここ)
        step: 0.01
        precision: 2
        min: -1000000.0
        max: 1000000.0
        suffix: ""
        prop: ""
        value: 0.0
        // 窪みはこの draw_bg が塗る。**埋め込み TextInput の draw_bg は塗らない** —
        // 2026-08-27 実測(欄の色を #xff0000 にしても1画素も出ない。文字と書体は
        // 効くので、応じないのは背景の面だけ)。掴み代も兼ねる
        draw_bg.color: mod.tokens.face.well
        text_input: TextInputFlat{
            width: Fill
            height: Fit
            padding: Inset{left: mod.tokens.space.s1 right: mod.tokens.space.s1}
            margin: 0.
            is_read_only: true
            is_numeric_only: false
            empty_text: ""
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: mod.tokens.ink.strong
            draw_text.color_hover: mod.tokens.ink.strong
            draw_text.color_focus: mod.tokens.ink.strong
            draw_text.color_down: mod.tokens.ink.strong
            draw_text.color_empty: mod.tokens.ink.faint
            draw_text.color_disabled: mod.tokens.ink.faint
            draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}
        }
    }
    let ValueCell = mod.widgets.ScrubValue

    // 折り三角 — makepad の FoldButton(SDF の三角がそのまま ▼/▶ の回転)。
    // FoldHeader は header の中の **fold_button という名前** を探すので変えない
    let FoldGlyph = FoldButton{
        width: 12
        height: 18
        margin: 0.
        draw_bg.color: mod.tokens.ink.faint
        draw_bg.color_hover: mod.tokens.ink.body
        draw_bg.color_active: mod.tokens.ink.faint
    }

    // 節見出し — 「TRANSFORM 3 · 2 keyed」。左が名、右が計数(薄)
    let SectionCap = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.area
        fold_button := FoldGlyph{}
        name := InkLabel{width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}}
        count := InkLabel{width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
    }

    // 節 = 見出し + 畳める本体。開閉は FoldHeader が持つ(自前の状態を作らない)
    let Section = FoldHeader{
        width: Fill
        height: Fit
        flow: Down
        body_walk: Walk{width: Fill height: Fit}
        header: SectionCap{}
        body: View{width: Fill height: Fit flow: Down new_batch: true}
    }

    // property 行 — 分母 25px(inspector-ratio-ledger 実測基準)。
    // 左 3px = 値型の色。◆=keyed / ◇=非 keyed。値3列は等幅
    let PropertyRow = SolidView{
        width: Fill
        height: 25
        flow: Right
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel
        type_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.ink.faint}
        name := InkLabel{width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
        vx := ValueCell{}
        vy := ValueCell{}
        vz := ValueCell{}
        keyed := InkLabel{width: Fill align: Align{x: 1.0} padding: Inset{right: mod.tokens.space.s4} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    // 値が1つしか無い property。空の欄を並べない — 欄に見える物は必ず触れる
    let PropertyRowOne = SolidView{
        width: Fill
        height: 25
        flow: Right
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel
        type_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.ink.faint}
        name := InkLabel{width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
        vx := ValueCell{}
        pad_yz := View{width: 104 height: Fill}
        keyed := InkLabel{width: Fill align: Align{x: 1.0} padding: Inset{right: mod.tokens.space.s4} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    // 数でない値(色の 16 進)。scrub もタイプもできないので **欄に見せない** —
    // 素のインクで置く(ピッカーは色見本の仕事、レーンE)
    let PropertyRowText = SolidView{
        width: Fill
        height: 25
        flow: Right
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel
        type_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.ink.faint}
        name := InkLabel{width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
        vx := InkLabel{width: 52 draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}}
        pad_yz := View{width: 104 height: Fill}
        keyed := InkLabel{width: Fill align: Align{x: 1.0} padding: Inset{right: mod.tokens.space.s4} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    // FX の ON — CheckBoxFlat の箱を欄いっぱいまで広げた物(size >= 欄の寸法だと
    // 箱が欄を埋め、印を透明にすれば「琥珀のチップ」そのものになる)。
    // 状態は色だけでなく **語** も変える(ON / OFF)ので、色覚に預けない
    let FxPower = CheckBoxFlat{
        width: 26
        height: mod.tokens.size.chip
        padding: 0
        margin: Inset{right: mod.tokens.space.s3}
        align: Align{x: 0.5 y: 0.5}
        text: "ON"
        active: true
        label_walk: Walk{width: Fit height: Fit margin: 0.}
        draw_bg.size: 26.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: mod.tokens.face.well
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.color_active: mod.tokens.accent.on
        draw_bg.color_focus: mod.tokens.face.hover
        draw_bg.color_disabled: mod.tokens.face.well
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #x00000000
        draw_bg.mark_color_active_hover: #x00000000
        draw_bg.mark_color_focus: #x00000000
        draw_bg.mark_color_disabled: #x00000000
        draw_text.color: mod.tokens.ink.muted
        draw_text.color_hover: mod.tokens.ink.body
        draw_text.color_down: mod.tokens.ink.muted
        draw_text.color_active: mod.tokens.ink.on_fill
        draw_text.color_focus: mod.tokens.ink.body
        draw_text.color_disabled: mod.tokens.ink.faint
        draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}
    }

    mod.widgets.InspectorSurfaceBase = #(InspectorSurface::register_widget(vm))
    mod.widgets.InspectorSurface = set_type_default() do mod.widgets.InspectorSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // selection summary — 46/25 = 1.84(inspector-ratio-ledger の実測比。モック表示の 40 は概形)
        summary := SolidView{width: Fill height: 46 flow: Down padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4 top: mod.tokens.space.s2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            sel_name := InkLabel{text: "Rectangle" width: Fill height: Fit draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.lg}}
            sel_kind := InkLabel{text: "Shape layer · 3D transform · 2 keys" width: Fill height: Fit padding: Inset{top: mod.tokens.space.s1} draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
        summary_rule := InspectorRule{}

        // 列見出し 21/25
        col_head := SolidView{width: Fill height: 21 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            pad := View{width: 3 height: Fill}
            c_prop := InkLabel{text: "Property" width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_x := InkLabel{text: "X" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_y := InkLabel{text: "Y" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_z := InkLabel{text: "Z" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }

        transform := Section{
            header.name.text: "TRANSFORM"
            header.count.text: "3 · 2 keyed"
            body +: {
                row_position := PropertyRow{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Position"
                    vx.value: 0.125 vx.precision: 3 vx.step: 0.001 vx.prop: "position.x"
                    vy.value: -0.075 vy.precision: 3 vy.step: 0.001 vy.prop: "position.y"
                    vz.value: 0.0 vz.precision: 3 vz.step: 0.001 vz.prop: "position.z"
                    keyed.text: "◆"
                }
                row_rotation := PropertyRow{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Rotation"
                    vx.value: 0.0 vx.precision: 1 vx.step: 0.25 vx.suffix: "°" vx.prop: "rotation.x"
                    vy.value: 0.0 vy.precision: 1 vy.step: 0.25 vy.suffix: "°" vy.prop: "rotation.y"
                    vz.value: 24.0 vz.precision: 1 vz.step: 0.25 vz.suffix: "°" vz.prop: "rotation.z"
                    keyed.text: "◆"
                }
                row_scale := PropertyRow{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Scale"
                    vx.value: 1.0 vx.precision: 3 vx.step: 0.005 vx.prop: "scale.x"
                    vy.value: 1.0 vy.precision: 3 vy.step: 0.005 vy.prop: "scale.y"
                    vz.value: 1.0 vz.precision: 3 vz.step: 0.005 vz.prop: "scale.z"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
            }
        }

        appearance := Section{
            header.name.text: "APPEARANCE"
            header.count.text: "2 · 1 keyed"
            body +: {
                row_fill := PropertyRowText{
                    type_bar.draw_bg.color: #xd8c97f
                    name.text: "Fill"
                    vx.text: "#D8C97F"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
                row_opacity := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.on
                    name.text: "Opacity"
                    vx.value: 100.0 vx.precision: 0 vx.step: 0.5 vx.min: 0.0 vx.max: 100.0 vx.suffix: "%" vx.prop: "opacity"
                    keyed.text: "◆"
                }
            }
        }

        fx_stack := Section{
            header.name.text: "FX STACK"
            header.count.text: "1 effect"
            body +: {
                // effect 行 — 選択は行の地、証は色付き左端(裁定: ○ボタン/FXバッジは置かない)
                fx_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                    fx_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.record}
                    fx_name := InkLabel{text: "TURBULENT DISPLACE" width: Fill padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
                    fx_params := InkLabel{text: "8 params" width: Fit padding: Inset{right: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                    fx_on := FxPower{}
                }
                row_amount := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Amount"
                    vx.value: 42.0 vx.precision: 1 vx.step: 0.1 vx.prop: "fx.turbulent_displace.amount"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
            }
        }

        advanced := Section{
            header.name.text: "ADVANCED"
            header.count.text: "4 parameters"
            body +: {
                row_size := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Size"
                    vx.value: 62.0 vx.precision: 1 vx.step: 0.1 vx.prop: "fx.turbulent_displace.size"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
                row_offset := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Offset"
                    vx.value: 0.0 vx.precision: 1 vx.step: 0.5 vx.prop: "fx.turbulent_displace.offset"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
                row_complexity := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Complexity"
                    vx.value: 2.0 vx.precision: 1 vx.step: 0.05 vx.min: 1.0 vx.max: 10.0 vx.prop: "fx.turbulent_displace.complexity"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
                row_evolution := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Evolution"
                    vx.value: 0.0 vx.precision: 1 vx.step: 0.5 vx.suffix: "°" vx.prop: "fx.turbulent_displace.evolution"
                    keyed.text: "◇"
                    keyed.draw_text.color: mod.tokens.ink.faint
                }
            }
        }

        body_fill := View{width: Fill height: Fill}

        footer_rule := InspectorRule{}
        hint_row := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            hint := InkLabel{text: "drag to scrub · click to type · Esc to cancel" width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
    }
}

/// クリックとドラッグを分ける遊び(px)。これ未満の移動は「押して離した」= タイプ入口。
const CLICK_SLOP_PX: f64 = 2.0;

/// 値セルが外へ出す唯一の口。`prop` は「何が動いたか」の名前だけを運ぶ —
/// **どのレイヤーか**は言わない。選択は `main.rs` の `session` が持つ真実で、
/// `Intent::SetAttrs` へ写すのはホストの仕事。Inspector は Document を持たない。
#[derive(Clone, Debug, Default)]
pub enum ScrubValueAction {
    #[default]
    None,
    /// ドラッグ途中の値。1フレームに何度でも出る(プレビュー用)。
    Changed { prop: String, value: f64 },
    /// 確定値。1ジェスチャにつき1回だけ出る(= 1 undo 相当)。
    Committed { prop: String, value: f64 },
}

#[derive(Clone, Copy, Debug)]
struct ScrubDrag {
    start_x: f64,
    start_value: f64,
    moved: bool,
}

/// 数値の欄。**フッターの3つの約束がここに実装されている**:
/// 横ドラッグ = scrub / 押して離す = タイプ / Esc = 取消。
///
/// 表示そのものは埋め込みの `TextInput`(自前の文字編集は書かない — wraps > scratch)。
/// 普段は `is_read_only` で表示専用にしておき、タイプに入る時だけ外す。
/// **掴みはこちらが先に取る**: makepad の finger capture は先着順なので、
/// `TextInput` へ先に配ると横ドラッグがキャレット移動になる。
#[derive(Script, Widget)]
pub struct ScrubValue {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// セル全域の掴み代。透明だが area はここが持つ(欄の外の余白も掴める)。
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    text_input: TextInput,

    #[live]
    value: f64,
    /// 1px 動かした時に足す量。宣言側(script_mod!)で行ごとに決める。
    #[live(0.01)]
    step: f64,
    #[live(2)]
    precision: usize,
    #[live(-1000000.0)]
    min: f64,
    #[live(1000000.0)]
    max: f64,
    /// 表示にだけ付く単位(`°` / `%`)。タイプ中は外し、読む時は落として解釈する。
    #[live]
    suffix: String,
    /// ホストが「何が動いたか」を読む名前。空なら誰も配線していない印。
    #[live]
    prop: String,

    #[rust]
    drag: Option<ScrubDrag>,
    #[rust]
    editing: bool,
    /// ジェスチャに入る前の値。Esc はここへ戻す。
    #[rust]
    origin: f64,
}

impl ScriptHook for ScrubValue {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        self.origin = self.value;
        vm.with_cx_mut(|cx| {
            self.sync_display(cx);
        });
    }
}

impl ScrubValue {
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        self.value = self.clamped(value);
        self.sync_display(cx);
    }

    fn clamped(&self, value: f64) -> f64 {
        value.max(self.min).min(self.max)
    }

    /// 読ませる形(単位つき)。
    fn display_text(&self) -> String {
        format!("{:.*}{}", self.precision, self.value, self.suffix)
    }

    /// 打たせる形(単位なし。select_all してそのまま上書きさせる)。
    fn edit_text(&self) -> String {
        format!("{:.*}", self.precision, self.value)
    }

    fn sync_display(&mut self, cx: &mut Cx) {
        let text = self.display_text();
        self.text_input.set_text(cx, &text);
    }

    fn parse(&self, raw: &str) -> Option<f64> {
        let trimmed = raw.trim();
        let trimmed = if self.suffix.is_empty() {
            trimmed
        } else {
            trimmed.trim_end_matches(self.suffix.as_str()).trim()
        };
        trimmed.parse::<f64>().ok()
    }

    fn emit(&self, cx: &mut Cx, committed: bool) {
        let action = if committed {
            ScrubValueAction::Committed {
                prop: self.prop.clone(),
                value: self.value,
            }
        } else {
            ScrubValueAction::Changed {
                prop: self.prop.clone(),
                value: self.value,
            }
        };
        cx.widget_action(self.uid, action);
    }

    /// 押して離しただけの時。欄を編集可能へ開けて全選択する。
    fn begin_typing(&mut self, cx: &mut Cx) {
        self.editing = true;
        let text = self.edit_text();
        self.text_input.set_text(cx, &text);
        self.text_input.set_is_read_only(cx, false);
        self.text_input.set_key_focus(cx);
        self.text_input.select_all(cx);
        self.text_input.redraw(cx);
    }

    /// タイプを閉じる。`take` が None なら打った内容を捨てて元へ戻す(Esc)。
    fn end_typing(&mut self, cx: &mut Cx, take: Option<f64>) {
        self.editing = false;
        self.value = match take {
            Some(value) => self.clamped(value),
            None => self.origin,
        };
        self.text_input.set_is_read_only(cx, true);
        self.text_input.force_new_edit_group();
        self.sync_display(cx);
        self.redraw(cx);
        self.origin = self.value;
        self.emit(cx, true);
    }

    /// ドラッグを Esc で捨てる。掴む前の値へ戻して確定を1回出す。
    fn cancel_drag(&mut self, cx: &mut Cx) {
        self.drag = None;
        self.value = self.origin;
        self.sync_display(cx);
        self.redraw(cx);
        cx.set_cursor(MouseCursor::Default);
        self.emit(cx, true);
    }
}

impl Widget for ScrubValue {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Esc = 取消。タイプ中は TextInput が Escaped を出すのでそちらで受ける。
        // ドラッグ中はどこにもキーフォーカスが無いので生の KeyDown を見る。
        if self.drag.is_some() {
            if let Event::KeyDown(key) = event {
                if key.key_code == KeyCode::Escape {
                    self.cancel_drag(cx);
                    return;
                }
            }
        }

        // タイプ中でない間は、セル全域の掴みを **先に** 取る。後回しにすると
        // 埋め込み TextInput が FingerDown を捕まえてしまい(先着順)、
        // 横ドラッグがキャレット移動に化ける。
        if !self.editing {
            match event.hits(cx, self.draw_bg.area()) {
                Hit::FingerHoverIn(_) | Hit::FingerHoverOver(_) => {
                    cx.set_cursor(MouseCursor::EwResize);
                }
                Hit::FingerHoverOut(_) => {
                    cx.set_cursor(MouseCursor::Default);
                }
                Hit::FingerDown(fe) if fe.is_primary_hit() => {
                    self.origin = self.value;
                    self.drag = Some(ScrubDrag {
                        start_x: fe.abs.x,
                        start_value: self.value,
                        moved: false,
                    });
                    cx.set_cursor(MouseCursor::EwResize);
                }
                Hit::FingerMove(fe) => {
                    let step = self.step;
                    let mut moved_to = None;
                    if let Some(drag) = self.drag.as_mut() {
                        let dx = fe.abs.x - drag.start_x;
                        if dx.abs() > CLICK_SLOP_PX {
                            drag.moved = true;
                        }
                        if drag.moved {
                            moved_to = Some(drag.start_value + dx * step);
                        }
                    }
                    if let Some(next) = moved_to {
                        let next = self.clamped(next);
                        if next != self.value {
                            self.value = next;
                            self.sync_display(cx);
                            self.redraw(cx);
                            self.emit(cx, false);
                        }
                    }
                }
                Hit::FingerUp(fe) if fe.is_primary_hit() => {
                    if let Some(drag) = self.drag.take() {
                        if drag.moved {
                            self.origin = self.value;
                            self.emit(cx, true);
                        } else if fe.is_over {
                            self.begin_typing(cx);
                        }
                    }
                }
                _ => (),
            }
        }

        for action in cx.capture_actions(|cx| self.text_input.handle_event(cx, event, scope)) {
            match action.as_widget_action().cast::<TextInputAction>() {
                TextInputAction::Returned(text, _modifiers) => {
                    let parsed = self.parse(&text);
                    self.end_typing(cx, parsed);
                }
                TextInputAction::Escaped => {
                    if self.editing {
                        self.end_typing(cx, None);
                    }
                }
                TextInputAction::KeyFocusLost => {
                    // 外を押して抜けた時は打った内容を活かす(取消は Esc だけ)。
                    if self.editing {
                        let parsed = self.parse(&self.text_input.text());
                        self.end_typing(cx, parsed);
                    }
                }
                _ => (),
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        let inner = self.text_input.walk(cx);
        let _ = self.text_input.draw_walk(cx, scope, inner);
        self.draw_bg.end(cx);
        DrawStep::done()
    }

    /// 見えている物をそのまま返す — `--remote` の `/snap` が値を読めるように
    /// (窓を叩いて確かめる時、欄の中身が外から見えないと合否が付けられない)。
    fn text(&self) -> String {
        self.display_text()
    }

    /// ホストが store の値を書き戻す口。数として読めない文字列は無視する。
    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        if let Some(value) = self.parse(v) {
            self.set_value(cx, value);
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct InspectorSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetMatchEvent for InspectorSurface {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // FX の ON は語も変える — 消えた琥珀を色だけで読ませない
        if let Some(active) = self.view.check_box(cx, ids!(fx_on)).changed(actions) {
            let chip = self.view.widget(cx, ids!(fx_on));
            chip.set_text(cx, if active { "ON" } else { "OFF" });
            chip.redraw(cx);
        }
    }
}

impl WidgetNode for InspectorSurface {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for InspectorSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
