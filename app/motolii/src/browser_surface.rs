//! Browser パネル枠。素材一覧の意味書きは波1。iced は引かない。
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素サンプル）
//! 色（画像実測）: パネル面 #4f4f4f / 上バー #3d3d3d / 検索窪み #282828 /
//!   リスト見出し帯 #606060 / 行明字 #d8d8d8〜#e4e4e4 / 節見出し・案内字 #9d9d9d /
//!   選択 = 行全体ベタ #6b8d96 + 濃字 #133342（`chrome/parts/nav.rs` と同一サンプル）/
//!   rail・リスト境の縦線 #343434 / 継ぎ目 1px #2d2d2d / Favorites 赤 #f20813
//! 形（Live の言語）: 行は低く詰める（16）。選択は行全体のベタ塗り+濃字。
//!   区切りは 1px 暗線か面の明度差だけ（枠線で囲まない）。角丸ゼロ・影ゼロ。
//!   検索欄は窪み矩形（暗面に薄字プレースホルダ）。
//! hover/down は静止画に写らないため nav.rs と同じ実測面 #5c5c5c / #2d2d2d を使う。
//!
//! ## 一覧はデータから出る（2026-08-27 C1〜C3）
//!
//! 以前ここは「8枚の手書きカードを col_a/col_b へ人力で振り分けた格子」で、
//! 検索欄は `InkLabel`（打てない面）だった。よって rail もフィルタも一覧へ
//! 届きようが無かった（[欠陥台帳](../../../docs/reviews/2026-08-27-layer-ui-parity-defects.md) C1〜C3）。
//!
//! - **並びは件数と幅が決める**: `card_grid` は `FlatList`（`flow: Right{wrap: true}`）。
//!   列数は turtle の折返しが幅から出す。入り切らなければ `ScrollBars` が縦へ送る。
//!   カードは `catalog` の1行に1つ、`FlatList` の template から作る
//! - **カタログは store の棚の投影**(2026-08-28 結線): 正本は `motolii-store` の
//!   `Composition:assets`(`AssetTable`、裁定162 の bin-first 台帳)。読み手は
//!   `BackendBridge`(`main.rs`)で、そこが [`BrowserAsset`] を組んで
//!   [`BrowserSurface::set_catalog`] へ渡す。**この widget は自分の一覧を持たない** —
//!   `TimelineSurface::set_model` と同じ「投影される側」の形
//! - **フィルタは1つの状態から導く**: rail = 置き場所（radio group が正本、`App` と
//!   同じ actions を読む）/ 検索語と tag = この widget が持つ。件数見出し・
//!   `Kind:` / `Tags:` の文言・`Clear` の出没は全部そこからの投影で、別に持たない
//!
//! ## 素材の口(2026-08-28、発注順序2「素材の配置」)
//!
//! ファイルを開く経路はこの面に居る。**ここに file browser を作らない** — 選ぶのは
//! OS の dialog(`rfd`、裁定176 で既に採った物)、調べるのは `motolii-media` の
//! probe、置くのは `motolii-store` の `Intent`。front が持つのは「どの口をどの順に
//! 呼ぶか」だけで、判断も計算も既にある物の中に在る。
//!
//! 別ファイルを作らなかった理由: 新しい module は `main.rs` の `mod` 宣言を要求し、
//! それは本レーンの write-set の外。`stage_import.rs` は名前が近いが**共有 GPU 面
//! (IOSurface)の取り込み**であって素材とは別室で、そこへ同居させると
//! `app/` が付け直した「名前が何であるかを言う」規律(AGENTS.md)を壊す。
//!
//! ## 棚 → タイムライン は **double-click**(2026-08-28、意図論=裁定271)
//!
//! カードを叩く人が求めているのは「**これを使いたい**」であって「ここへ置きたい」
//! ではない。drag は着地点(どの行・どのフレーム)の精密な指定という**別の意図**を
//! 含むので、最短の動詞にならない。よって [`BrowserEditAction::PlaceAsset`] は
//! double-click で出る。着地点は意図が名指していないので shell が既定
//! (playhead / 最前面 / `LayerTiming::place`)で埋める。
//!
//! **single click は何もしない。** 押した感じ(hover の面と Hand カーソル)は出すが、
//! 選択という身分が Browser にまだ無いので、選択の見た目だけ作って何も起きない
//! 方を避けた(裁定(a)「効いたように見えてから黙って戻る」はできないより悪い)。
use makepad_widgets::*;
use motolii_shell_state::Session;
use motolii_store::{
    AssetDraft, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    SourceFingerprintV1,
};
use std::path::{Path, PathBuf};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 素通しグリフ — 面を持たない。角丸・枠線なし
    let IconButton = ButtonFlatterIcon{
        margin: 0
        width: 24
        height: 16
        icon_walk: Walk{width: 11 height: 11}
        padding: 0
        draw_icon +: {color: mod.tokens.ink.glyph}
    }

    // tab — 「N のうち1つ」は makepad の radio。選択は `active`(instance)。
    // 面の色は uniform(draw call 共有)なので個体ごとに効かない。よって面は
    // instance の hover/down/active から shader で作る(実測 2026-08-27)。
    let TabIcon = RadioButtonTabFlat{
        width: Fill
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.row_glyph height: mod.tokens.size.row_glyph}
        label_walk: Walk{width: 0 height: 0}
        padding: Inset{left: 0 right: 0}
        align: Align{x: 0.5 y: 0.5}
        // 反応は即時。ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sunk = max(self.active, self.down)
                let face = mod.tokens.face.bar.mix(mod.tokens.face.hover, self.hover).mix(mod.tokens.face.pressed, sunk)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                // 押し込みは縁で語る: 上が暗く、下が明るい。枠線では囲まない
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.area, sunk))
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.raised, sunk))
                return sdf.result
            }
        }
        draw_icon +: {color: mod.tokens.ink.glyph}
    }

    // rail 行 — 低く詰める(16)。選択は押し込まれた面で語る(文字色は反転しない)
    let RailRow = RadioButtonTabFlat{
        width: Fill
        height: mod.tokens.size.row
        icon_walk: Walk{width: mod.tokens.size.row_glyph height: mod.tokens.size.row_glyph}
        label_walk: Walk{width: Fill height: Fit margin: Inset{left: mod.tokens.space.s4}}
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        // 反応は即時。ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sunk = max(self.active, self.down)
                let face = mod.tokens.face.panel.mix(mod.tokens.face.hover, self.hover).mix(mod.tokens.face.pressed, sunk)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                // 押し込みは縁で語る: 上が暗く、下が明るい。枠線では囲まない
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.area, sunk))
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(mod.tokens.face.raised, sunk))
                return sdf.result
            }
        }
        draw_icon +: {color: mod.tokens.ink.glyph}
        draw_text.color: mod.tokens.ink.strong
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    // 予約地の rail 行 — モックの慣習で半透明(意味が起草されるまで操作は無い)。
    // radio に入れない: 押せそうで押せない物を作らない(Q0)
    let RailRowReserved = View{
        width: Fill
        height: mod.tokens.size.row
        flow: Right
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        spacing: mod.tokens.space.s4
        glyph := Icon{width: mod.tokens.size.icon height: mod.tokens.size.icon align: Align{x: 0.5 y: 0.5} icon_walk: Walk{width: mod.tokens.size.row_glyph height: mod.tokens.size.row_glyph} draw_icon +: {color: mod.tokens.ink.faint}}
        label := InkLabel{width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    // 素材カード — Browser のヒーロー(正本: browser-semantics.html、旧 124×84 の縮約)。
    // ● = 配置済み(bin-first の可視化)。**double-click = 置く**(mod doc 参照)。
    // ● は「席」と「点」を分ける: 席は常に 5×5 を占め、点だけ出没する。
    // 色を透明へ差し替える旧手だと、値が uniform 側へ落ちた瞬間に全カードが道連れになる
    //
    // hover は **animator が `draw_bg.color` を差し替える**形(`chrome/parts/fold.rs`
    // と同じ)。`draw_bg +:` へ instance を足すと GPU layout はコンパイル時なので
    // "cannot push to frozen vec" で eval ごと落ちる。`SolidView` の `draw_bg.color` は
    // instance(`widgets/src/view_ui.rs`)なので animator が個体ごとに動かせる。
    // `down` 群は置かない — 無い群への `animator_play` は no-op。
    // **`cursor` が hit の引き金**: View が hit を拾う条件は
    // `cursor.is_some() || animator.is_defined`(`widgets/src/view.rs`)で、
    // 拾った FingerDown は `ViewAction::FingerDown`(`tap_count` 付き)として出る。
    let AssetCard = SolidView{
        width: 76
        height: 58
        flow: Down
        show_bg: true
        new_batch: true
        cursor: MouseCursor.Hand
        draw_bg.color: mod.tokens.face.area
        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: mod.tokens.face.area}}
                }
                on: AnimatorState{
                    cursor: MouseCursor.Hand
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: mod.tokens.face.hover}}
                }
            }
        }
        thumb := View{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5}
            glyph := Icon{width: mod.tokens.size.icon_lg + mod.tokens.space.s2 height: mod.tokens.size.icon_lg + mod.tokens.space.s2 align: Align{x: 0.5 y: 0.5} icon_walk: Walk{width: mod.tokens.size.icon_lg height: mod.tokens.size.icon_lg} draw_icon +: {color: mod.tokens.ink.faint}}
        }
        meta := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s2 right: mod.tokens.space.s2} spacing: mod.tokens.space.s2 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.well
            name := InkLabel{width: Fill margin: Inset{right: mod.tokens.space.s1} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            placed_slot := View{width: 5 height: 5 flow: Overlay
                dot := SolidView{width: Fill height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.on}
            }
        }
    }

    // 一覧が空のときの面 — 「無い」を無言の灰色で語らない
    let AssetEmpty = View{
        width: Fill
        height: Fit
        padding: Inset{left: mod.tokens.space.s3 top: mod.tokens.space.s4}
        note := InkLabel{width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    // 節見出し — Collections / Library / Places。薄字、上に群間の余白
    let RailCap = Label{
        width: Fill
        height: mod.tokens.size.cap
        padding: Inset{left: mod.tokens.space.s5 top: mod.tokens.space.s3}
        draw_text.color: mod.tokens.ink.muted
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    // 群の境は線で引く(利用者裁定 2026-08-27)。面の明度差が既に境になっている
    // tab strip 直下だけは引かない — 二重の境は境でなくなる
    let RailRule = SolidView{
        width: Fill
        height: mod.tokens.rule.size
        margin: Inset{top: mod.tokens.space.s3}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.owner
    }

    // 有効フィルタ chip — 選択の言語（ベタ #6b8d96 + 濃字 #133342、角丸なし）。
    // `chrome/parts/nav.rs` の ChromeChipOn と同一サンプル値。本 mod は chrome より先に
    // eval されるため参照でなく値で置く（未登録名の参照は葉が落ちる）
    let FilterChip = SolidView{
        width: Fit
        height: mod.tokens.size.chip
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.sel.standby
        label := ButtonFlatter{
            width: Fit
            height: Fit
            padding: Inset{left: 4 right: 4}
            draw_text.color: mod.tokens.sel.ink
            draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
        }
    }

    // shelf の押せる字 — 面を持たない。反応は字の明度だけ。
    // ButtonFlat の `draw_text.color` / `color_hover` は instance(個体ごとに効く)、
    // `color_down` は uniform なので触らない(実測 2026-08-27 の面=uniform と同じ話)
    let ShelfButton = ButtonFlatter{
        width: Fit
        height: Fit
        margin: 0
        padding: Inset{left: mod.tokens.space.s1 right: mod.tokens.space.s1}
        draw_text.color: mod.tokens.ink.faint
        draw_text.color_hover: mod.tokens.ink.strong
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}
    }

    // ファイル行 — Live 右リストの1行。低く詰める、ベタ、角丸なし
    let FileRow = ButtonFlat{
        width: Fill
        height: mod.tokens.size.row
        icon_walk: Walk{width: mod.tokens.size.row_glyph height: mod.tokens.size.row_glyph}
        spacing: mod.tokens.space.s4
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        draw_bg.color: mod.tokens.face.panel
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: mod.tokens.ink.glyph}
        draw_text.color: mod.tokens.ink.body
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    // rail とリストの境 — 画像実測 #343434 の縦 1px
    let PaneDivider = SolidView{
        width: mod.tokens.rule.size
        height: Fill
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.pane
    }

    // 継ぎ目 — 横 1px 暗線（枠線で囲まない）
    let SeamRule = SolidView{
        width: Fill
        height: mod.tokens.rule.size
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.seam
    }

    mod.widgets.BrowserSurfaceBase = #(BrowserSurface::register_widget(vm))
    mod.widgets.BrowserSurface = set_type_default() do mod.widgets.BrowserSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        browser_toolbar := SolidView{width: Fill height: mod.tokens.size.toolbar flow: Right spacing: mod.tokens.space.s2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            back := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/back.svg")}}
            forward := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/forward.svg")}}
            // 素材の口。Browser の動詞はこれ1つなので、絞り込みの道具
            // (filters/tags)ではなく前進の道具(back/forward)の隣に置く。
            // 配線が来るまでは席ごと畳む(`BrowserSurface::IMPORT_WIRED`)
            import := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/create.svg") color: mod.tokens.ink.strong}}
            search := SolidView{width: Fill height: mod.tokens.size.field flow: Right spacing: mod.tokens.space.s1 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.sunk
                search_glyph := IconButton{width: 15 draw_icon +: {svg: crate_resource("self://resources/icons/search.svg") color: mod.tokens.ink.muted}}
                // 本物の入力欄。窪みの面は親の SolidView が既に塗っているので、
                // 欄そのものは全状態で同じ窪み色に固定する(`chrome/parts/search.rs` と同型)。
                // TextInputFlat の既定は theme の inset 色へ飛ぶのでフラットに反する
                search_field := TextInputFlat{
                    width: Fill
                    height: Fill
                    margin: 0
                    padding: Inset{left: 2 right: 2}
                    label_align: Align{y: 0.5}
                    empty_text: "Search (Cmd + F)"
                    draw_bg.color: mod.tokens.face.sunk
                    draw_bg.color_hover: mod.tokens.face.sunk
                    draw_bg.color_focus: mod.tokens.face.sunk
                    draw_bg.color_down: mod.tokens.face.sunk
                    draw_bg.color_empty: mod.tokens.face.sunk
                    draw_bg.color_disabled: mod.tokens.face.sunk
                    draw_bg.border_size: 0.0
                    draw_bg.border_radius: 0.0
                    draw_text.color: mod.tokens.ink.strong
                    draw_text.color_hover: mod.tokens.ink.strong
                    draw_text.color_focus: mod.tokens.ink.strong
                    draw_text.color_down: mod.tokens.ink.strong
                    draw_text.color_empty: mod.tokens.ink.muted
                    draw_text.color_empty_hover: mod.tokens.ink.muted
                    draw_text.color_empty_focus: mod.tokens.ink.muted
                    draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
                }
            }
            filters := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            tags := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg")}}
            local := InkLabel{text: "LOCAL" width: Fit padding: Inset{left: mod.tokens.space.s2} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
        tabs := SolidView{width: Fill height: 22 flow: Right show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            media := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
            effects := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg")}}
            create := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/create.svg")}}
            panels := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")}}
        }

        browser_body := SolidView{width: Fill height: Fill flow: Right
            rail := SolidView{width: mod.tokens.size.rail height: Fill flow: Down padding: Inset{bottom: 2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                // Collections は予約地(タグ束、map D)。半透明で意味だけ predoc
                collections := RailCap{text: "Collections"}
                favorite := RailRowReserved{label.text: "Favorite" glyph.draw_icon.svg: crate_resource("self://resources/icons/star.svg")}
                brand := RailRowReserved{label.text: "Brand" glyph.draw_icon.svg: crate_resource("self://resources/icons/tag.svg")}
                library_rule := RailRule{}
                library := RailCap{text: "Library"}
                all_media := RailRow{text: "All media" draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                video := RailRow{text: "Video" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")}}
                images := RailRow{text: "Images" draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")}}
                audio := RailRow{text: "Audio" draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")}}
                project := RailRow{text: "Project" draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")}}
                recent := RailRow{text: "Recent" draw_icon +: {svg: crate_resource("self://resources/icons/loop.svg")}}
                places_rule := RailRule{}
                places := RailCap{text: "Places"}
                starter := RailRowReserved{label.text: "Starter Media" glyph.draw_icon.svg: crate_resource("self://resources/icons/folder.svg")}
                project_assets := RailRowReserved{label.text: "Project assets" glyph.draw_icon.svg: crate_resource("self://resources/icons/project.svg")}
                motion_assets := RailRowReserved{label.text: "Motion assets" glyph.draw_icon.svg: crate_resource("self://resources/icons/folder.svg")}
                add_folder := RailRowReserved{label.text: "Add Folder..." glyph.draw_icon.svg: crate_resource("self://resources/icons/create.svg")}
            }
            browser_owner_divider := PaneDivider{}
            catalog := SolidView{width: Fill height: Fill flow: Down show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                catalog_head_row := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right spacing: mod.tokens.space.s2 align: Align{y: 0.5} padding: Inset{left: 6 right: 2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.head
                    catalog_head := InkLabel{text: "Name" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                    // 件数は投影。宣言側の文言は初回描画までの繋ぎでしかない
                    result_count := InkLabel{text: "" width: Fit padding: Inset{right: mod.tokens.space.s2} draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                    view_modes := View{width: Fit height: Fit flow: Right align: Align{y: 0.5}
                        mode_thumb := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                        mode_grid := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/grid.svg") color: mod.tokens.ink.strong}}
                        mode_list := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/list.svg")}}
                    }
                }
                // filter shelf — filter だけを言う(件数は見出しへ移した)。UI 文字は英語。
                // `kind_filter` / `tag_filter` は投影（rail と tag 状態を読む）で、
                // `clear_filters` だけが動詞。消せる物が無いときは席ごと畳む
                filter_shelf := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right spacing: mod.tokens.space.s4 align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                    kind_filter := InkLabel{text: "" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                    tag_filter := InkLabel{text: "" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                    clear_filters := ShelfButton{text: "Clear"}
                }
                shelf_rule := SeamRule{}
                // ヒーロー = カード grid。並びは件数と幅が決める — 列は turtle の折返し、
                // 溢れは ScrollBars。カードは catalog の1行から作る(手書きの列は無い)。
                // 動詞は double-click = 置く(mod doc の意図論)。
                card_grid := FlatList{width: Fill height: Fill flow: Right{wrap: true} padding: mod.tokens.space.s3 spacing: mod.tokens.space.s3 wrap_spacing: mod.tokens.space.s3
                    CardVideo := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/video.svg")}
                    CardImage := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/image.svg")}
                    CardAudio := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/audio.svg")}
                    // 種を1つに決められない素材(点群など)。video の絵を借りると嘘になる
                    CardOther := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/media.svg")}
                    CardEmpty := AssetEmpty{}
                }
            }
        }
    }
}

/// Browser がもう1本だけ持つ、**編集意図**の口
/// (`TimelineEditAction` と同じ形 — widget は意図だけを言い、書くのは shell)。
///
/// **shell 側の受け口**: `main.rs` の `BackendBridge::place_asset_from_browser`。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum BrowserEditAction {
    #[default]
    None,
    /// カードの double-click = 「**これを使いたい**」。
    ///
    /// 運ぶのは `AssetId` の生値**だけ**。どの行へ・どのフレームへ置くかは
    /// この意図が名指していないので、shell が既定(playhead / 最前面 /
    /// `LayerTiming::place`)で埋める(裁定271: 操作は意図が名指した物だけを変える)。
    PlaceAsset { asset: u64 },
}

/// 素材の種。カードの絵は種で決まるので、種ごとに template を1枚持つ
/// (svg を実行時に差し替えると、どのカードがどの svg を持っているかが
/// 描画順に依存する)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AssetKind {
    Video,
    Image,
    Audio,
    /// 上の3つのどれとも言えない素材(点群・生成系など)。**video の絵を借りない** —
    /// 借りた瞬間、rail `Video` の絞り込みと絵が食い違う。
    Other,
}

impl AssetKind {
    /// `Asset::asset_type`(opaque な type 文字列、例 `video/mp4` / `image/png` /
    /// `pointcloud.octree.v1`)から種を読む。**store の語彙をここで発明しない** —
    /// 読むのは mime の主型だけで、知らない物は [`AssetKind::Other`] へ落とす。
    pub(crate) fn from_asset_type(asset_type: &str) -> Self {
        match asset_type.split('/').next().unwrap_or_default() {
            "video" => Self::Video,
            "image" => Self::Image,
            "audio" => Self::Audio,
            _ => Self::Other,
        }
    }

    fn template(self) -> LiveId {
        match self {
            AssetKind::Video => live_id!(CardVideo),
            AssetKind::Image => live_id!(CardImage),
            AssetKind::Audio => live_id!(CardAudio),
            AssetKind::Other => live_id!(CardOther),
        }
    }
}

/// 一覧の1行。**front の投影型であって保存形ではない。**
///
/// 正本は `motolii-store` の `Composition:assets`(`AssetTable` / `Asset`、裁定162)。
/// 組み立てるのは `BackendBridge::browser_catalog`(`main.rs`)で、この widget は
/// 受け取った物を絞って描くだけ。store 側へ新しい型は足していない
/// (`Asset` に必要な欄は既に在る)。
#[derive(Clone, Debug)]
pub(crate) struct BrowserAsset {
    /// `AssetId` の生値。**double-click が運ぶ唯一の名前**であり、
    /// `FlatList` の entry id もこれから作る(絞り込みで並びが変わっても
    /// 同じ素材が同じ widget を使い続ける)。
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) kind: AssetKind,
    /// この素材を指す layer がタイムラインに居るか(bin-first の可視化 = カード右下の ●)。
    pub(crate) placed: bool,
    /// タグ束。**store にタグの語彙がまだ無いので今は常に空** — 空である限り
    /// toolbar のタグ glyph は席ごと畳む(押せて何も起きない物を作らない)。
    /// `Collections`(Favorite / Brand)が起草されたらここへ入る。
    pub(crate) tags: Vec<String>,
}

/// rail = 置き場所。**radio group が正本**(`main.rs` の `browser_rail_ids!` と同じ並び)
/// で、この widget は同じ actions を読んで自分の投影を合わせるだけ。二重に持たない。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum BrowserScope {
    #[default]
    AllMedia,
    Video,
    Images,
    Audio,
    Project,
    Recent,
}

impl BrowserScope {
    /// rail の並び順。索引が意味を運ぶので `browser_rail_ids!` から離さない。
    const RAIL_ORDER: [Self; 6] = [
        Self::AllMedia,
        Self::Video,
        Self::Images,
        Self::Audio,
        Self::Project,
        Self::Recent,
    ];

    fn from_rail_index(index: usize) -> Self {
        Self::RAIL_ORDER.get(index).copied().unwrap_or_default()
    }

    /// `session_floor` = この窓が開いた時点で台帳に居なかった最初の `AssetId`
    /// ([`BrowserSurface::session_floor`])。`Recent` はそこから導く。
    fn accepts(self, asset: &BrowserAsset, session_floor: u64) -> bool {
        match self {
            Self::AllMedia => true,
            Self::Video => asset.kind == AssetKind::Video,
            Self::Images => asset.kind == AssetKind::Image,
            Self::Audio => asset.kind == AssetKind::Audio,
            // **台帳に居る = この project に入っている**(裁定162 の台帳は Document 所有)。
            // 今は `All media` と同じ集合になるが、これは嘘ではなく事実の一致である。
            Self::Project => true,
            // `Asset` に時刻は無い。だが `AssetId` は admit 順の単調増加なので、
            // 「この窓が開いた後に admit された物」は id だけで言える。**発明ではない。**
            Self::Recent => asset.id >= session_floor,
        }
    }

    /// shelf の `Kind:` 文言。置き場所であって種でない rail は "All" と言う —
    /// 嘘をつくくらいなら絞っていないと言う。
    fn kind_label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Images => "Images",
            Self::Audio => "Audio",
            Self::AllMedia | Self::Project | Self::Recent => "All",
        }
    }
}

/// 空一覧の面に使う entry id。カードは `AssetId + 1` を使うので 0 は空く。
const EMPTY_ENTRY: LiveId = LiveId(0);

/// `AssetId` の生値 ⇄ `FlatList` の entry id。`AssetId` は 0 から始まるので
/// [`EMPTY_ENTRY`] と衝突しないよう1つずらす。**この2つは互いの逆**。
fn entry_of(asset_id: u64) -> LiveId {
    LiveId(asset_id.wrapping_add(1))
}

fn asset_of(entry: LiveId) -> Option<u64> {
    (entry != EMPTY_ENTRY).then(|| entry.0.wrapping_sub(1))
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct BrowserSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// 一覧の中身。宣言側にカードは1枚も無い — 全部ここから出る。
    /// **正本は store の `Composition:assets`**、ここはその投影
    /// ([`BrowserSurface::set_catalog`] が入れる)。
    #[rust]
    catalog: Vec<BrowserAsset>,
    /// この窓が開いた時点で台帳に**居なかった**最初の `AssetId`。`Recent` の境目。
    /// 最初の投影で決まり、以後動かない(admit は単調増加なので、これより上は
    /// 全部「この窓が開いた後に入ってきた物」)。
    #[rust]
    session_floor: Option<u64>,
    #[rust]
    scope: BrowserScope,
    /// 検索欄の中身。`TextInputFlat` が正本だが、絞り込みは毎フレーム読むのでここへ写す。
    #[rust]
    query: String,
    /// tag 絞り込み。`None` = 絞っていない。
    #[rust]
    tag: Option<String>,
}

impl BrowserSurface {
    /// 素材の口が `main.rs` の受け口(`browser_surface::handle_import_actions`)へ
    /// 繋がっているか。**繋がるまで Import の席ごと畳む。**
    ///
    /// 押せば OS の dialog は開き、probe も走るが、`Intent` を受け取る者が居なければ
    /// Document は変わらない — 「効いたように見えてから黙って戻る」は「できない」より
    /// 悪い(2026-08-28 裁定(a)、`TimelineSurface::TRIM_HANDLE_WIRED` と同じ扱い)。
    ///
    /// **`main.rs` に受け口(WIRE-2、報告の NOT_DONE 参照)が着地したら `true` へ。**
    #[allow(dead_code)]
    const IMPORT_WIRED: bool = false;

    /// store の棚を投影する(`TimelineSurface::set_model` と同じ口)。catalog の正本は
    /// Document の台帳で、この widget は投影を持つだけ。
    ///
    /// **絞り込みの状態(rail / 検索語 / tag)は持ち越す** — 取り込みや配置のたびに
    /// 絞りが解けたら、利用者が名指していない物が動くことになる(裁定271)。ただし
    /// 今の一覧に無い tag を指したまま残ると0件の理由が読めなくなるので、
    /// 実在しなくなった tag だけは畳む。
    pub(crate) fn set_catalog(&mut self, cx: &mut Cx, catalog: Vec<BrowserAsset>) {
        // `Recent` の境目は最初の投影で決まる。開いた時に既に居た物は Recent ではない。
        if self.session_floor.is_none() {
            self.session_floor = Some(
                catalog
                    .iter()
                    .map(|asset| asset.id)
                    .max()
                    .map_or(0, |max| max.saturating_add(1)),
            );
        }
        self.catalog = catalog;
        if let Some(tag) = self.tag.clone() {
            if !self.catalog.iter().any(|asset| asset.tags.contains(&tag)) {
                self.tag = None;
            }
        }
        self.view.redraw(cx);
    }

    fn session_floor(&self) -> u64 {
        self.session_floor.unwrap_or(u64::MAX)
    }

    /// catalog に実在する tag を辞書順で1周する環。UI が発明した語彙を持たない。
    fn tag_cycle(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .catalog
            .iter()
            .flat_map(|asset| asset.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    fn advance_tag(&mut self) {
        let cycle = self.tag_cycle();
        if cycle.is_empty() {
            self.tag = None;
            return;
        }
        self.tag = match &self.tag {
            None => Some(cycle[0].clone()),
            Some(current) => match cycle.iter().position(|tag| tag == current) {
                Some(index) if index + 1 < cycle.len() => Some(cycle[index + 1].clone()),
                _ => None,
            },
        };
    }

    fn clearable(&self) -> bool {
        !self.query.trim().is_empty() || self.tag.is_some()
    }

    /// 見えている行の索引。**rail・検索語・tag の3つだけ**から出る。
    fn visible_rows(&self) -> Vec<usize> {
        let needle = self.query.trim().to_lowercase();
        let floor = self.session_floor();
        self.catalog
            .iter()
            .enumerate()
            .filter(|(_, asset)| self.scope.accepts(asset, floor))
            .filter(|(_, asset)| needle.is_empty() || asset.name.to_lowercase().contains(&needle))
            .filter(|(_, asset)| match &self.tag {
                None => true,
                Some(tag) => asset.tags.iter().any(|owned| owned == tag),
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// 見出しと shelf は状態の投影。描く前に合わせるので、宣言側の文言は繋ぎでよい。
    fn project_shelf(&self, cx: &mut Cx, shown: usize) {
        let count = match shown {
            0 => "No items".to_string(),
            1 => "1 item".to_string(),
            n => format!("{n} items"),
        };
        self.view
            .label(cx, ids!(browser_body.catalog.catalog_head_row.result_count))
            .set_text(cx, &count);
        self.view
            .label(cx, ids!(browser_body.catalog.filter_shelf.kind_filter))
            .set_text(cx, &format!("Kind: {}", self.scope.kind_label()));
        let tags = match &self.tag {
            Some(tag) => format!("Tags: {tag}"),
            None => "Tags: —".to_string(),
        };
        self.view
            .label(cx, ids!(browser_body.catalog.filter_shelf.tag_filter))
            .set_text(cx, &tags);
        self.view
            .button(cx, ids!(browser_body.catalog.filter_shelf.clear_filters))
            .set_visible(cx, self.clearable());
        // tag グリフは「一覧に在る tag を1周する」道具。環が空の一覧
        // (= 台帳から来た実データ。tag をまだ誰も付けていない)では、押しても
        // 何も起きない物になる — 押せそうで押せない物を作らない(Q0)
        self.view
            .button(cx, ids!(browser_toolbar.tags))
            .set_visible(cx, !self.tag_cycle().is_empty());
        // 素材の口。store へ届かない間は席ごと畳む(`Self::IMPORT_WIRED` の doc)
        self.view
            .button(cx, ids!(browser_toolbar.import))
            .set_visible(cx, Self::IMPORT_WIRED);
    }

    /// 空一覧の文言。「何も無い」と「絞り込んだ結果ゼロ」を混同しない。
    fn empty_note(&self) -> &'static str {
        if self.clearable() {
            "No media matches this filter."
        } else {
            "No media here yet."
        }
    }
}

impl WidgetNode for BrowserSurface {
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

impl Widget for BrowserSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // 子の action を一度受け取ってから流し直す。`main.rs` の rail 読みが
        // 同じ action を待っているので、握り潰さない(`extend_actions` が命綱)。
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        let mut dirty = false;

        let rail = self.view.radio_button_set(
            cx,
            ids_array!(
                browser_body.rail.all_media,
                browser_body.rail.video,
                browser_body.rail.images,
                browser_body.rail.audio,
                browser_body.rail.project,
                browser_body.rail.recent
            ),
        );
        if let Some(index) = rail.selected(cx, &actions) {
            let next = BrowserScope::from_rail_index(index);
            if next != self.scope {
                self.scope = next;
                dirty = true;
            }
        }

        if let Some(text) = self
            .view
            .text_input(cx, ids!(browser_toolbar.search.search_field))
            .changed(&actions)
        {
            if text != self.query {
                self.query = text;
                dirty = true;
            }
        }

        // tag グリフは一覧に実在する tag を1周する。押した結果は shelf の
        // `Tags:` に出るので、状態の見えない切替にはならない。
        if self
            .view
            .button(cx, ids!(browser_toolbar.tags))
            .clicked(&actions)
        {
            self.advance_tag();
            dirty = true;
        }

        if self
            .view
            .button(cx, ids!(browser_body.catalog.filter_shelf.clear_filters))
            .clicked(&actions)
        {
            self.query.clear();
            self.tag = None;
            self.view
                .text_input(cx, ids!(browser_toolbar.search.search_field))
                .set_text(cx, "");
            dirty = true;
        }

        // 素材の口。**この widget は Document を持たない** — 選ぶ(OS dialog)まで
        // をここで行い、置く判断は意図の action として外へ出す
        // (`TimelineSurface` が編集意図を出すのと同じ形)。
        if self
            .view
            .button(cx, ids!(browser_toolbar.import))
            .clicked(&actions)
        {
            if let Some(path) = pick_media_path() {
                cx.widget_action(self.uid, BrowserSurfaceAction::ImportMedia(path));
            }
        }

        // 棚 → タイムライン。カードの double-click だけが動詞
        // (single click は何も名指していない — mod doc の意図論)。
        //
        // `FlatList` は item の action を `group_widget_actions` で自分の uid の下へ
        // 束ねるので、どのカードが鳴ったかは `items_with_actions` が entry id で返す。
        // uid の台帳をこちらで持たない(持つと描画順と食い違う二重帳簿になる)。
        let grid = self
            .view
            .widget(cx, ids!(browser_body.catalog.card_grid))
            .as_flat_list();
        let mut place: Option<u64> = None;
        for (entry, item) in grid.items_with_actions(&actions) {
            let Some(asset) = asset_of(entry) else {
                continue;
            };
            // **ちょうど 2**。`>= 2` にすると3連打で3回目の down がもう1枚置く
            // (`tap_count` は 1,2,3,… と数え上がる)。
            let double = actions
                .filter_widget_actions(item.widget_uid())
                .any(|action| {
                    matches!(action.cast::<ViewAction>(), ViewAction::FingerDown(fe) if fe.tap_count == 2)
                });
            if double {
                place = Some(asset);
            }
        }

        cx.extend_actions(actions);
        if let Some(asset) = place {
            cx.widget_action(self.uid, BrowserEditAction::PlaceAsset { asset });
        }
        if dirty {
            self.view.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rows = self.visible_rows();
        self.project_shelf(cx, rows.len());
        let note = self.empty_note();

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            let grid = step.as_flat_list();
            let Some(mut list) = grid.borrow_mut() else {
                continue;
            };
            if rows.is_empty() {
                if let Some(item) = list.item(cx, EMPTY_ENTRY, live_id!(CardEmpty)) {
                    item.label(cx, ids!(note)).set_text(cx, note);
                    item.draw_all_unscoped(cx);
                }
                continue;
            }
            for row in rows.iter().copied() {
                let Some(asset) = self.catalog.get(row) else {
                    continue;
                };
                // entry id は `AssetId` に紐づける — 絞り込みや取り込みで並びが
                // 変わっても同じ素材が同じ widget(= 同じ template)を使い続け、
                // double-click の受け口(`items_with_actions`)がそのまま素材を名指せる。
                let entry = entry_of(asset.id);
                let Some(item) = list.item(cx, entry, asset.kind.template()) else {
                    continue;
                };
                item.label(cx, ids!(meta.name)).set_text(cx, &asset.name);
                item.view(cx, ids!(meta.placed_slot.dot))
                    .set_visible(cx, asset.placed);
                item.draw_all_unscoped(cx);
            }
        }

        DrawStep::done()
    }
}

// ============================================================================
// 素材の口 — ファイルを開いて、この comp に置く
//
// 通す道: 選ぶ(`rfd`)→ 調べる(`motolii_media::probe` + `SourceFingerprintV1`)
// → 記帳して置く(`Intent::AdmitAsset` + `AddLayer` + `SetMeta`)→ 見える
// (Timeline のレーンも Stage の絵も Document から導出されるので、置いた時点で出る)。
//
// **判断はこの節に閉じる。** `main.rs` 側が要るのは呼び出し2本だけで、置き方の規則
// (尺・重ね順・記帳)を shell へ書き写さない — 書き写すと面ごとに違う置き方が生まれる
// (`LayerTiming::place` の doc「この規則を shell に書かせない」)。
// ============================================================================

/// dialog が受け取る拡張子。**種別判定の正本ではない** — 台帳の `asset_type` は
/// [`asset_type_for`] が決め、ここは「開ける物だけを見せる」ための一覧。
///
/// 出所は `next/shell/motolii-shell/src/file_dialogs.rs`(裁定176 の File 束)の
/// 3本の一覧。あちらは歴史(`next/`)なので依存としては引けず、値だけ引き写す。
///
/// **音声は v1 に入れない。** `motolii_media::probe` は先頭 video stream を要求する
/// (`probe.rs:163`)ので audio-only は必ず `Err` になり、開けない物を dialog に
/// 並べることになる。音を持ち込む道は別の裁定が要る(報告の EVIDENCE_GAP)。
const IMPORT_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "mkv", "webm", "avi", "m4v", "jpg", "jpeg", "png", "gif", "webp", "bmp",
];

/// 素材の口が運ぶ意図。**値ではなく意図**(どの path を使いたいか)だけを運ぶ —
/// 置き方(尺・重ね順)の計算は Document を読める側([`place_media`])の仕事で、
/// widget 側で先に計算すると同じ規則が2箇所に住む。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum BrowserSurfaceAction {
    #[default]
    None,
    /// 利用者が選んだ1本。v1 は1ファイルずつ(複数選択・フォルダは後)。
    ImportMedia(PathBuf),
}

/// OS の file dialog。**自前のファイルブラウザを作らない**(裁定176、
/// wraps > 移植 > スクラッチ)。
///
/// 同期版を使う: dialog が開いている間 makepad のイベントループは止まる(native
/// dialog は本来モーダル)。iced 二代目は `Task::perform` という執行者を持っていた
/// ので非同期版へ移したが、makepad 側に同じ物が無いところへ channel + 毎フレーム
/// poll を自作すると「繋げるだけ」が機構になる。止まるのは **OS の dialog が前に
/// 居る間だけ**で、利用者から見れば普通の挙動。
fn pick_media_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import Media")
        .add_filter("Media", IMPORT_EXTENSIONS)
        .pick_file()
}

/// 台帳の opaque `asset_type`。拡張子からの粗い推定で、精度はこの切片の非目標
/// (`next/shell/motolii-shell/src/assets.rs::guess_asset_type` と同じ割り切り)。
fn asset_type_for(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v" => format!("video/{ext}"),
        "jpg" | "jpeg" => "image/jpeg".to_owned(),
        "png" | "gif" | "webp" | "bmp" | "svg" => format!("image/{ext}"),
        "wav" | "mp3" | "aac" | "flac" | "ogg" | "m4a" => format!("audio/{ext}"),
        "" => "application/octet-stream".to_owned(),
        other => format!("application/{other}"),
    }
}

/// 表示用の短い名前。path 全体を状態行へ出すと、長い path で他が読めなくなる。
fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// 内容の指紋(ファイル IO)。probe とは独立 — 記帳は「読めるか」だけを見る。
fn fingerprint(path: &Path) -> Result<SourceFingerprintV1, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    SourceFingerprintV1::from_reader(file).map_err(|error| error.to_string())
}

/// 選んだ1本を**台帳へ記帳し、そのまま comp へ置く**。
///
/// 意図論(裁定271): 利用者が file を開く時に求めているのは「**この素材を使いたい**」
/// であって、置き場所やレイヤー順を名指してはいない。だから開いた時点で使える状態
/// — playhead の位置に、素材の実尺で、一番手前に — にする。「取り込んだので配置して
/// ください」で止めるのは、意図を半分しか叶えていない。
///
/// **名指していない物は変えない**: playhead は動かさない。既存レイヤーの順序も
/// 触らない(新しい layer が最大 order + 1 を取るだけ)。
///
/// 置き方の規則は全部 store の物を呼ぶだけ:
/// - 尺は [`LayerTiming::place`](= `min(素材の尺, comp の残り)`)
/// - 静止画は「尺が分からない物」として `None` を渡す。probe は静止画へ
///   `nb_frames = Some(1)` を返すが、それは demuxer が置いた作り値であって絵の尺では
///   ない(`probe.rs:376`)。1 フレームの layer を置くのは、probe の実装詳細を
///   利用者の作業へ漏らすことになる
/// - 記帳の重複統合は `AssetTable::admit`(同じ content_hash は台帳を増やさない)
///
/// 1本 = **1 undo**(`apply_all` 1回)。記帳と配置を別の undo にしない。
pub(crate) fn place_media(
    doc: &mut Document,
    session: &mut Session,
    path: &Path,
) -> Result<String, String> {
    let label = file_label(path);
    let store = doc.view();
    let composition = store
        .composition()
        .map_err(|error| format!("IMPORT FAILED  ·  {label}  ·  {error}"))?
        .ok_or_else(|| format!("IMPORT FAILED  ·  {label}  ·  no composition yet"))?;
    let layer = LayerId(store.next_layer_id());
    // 一番手前 = 既存の最大 order の1つ上(Timeline は order 降順で並べる)。
    let order = store
        .layers()
        .into_iter()
        .filter_map(|id| store.meta(id).ok().flatten().map(|meta| meta.order))
        .max()
        .map(|max| max.saturating_add(1))
        .unwrap_or(0);
    drop(store);

    let asset_type = asset_type_for(path);
    let info = motolii_media::probe(path)
        .map_err(|error| format!("IMPORT FAILED  ·  {label}  ·  {error}"))?;
    let still = AssetKind::from_asset_type(&asset_type) == AssetKind::Image;
    let source_frames = if still { None } else { info.nb_frames };
    let timing = LayerTiming::place(session.playhead, source_frames, composition.duration_frames);

    let mut intents = Vec::new();
    // 記帳は「指紋が読めたか」だけを見る。読めなくても配置は続ける
    // (bin-first: 取り込みと配置は別の判断 — 裁定162)。
    let content_hash = match fingerprint(path) {
        Ok(source) => {
            let mut draft = AssetDraft::from_probed_source(asset_type, &source, path, None);
            draft.duration = info.duration;
            intents.push(Intent::AdmitAsset { draft });
            Some(source.content_hash())
        }
        Err(_) => None,
    };
    intents.push(Intent::AddLayer(layer));
    // 新規配置は `SetMeta` 1本。`SetSource`/`SetOrder`/`SetTiming` は**既存の meta を
    // 読んで1フィールドだけ書き換える口**で、meta の無い layer には使えない
    // (`document/apply.rs:151` が理由つきで拒む)。
    intents.push(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path: path.to_string_lossy().into_owned(),
                fingerprint: content_hash,
            },
            order,
            timing,
        },
    });

    doc.apply_all(intents)
        .map_err(|error| format!("IMPORT FAILED  ·  {label}  ·  {error}"))?;

    // 置いた物が選ばれている(AE と同じ)。Inspector はこの選択を読む。
    session.selection = Some(layer);
    session.selected_layers = vec![layer];

    Ok(format!(
        "IMPORT  ·  {label}  ·  {}×{}  ·  {} frames from {}",
        info.width, info.height, timing.duration, timing.start
    ))
}

