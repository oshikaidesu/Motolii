//! ホスト向け公開型: ChromeCheck / ChromeToggle / ChromeLock
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素実測）
//! 形: 矩形のベタ面。枠線で囲まず面の明度差で領域を示す。状態は面色の切替だけ
//! （活性=色付き面に濃字 #252525、非活性=暗面 #323232 に灰字 #505050）。
//! 状態色（画像サンプル）:
//!   Check 活性 = ミキサー Track Activator「1」の明灰面 #a0a0a0（(950,74) 付近）
//!   Toggle 活性 = 上バー / Device「RMS」の活性橙 #xe89b3f（(157,445) 付近）
//!   Lock 押下 = ソロ「S」の青 #x2b7ad0（(969,177) 付近）
//! checkbox / toggle / lock。Document を持たない。
//! 3つとも**状態を持つ**部品 — 押している間だけの `color_down` に状態を預けない。
//! 技能の `CheckBoxFlat` / `ToggleFlat` を載せる。iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // checkbox — CheckBoxFlat。箱は窪み #282828、活性で明灰面 + 濃印。枠線ゼロ、行高 20
    mod.widgets.ChromeCheck = CheckBoxFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}        text: "Check"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 16}}
        draw_bg.size: 12.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x282828
        draw_bg.color_hover: #x343434
        draw_bg.color_down: #x282828
        draw_bg.color_active: #xa0a0a0
        draw_bg.color_focus: #x343434
        draw_bg.color_disabled: #x323232
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #x252525
        draw_bg.mark_color_active_hover: #x252525
        draw_bg.mark_color_focus: #xefefef
        draw_bg.mark_color_disabled: #x505050
        draw_text.color: #xa0a0a0
        draw_text.color_hover: #xefefef
        draw_text.color_down: #xa0a0a0
        draw_text.color_active: #xefefef
        draw_text.color_focus: #xefefef
        draw_text.color_disabled: #x505050
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // toggle — ToggleFlat。槽は窪み #282828、活性で橙面 + 濃摘み。枠線ゼロ
    mod.widgets.ChromeToggle = ToggleFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}        text: "On"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 28}}
        draw_bg.size: 12.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x282828
        draw_bg.color_hover: #x343434
        draw_bg.color_down: #x282828
        draw_bg.color_active: #xe89b3f
        draw_bg.color_focus: #x343434
        draw_bg.color_disabled: #x323232
        draw_bg.mark_color: #xa0a0a0
        draw_bg.mark_color_hover: #xefefef
        draw_bg.mark_color_down: #xa0a0a0
        draw_bg.mark_color_active: #x252525
        draw_bg.mark_color_active_hover: #x252525
        draw_bg.mark_color_focus: #xefefef
        draw_bg.mark_color_disabled: #x505050
        draw_text.color: #xa0a0a0
        draw_text.color_hover: #xefefef
        draw_text.color_down: #xa0a0a0
        draw_text.color_active: #xefefef
        draw_text.color_focus: #xefefef
        draw_text.color_disabled: #x505050
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // lock — 錠は**掛かっているか**であって、押している間の事ではない
    // (2026-08-27 台帳 E4)。ButtonFlatIcon の `color_down` はマウスを押している
    // 間だけの状態なので、離した瞬間に錠が外れて見えていた。
    // CheckBoxFlat の `active`(instance)へ載せ替える — makepad は「入っている」を
    // 色ではなく active で表す設計で、hot reload で宣言状態へ戻っても
    // ホストが `active` を投影し直せる。初期値は `ChromeLock{active: true}`。
    // 箱と印は描かず、面は 24 の矩形1枚。CheckBoxFlat の pixel を上書きするので
    // ChromeCheck とは別 shader になり、uniform(draw call 共有)も衝突しない
    // (memory `makepad-surface-colors-are-uniform`, 2026-08-27 実測)。
    // 掛かった面はソロ青 #x2b7ad0、その上の hover は同じ青の明度違い #x488ed7。
    // 反応は即時 — ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
    mod.widgets.ChromeLock = CheckBoxFlat{
        width: 24
        height: 24
        padding: 0
        margin: 0.
        spacing: 0
        align: Center        text: ""
        label_walk: Walk{width: 0 height: 0}
        icon_walk: Walk{width: 13 height: 13}
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let open = mix(#x3d3d3d, #x4f4f4f, self.hover)
                let held = mix(#x2b7ad0, #x488ed7, self.hover)
                let face = mix(mix(open, held, self.active), #x323232, self.disabled)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                return sdf.result
            }
        }
        draw_icon +: {svg: crate_resource("self://resources/icons/lock.svg") color: #xefefef}
    }
}
