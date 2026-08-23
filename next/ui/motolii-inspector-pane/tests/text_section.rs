//! B46 第1切片(裁定184): Inspector TEXT section — テキストレイヤー選択時
//! **のみ** section が現れ、Font/Size/Line Height/Tracking/Justify が
//! `Intent::SetTextDocument`(丸ごと差し替え、`SetMasks` と同じ形)経由で
//! 書けること。`TextDocument::justify` は裁定92により `KeyframeTrack` に
//! 乗らない(v1)ので Key 列は対象外のまま。**`Size`/`LineHeight`/`Tracking`
//! は D-1(track 書き口)+ E-3(2026-08-23、見た目の3状態結線)で
//! Position/Scale と同じ Key oracle が乗る** — 下の
//! `size_key_state_differs_once_a_keyframe_is_placed` がその検収点。
//!
//! view の存在照合は `mask_section.rs` と同じ iced_test 手口
//! (`Target::Text` 列挙)。

use iced_test::selector::{Candidate, Target};

use motolii_core::Fps;
use motolii_inspector_pane::{
    commit_text_field, commit_text_font_pick, cycle_text_justify, project, reset_text_line_height,
    reset_text_tracking, toggle_text_style_key, view, KeyCellState, Message, TextField,
    TextFieldDraft, TextStyleField,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, TextDocument,
    TextJustify, TextStyleId,
};
use motolii_tokens_rs::{Colors, Dimensions};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`source` を呼び手が選べる — text/非text
/// 両方のテストで使い回す、`mask_section.rs::doc_with_layer` の派生形)。
fn doc_with_layer(source: LayerSource) -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source,
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置けるはず");
    (doc, layer)
}