/// 台帳(`Composition:assets`)から一覧を作る。**一覧の正本は台帳**(裁定162)で、
/// [`demo_catalog`] は誰も台帳を渡してこない間の繋ぎ。
///
/// `placed`(カード右下の ●)は「この素材を指す layer が居るか」の実測 — 台帳と
/// layer を突き合わせて毎回導出する(別に持つと必ずずれる)。
fn catalog_from_document(doc: &Document) -> Vec<BrowserAsset> {
    let store = doc.view();
    let placed_paths: Vec<String> = store
        .layers()
        .into_iter()
        .filter_map(|id| store.meta(id).ok().flatten())
        .filter_map(|meta| match meta.source {
            LayerSource::Media { path, .. } => Some(path),
            _ => None,
        })
        .collect();
    store
        .assets()
        .unwrap_or_default()
        .into_iter()
        .map(|asset| {
            let placed = asset
                .path_absolute
                .as_ref()
                .is_some_and(|path| placed_paths.iter().any(|used| used == path));
            // NOTE(統合 2026-08-28): 一覧の正本 builder は `main.rs::browser_catalog`
            // (指紋一致で ● を導く)。ここは import 直後の再投影だけが使う休眠経路で、
            // WIRE-2 が着地したら正本へ畳むこと。
            BrowserAsset {
                id: asset.id.get(),
                name: asset.file_name.clone().unwrap_or_else(|| asset.name.clone()),
                kind: AssetKind::from_asset_type(&asset.asset_type),
                placed,
                // 台帳に tag 欄が無い(`motolii_store::Asset`)。front で発明すると
                // 保存で消えるので、空のまま渡す
                tags: Vec::new(),
            }
        })
        .collect()
}

