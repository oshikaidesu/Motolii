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
use std::collections::HashMap;

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

    // 区間の緩急 — 押すと INTERVAL EASING 板を開き、この行を名指す(Flow/Alight
    // Motion 様式、裁定「6型が見える板が正」)。**語が状態を運ぶ**(FX の ON と同じ
    // 規律)。巡回はしない — 型を選ぶのは板の6チップの仕事
    let EaseOpen = ButtonFlat{
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

    // イージング型の札。**6つとも常に見えている**(隠れていないから読める —
    // 可視性原理)。選ばれている1つだけが左端 3px の琥珀を出す。この panel が
    // 既に使っている「左端が値型を語る」文法をそのまま状態に転用する。
    // 面の色を Rust から差し替えないのは makepad の都合 — Button/CheckBox の
    // 面の色は uniform で個体ごとに効かない(2026-08-27 実測)。`visible` は
    // 個体ごとに効くので、印の出し入れで状態を語る
    let EasingChip = SolidView{
        width: Fill
        height: mod.tokens.size.chip
        flow: Right
        align: Align{y: 0.5}
        cursor: MouseCursor.Hand
        margin: Inset{right: mod.tokens.space.s1}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.well
        // 印は**幅を予約した枠の中**で出し入れする。枠なしで visible を振ると
        // 選ぶたびに札の文字が 3px ずれる(makepad の非表示は場所も取らない)
        on_slot := View{width: 3 height: Fill
            on := SolidView{width: Fill height: Fill visible: false show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.on}
        }
        cap := InkLabel{width: Fill padding: Inset{left: mod.tokens.space.s2} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
    }

    // 型ごとに意味も範囲も変わる値の欄。見出しは Rust から入れる —
    // X1 と DECAY と BOUNCES を1つの宣言では書けない
    let EasingParam = View{
        width: Fill
        height: Fill
        flow: Down
        visible: false
        margin: Inset{right: mod.tokens.space.s1}
        cap := InkLabel{text: "" width: Fill height: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        val := ValueCell{width: Fill height: mod.tokens.size.field margin: Inset{top: 2.0}}
    }

    mod.widgets.EasingCurveBase = #(EasingCurve::register_widget(vm))
    mod.widgets.EasingCurve = set_type_default() do mod.widgets.EasingCurveBase{
        width: Fill
        height: 84
        margin: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4 bottom: mod.tokens.space.s2}
        stroke: 1.5
        samples: 96
        // 表示(display)の窪み — 値そのものを見せる面(Ableton の DisplayBackground)
        draw_bg +: {color: mod.tokens.face.display}
        draw_grid +: {color: mod.tokens.rule.owner}
        draw_curve +: {color: mod.tokens.accent.on}
    }
    // `use mod.widgets.*`(この節の先頭)はこの節より前に登録された物しか掴んで
    // いない — `ValueCell`/`KeyEaseCell` と同じ理由で完全修飾を引き直す
    let EasingCurve = mod.widgets.EasingCurve

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
        ease: EaseOpen{}
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

        // 区間イージング(wf4、Flow / Alight Motion 様式)— **1区間の正規化 time
        // remap だけ**を扱う面。Graph View(時間方向の値グラフ)でも空間モーション
        // パス(位置の 2D 経路)でもない(2026-07-10 決定、`docs/concept.md`)。
        //
        // 既定は非表示 — **どの区間かを名指す前に開いていると、何を編集して
        // いるのか言えない板**になる(Q0)。EASE を押した行の名が見出しに出る
        easing := SolidView{
            width: Fill
            height: Fit
            flow: Down
            visible: false
            show_bg: true
            new_batch: true
            draw_bg.color: mod.tokens.face.panel

            easing_rule := InspectorRule{}
            easing_cap := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4}
                show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
                easing_name := InkLabel{text: "INTERVAL EASING" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}}
                easing_target := InkLabel{text: "" width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            }

            easing_types_a := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s3}
                show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                chip_linear := EasingChip{cap.text: "Linear"}
                chip_bezier := EasingChip{cap.text: "Bezier"}
                chip_hold := EasingChip{cap.text: "Hold"}
            }
            // 下段の3つが、AE では式(`valueAtTime` の物理シミュ)が要った領域。
            // 動きの"性格"を区間の補間型として持つ(2026-07-10、先例 Alight Motion)
            easing_types_b := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s3}
                show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                chip_bounce := EasingChip{cap.text: "Bounce"}
                chip_elastic := EasingChip{cap.text: "Elastic"}
                chip_steps := EasingChip{cap.text: "Steps"}
            }

            easing_curve := EasingCurve{}

            easing_params := SolidView{width: Fill height: 34 flow: Right
                padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s3 bottom: mod.tokens.space.s2}
                show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                ep0 := EasingParam{}
                ep1 := EasingParam{}
                ep2 := EasingParam{}
                ep3 := EasingParam{}
            }

            // この板が何であるかを言う1行。**fps に依らない**のがこの表現の要点で、
            // それが読めないと Graph View と取り違える
            easing_note := SolidView{width: Fill height: 16 flow: Right align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                note := InkLabel{text: "one interval · 0→1 normalized · fps-independent" width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            }
        }

        // FX STACK と ADVANCED の2節はここに直書きされていたが、**中身が実在しな
        // かった**(2026-08-28、FX レーンで撤去)。`TURBULENT DISPLACE` という
        // plugin は無く、`8 params` も `Amount`/`Size`/`Offset`/`Complexity`/
        // `Evolution` も、engine の `translate_effect_passes` が一つも知らない
        // (写せる `plugin_id` は `known_effects()` に載っている分だけ——2026-08-29
        // 時点で `"motolii.glow"`/`"motolii.isf_bloom"` の2本)。触れそうで触れない
        // 物は不合格(Q0)なので、実在する効果だけを出す `FxStack`(`fx_stack.rs`)へ
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
        // パラメトリック補間型(2026-08-28、`Interp::ease` が意味の家)。型は
        // INTERVAL EASING 板(wf4)の6チップから選べる。
        Interp::Bounce { .. } => "BOUNCE",
        Interp::Elastic { .. } => "ELASTIC",
        Interp::Steps { .. } => "STEPS",
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