fn text_layer() -> (Document, LayerId) {
    doc_with_layer(LayerSource::Text)
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

#[test]
fn multi_text_projection_reports_the_selection_and_text_counts() {
    let (mut doc, first) = text_layer();
    let second = LayerId(2);
    doc.apply_all([
        Intent::AddLayer(second),
        Intent::SetMeta {
            layer: second,
            meta: LayerMeta {
                source: LayerSource::Text,
                order: 1,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("2枚目のText layerを置けるはず");
    let session = Session {
        selection: None,
        selected_layers: vec![first, second],
        ..Session::default()
    };

    let projection = project(&doc.view(), &session)
        .expect("複数選択の投影を組めるはず")
        .expect("選択中なのでSomeのはず");
    assert_eq!(projection.selection_count, 2);
    assert_eq!(projection.text_layer_count, 2);
    assert_eq!(projection.kind, "text");
    assert!(projection.text.is_some(), "複数Text選択にはTEXT entryが必要");
}

/// `mask_section.rs::collect_targets` と同じ「`find` を尽きるまで繰り返す」手口。
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
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

fn has_text(targets: &[Target], content: &str) -> bool {
    targets
        .iter()
        .any(|t| matches!(t, Target::Text { content: c, .. } if c == content))
}

fn text_document_of(doc: &Document, layer: LayerId) -> Option<TextDocument> {
    doc.view()
        .text_document(layer)
        .expect("text document を読めるはず")
}

// ---------------------------------------------------------------------------
// 投影: テキストレイヤーでのみ section の中身が生える(Q0)
// ---------------------------------------------------------------------------

/// 非テキストレイヤー(solid)の投影は `text` が `None` — view は TEXT section
/// を出さない(`mask_section.rs` の「mask 無し layer」テストと同型)。
#[test]
fn a_non_text_layer_projects_no_text_section_and_shows_no_text_header() {
    let (doc, layer) = doc_with_layer(LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    });
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert!(
        projection.text.is_none(),
        "非テキストレイヤーに TEXT 投影が生えている"
    );

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(
        !has_text(&targets, "TEXT"),
        "非テキストレイヤーに TEXT section header が出ている"
    );
}

/// text_document が未着手のテキストレイヤーは、[`motolii_inspector_pane::
/// default_text_document`]/[`motolii_inspector_pane::default_text_style`] の
/// 既定値を**表示専用**で投影する(store には何も書かない)。
#[test]
fn a_text_layer_without_a_document_projects_display_only_defaults() {
    let (doc, layer) = text_layer();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let text_projection = projection
        .text
        .as_ref()
        .expect("テキストレイヤーは Some のはず");
    assert_eq!(text_projection.content, "", "既定 content は空文字列のはず");
    // 2026-08-22 追い発注「フォントが選べる・選ばなくても落ちない」——
    // 以前はここが空文字列だった(`FontRef::default()`)が、それこそが
    // `lyric_text_layer_drive.rs` FINDING の発生源(文字を打った瞬間に空 path
    // でのフォント読込へ進み `render_frame` 全体が `Err` になる)だった。
    // 既定フォントは必ず解決できる family/path を持つ(`default_font_ref`)。
    assert!(
        !text_projection.font_family.is_empty(),
        "既定フォントの family が空(フォント未解決 — FINDING の再発)"
    );
    assert!(
        !text_projection.style.font.path.is_empty(),
        "既定フォントの path が空(FINDING の再発 — 文字を打つと render_frame が Err になる)"
    );
    assert!(
        std::path::Path::new(&text_projection.style.font.path).is_file(),
        "既定フォントの path が実在しない: {}",
        text_projection.style.font.path
    );
    assert_eq!(text_projection.size, 100.0, "既定サイズがずれている");
    assert_eq!(text_projection.line_height, None, "既定 Line Height は Auto のはず");
    assert_eq!(text_projection.tracking, 0.0, "既定 Tracking は0のはず");
    assert_eq!(text_projection.justify, TextJustify::Left, "既定 Justify は Left のはず");

    // 表示専用 — 投影を組んだだけでは Document に何も書かれていない。
    assert!(
        text_document_of(&doc, layer).is_none(),
        "投影を読んだだけで text_document が実体化している(表示専用の約束違反)"
    );

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(has_text(&targets, "TEXT"), "TEXT section header が出ない");
    assert!(has_text(&targets, "Content"), "Content 行が出ない");
    assert!(has_text(&targets, "Font"), "Font 行が出ない");
    assert!(has_text(&targets, "Size"), "Size 行が出ない");
    assert!(has_text(&targets, "Line Height"), "Line Height 行が出ない");
    assert!(has_text(&targets, "Tracking"), "Tracking 行が出ない");
    assert!(has_text(&targets, "Justify"), "Justify 行が出ない");
    assert!(has_text(&targets, "Auto"), "Line Height の Auto 表示が出ない");
    assert!(has_text(&targets, "Left"), "Justify の Left 表示が出ない");
    assert!(has_text(&targets, "Fill Color"), "Fill Color 行が出ない(色エディタ未結線)");
    assert!(has_text(&targets, "Stroke Color"), "Stroke Color 行が出ない(色エディタ未結線)");
}

// ---------------------------------------------------------------------------
// 編集: text_input 系フィールドは下書き→Enter で `Intent::SetTextDocument` 1回
// ---------------------------------------------------------------------------

/// **本命(歌詞動画/MV ペルソナ、2026-08-22 発注「歌詞が入れられる道を通す」)**:
/// Content の Enter は `TextDocument::content` を1回で書く(1 gesture = 1
/// undo)。日本語の歌詞をそのまま通す — `docs/reviews/2026-08-22-persona-lyric-mv.md`
/// が指摘した致命的欠落(本文の入力欄が無い)の直接の検収。
#[test]
fn committing_a_content_draft_writes_the_text_document_content() {
    let (mut doc, layer) = text_layer();
    let field = TextField::Content;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "サビ歌詞がここに出る".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field)
        .expect("content の確定は成功するはず");
    assert!(draft.is_none(), "確定後も下書きが残っている");

    let document = text_document_of(&doc, layer).expect("確定後は document が有るはず");
    assert_eq!(
        document.content.eval(motolii_core::RationalTime::ZERO),
        "サビ歌詞がここに出る",
        "content が書けていない"
    );

    doc.undo();
    assert!(
        text_document_of(&doc, layer).is_none(),
        "1 undo で書く前(未着手)へ戻らない(1操作=1 undo 違反)"
    );
}

/// Content の投影(`text_projection.content`)は書いた内容をそのまま表示する
/// (プレイヘッド位置に関わらず — v1 は静止フィールド、`text.rs`
/// `text_document_content` doc 参照)。
#[test]
fn committing_content_updates_the_projection() {
    let (mut doc, layer) = text_layer();
    let field = TextField::Content;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "Lyric line one".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field)
        .expect("content の確定は成功するはず");

    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert_eq!(
        projection.text.expect("テキストレイヤーは Some のはず").content,
        "Lyric line one"
    );
}

/// Font Family の Enter は `styles[0].font.family` だけを書き換えた
/// `TextDocument` を1回で書く(1 gesture = 1 undo)。
///
/// **"Helvetica" は使わない**(2026-08-22 追い発注以降の既定値の副作用):
/// 既定フォント([`motolii_inspector_pane::default_text_style`])が実機の
/// システムフォントから解決されるようになったため、"Helvetica" が既定と
/// 一致する環境では「既定と同値」判定([`apply_text_document_edit`] の
/// 決定7)で Intent が出ず、この試験が意図する「実際に書き換わる」ことの
/// 検収にならない。既定と衝突しない架空の family 名を使うことで、この試験を
/// 実行環境(どの実フォントが入っているか)から独立させる。
#[test]
fn committing_a_font_family_draft_writes_the_text_document_style() {
    let (mut doc, layer) = text_layer();
    let field = TextField::FontFamily;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "Definitely Not The Default Font".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field)
        .expect("font family の確定は成功するはず");

    let document = text_document_of(&doc, layer).expect("確定後は document が有るはず");
    assert_eq!(document.styles[0].font.family, "Definitely Not The Default Font");

    doc.undo();
    assert!(
        text_document_of(&doc, layer).is_none(),
        "1 undo で書く前(未着手)へ戻らない(1操作=1 undo 違反)"
    );
}

// ---------------------------------------------------------------------------
// フォント pick_list(2026-08-22 追い発注「フォントが選べる・選ばなくても
// 落ちない」)— `commit_text_font_pick` は family と path を**同時に**書く。
// ---------------------------------------------------------------------------

/// 実在するシステムフォントを選ぶと、family と path が**両方**1回の
/// `Intent::SetTextDocument` で書かれる(手打ち欄との違い — FINDING の
/// 直接対処)。
#[test]
fn picking_a_known_system_font_writes_family_and_path_together() {
    let (mut doc, layer) = text_layer();
    let entry = motolii_font_catalog::system_fonts()
        .first()
        .expect("このマシンにシステムフォントが1つも無い");
    let family = entry.family.clone();

    commit_text_font_pick(&mut doc, Some(layer), &family)
        .expect("既知の family の選択は成功するはず");

    let document = text_document_of(&doc, layer).expect("確定後は document が有るはず");
    assert_eq!(document.styles[0].font.family, family, "family が書けていない");
    assert_eq!(document.styles[0].font.path, entry.path, "path が書けていない");
    assert!(
        std::path::Path::new(&document.styles[0].font.path).is_file(),
        "書かれた path が実在しない"
    );

    doc.undo();
    assert!(
        text_document_of(&doc, layer).is_none(),
        "1 undo で書く前(未着手)へ戻らない(1操作=1 undo 違反)"
    );
}

/// カタログに存在しない family を選ぼうとしても(呼び出し側のバグで options
/// とカタログがずれた形を想定)`Err` を返し、Document には一切触らない —
/// 「壊れた path を書けない」頑健化の直接の検収。
#[test]
fn picking_an_unknown_font_family_fails_without_writing_a_broken_path() {
    let (mut doc, layer) = text_layer();

    let result = commit_text_font_pick(
        &mut doc,
        Some(layer),
        "Definitely Not A Real Font Family 12345",
    );
    assert!(result.is_err(), "存在しない family の選択は Err のはず");
    assert!(
        text_document_of(&doc, layer).is_none(),
        "失敗した選択で Document が書かれている(壊れた path 混入)"
    );
}

/// 選択なしは黙って no-op(他の TEXT section 腕と同じ安全側の柵)。
#[test]
fn picking_a_font_without_a_selection_is_a_silent_no_op() {
    let mut doc = Document::new();
    let entry = motolii_font_catalog::system_fonts()
        .first()
        .expect("このマシンにシステムフォントが1つも無い");
    commit_text_font_pick(&mut doc, None, &entry.family).expect("選択なしは no-op のはず");
}

/// Size の Enter は `styles[0].size` を書き換える。数値として読めない入力は
/// `Err` の理由文(M13)。
#[test]
fn committing_a_size_draft_writes_the_text_document_style_size() {
    let (mut doc, layer) = text_layer();
    let field = TextField::Size;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "48".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field).expect("size の確定は成功するはず");

    let document = text_document_of(&doc, layer).expect("確定後は document が有るはず");
    assert_eq!(document.styles[0].size, 48.0);
}

