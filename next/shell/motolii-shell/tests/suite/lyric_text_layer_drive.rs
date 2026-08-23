//! 通し(2026-08-22 発注「歌詞が入れられる道を通す」)— `docs/reviews/
//! 2026-08-22-persona-lyric-mv.md` が実証した致命的欠落(テキストレイヤーを
//! 作る入口がリポ全体に存在しない・TEXT section に本文の入力欄が無い・
//! 塗り色の editor が無い)を実際に塞いだことの固定。
//!
//! 「New Text Layer → 文字を打つ → 色とサイズを変える」を**実 Message 経路
//! だけ**で通す(create タブは 2026-08-22 利用者裁定「追加するものは Browser
//! の中に全部入れる」で `browser_pane::Message::CreateFromCard` へ統一 —
//! Layer メニュー等の別入口は作らない)。
//!
//! ## フォント選択 UI は本発注の範囲外だった(FINDING) — 2026-08-22 追い発注
//! 「フォントが選べる・選ばなくても落ちない」で解消済み
//!
//! 下の doc は発見時点(本発注)の記録としてそのまま残す。**その後**の追い発注
//! (`motolii-font-catalog` crate 新設 + `inspector_pane::text::
//! default_text_style`/`font_family_row`/`commit_text_font_pick`)がこの穴を
//! 塞いだ ——
//! [`lyrics_render_without_ever_touching_the_font_ui`] がそのまま解消の検収
//! (Shell だけを経由し、フォント欄に一切触れずに Stage へ画素が届く)。
//!
//! `TextField::FontFamily` は `FontRef::family` しか編集しない —
//! `FontRef::path` を編集する入口はリポ全体のどこにも無い(`text.rs` 実装・
//! `motolii-inspector-pane` 全 grep で確認)。エンジンのラスタライズ
//! (cosmic-text→swash)は `db.load_font_file(&font.path)` が実在するファイルを
//! 要求する(`motolii-vector::text::shape_text` 実装)ので、**現在のビルドでは
//! Browser の Text カードで作った layer に利用者が辿れる道だけでは Stage に
//! 文字が出ない**(content が空の間は早期 return で無害だが、文字を打った
//! 瞬間に空 path でのフォント読み込みへ進み `TextShapeError::FontFile` で
//! `render_frame` 全体が `Err` になる — `motolii-engine/src/text.rs`
//! `rasterize_text_document` 実装参照)。これは本発注が作った欠落ではなく、
//! 本発注が初めて可視化した**別の**既存の穴(フォントピッカー UI 皆無)。
//!
//! 下の2本目の試験(`*_reaches_the_stage_once_a_font_is_present`)は、この
//! 穴を隠さず明記した上で、**Shell を経由しない生の `Document`**(motolii-store
//! を直接叩く——`Shell` は書き口を1箇所に絞っているので、フォント path を
//! 差し込む Message が無い以上ここからは触れない)へ本発注の実関数
//! (`inspector_pane::commit_text_field`/`inspector_pane::color::
//! commit_text_style_color`)をそのまま適用し、フォントだけ試験側が直接
//! 注入して、配線そのもの(content/color が実際に画素へ届くこと)を確かめる。
//! (**注**: 追い発注後は「フォント path を差し込む Message」が
//! `inspector_pane::Message::PickFont` として実在する。この2本目の試験は
//! それでも生 `Document` のままにしてある——font pick_list を経由せず
//! `commit_text_field`/`commit_text_style_color` という実発注の関数そのものが
//! 画素まで届くことを確かめる、という元の試験意図を変えないため。)

use motolii_shell::browser_pane;
use motolii_shell::inspector_pane::{
    self,
    color::{ColorChannel, ColorTarget},
};
use motolii_shell::{Message, Shell};
use motolii_store::RationalTime;

fn dispatch(shell: &mut Shell, message: Message) {
    let _ = shell.update(message);
}

/// Browser の Text カードをダブルクリックした時に shell が受け取る Message
/// と同じ形で layer を作る(`create_from_card_drive.rs` と同じ手口)。
fn create_text_layer(shell: &mut Shell) -> motolii_store::LayerId {
    dispatch(
        shell,
        Message::Browser(browser_pane::Message::CreateFromCard {
            kind: browser_pane::model::CreateKind::Text,
        }),
    );
    shell
        .session()
        .selection
        .expect("Text カード生成後に選択されていないはず")
}

fn set_text_field(shell: &mut Shell, field: inspector_pane::TextField, text: &str) {
    dispatch(
        shell,
        Message::Inspector(inspector_pane::Message::TextFieldInput(
            field,
            text.to_owned(),
        )),
    );
    dispatch(
        shell,
        Message::Inspector(inspector_pane::Message::TextFieldSubmit(field)),
    );
}