/// [`ScrubValue::configure`] に渡す1欄分の目盛り。**新しい抽象ではなく、
/// 既に `ScrubValue` が `#[live]` で持っている6つの値の束**(引数が6本並ぶのを
/// 避けるためだけの記録)。
#[derive(Clone, Copy, Debug)]
pub struct ScrubSpec<'a> {
    pub prop: &'a str,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub precision: usize,
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

    /// 目盛りごと**実行時に**入れ替える口。区間イージングの値は型が変わるたびに
    /// 意味も範囲も変わる(`X1` は [0,1]、`DECAY` も [0,1] だが桁が違い、
    /// `BOUNCES` は整数)ので、宣言側で行ごとに固定できない唯一の欄になる。
    ///
    /// **掴んでいる最中とタイプ中は何もしない** — 指の下の値を外から書き換えると
    /// 「効いたように見えてから黙って戻る」が起きる。
    pub fn configure(&mut self, cx: &mut Cx, spec: ScrubSpec<'_>) {
        if self.drag.is_some() || self.editing {
            return;
        }
        self.prop.clear();
        self.prop.push_str(spec.prop);
        self.suffix.clear();
        self.min = spec.min;
        self.max = spec.max;
        self.step = spec.step;
        self.precision = spec.precision;
        self.value = self.clamped(spec.value);
        self.origin = self.value;
        self.sync_display(cx);
        self.redraw(cx);
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

// ---------------------------------------------------------------------------
// 区間イージング(wf4、2026-07-10 決定 `docs/concept.md`、先例 Alight Motion)
// ---------------------------------------------------------------------------

/// 型を選び直した時の入り口の値。**空の型を渡して利用者に数字を発明させない** —
/// どれも先例の既定(Bezier は CSS `ease-in-out`、Elastic は Penner の
/// `a=1, p=0.3`)。
const EASE_BEZIER: Interp = Interp::Bezier {
    x1: 0.42,
    y1: 0.0,
    x2: 0.58,
    y2: 1.0,
};
const EASE_BOUNCE: Interp = Interp::Bounce {
    bounces: 3,
    decay: 0.45,
};
const EASE_ELASTIC: Interp = Interp::Elastic {
    amplitude: 1.0,
    period: 0.3,
};
const EASE_STEPS: Interp = Interp::Steps { count: 4 };

/// 欄の `prop` に付ける印。ホスト(`main.rs`)が property の名前と取り違えない
/// ように、property 名には現れない接頭辞にしてある。
const EASING_PARAM_PREFIX: &str = "easing.p";

/// いま選ばれている型が持つ、編集できる値の並び。**型が変わると本数も意味も
/// 変わる**ので、宣言側ではなくここが正本になる。
fn easing_params(interp: Interp) -> Vec<(&'static str, ScrubSpec<'static>)> {
    // y は [0,1] の外へ出てよい(オーバーシュート = 行き過ぎ)。x は
    // `cubic_bezier_ease` が単調性のために [0,1] を要求する。
    let bezier_y = (-4.0, 5.0);
    match interp {
        Interp::Hold | Interp::Linear => Vec::new(),
        Interp::Bezier { x1, y1, x2, y2 } => vec![
            ("X1", scrub(x1, 0.0, 1.0, 0.004, 3)),
            ("Y1", scrub(y1, bezier_y.0, bezier_y.1, 0.004, 3)),
            ("X2", scrub(x2, 0.0, 1.0, 0.004, 3)),
            ("Y2", scrub(y2, bezier_y.0, bezier_y.1, 0.004, 3)),
        ],
        Interp::Bounce { bounces, decay } => vec![
            (
                "BOUNCES",
                scrub(bounces as f64, 0.0, Interp::MAX_BOUNCES as f64, 0.05, 0),
            ),
            ("DECAY", scrub(decay, 0.0, 1.0, 0.004, 3)),
        ],
        Interp::Elastic { amplitude, period } => vec![
            // 振幅 1 未満は表現できない(`Interp::Elastic` の doc 参照)ので
            // 欄の下限もそこに置く — 触れるのに何も起きない帯を作らない
            ("AMPLITUDE", scrub(amplitude, 1.0, 8.0, 0.01, 2)),
            ("PERIOD", scrub(period, 0.05, 2.0, 0.004, 3)),
        ],
        Interp::Steps { count } => vec![("STEPS", scrub(count as f64, 1.0, 64.0, 0.05, 0))],
    }
}

fn scrub(value: f64, min: f64, max: f64, step: f64, precision: usize) -> ScrubSpec<'static> {
    ScrubSpec {
        prop: "",
        value,
        min,
        max,
        step,
        precision,
    }
}

