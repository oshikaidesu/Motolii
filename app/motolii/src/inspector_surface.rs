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
//! **ホストとの継ぎ目**: 値の確定・キーの打ち消し・区間の緩急は、どれも
//! [`InspectorSurfaceAction`] という**1つの型**で、`InspectorSurface` 自身の uid から
//! 出る(`TimelineSurfaceAction` と同じ形 — ホストは
//! `actions.filter_widget_actions_cast(inspector_uid)` を1本回すだけで済む)。
//! `prop`(例 `"position.x"` / `"position"`)が「何が動いたか」を運ぶだけで、
//! どのレイヤーかは言わない — 選択は `main.rs` の `session` が持っている真実だから。
//! ここは Document を持たない。
//!
//! **`prop` は store の `PropertyId` ではなく、この面の表示 id である。** 写像は
//! ホストが行う: 同じ「Position」でも Document が split-position(`position.x` /
//! `position.y` の別 track)を使っているか vec 1本(`position`)かは**書類ごとに違う**
//! ので、静的な宣言に焼けない。front が写像を持つと、書類によって嘘になる。
//!
//! **投影が来るまで、キーと緩急の口は現れない**([`KeyEase`])。store の今を知らない
//! まま「打った」「緩めた」と見せると、効いたように見えてから黙って戻る —
//! 裁定(a)により、それは「できない」より悪い。だから配線の有無を `const` の
//! スイッチではなく**投影が来たかどうか**そのもので決める(切り替え忘れが起きない)。
use makepad_widgets::*;
use motolii_store::Interp;

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

    // ◆ の口 — 「この時刻に、この値」を打つ/消す。`fx_stack.rs` の `FxOnChip` と
    // 同じ CheckBoxFlat の
    // 活性そのもの(自前の状態を持たない)。印は箱ではなく **字の菱形**なので、
    // 箱の mark は全部透明にして、面と字だけで状態を言う
    let KeyDot = CheckBoxFlat{
        width: 18
        height: mod.tokens.size.chip
        padding: 0
        margin: 0
        align: Align{x: 0.5 y: 0.5}
        text: "◇"
        active: false
        label_walk: Walk{width: Fit height: Fit margin: 0.}
        draw_bg.size: 18.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x00000000
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.color_active: #x00000000
        draw_bg.color_focus: mod.tokens.face.hover
        draw_bg.color_disabled: #x00000000
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #x00000000
        draw_bg.mark_color_active_hover: #x00000000
        draw_bg.mark_color_focus: #x00000000
        draw_bg.mark_color_disabled: #x00000000
        draw_text.color: mod.tokens.ink.faint
        draw_text.color_hover: mod.tokens.ink.body
        draw_text.color_down: mod.tokens.ink.faint
        draw_text.color_active: mod.tokens.accent.on
        draw_text.color_focus: mod.tokens.ink.body
        draw_text.color_disabled: mod.tokens.ink.faint
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    // 区間の緩急 — 押すたび LINEAR → EASE → HOLD と巡る。**語が状態を運ぶ**
    // (FX の ON と同じ規律: 色覚に預けない)。掴めるハンドルとグラフエディタは
    // 次の波で、ここは「選べる型」だけ
    let EaseCycle = ButtonFlat{
        width: Fill
        height: mod.tokens.size.chip
        margin: Inset{right: mod.tokens.space.s2}
        padding: Inset{left: mod.tokens.space.s2 right: mod.tokens.space.s2}
        align: Align{x: 1.0 y: 0.5}
        text: "LINEAR"
        draw_bg.color: mod.tokens.face.well
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: mod.tokens.ink.muted
        draw_text.color_hover: mod.tokens.ink.strong
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}
    }

    // property 行の右端。**投影が来るまでは `mark`(読むだけの ◆/◇)しか描かない** —
    // store の今を知らないまま打てるように見せない(Q0/裁定(a))
    mod.widgets.KeyEaseBase = #(KeyEase::register_widget(vm))
    mod.widgets.KeyEase = set_type_default() do mod.widgets.KeyEaseBase{
        width: Fill
        height: Fill
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{right: mod.tokens.space.s4}
        // 面は持たない。行の地がそのまま透ける
        draw_bg.color: #x00000000
        // 行の表示 id(`"position"` / `"opacity"`)。空なら誰も配線していない印
        prop: ""
        mark: InkLabel{width: Fill align: Align{x: 1.0} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
        key: KeyDot{}
        ease: EaseCycle{}
    }
    // `use mod.widgets.*`(この節の先頭)は**この節より前に登録された物**しか
    // 掴んでいない。同じ節の中で足した型は完全修飾で引き直す — `ValueCell` が
    // `mod.widgets.ScrubValue` を引いているのと同じ理由
    let KeyEaseCell = mod.widgets.KeyEase

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
        keyed := KeyEaseCell{}
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
        keyed := KeyEaseCell{}
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
        keyed := KeyEaseCell{}
    }

    // FX の ON バッジはここから出た(2026-08-28、FX レーン)。効果の面は
    // `fx_stack.rs` の `FxStack` が持っており、その中の `FxOnChip` が同じ物の
    // **唯一の家**である — 同じ意味の宣言を2箇所に置かない。

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
                    keyed.prop: "position"
                    keyed.mark.text: "◆"
                }
                row_rotation := PropertyRow{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Rotation"
                    vx.value: 0.0 vx.precision: 1 vx.step: 0.25 vx.suffix: "°" vx.prop: "rotation.x"
                    vy.value: 0.0 vy.precision: 1 vy.step: 0.25 vy.suffix: "°" vy.prop: "rotation.y"
                    vz.value: 24.0 vz.precision: 1 vz.step: 0.25 vz.suffix: "°" vz.prop: "rotation.z"
                    keyed.prop: "rotation"
                    keyed.mark.text: "◆"
                }
                row_scale := PropertyRow{
                    type_bar.draw_bg.color: mod.tokens.accent.alt
                    name.text: "Scale"
                    vx.value: 1.0 vx.precision: 3 vx.step: 0.005 vx.prop: "scale.x"
                    vy.value: 1.0 vy.precision: 3 vy.step: 0.005 vy.prop: "scale.y"
                    vz.value: 1.0 vz.precision: 3 vz.step: 0.005 vz.prop: "scale.z"
                    keyed.prop: "scale"
                    keyed.mark.text: "◇"
                    keyed.mark.draw_text.color: mod.tokens.ink.faint
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
                    // `keyed.prop` を書かない = この行の◆は読むだけ。色に写る
                    // store の property がまだ無いので、宛先の無い意図を出さない
                    // (2026-08-28 の統合裁定。`prop` が空 = 誰も配線していない印)
                    keyed.mark.text: "◇"
                    keyed.mark.draw_text.color: mod.tokens.ink.faint
                }
                row_opacity := PropertyRowOne{
                    type_bar.draw_bg.color: mod.tokens.accent.on
                    name.text: "Opacity"
                    vx.value: 100.0 vx.precision: 0 vx.step: 0.5 vx.min: 0.0 vx.max: 100.0 vx.suffix: "%" vx.prop: "opacity"
                    keyed.prop: "opacity"
                    keyed.mark.text: "◆"
                }
            }
        }

        // FX STACK と ADVANCED の2節はここに直書きされていたが、**中身が実在しな
        // かった**(2026-08-28、FX レーンで撤去)。`TURBULENT DISPLACE` という
        // plugin は無く、`8 params` も `Amount`/`Size`/`Offset`/`Complexity`/
        // `Evolution` も、engine の `translate_effect_passes` が一つも知らない
        // (写せる `plugin_id` は `"motolii.glow"` 1本だけ)。触れそうで触れない物は
        // 不合格(Q0)なので、実在する効果だけを出す `FxStack`(`fx_stack.rs`)へ
        // 置き換えた。差し込み口は `main.rs` の `InspectorPane` にある。

        // ここに `body_fill := View{width: Fill height: Fill}` が居た。**Fit の turtle の
        // 中に Fill の子は置けない** — 残り高さが決まらないので `move_align_list` が
        // `dy = NaN` で assert に落ち、窓が起動直後に死ぬ(2026-08-28 の統合で実測)。
        // この面は `main.rs` の `InspectorPane` で `height: Fit` に置かれ、余りは
        // 下の兄弟 `FxStack{height: Fill}` が取る。押し下げる詰め物はもう要らない。

        footer_rule := InspectorRule{}
        hint_row := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            hint := InkLabel{text: "drag to scrub · click to type · Esc to cancel" width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
    }
}

