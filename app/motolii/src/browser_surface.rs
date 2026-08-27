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
//! - **カタログは front ローカル**: 出所は本来 `motolii-store` の `Composition:assets`
//!   (`AssetTable`) だが、その読み手（`BackendBridge`）を持つのは `App`（`main.rs`）で、
//!   本レーンの write-set の外。store の型を発明せず、front の投影型
//!   [`BrowserAsset`] を置いて既定値に旧8件を入れてある。結線は WIRE 1本の仕事
//! - **フィルタは1つの状態から導く**: rail = 置き場所（radio group が正本、`App` と
//!   同じ actions を読む）/ 検索語と tag = この widget が持つ。件数見出し・
//!   `Kind:` / `Tags:` の文言・`Clear` の出没は全部そこからの投影で、別に持たない
use makepad_widgets::*;

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
    // ● = 配置済み(bin-first の可視化)。drag=配置は未配線。
    // ● は「席」と「点」を分ける: 席は常に 5×5 を占め、点だけ出没する。
    // 色を透明へ差し替える旧手だと、値が uniform 側へ落ちた瞬間に全カードが道連れになる
    let AssetCard = View{
        width: 76
        height: 58
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.area
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
                // drag=配置 / double-click は未決 — 動詞は配線しない(Q0: 押せそうで押せない物を作らない)
                card_grid := FlatList{width: Fill height: Fill flow: Right{wrap: true} padding: mod.tokens.space.s3 spacing: mod.tokens.space.s3 wrap_spacing: mod.tokens.space.s3
                    CardVideo := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/video.svg")}
                    CardImage := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/image.svg")}
                    CardAudio := AssetCard{thumb.glyph.draw_icon.svg: crate_resource("self://resources/icons/audio.svg")}
                    CardEmpty := AssetEmpty{}
                }
            }
        }
    }
}

/// 素材の種。カードの絵は種で決まるので、種ごとに template を1枚持つ
/// (svg を実行時に差し替えると、どのカードがどの svg を持っているかが
/// 描画順に依存する)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AssetKind {
    Video,
    Image,
    Audio,
}

impl AssetKind {
    fn template(self) -> LiveId {
        match self {
            AssetKind::Video => live_id!(CardVideo),
            AssetKind::Image => live_id!(CardImage),
            AssetKind::Audio => live_id!(CardAudio),
        }
    }
}

/// 一覧の1行。**front の投影型であって保存形ではない。**
///
/// 正本は `motolii-store` の `Composition:assets`(`AssetTable` / `Asset`)。ただし
/// その読み手 `BackendBridge` を持つのは `App`(`main.rs`)で、本レーンの write-set の
/// 外にある。store 側へ新しい型を足す理由も無い(`Asset` に必要な欄は既に在る)ので、
/// ここは「一覧が読む形」だけを置き、既定値に旧8件を入れてある。
#[derive(Clone, Debug)]
pub(crate) struct BrowserAsset {
    pub(crate) name: String,
    pub(crate) kind: AssetKind,
    /// タイムラインへ既に置いてあるか(bin-first の可視化 = カード右下の ●)。
    pub(crate) placed: bool,
    /// この project へ取り込み済みか(rail `Project`)。
    pub(crate) in_project: bool,
    /// 直近に触った物か(rail `Recent`)。
    pub(crate) recent: bool,
    pub(crate) tags: Vec<String>,
}

impl BrowserAsset {
    fn new(
        name: &str,
        kind: AssetKind,
        placed: bool,
        in_project: bool,
        recent: bool,
        tags: &[&str],
    ) -> Self {
        Self {
            name: name.to_string(),
            kind,
            placed,
            in_project,
            recent,
            tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        }
    }
}

/// 旧「直書き8枚」と同じ中身。**手書きの列ではなく行として**持つ。
/// store 結線が来たらこの関数の呼び出しが差し替わるだけで、以下は動かない。
fn demo_catalog() -> Vec<BrowserAsset> {
    vec![
        BrowserAsset::new("intro_take2.mp4", AssetKind::Video, true, true, true, &["footage"]),
        BrowserAsset::new("logo.png", AssetKind::Image, true, true, false, &["brand"]),
        BrowserAsset::new("bass_drop.wav", AssetKind::Audio, false, true, true, &["sfx"]),
        BrowserAsset::new("cutaway_b.mp4", AssetKind::Video, false, true, false, &["footage"]),
        BrowserAsset::new("texture_grain.png", AssetKind::Image, true, false, false, &["texture"]),
        BrowserAsset::new("vo_line3.wav", AssetKind::Audio, false, true, true, &["voice"]),
        BrowserAsset::new("ending_loop.mp4", AssetKind::Video, false, false, false, &["footage"]),
        BrowserAsset::new("title_bg.png", AssetKind::Image, false, false, true, &["brand"]),
    ]
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

    fn accepts(self, asset: &BrowserAsset) -> bool {
        match self {
            Self::AllMedia => true,
            Self::Video => asset.kind == AssetKind::Video,
            Self::Images => asset.kind == AssetKind::Image,
            Self::Audio => asset.kind == AssetKind::Audio,
            Self::Project => asset.in_project,
            Self::Recent => asset.recent,
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

/// 空一覧の面に使う entry id。カードは catalog の索引 +1 を使うので 0 は空く。
const EMPTY_ENTRY: LiveId = LiveId(0);

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct BrowserSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// 一覧の中身。宣言側にカードは1枚も無い — 全部ここから出る。
    #[rust(demo_catalog())]
    catalog: Vec<BrowserAsset>,
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
        self.catalog
            .iter()
            .enumerate()
            .filter(|(_, asset)| self.scope.accepts(asset))
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

        cx.extend_actions(actions);
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
                // entry id は catalog の索引に紐づける — 絞り込みで並びが変わっても
                // 同じ素材が同じ widget(= 同じ template)を使い続ける。
                let entry = LiveId(row as u64 + 1);
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