/// [`easing_params`] の `index` 番目を `value` にした型を返す。並びは
/// [`easing_params`] と同じ順(片方だけ足すと欄と値がずれるので、必ず対で直す)。
fn easing_with_param(interp: Interp, index: usize, value: f64) -> Interp {
    match (interp, index) {
        (Interp::Bezier { y1, x2, y2, .. }, 0) => Interp::Bezier {
            x1: value.clamp(0.0, 1.0),
            y1,
            x2,
            y2,
        },
        (Interp::Bezier { x1, x2, y2, .. }, 1) => Interp::Bezier { x1, y1: value, x2, y2 },
        (Interp::Bezier { x1, y1, y2, .. }, 2) => Interp::Bezier {
            x1,
            y1,
            x2: value.clamp(0.0, 1.0),
            y2,
        },
        (Interp::Bezier { x1, y1, x2, .. }, 3) => Interp::Bezier { x1, y1, x2, y2: value },
        (Interp::Bounce { decay, .. }, 0) => Interp::Bounce {
            bounces: value.round().clamp(0.0, Interp::MAX_BOUNCES as f64) as u32,
            decay,
        },
        (Interp::Bounce { bounces, .. }, 1) => Interp::Bounce {
            bounces,
            decay: value.clamp(0.0, 1.0),
        },
        (Interp::Elastic { period, .. }, 0) => Interp::Elastic {
            amplitude: value,
            period,
        },
        (Interp::Elastic { amplitude, .. }, 1) => Interp::Elastic {
            amplitude,
            period: value.max(f64::MIN_POSITIVE),
        },
        (Interp::Steps { .. }, 0) => Interp::Steps {
            count: value.round().max(1.0) as u32,
        },
        (other, _) => other,
    }
}

