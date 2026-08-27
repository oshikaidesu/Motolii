//! 全パネル共通の目盛り。Tailwind と同じ考え方 — 値は数直線から選ぶものであって、
//! その場で書くものではない。`14` と `13` が並ぶ理由を後から誰も説明できないため。
//!
//! `scale` だけが可変で、寸法系は全部それに掛かる。UI 全体を 1% 刻みで拡縮できるのは
//! これが1箇所しか無いから。色は拡縮しないので `scale` を掛けない。
use std::sync::atomic::{AtomicU32, Ordering};

/// UI 全体の拡縮(%)。100 が等倍。`script_mod!` の式へ焼き込まれるので、
/// 変更後は `cx.request_live_edit()` で再評価させる(makepad が iOS の safe-area
/// inset に使っているのと同じ経路)。
static UI_SCALE_PERCENT: AtomicU32 = AtomicU32::new(100);

pub const UI_SCALE_MIN: i32 = 50;
pub const UI_SCALE_MAX: i32 = 300;

pub fn ui_scale_percent() -> i32 {
    UI_SCALE_PERCENT.load(Ordering::Relaxed) as i32
}

pub fn set_ui_scale_percent(percent: i32) -> i32 {
    let percent = percent.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
    UI_SCALE_PERCENT.store(percent as u32, Ordering::Relaxed);
    percent
}