/// クリックとドラッグを分ける遊び(px)。これ未満の移動は「押して離した」= タイプ入口。
const CLICK_SLOP_PX: f64 = 2.0;

/// AE の Easy Ease(既定 influence 33%)を cubic-bezier へ写した値。
///
/// **AE parity の審判は利用者ではなく Lottie**(裁定272)。keyframe の
/// `o`(Out Tangent)と `i`(In Tangent)は `easing-handle`(x,y ∈ [0,1])で、
/// store の [`Interp::Bezier`] の `x1,y1` がそのまま `o`、`x2,y2` が `i` である
/// (`app/reference/lottie.schema.json` の `base-keyframe`)。つまり **AE の Easy Ease が
/// 何なのかはこの4数に外出しされている**。
///
/// 同じ4数は二代目 UI(`next/ui/motolii-timeline-pane/src/write/mod.rs` の `EASY_EASE`)に
/// 既に居る。**正しい家は front ではなく `motolii-eval`**(意味の層)だが、生きている側に
/// まだ無いので当面ここが写しを持つ。写しが2つある状態なので FINDING に上げてある。
const EASY_EASE: Interp = Interp::Bezier {
    x1: 0.333,
    y1: 0.0,
    x2: 0.667,
    y2: 1.0,
};

/// 区間の緩急の名前。**語が状態を運ぶ** — 色覚に預けない(FX の ON と同じ規律)。
fn interp_label(interp: Interp) -> &'static str {
    match interp {
        Interp::Hold => "HOLD",
        Interp::Linear => "LINEAR",
        found if found == EASY_EASE => "EASE",
        // 名前の付いていない曲線。グラフエディタが来るまで front は形を作れないので、
        // 「これは EASE でも LINEAR でもない」とだけ言う(嘘の名前を付けない)
        Interp::Bezier { .. } => "BEZIER",
        // パラメトリック補間型(2026-08-28、`Interp::ease` が意味の家)。名前は
        // 読めるが、front からの入口はまだ無い(INTERVAL EASING の板は次の波)
        Interp::Bounce { .. } => "BOUNCE",
        Interp::Elastic { .. } => "ELASTIC",
        Interp::Steps { .. } => "STEPS",
    }
}

