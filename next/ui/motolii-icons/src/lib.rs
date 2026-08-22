//! wraps: Material Symbols(Apache 2.0)の SVG を iced svg widget へ
//!
//! 裁定186(2026-08-22「再生ボタンなどかなり無理やり作っていて痛ましかった。
//! 今後は SVG アイコンを有効活用」)+裁定187(icon-first — 文字を読まなくても
//! 意味が分かる UI)の基盤 crate。
//!
//! ## 何を包むか
//! - Material Symbols(outlined・24px 枠)の SVG を `assets/` へ vendoring
//!   (出典 rev・Apache 2.0 LICENSE・NOTICE は `assets/` に同梱)。
//! - `include_bytes!` で全アイコンをバイナリへ embed — 実行時のファイル I/O も
//!   パス解決も無い(tokens の debug watch とは別物: アイコンの絵は意味の一部
//!   なので hot reload の対象にしない)。
//!
//! ## 色(tint)の仕組み
//! iced の svg renderer は `svg::Style { color: Some(c) }` の時、rasterize 後の
//! RGB を `c` へ置換して alpha だけ残す(fork `wgpu/src/image/vector.rs` 実測)。
//! つまり SVG 側の fill が何であれ呼び手の色で塗られる — vendoring した SVG は
//! fill 属性を持たない(暗黙 black)ので `currentColor` 化の前処理は不要だった。
//! 色は**必ず呼び手が tokens から渡す**(この crate は色の意見を持たない)。
//!
//! ## 寸法の作法(裁定178: Rust 直書き禁止との整合)
//! `icon()` の `size` は呼び手が tokens 由来の値を渡す。文字グリフからの置換で
//! 視覚同等を保ちたい時は [`frame_px_for_glyph_px`](グリフ字寸→アイコン枠寸の
//! 純関数、Material の公式 live area 比 20/24 由来)を使う — 中間比の発明では
//! なく上流(Material Design guideline)の定数の転写。
//!
//! ## tooltip との標準ペア(裁定187)
//! アイコンは tooltip と対で使うのが標準(実装は将来レーン)。呼び出し例:
//! `tooltip(icon(Icon::PlayArrow, size, ink), "Play", tooltip::Position::Bottom)`

use iced::widget::svg::{self, Svg};
use iced::{Color, Element, Length};

/// Icon enum・embed・名前表を1箇所で宣言するマクロ(variant を足す時に
/// `ALL`/`svg_bytes`/`name` の3表がずれない構造 — 台帳の一本化)。
macro_rules! declare_icons {
    ($( $variant:ident => $file:literal, )+) => {
        /// vendoring 済みアイコンの全語彙(Material Symbols 名は [`Icon::name`])。
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Icon {
            $( $variant, )+
        }

        impl Icon {
            /// 全 variant(テストの全数検査と、将来の一覧 UI 用)。
            pub const ALL: &'static [Icon] = &[ $( Icon::$variant, )+ ];

            /// embed 済み SVG バイト列(Material Symbols outlined 24px 枠)。
            pub const fn svg_bytes(self) -> &'static [u8] {
                match self {
                    $( Icon::$variant => include_bytes!(concat!("../assets/", $file, ".svg")), )+
                }
            }

            /// Material Symbols での名前(= `assets/` のファイル名)。
            pub const fn name(self) -> &'static str {
                match self {
                    $( Icon::$variant => $file, )+
                }
            }
        }
    };
}