/// 数値として読めない Size の下書きは `Err` を返し、Document を書かない
/// (`commit_inspector_field` の「数値として読めない」と同じ形)。
#[test]
fn committing_an_unparsable_size_draft_fails_without_writing() {
    let (mut doc, layer) = text_layer();
    let field = TextField::Size;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "not-a-number".to_owned(),
    });
    let result = commit_text_field(&mut doc, &mut draft, Some(layer), field);
    assert!(result.is_err(), "数値として読めない入力は Err のはず");
    assert!(
        text_document_of(&doc, layer).is_none(),
        "失敗した確定で Document が書かれている"
    );
}

/// Line Height の Enter は `Some(値)` を書く。Auto ボタン
/// ([`reset_text_line_height`])は `None` へ戻す — 既に Auto(未着手)なら
/// no-op(決定7 と同じ判断、`apply_text_document_edit` の同値判定)。
#[test]
fn committing_line_height_then_resetting_returns_to_auto() {
    let (mut doc, layer) = text_layer();
    let field = TextField::LineHeight;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "1.2".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field)
        .expect("line height の確定は成功するはず");
    assert_eq!(
        text_document_of(&doc, layer).unwrap().styles[0].line_height,
        Some(1.2)
    );

    reset_text_line_height(&mut doc, Some(layer)).expect("Auto へのリセットは成功するはず");
    assert_eq!(
        text_document_of(&doc, layer).unwrap().styles[0].line_height,
        None,
        "Auto ボタンが line_height を None へ戻していない"
    );
}