/// 押すたびに巡る順。LINEAR → EASE → HOLD → LINEAR。
///
/// **名前の無い Bezier は LINEAR へ落とす。** 押した人に次が読めない状態を作らない
/// ためで、曲線そのものを front が編集できるようになる(掴めるハンドル/グラフ
/// エディタ)のは次の波。
fn next_interp(interp: Interp) -> Interp {
    match interp {
        Interp::Linear => EASY_EASE,
        found if found == EASY_EASE => Interp::Hold,
        Interp::Hold => Interp::Linear,
        Interp::Bezier { .. } => Interp::Linear,
        // パラメトリック型も名前の無い Bezier と同じ扱いで LINEAR へ戻す —
        // 押した人に次が読める。型の選択板(wf-4 の INTERVAL EASING)は次の波
        Interp::Bounce { .. } | Interp::Elastic { .. } | Interp::Steps { .. } => Interp::Linear,
    }
}

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

/// property 行の右端が外へ出す口。`prop` は**行の**表示 id(`"position"`)で、
/// 値セルの `prop`(`"position.x"`)より1段浅い — キーと緩急は track ごとに1組で、
/// 成分ごとには無いから(store の `KeyframeTrack` がそう出来ている)。
#[derive(Clone, Debug, Default)]
pub enum KeyEaseAction {
    #[default]
    None,
    /// playhead にキーを打つ(`keyed: true`)/ 消す(`false`)。
    ToggleKey { prop: String, keyed: bool },
    /// playhead を含む区間の緩急を変える。
    SetInterp { prop: String, interp: Interp },
}