/// 1区間の正規化イージングを、そのまま絵にする板。
///
/// **評価器と同じ関数を呼ぶ**([`Interp::ease`])。front が曲線を描き直すと
/// 「見えている絵」と「実際に動く物」が黙ってずれるので、規則の家を2つ作らない。
///
/// 縦は自動で伸縮する — `Elastic` は 1 を越える(行き過ぎ)ので、[0,1] 固定だと
/// バネの山が画面の外へ出て「バネに見えない」。0 と 1 の高さ(= 前後のキーの値
/// そのもの)には線を引き、どこから先が行き過ぎかを読めるようにする。
#[derive(Script, ScriptHook, Widget)]
pub struct EasingCurve {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,

    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_grid: DrawColor,
    #[live]
    draw_curve: DrawColor,

    /// 曲線の最小の太さ(px)。
    #[live(1.5)]
    stroke: f64,
    /// 標本数。上げるほど滑らかになる。`Steps` の段は標本数に関係なく出る
    /// (段の縦線は隣り合う標本の差として描かれるため)。
    #[live(96)]
    samples: usize,

    /// `None` = まだ何も入っていない(= Linear と同じ絵)。
    #[rust]
    interp: Option<Interp>,
}

impl EasingCurve {
    pub fn set_interp(&mut self, cx: &mut Cx, interp: Interp) {
        if self.interp != Some(interp) {
            self.interp = Some(interp);
            self.redraw(cx);
        }
    }
}

impl Widget for EasingCurve {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);
        if rect.size.x <= 4.0 || rect.size.y <= 4.0 {
            return DrawStep::done();
        }

        let interp = self.interp.unwrap_or(Interp::Linear);
        let count = self.samples.clamp(8, 512);
        let mut ys = Vec::with_capacity(count + 1);
        // 端の 0/1 は常に見せる — 行き過ぎの量は「1 からどれだけ」なので、
        // 1 が画面の中に無いと読めない。
        let (mut lo, mut hi) = (0.0f64, 1.0f64);
        for i in 0..=count {
            let y = interp.ease(i as f64 / count as f64);
            let y = if y.is_finite() { y } else { 0.0 };
            lo = lo.min(y);
            hi = hi.max(y);
            ys.push(y);
        }
        let pad = ((hi - lo) * 0.08).max(0.02);
        let (lo, hi) = (lo - pad, hi + pad);
        let span = (hi - lo).max(1e-6);

        let inset = 2.0;
        let x0 = rect.pos.x + inset;
        let width = (rect.size.x - inset * 2.0).max(1.0);
        let y0 = rect.pos.y + inset;
        let height = (rect.size.y - inset * 2.0).max(1.0);
        let to_y = |v: f64| y0 + (hi - v) / span * height;

        self.draw_grid.new_draw_call(cx);
        for v in [0.0, 1.0] {
            self.draw_grid.draw_abs(
                cx,
                Rect {
                    pos: dvec2(x0, to_y(v)),
                    size: dvec2(width, 1.0),
                },
            );
        }

        self.draw_curve.new_draw_call(cx);
        let column = (width / count as f64).max(1.0);
        for i in 0..count {
            let a = to_y(ys[i]);
            let b = to_y(ys[i + 1]);
            self.draw_curve.draw_abs(
                cx,
                Rect {
                    pos: dvec2(x0 + width * i as f64 / count as f64, a.min(b)),
                    size: dvec2(column, (a - b).abs().max(self.stroke)),
                },
            );
        }
        DrawStep::done()
    }
}