/// 既に Auto(text_document 未着手)の layer へ Auto ボタンを押しても
/// Document は書かれない(決定7: 同値は Undo を積まない)。
#[test]
fn resetting_line_height_when_already_auto_is_a_silent_no_op() {
    let (mut doc, layer) = text_layer();
    reset_text_line_height(&mut doc, Some(layer)).expect("no-op でも Ok のはず");
    assert!(
        text_document_of(&doc, layer).is_none(),
        "既に Auto なのに Intent を出している(決定7 違反)"
    );
}

/// Tracking の Enter → Reset は 0 へ戻す(map「Reset tracking to 0」)。
#[test]
fn committing_tracking_then_resetting_returns_to_zero() {
    let (mut doc, layer) = text_layer();
    let field = TextField::Tracking;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "25".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, Some(layer), field)
        .expect("tracking の確定は成功するはず");
    assert_eq!(text_document_of(&doc, layer).unwrap().styles[0].tracking, 25.0);

    reset_text_tracking(&mut doc, Some(layer)).expect("0 へのリセットは成功するはず");
    assert_eq!(
        text_document_of(&doc, layer).unwrap().styles[0].tracking,
        0.0,
        "Reset ボタンが tracking を0へ戻していない"
    );
}

// ---------------------------------------------------------------------------
// 編集: Justify 巡回は即1回の `Intent::SetTextDocument`
// ---------------------------------------------------------------------------

/// Justify 巡回は `Left → Right → Center → Left` を辿り、1 undo で戻る
/// (`cycling_a_mask_mode_advances...` と同じ形の検収)。
#[test]
fn cycling_justify_advances_through_left_right_center_and_undoes_in_one_step() {
    let (mut doc, layer) = text_layer();

    cycle_text_justify(&mut doc, Some(layer)).expect("Justify 巡回は成功するはず");
    assert_eq!(
        text_document_of(&doc, layer).unwrap().justify,
        TextJustify::Right,
        "既定 Left の次は Right のはず"
    );

    cycle_text_justify(&mut doc, Some(layer)).expect("Justify 巡回は成功するはず");
    assert_eq!(
        text_document_of(&doc, layer).unwrap().justify,
        TextJustify::Center,
        "Right の次は Center のはず"
    );

    doc.undo();
    assert_eq!(
        text_document_of(&doc, layer).unwrap().justify,
        TextJustify::Right,
        "1 undo で1歩だけ戻らない(1操作=1 undo 違反)"
    );
}

// ---------------------------------------------------------------------------
// 安全側: 選択なしは黙って no-op(`commit_inspector_field`/mask 系と同じ柵)
// ---------------------------------------------------------------------------

#[test]
fn text_field_edits_without_a_selection_are_silent_no_ops() {
    let mut doc = Document::new();
    let mut content_draft = Some(TextFieldDraft {
        field: TextField::Content,
        text: "誰も見ない歌詞".to_owned(),
    });
    commit_text_field(&mut doc, &mut content_draft, None, TextField::Content)
        .expect("選択なしは no-op のはず");

    let field = TextField::Tracking;
    let mut draft = Some(TextFieldDraft {
        field,
        text: "25".to_owned(),
    });
    commit_text_field(&mut doc, &mut draft, None, field).expect("選択なしは no-op のはず");
    cycle_text_justify(&mut doc, None).expect("選択なしは no-op のはず");
    reset_text_line_height(&mut doc, None).expect("選択なしは no-op のはず");
    reset_text_tracking(&mut doc, None).expect("選択なしは no-op のはず");
}