/// property 行の右端。**投影が来るまでは読むだけの ◆/◇ しか描かない。**
///
/// キーを打つ口と区間の緩急の口は、ホストが [`InspectorSurface::set_property_keys`] で
/// **store の今を1度でも押し込んだ行にだけ**現れる。これは横着ではなく Q0 そのもの —
/// store の今を知らないまま「打った」「緩めた」と見せると、効いたように見えてから
/// 黙って戻る。裁定(a)により、それは「できない」より悪い。
///
/// **`const ..._WIRED` を置かないのは意図的**(A1/A5 の先例との違い)。配線の有無を
/// 人が切り替えるスイッチにすると、切り替え忘れが「実装はあるのに死んでいる」を作る。
/// 投影が来たかどうかそのものを条件にすれば、ホストが1本繋いだ瞬間に自動で本物になる。
///
/// 緩急の口はさらに `interp` が来た行だけ(= キーが2つ以上あって区間が実在する行だけ)。
/// キーが1つも無い property に「区間の緩急」は無いので、出したら嘘になる。
#[derive(Script, ScriptHook, Widget)]
pub struct KeyEase {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// 面は持たない(透明)。`#[redraw]` の area の持ち主としてだけ居る。
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    /// 投影が来る前の、読むだけの ◆/◇。
    #[live]
    mark: Label,
    /// 打つ/消す。活性そのものが「キーが在る」。
    #[live]
    key: CheckBox,
    /// 区間の緩急。押すたび LINEAR → EASE → HOLD と巡る。
    #[live]
    ease: Button,

    /// 行の表示 id。空なら誰も配線していない印(値セルの `prop` と同じ規約)。
    #[live]
    prop: String,

    /// ホストが1度でも投影したか。**これが配線の有無そのもの。**
    #[rust]
    projected: bool,
    #[rust]
    keyed: bool,
    /// playhead を含む区間の緩急。区間が無ければ `None`(= 緩急の口を出さない)。
    #[rust]
    interp: Option<Interp>,
}

impl KeyEase {
    /// ホストが store から読んだ「この track の今」を押し込む口。
    /// **呼ばれた瞬間からこの行は本物になる**(それまでは読むだけ)。
    fn set_state(&mut self, cx: &mut Cx, keyed: bool, interp: Option<Interp>) {
        self.projected = true;
        self.keyed = keyed;
        self.interp = interp;
        self.key.set_active(cx, keyed, Animate::No);
        self.key.set_text(cx, if keyed { "◆" } else { "◇" });
        if let Some(interp) = interp {
            self.ease.set_text(cx, interp_label(interp));
        }
        self.redraw(cx);
    }
}