pub fn ui_scale() -> f64 {
    ui_scale_percent() as f64 / 100.0
}

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    let s = #(crate::tokens::ui_scale())

    // 余白 — 2px 刻み。名前は Tailwind と同じく「いくつ分か」で読む
    let space = {
        s1: 2.0 * s
        s2: 4.0 * s
        s3: 6.0 * s
        s4: 8.0 * s
        s5: 10.0 * s
        s6: 12.0 * s
        s8: 16.0 * s
        s10: 20.0 * s
        s12: 24.0 * s
    }

    // 寸法 — 行の高さと帯の高さ。ここが UI の密度そのもの
    let size = {
        // Live 実機の計量(2026-08-27): rail 行ピッチ 15.0pt / 文字 cap 8.0pt(比 1.88)。
        // 行の太さでなく比が手触りを決める
        row: 15.0 * s
        row_tight: 14.0 * s
        bar: 22.0 * s
        toolbar: 26.0 * s
        cap: 18.0 * s
        // 入力欄を持つ行は一覧行より高い。同じ row を使い回すと欄が潰れる
        form_row: 24.0 * s
        well: 18.0 * s
        chip: 16.0 * s
        field: 17.0 * s
        icon_sm: 11.0 * s
        icon: 12.0 * s
        icon_lg: 17.0 * s
        rail: 132.0 * s
        // 行の中のグリフも同じ比率から導出する。Live 実測: 行の中身の帯(アイコン∪文字)が
        // 行高の 0.53 — アイコンだけ大きいと字と喧嘩する(利用者指摘の「不自然」の一部)
        row_glyph: 15.0 * 0.53 * s
        pane: 300.0 * s
        chrome: 32.0 * s
        status: 20.0 * s
        menu: 24.0 * s
        transport: 24.0 * s
        // dock のタブ行。makepad 側に 25pt の下限がある(`max(theme.tab_flat_height, 25.)`)
        tab_bar: 25.0 * s
        // 浮くタブ行を「開く」隅の大きさ。開いた後は帯の全幅で保持する(hysteresis)
        tab_reveal: 140.0 * s
    }

    // 字の大きさは**比率で導出する**(利用者裁定 2026-08-27「文字は比率で見るべき」、
    // 比率の原則と同じ)。Live 実測: 行 15pt に字インク 8pt = 0.53 —
    // 目盛りの target_cell_ratio 0.52 とほぼ同じ数を Ableton は字にも使っている。
    // ink_k = このテーマ書体の「フォント指定 1pt あたりの実インク高」。
    // (font 10 での実測 1.10。計量器の「cap」は行の中身の帯=アイコン∪文字を測るので、
    // 文字だけを k に混ぜない)
    // よって本文 = row * 0.53 / 1.1。密度(size.row)を変えると字が自動で追随する。
    // 残りの規則(裁定270): 等幅は値だけ / 薄インクは本文サイズ以上でだけ。
    let type_ratio = 0.53
    let ink_k = 1.1
    let body_pt = size.row * type_ratio / ink_k
    let text = {
        // caption 級には床を置く(7.5)。比率導出だけだと 6pt 台まで落ちて、
        // 「小さい字に薄インクを使わない」(裁定270)を自分で破る
        xs: max(body_pt * 0.85, 7.5)
        sm: body_pt
        base: body_pt
        md: body_pt * 0.95
        lg: body_pt * 1.08
        xl: body_pt * 1.18
    }

    // 境の線。太さは1つしか無い。色は「何の境か」で分ける —
    // 線は操作の持ち主か座標系が変わる所にだけ引く。
    // Ableton の線は両方の面より暗い(ControlContrastFrame #111111)。盛り上げ線は無い
    let rule = {
        size: 1.0
        owner: #x4a4a4a
        seam: #x1c1c1c
        pane: #x1c1c1c
        surface: #x111111
    }

    // 面 — 深さは影ではなく明度で作る。**暗いほど奥**。
    // 出典: Live 12 Beta `Default Dark Neutral Medium.ask`(実機、2026-08-27 抽出)
    //   Desktop #2a2a2a / SurfaceBackground #363636 / SurfaceArea #242424 /
    //   SurfaceHighlight #464646 / ControlBackground #1e1e1e / DisplayBackground #181818
    let face = {
        desktop: #x2a2a2a
        panel: #x363636
        bar: #x2f2f2f
        area: #x242424
        well: #x1e1e1e
        display: #x181818
        raised: #x464646
        head: #x464646
        sunk: #x242424
        hover: #x404040
        down: #x1e1e1e
        pressed: #x2e2e2e
    }

    // 字と記号 — Ableton のインクは3段(ControlForeground #b5b5b5 /
    // TextDisabled #757575 / 白は稀)。明るい面の上だけ #070707
    let ink = {
        strong: #xd0d0d0
        body: #xb5b5b5
        muted: #x757575
        faint: #x5d5d5d
        glyph: #xb5b5b5
        on_fill: #x070707
    }

    // 選択は色ではなく**極性反転** — 明るい面 + 暗いインク。
    // フォーカス外は脱彩して沈む(StandbySelectionBackground)。
    // 出典: .ask SelectionBackground #b0ddeb / Standby #637e86 / Foreground #070707
    let sel = {
        focus: #xb0ddeb
        standby: #x637e86
        ink: #x070707
    }

    // 「on」は琥珀1色(.ask ViewCheckControlEnabledOn / ChosenDefault /
    // TransportProgress = #ffad56)。record 赤・alt シアンは意味を持つ時だけ
    let accent = {
        on: #xffad56
        record: #xff5559
        alt: #x03c3d5
    }

    // 固定行高の中に置く裸のインク。theme 自身が H1 で `padding: 0.` とするのと同型の
    // 派生役割(全域上書きはしない)。理由: Label の padding は実装上 margin へ変換され
    // (`with_add_padding`)、Fit 行が抱き込む前提の設計 — makepad の正典は
    // `caption_bar := SolidView{height: Fit, ...}` で行側が Fit。固定行に既定 Label を
    // 置くと align 後に top margin 分だけ沈む(実測 Δ+1.5〜+2.8pt、「下にめり込む」の根)
    mod.widgets.InkLabel = mod.widgets.Label{padding: 0.}

    mod.tokens = {
        scale: s
        space: space
        text: text
        size: size
        rule: rule
        face: face
        ink: ink
        sel: sel
        accent: accent
    }
}