// ---------------------------------------------------------------------------
// ±1px 柵: TEXT section の行も既存の行グラマー(`bordered_row`/
// `row_band_style`)をそのまま再利用しており、新しい寸法・border 幅を発明
// していない。行の幾何契約そのものの機械照合は `row_line_fence.rs`(D5
// ORACLE)がすでに担っており、この section はそこを経由するだけ
// (`text_section` 内の各行はすべて `bordered_row(..., dims)` 経由 — lib.rs
// 参照)なので、ここで契約を複製しない。**このテストは「新しい border/
// padding トークンを足していない」ことだけを再確認する**(回帰の早期発見)。
// ---------------------------------------------------------------------------

#[test]
fn text_section_rows_reuse_the_existing_row_geometry_contract() {
    use motolii_inspector_pane::row_band_style;

    let dims = Dimensions::default();
    let style = row_band_style(dims);
    assert_eq!(
        style.border.width, dims.border_width,
        "TEXT section が依拠する行 border 幅が既存トークンから動いた(幾何不変違反)"
    );
    assert_eq!(
        style.border.color.a, 0.0,
        "TEXT section が依拠する行 border が透明でない(D5 違反)"
    );
}

// ---------------------------------------------------------------------------
// E-3(2026-08-23): Key 列の見た目結線 — 投影の値で示す(`iced_test::
// Simulator` は canvas を構造的に見られないため、widget を作る前の投影の層で
// 押さえる、発注書の指示どおり)。
// ---------------------------------------------------------------------------

/// **検収点**: `Size` の track に打点があると、投影の Key 状態
/// (`TextSectionProjection::size_key`)が未打点(`Static`)と異なる。
/// [`toggle_text_style_key`](本物の書き口、click の意味そのもの)で
/// playhead=0 へ打点し、打点前後で投影を撮り直して比較する。
#[test]
fn size_key_state_differs_once_a_keyframe_is_placed() {
    let (mut doc, layer) = text_layer();
    let session = session_selecting(layer);
    let fps = fps30();

    let before = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず")
        .text
        .expect("text layer なので TEXT 投影は Some のはず")
        .size_key;
    assert_eq!(
        before,
        KeyCellState::Static,
        "track を置く前は Static(未打点)のはず"
    );

    toggle_text_style_key(
        &mut doc,
        Some(layer),
        session.playhead,
        fps,
        TextStyleId(0),
        TextStyleField::Size,
    )
    .expect("Key 列 click の意味そのもの ── 打てるはず");

    let after = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず")
        .text
        .expect("text layer なので TEXT 投影は Some のはず")
        .size_key;

    assert_ne!(
        before, after,
        "打点しても投影の Key 状態が変わっていない(見た目が結線されていない)"
    );
    assert_eq!(
        after,
        KeyCellState::AtKey,
        "playhead(=0)にちょうど打ったので AtKey(実菱形・濃)のはず"
    );
}

/// [`toggle_text_style_key`] をもう一度呼ぶと同じキーが除去され、投影は
/// `Static` へ戻る(3状態 oracle が往復対称であることの確認 ── AE のストップ
/// ウォッチ解除と等価、`crate::transform::toggled_key_track` doc 参照)。
#[test]
fn size_key_state_returns_to_static_after_toggling_the_same_keyframe_off() {
    let (mut doc, layer) = text_layer();
    let session = session_selecting(layer);
    let fps = fps30();

    toggle_text_style_key(
        &mut doc,
        Some(layer),
        session.playhead,
        fps,
        TextStyleId(0),
        TextStyleField::Size,
    )
    .expect("1回目: 打てるはず");
    toggle_text_style_key(
        &mut doc,
        Some(layer),
        session.playhead,
        fps,
        TextStyleId(0),
        TextStyleField::Size,
    )
    .expect("2回目: 外せるはず(最後の1個 → 静的化)");

    let after = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず")
        .text
        .expect("text layer なので TEXT 投影は Some のはず")
        .size_key;
    assert_eq!(
        after,
        KeyCellState::Static,
        "唯一のキーを外したら Static(静的化)へ戻るはず"
    );
}