fn set_color_channel(shell: &mut Shell, target: ColorTarget, channel: ColorChannel, text: &str) {
    dispatch(
        shell,
        Message::Inspector(inspector_pane::Message::Color(
            inspector_pane::color::Message::ChannelInput(target, channel, text.to_owned()),
        )),
    );
    dispatch(
        shell,
        Message::Inspector(inspector_pane::Message::Color(
            inspector_pane::color::Message::ChannelSubmit(target, channel),
        )),
    );
}

/// **本命**: Text カード生成 → Content(日本語歌詞)→ Size → Fill 色、全操作が
/// 実 Message 経路(`Message::Browser` → `Message::Inspector`)だけで
/// `Document` へ通る。
#[test]
fn create_text_layer_type_lyrics_and_change_size_and_color_through_messages() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.layer_count();
    let layer = create_text_layer(&mut shell);
    assert_eq!(shell.layer_count(), before + 1, "Text カードが layer を増やしていない");

    set_text_field(&mut shell, inspector_pane::TextField::Content, "サビ歌詞がここに出る");
    set_text_field(&mut shell, inspector_pane::TextField::Size, "64");
    set_color_channel(&mut shell, ColorTarget::Fill, ColorChannel::R, "255");
    set_color_channel(&mut shell, ColorTarget::Fill, ColorChannel::G, "255");
    set_color_channel(&mut shell, ColorTarget::Fill, ColorChannel::B, "255");

    let document = shell
        .store_view()
        .text_document(layer)
        .ok()
        .flatten()
        .expect("Content/Size/Color を書いたのに TextDocument が無い");
    assert_eq!(
        document.content.eval(RationalTime::ZERO),
        "サビ歌詞がここに出る",
        "Content が Message 経路で書けていない"
    );
    // **D-1(2026-08-23)**: `TextField::Size`/`LineHeight`/`Tracking` の Enter
    // 確定は `text_field_track_target` 経由で track 書き口
    // (`commit_text_style_track_field`)へ橋渡しされるようになった——track が
    // 正本(`StoreView::resolved_text_document`、A-1b 裁定214)なので、書いた
    // 値は `TextDocument::styles[0].size`(静的フィールド、もう更新されない)
    // ではなく `resolved_text_document` 経由で読む必要がある(Preview/Export
    // が実際に読む経路と同じ、`render.rs`/`lottie.rs` 参照)。
    let resolved = shell
        .store_view()
        .resolved_text_document(layer, RationalTime::ZERO)
        .ok()
        .flatten()
        .expect("resolved_text_document が読めない");
    assert_eq!(resolved.styles[0].size, 64.0, "Size が Message 経路で書けていない");
    assert_eq!(
        document.styles[0].fill,
        [1.0, 1.0, 1.0, 1.0],
        "Fill 色が Message 経路で書けていない(R/G/B を255にしたのに白になっていない)"
    );
}

/// Stroke 色も同じ経路で書ける(`ColorTarget::Stroke` — `None` から
/// 昇格させる分岐、`color.rs::set_text_style_color_channel` doc 参照)。
#[test]
fn stroke_color_promotes_from_none_through_the_same_message_path() {
    let mut shell = Shell::new_fixture().0;
    let layer = create_text_layer(&mut shell);

    let before = shell
        .store_view()
        .text_document(layer)
        .ok()
        .flatten()
        .expect("生成直後から TextDocument があるはず(空 layer にならない)");
    assert_eq!(before.styles[0].stroke_color, None, "既定は stroke 無しのはず");

    set_color_channel(&mut shell, ColorTarget::Stroke, ColorChannel::R, "200");

    let after = shell
        .store_view()
        .text_document(layer)
        .ok()
        .flatten()
        .expect("stroke 色を書いたのに TextDocument が無い");
    assert!(
        after.styles[0].stroke_color.is_some(),
        "Stroke 色欄への commit が None のまま(昇格していない)"
    );
}