/// 「押して、その上で離した」= クリック。外へ滑らせて離した物は取り消し。
fn released_over(view: &ViewRef, actions: &Actions) -> bool {
    view.finger_up(actions).is_some_and(|fe| fe.is_over)
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
    /// EASE を押した。INTERVAL EASING 板をこの行で開く(front ローカル — 板の
    /// 開閉は host の Document を触らない)。板が実際に緩急を変えたら
    /// [`InspectorSurfaceAction::SetInterp`] がそこから直接出る。
    OpenEasing { prop: String },
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
                if self.interp.is_some() {
                    cx.widget_action(
                        self.uid,
                        KeyEaseAction::OpenEasing {
                            prop: self.prop.clone(),
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

    /// いま INTERVAL EASING 板が開いている行の表示 id。空 = 板を閉じている。
    #[rust]
    easing_prop: String,
    /// 行ごとの区間イージング。**`set_property_keys` が投影で埋める** — front
    /// ローカルの写しであって store の正本ではない(`ScrubValue::value` が
    /// 投影を待つのと同じ状態)。
    #[rust]
    easing: HashMap<String, Interp>,
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
        // INTERVAL EASING 板の投影もここに乗る(呼び場所を増やさない) — `interp`
        // が来た行だけ板の写しを更新し、その行がいま開いていれば絵も引き直す。
        if let Some(interp) = interp {
            self.easing.insert(prop.to_owned(), interp);
            if self.easing_prop == prop {
                self.refresh_easing(cx);
            }
        }
    }

    fn current_easing(&self) -> Interp {
        self.easing
            .get(&self.easing_prop)
            .copied()
            .unwrap_or(Interp::Linear)
    }

    /// EASE を押した行の区間を開く。**見出しにその property の名を出す** —
    /// 何を編集しているか言えない板にしないため(Q0)。
    fn open_easing(&mut self, cx: &mut Cx, prop: &str) {
        self.easing_prop = prop.to_owned();
        self.view.view(cx, ids!(easing)).set_visible(cx, true);
        self.view
            .label(cx, ids!(easing_target))
            .set_text(cx, &prop.to_uppercase());
        self.refresh_easing(cx);
    }

    /// 型を差し替える。**同じ型を押し直しても値は捨てない** — 押すたびに
    /// 手で詰めた4値が既定へ戻るのは「効いたように見えてから黙って戻る」側。
    fn pick_easing_kind(&mut self, cx: &mut Cx, interp: Interp) {
        if self.easing_prop.is_empty() || self.current_easing().kind() == interp.kind() {
            return;
        }
        self.easing.insert(self.easing_prop.clone(), interp);
        self.refresh_easing(cx);
        self.emit_easing(cx);
    }

    fn apply_easing_param(&mut self, cx: &mut Cx, index: usize, value: f64, committed: bool) {
        if self.easing_prop.is_empty() {
            return;
        }
        let next = easing_with_param(self.current_easing(), index, value);
        self.easing.insert(self.easing_prop.clone(), next);
        // 掴んでいる最中は曲線だけ動かす。欄を書き戻すのは確定した時だけ
        // (指の下の値を外から書き換えない — `ScrubValue::configure` の約束)。
        if let Some(mut curve) = self
            .view
            .widget(cx, ids!(easing_curve))
            .borrow_mut::<EasingCurve>()
        {
            curve.set_interp(cx, next);
        }
        if committed {
            self.refresh_easing(cx);
            self.emit_easing(cx);
        }
    }

    /// 板が確定した緩急を host へ渡す唯一の口。**store へ書けるのは host だけ**
    /// (Inspector は Document を持たない) — `InspectorSurfaceAction::SetInterp` を
    /// 直接この面の uid から出す(`KeyEase::ToggleKey` と同じ経路)。
    fn emit_easing(&self, cx: &mut Cx) {
        if self.easing_prop.is_empty() {
            return;
        }
        cx.widget_action(
            self.uid,
            InspectorSurfaceAction::SetInterp {
                prop: self.easing_prop.clone(),
                interp: self.current_easing(),
            },
        );
    }

    /// 板の全部を、いまの型から描き直す。
    fn refresh_easing(&mut self, cx: &mut Cx) {
        let interp = self.current_easing();
        let kind = interp.kind();
        for (chip, name) in [
            (ids!(chip_linear.on), "Linear"),
            (ids!(chip_bezier.on), "Bezier"),
            (ids!(chip_hold.on), "Hold"),
            (ids!(chip_bounce.on), "Bounce"),
            (ids!(chip_elastic.on), "Elastic"),
            (ids!(chip_steps.on), "Steps"),
        ] {
            self.view.view(cx, chip).set_visible(cx, kind == name);
        }

        if let Some(mut curve) = self
            .view
            .widget(cx, ids!(easing_curve))
            .borrow_mut::<EasingCurve>()
        {
            curve.set_interp(cx, interp);
        }

        let params = easing_params(interp);
        let columns: [(&[LiveId], &[LiveId], &[LiveId]); 4] = [
            (ids!(ep0), ids!(ep0.cap), ids!(ep0.val)),
            (ids!(ep1), ids!(ep1.cap), ids!(ep1.val)),
            (ids!(ep2), ids!(ep2.cap), ids!(ep2.val)),
            (ids!(ep3), ids!(ep3.cap), ids!(ep3.val)),
        ];
        for (index, (col, cap, val)) in columns.into_iter().enumerate() {
            let Some((caption, spec)) = params.get(index) else {
                // 使わない欄は**消す**。空の欄を残すと「打てるのに何も起きない」
                // 物が並ぶ(Q0)
                self.view.view(cx, col).set_visible(cx, false);
                continue;
            };
            self.view.view(cx, col).set_visible(cx, true);
            self.view.label(cx, cap).set_text(cx, caption);
            let prop = format!("{EASING_PARAM_PREFIX}{index}");
            if let Some(mut cell) = self.view.widget(cx, val).borrow_mut::<ScrubValue>() {
                cell.configure(
                    cx,
                    ScrubSpec {
                        prop: &prop,
                        ..*spec
                    },
                );
            }
        }
        self.view.redraw(cx);
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
            // 値セルの確定。`easing.p{n}` 接頭辞は INTERVAL EASING 板の4欄自身
            // (`EASING_PARAM_PREFIX` の doc)— host の SetValue へは出さず、板の
            // 中で完結させる(板が確定した緩急は `emit_easing` が別途出す)。
            match widget_action.cast::<ScrubValueAction>() {
                ScrubValueAction::Changed { prop, value } => {
                    if let Some(index) = prop
                        .strip_prefix(EASING_PARAM_PREFIX)
                        .and_then(|rest| rest.parse::<usize>().ok())
                    {
                        self.apply_easing_param(cx, index, value, false);
                    }
                }
                ScrubValueAction::Committed { prop, value } => {
                    if let Some(index) = prop
                        .strip_prefix(EASING_PARAM_PREFIX)
                        .and_then(|rest| rest.parse::<usize>().ok())
                    {
                        self.apply_easing_param(cx, index, value, true);
                    } else if !prop.is_empty() {
                        out.push(InspectorSurfaceAction::SetValue { prop, value });
                    }
                }
                ScrubValueAction::None => {}
            }
            match widget_action.cast() {
                KeyEaseAction::ToggleKey { prop, keyed } => {
                    if !prop.is_empty() {
                        out.push(InspectorSurfaceAction::ToggleKey { prop, keyed });
                    }
                }
                // 板の開閉は front ローカル(host へ出さない) — `open_easing` が
                // 見出しへ行の名を出し、板を開く。
                KeyEaseAction::OpenEasing { prop } => {
                    if !prop.is_empty() {
                        self.open_easing(cx, &prop);
                    }
                }
                KeyEaseAction::None => {}
            }
        }

        // 6型チップ。**押した瞬間に確定**(値セルと違いドラッグで途中経過を
        // 見せる物ではない)。`emit_easing` が host への SetInterp を出す。
        for (chip, interp) in [
            (ids!(chip_linear), Interp::Linear),
            (ids!(chip_bezier), EASE_BEZIER),
            (ids!(chip_hold), Interp::Hold),
            (ids!(chip_bounce), EASE_BOUNCE),
            (ids!(chip_elastic), EASE_ELASTIC),
            (ids!(chip_steps), EASE_STEPS),
        ] {
            if released_over(&self.view.view(cx, chip), actions) {
                self.pick_easing_kind(cx, interp);
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