/// 一覧を Document の台帳へ合わせる。`main.rs` は起動時・`Event::LiveEdit`・
/// 取り込み後にこれを呼ぶ(live edit は widget を宣言状態へ戻すので、投影し直すのは
/// 呼び手の責任 — `apply_browser_selection` と同じ形)。
#[allow(dead_code)]
pub(crate) fn install_catalog(cx: &mut Cx, browser: &WidgetRef, doc: &Document) {
    let catalog = catalog_from_document(doc);
    let surface = browser.child_by_path(ids!(browser_surface));
    if let Some(mut surface) = surface.borrow_mut::<BrowserSurface>() {
        surface.set_catalog(cx, catalog);
    };
}

/// 素材の口の受け口。**`main.rs` の `handle_actions` から1本だけ呼ぶ。**
///
/// `browser` は Dock の `browser` タブの中身(`main.rs::browser`)。戻り値は状態行の
/// 文言で、`None` は「この action 群に取り込みは無かった」。
///
/// ```ignore
/// // main.rs: App::handle_actions の中
/// let browser = self.browser(cx);
/// if let Some(backend) = self.backend.as_mut() {
///     if let Some(status) = browser_surface::handle_import_actions(
///         cx, &browser, actions, &mut backend.doc, &mut backend.session,
///     ) {
///         backend.frame = None;
///         self.install_timeline_model(cx);
///         self.request_stage_frame(cx);
///         self.set_status(cx, &status);
///     }
/// }
/// ```
#[allow(dead_code)]
pub(crate) fn handle_import_actions(
    cx: &mut Cx,
    browser: &WidgetRef,
    actions: &Actions,
    doc: &mut Document,
    session: &mut Session,
) -> Option<String> {
    let surface = browser.child_by_path(ids!(browser_surface));
    if surface.is_empty() {
        return None;
    }
    let uid = surface.widget_uid();
    let mut status = None;
    for action in actions.filter_widget_actions_cast::<BrowserSurfaceAction>(uid) {
        let BrowserSurfaceAction::ImportMedia(path) = action else {
            continue;
        };
        // 失敗も状態行へ出す。**黙って落とさない** — 落ちた物が何も言わずに消えると、
        // 利用者は「開いたのに何も起きない」としか分からない。
        status = Some(place_media(doc, session, &path).unwrap_or_else(|reason| reason));
    }
    if status.is_some() {
        install_catalog(cx, browser, doc);
    }
    status
}