impl Widget for KeyEase {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 投影が来ていない行は描いてもいないので当たり判定も無い。子へ配らない。
        if !self.projected {
            return;
        }
        let has_segment = self.interp.is_some();
        for action in cx.capture_actions(|cx| {
            self.key.handle_event(cx, event, scope);
            if has_segment {
                self.ease.handle_event(cx, event, scope);
            }
        }) {
            if let CheckBoxAction::Change(keyed) = action.as_widget_action().cast() {
                // 見た目は倒したままにする。ホストが書いて投影を返すので、
                // 次の `set_state` が store の答えで上書きする(倒しっぱなしにしない)。
                self.keyed = keyed;
                self.key.set_text(cx, if keyed { "◆" } else { "◇" });
                self.redraw(cx);
                cx.widget_action(
                    self.uid,
                    KeyEaseAction::ToggleKey {
                        prop: self.prop.clone(),
                        keyed,
                    },
                );
            }
            if let ButtonAction::Clicked(_) = action.as_widget_action().cast() {
                if let Some(current) = self.interp {
                    let next = next_interp(current);
                    self.interp = Some(next);
                    self.ease.set_text(cx, interp_label(next));
                    self.redraw(cx);
                    cx.widget_action(
                        self.uid,
                        KeyEaseAction::SetInterp {
                            prop: self.prop.clone(),
                            interp: next,
                        },
                    );
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        if self.projected {
            // 緩急が先(Fill)、◆ が右端。行の右端に菱形が来る並びは投影前と同じ
            if self.interp.is_some() {
                let inner = self.ease.walk(cx);
                let _ = self.ease.draw_walk(cx, scope, inner);
            }
            let inner = self.key.walk(cx);
            let _ = self.key.draw_walk(cx, scope, inner);
        } else {
            let inner = self.mark.walk(cx);
            let _ = self.mark.draw_walk(cx, scope, inner);
        }
        self.draw_bg.end(cx);
        DrawStep::done()
    }

    /// `--remote` の `/snap` から状態が読めるように、見えている物をそのまま返す。
    fn text(&self) -> String {
        if !self.projected {
            return self.mark.text();
        }
        match self.interp {
            Some(interp) => format!(
                "{} {}",
                if self.keyed { "◆" } else { "◇" },
                interp_label(interp)
            ),
            None => if self.keyed { "◆" } else { "◇" }.to_owned(),
        }
    }
}

/// Inspector が外へ出す唯一の口。**どのレイヤーかは言わない** — 選択は `main.rs` の
/// `session` が持つ真実で、`prop` を store の `PropertyId` へ写すのもホストの仕事。
///
/// 子(`ScrubValue` / `KeyEase`)の action をここで1つの型へまとめ直しているのは、
/// `TimelineSurfaceAction` と同じ形にするため — ホストは
/// `actions.filter_widget_actions_cast(inspector_uid)` を1本回すだけでよく、
/// 欄が増えるたびにホストが uid を数え直す必要が無い。
#[derive(Clone, Debug, Default)]
pub enum InspectorSurfaceAction {
    #[default]
    None,
    /// 値の確定。**1ジェスチャ = 1回 = 1 undo。** ドラッグ途中の
    /// [`ScrubValueAction::Changed`] はここへ上げない(プレビューは欄の中の話)。
    SetValue { prop: String, value: f64 },
    /// playhead にキーを打つ/消す。**「この時刻に、この値」だけを名指す** —
    /// グループの尺もレイヤーの尺もキーを縛らない(裁定272。store の `value_at` も
    /// 既に `LayerTiming` を参照しない)。
    ToggleKey { prop: String, keyed: bool },
    /// playhead を含む区間の緩急。値は store の [`Interp`] そのもの
    /// (front に第二の緩急の語彙を作らない)。
    SetInterp { prop: String, interp: Interp },
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

impl InspectorSurface {
    /// 面の中の widget を全部たどる。**欄を名指ししない** — 行は他レーンが増やし
    /// 続けるので、名指しにすると増えるたびここが古くなる(`main.rs` の
    /// `text_entry_has_focus` が同じ理由で同じ形をしている)。
    fn each_widget(&self, visit: &mut dyn FnMut(&WidgetRef)) {
        fn walk(node: &WidgetRef, visit: &mut dyn FnMut(&WidgetRef)) {
            visit(node);
            node.children(&mut |_id, child| walk(&child, visit));
        }
        self.view.children(&mut |_id, child| walk(&child, visit));
    }

    /// store から読んだ値を欄へ戻す口。`prop` は値セルの表示 id(`"position.x"`)。
    ///
    /// **ホストは書いた後これを呼ぶ。** 欄が持っているのは「見せていた形」であって
    /// 正本ではないので、書けた値と欄の値が食い違ったまま残らないようにする
    /// (`main.rs` の5手の1「今の値を store から読む」の裏返し)。
    pub fn set_property_value(&mut self, cx: &mut Cx, prop: &str, value: f64) {
        let mut cells = Vec::new();
        self.each_widget(&mut |node| {
            if node
                .borrow::<ScrubValue>()
                .is_some_and(|cell| cell.prop == prop)
            {
                cells.push(node.clone());
            }
        });
        for node in cells {
            if let Some(mut cell) = node.borrow_mut::<ScrubValue>() {
                cell.set_value(cx, value);
            }
        }
    }

    /// selection summary(ヘッダの名前+種別)の投影口(発注 S5b)。**選択は
    /// `main.rs` の `session` が持つ真実**(モジュール doc 参照) — ここは押し込まれた
    /// 文字列をそのまま2つのラベルへ流すだけで、選択が何かを解釈しない。
    /// 選択が無い時に何を表示するかは呼び手(host)が決める("Select a layer to
    /// inspect" 等) — ここは器であって語彙の家ではない。
    pub fn set_selection_summary(&mut self, cx: &mut Cx, name: &str, kind: &str) {
        self.view
            .widget(cx, ids!(summary.sel_name))
            .as_label()
            .set_text(cx, name);
        self.view
            .widget(cx, ids!(summary.sel_kind))
            .as_label()
            .set_text(cx, kind);
    }

    /// この track の playhead での今を押し込む口。`prop` は**行の**表示 id
    /// (`"position"`)。`keyed` = playhead にキーが在るか、`interp` = playhead を
    /// 含む区間の緩急(区間が無ければ `None`)。
    ///
    /// **この呼び出しが「配線されている」の定義そのもの。** 呼ばれるまで、その行の
    /// キーの口も緩急の口も現れない(Q0 — store へ届かない物を触れるように見せない)。
    pub fn set_property_keys(
        &mut self,
        cx: &mut Cx,
        prop: &str,
        keyed: bool,
        interp: Option<Interp>,
    ) {
        let mut cells = Vec::new();
        self.each_widget(&mut |node| {
            if node
                .borrow::<KeyEase>()
                .is_some_and(|cell| cell.prop == prop)
            {
                cells.push(node.clone());
            }
        });
        for node in cells {
            if let Some(mut cell) = node.borrow_mut::<KeyEase>() {
                cell.set_state(cx, keyed, interp);
            }
        }
    }
}

impl WidgetMatchEvent for InspectorSurface {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // FX の ON バッジの世話はここから出た(2026-08-28 の統合)。効果の面は
        // `fx_stack.rs` の `FxStack` が丸ごと持っており、ON の語(ON/OFF)も
        // そちらの `draw_walk` が model から入れる。ここに残すと、存在しない
        // `ids!(fx_on)` を毎回引きにいく死んだ枝になる。

        // 子の action を1つの型へまとめ直して面の uid から出す。ホストが読むのは
        // ここだけ(`TimelineSurfaceAction` と同じ形)。**`prop` が空の物は落とす** —
        // 空は「誰も配線していない印」なので、外へ出すと宛先の無い意図になる。
        let mut out = Vec::new();
        for action in actions {
            let widget_action = action.as_widget_action();
            if let ScrubValueAction::Committed { prop, value } = widget_action.cast() {
                if !prop.is_empty() {
                    out.push(InspectorSurfaceAction::SetValue { prop, value });
                }
            }
            match widget_action.cast() {
                KeyEaseAction::ToggleKey { prop, keyed } => {
                    if !prop.is_empty() {
                        out.push(InspectorSurfaceAction::ToggleKey { prop, keyed });
                    }
                }
                KeyEaseAction::SetInterp { prop, interp } => {
                    if !prop.is_empty() {
                        out.push(InspectorSurfaceAction::SetInterp { prop, interp });
                    }
                }
                KeyEaseAction::None => {}
            }
        }
        for action in out {
            cx.widget_action(self.uid, action);
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