/// **配線の直接固定(FINDING、モジュール冒頭 doc 参照)**: フォント選択 UI が
/// 無いという別の既存の穴を明記した上で、本発注の実関数
/// (`inspector_pane::commit_text_field`/`inspector_pane::color::
/// commit_text_style_color`)がそのまま `Engine::render_frame` まで届くこと
/// (Stage に実際の画素として出る)を確かめる。Shell 経由ではなく生の
/// `Document`(Shell に「フォント path を差し込む Message」が無いため —
/// 意図的に Shell を経由しない、上記doc参照)。
#[test]
fn lyric_content_and_color_reach_the_stage_once_a_font_is_present() {
    use motolii_engine::Engine;
    use motolii_store::{
        Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    };

    const HIRAGINO: &str = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc";

    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 512,
        height: 200,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Text,
                order: 0,
                timing: LayerTiming::place(0, None, 60),
            },
        },
        // `create_from_card` が書くのと同じ既定 TextDocument(空 content)。
        Intent::SetTextDocument {
            layer,
            document: inspector_pane::default_text_document(),
        },
    ])
    .unwrap();

    // Content は実発注の関数(commit_text_field)経由 — `Shell::update_inspector`
    // が呼ぶのと同じ関数を直接叩く。
    let mut text_draft = Some(inspector_pane::TextFieldDraft {
        field: inspector_pane::TextField::Content,
        text: "文字が画素になる".to_owned(),
    });
    inspector_pane::commit_text_field(
        &mut doc,
        &mut text_draft,
        Some(layer),
        inspector_pane::TextField::Content,
    )
    .expect("content を書けるはず");

    // 色も同じく実発注の関数(commit_text_style_color)経由。
    for (channel, value) in [
        (ColorChannel::R, "0"),
        (ColorChannel::G, "0"),
        (ColorChannel::B, "255"),
    ] {
        let mut color_draft = Some(inspector_pane::color::ColorFieldDraft {
            target: ColorTarget::Fill,
            channel,
            text: value.to_owned(),
        });
        inspector_pane::color::commit_text_style_color(
            &mut doc,
            &mut color_draft,
            Some(layer),
            ColorTarget::Fill,
            channel,
        )
        .expect("色を書けるはず");
    }

    // **FINDING の直接注入**: フォント path を編集する UI が無いので、ここだけ
    // 生の Intent で土台を差し込む(利用者が辿れる道ではない、モジュール冒頭
    // doc 参照)。content/color は上で実関数を通したままなので、その部分の
    // 検収は崩れない。
    let mut document = doc.view().text_document(layer).unwrap().unwrap();
    document.styles[0].font.path = HIRAGINO.to_owned();
    document.styles[0].font.family = "Hiragino Sans".to_owned();
    doc.apply(Intent::SetTextDocument { layer, document }).unwrap();

    let mut engine = Engine::new().expect("engine");
    let t = RationalTime::try_from_frame(0, Fps::try_new(30, 1).unwrap()).unwrap();
    let frame = engine.render_frame(&doc.view(), t).expect("render");

    // 背景は不透明黒 — 青チャンネルが立っている画素を「文字が出た画素」として
    // 数える(`motolii-engine/tests/text_layer.rs::colored_pixel_count` と
    // 同じ手口)。
    let blue_pixels = frame.chunks_exact(4).filter(|p| p[2] > 40 && p[0] < 40).count();
    assert!(
        blue_pixels > 500,
        "Content(commit_text_field)/Color(commit_text_style_color)経由で書いた \
         日本語テキストが Stage に画素として出ていない"
    );
}

// ---------------------------------------------------------------------------
// FINDING の解消固定(2026-08-22 追い発注「フォントが選べる・選ばなくても
// 落ちない」)— モジュール冒頭 doc 参照。
// ---------------------------------------------------------------------------

/// **本命の解消固定**: Browser の Text カードで layer を作り、フォント欄
/// (`Font` の text_input/pick_list)に一切触れず Content だけ打っても Stage が
/// 描ける。以前は [`inspector_pane::default_text_document`] の font が空
/// path のままだったため、この操作(Content の Enter)だけで
/// `Engine::render_frame` が `Err` になっていた——`Shell::refresh_frame` は
/// M16(「絵が出せなくても画面は空にしない」)により Stage 自体が消えることは
/// 無いが、status 帯に「Stage を描けない」が出て**実際には文字が画素として
/// 一切届かない**状態だった。追い発注(`motolii-font-catalog` による既定
/// フォント解決)後は、フォント欄に触れなくても解決可能な `FontRef` が
/// 最初から入っているので、この操作だけで正常に描ける。
#[test]
fn lyrics_render_without_ever_touching_the_font_ui() {
    let mut shell = Shell::new_fixture().0;
    create_text_layer(&mut shell);
    set_text_field(
        &mut shell,
        inspector_pane::TextField::Content,
        "フォント欄に触れずにここまで届く",
    );

    let status = shell.status().unwrap_or("").to_owned();
    assert!(
        !status.contains("Stage を描けない"),
        "フォント欄に触れず Content を打っただけで Stage が描けなくなっている \
         (FINDING の再発): {status}"
    );

    let (width, height, _rgba) = shell
        .frame_rgba()
        .expect("フレームが一度も描けていない(FINDING の再発)");
    assert!(width > 0 && height > 0, "描けたフレームの寸法が0");
}