declare_icons! {
    // -- transport(裁定186 の当座の置換対象)------------------------------
    SkipPrevious => "skip_previous",
    SkipNext => "skip_next",
    PlayArrow => "play_arrow",
    Pause => "pause",
    // 1コマ移動: chevron(山括弧)より arrow_left/right(小三角)が旧グリフ
    // ◂/▸(fold 三角と同族の小三角)の直訳 — chevron 系は「ページ送り」等の
    // 別動詞のために在庫として持つ。
    ArrowLeft => "arrow_left",
    ArrowRight => "arrow_right",
    ChevronLeft => "chevron_left",
    ChevronRight => "chevron_right",
    // ループ再生トグル(B21 transport 帯)の Repeat は第2弾セクション側で宣言。
    // 第5波 shell 結線と先読み第2弾(IC3)が同じ rev から同一バイトを vendoring し、
    // merge がテキスト上は綺麗に通って E0428 になった — 意味衝突を型が捕捉した実例。
    // -- 近い将来分(裁定186 発注書)---------------------------------------
    Search => "search",
    Close => "close",
    Settings => "settings",
    Folder => "folder",
    Add => "add",
    // -- 常用語彙(裁定187 icon-first — 今後のレーンの在庫)-----------------
    Visibility => "visibility",
    VisibilityOff => "visibility_off",
    Lock => "lock",
    LockOpen => "lock_open",
    ContentCopy => "content_copy",
    ContentPaste => "content_paste",
    Delete => "delete",
    Undo => "undo",
    Redo => "redo",
    DragIndicator => "drag_indicator",
    Tune => "tune",
    FilterList => "filter_list",
    GridView => "grid_view",
    ViewList => "view_list",
    VolumeUp => "volume_up",
    VolumeOff => "volume_off",
    Fullscreen => "fullscreen",
    ZoomIn => "zoom_in",
    ZoomOut => "zoom_out",
    MoreVert => "more_vert",
    Check => "check",
    Warning => "warning",
    Error => "error",
    Info => "info",
    // -- 第2弾(裁定186/187 — 走行中レーンの予想需要+次波の在庫)-----------
    // メディア/アセット種別
    Movie => "movie",
    AudioFile => "audio_file",
    Image => "image",
    TextFields => "text_fields",
    // マーキング/分類
    Flag => "flag",
    Bookmark => "bookmark",
    Label => "label",
    Palette => "palette",
    // 変形/レイアウト
    Crop => "crop",
    Straighten => "straighten",
    GridOn => "grid_on",
    CenterFocusStrong => "center_focus_strong",
    Layers => "layers",
    LayersClear => "layers_clear",
    // 窓/パネル
    OpenInNew => "open_in_new",
    DockToRight => "dock_to_right",
    PictureInPicture => "picture_in_picture",
    // ファイル I/O
    Save => "save",
    FileDownload => "file_download",
    FileUpload => "file_upload",
    // transport 続き
    PlayCircle => "play_circle",
    Stop => "stop",
    FastForward => "fast_forward",
    FastRewind => "fast_rewind",
    Repeat => "repeat",
    RepeatOne => "repeat_one",
    Shuffle => "shuffle",
    // -- 第3弾(裁定189 — 走行中レーン ST3 の要求+波の需要先回り)-----------
    // 映像品質/撮影
    Videocam => "videocam",
    Hd => "hd",
    Sd => "sd",
    // 色/合成
    Opacity => "opacity",
    Gradient => "gradient",
    Colorize => "colorize",
    // 枠/整形
    BorderAll => "border_all",
    CropFree => "crop_free",
    FitScreen => "fit_screen",
    ZoomOutMap => "zoom_out_map",
    CenterFocusWeak => "center_focus_weak",
    // 整列/文字組
    VerticalAlignCenter => "vertical_align_center",
    FormatAlignLeft => "format_align_left",
    FormatAlignCenter => "format_align_center",
    FormatAlignRight => "format_align_right",
    FormatBold => "format_bold",
    FormatItalic => "format_italic",
    LineWeight => "line_weight",
    TextRotationNone => "text_rotation_none",
    // 時間/履歴
    Timer => "timer",
    Schedule => "schedule",
    History => "history",
    // "restore" は Material Symbols に単独名が無いため settings_backup_restore
    // (循環矢印=restore の定番グリフ)で代替。対応表は発注 RETURN 参照。
    Restore => "settings_backup_restore",
    Backup => "backup",
    CloudDone => "cloud_done",
    // 結線/復帰
    Link => "link",
    LinkOff => "link_off",
    LockReset => "lock_reset",
    Flip => "flip",
    RotateLeft => "rotate_left",
    RotateRight => "rotate_right",
    // tune_vertical は Material Symbols に存在しない(404実測)。代替指定の
    // "tune" は既存 Icon::Tune と重複するため追加せずスキップ(対応表参照)。
}

impl Icon {
    /// iced の svg [`Handle`](svg::Handle)。id は内容 hash(fork
    /// `core/src/svg.rs` 実測)なので毎フレーム呼んでも rasterize cache は
    /// 1アイコン1エントリに畳まれる。
    pub fn handle(self) -> svg::Handle {
        svg::Handle::from_memory(self.svg_bytes())
    }
}

/// アイコンの widget(公開 API)。`size` は正方形の枠寸(px)、`color` は
/// tint(呼び手が tokens から渡す — module doc「色」参照)。
pub fn icon<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    Svg::new(icon.handle())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme, _status| svg::Style { color: Some(color) })
        .into()
}

/// 文字グリフ(字寸 `glyph_px` の text)を置換する時のアイコン枠寸。
///
/// Material Symbols は 24px 枠に 20px の live area(公式 guideline の余白 2px
/// ×両側)を持つ — 枠 = 描画部 × 24/20。旧 text グリフの字寸を「描画部の寸」
/// と読み、視覚同等の枠寸へ写す。中間比の発明ではなく上流定数の転写
/// (`lane_bar::glyph_text_size_px` の mock 比転写と同型)。
pub fn frame_px_for_glyph_px(glyph_px: f32) -> f32 {
    glyph_px * (24.0 / 20.0)
}
