//! `--fixture`(トンマナ検分器具)の受入条件。発注書の箇条書きをそのまま検分する:
//! 層15本・timing をずらした配置・マーカー3個・数層に position/opacity キー・
//! 1層選択済み・playhead を中盤へ・status 帯に文言1件。

use motolii_shell::Shell;

fn fixture() -> Shell {
    Shell::new_fixture().0
}

#[test]
fn fixture_places_fifteen_layers() {
    let shell = fixture();
    assert_eq!(shell.layer_count(), 15, "fixture の層数が15本でない");
}

#[test]
fn fixture_layers_have_staggered_timing() {
    let shell = fixture();
    let rows = shell.timeline_rows();
    assert_eq!(rows.len(), 15);

    // 「timing をずらした配置」— 全層が同じ start に揃っていては fixture の
    // 意図(MV制作の途中)を満たさない。start の異なり数が複数あることを見る。
    let mut starts: Vec<i64> = rows.iter().map(|r| r.start).collect();
    starts.sort_unstable();
    starts.dedup();
    assert!(
        starts.len() > 1,
        "全層が同じ start に揃っている — timing がずれていない"
    );

    // 尺も一様でないこと(全層が comp 尺いっぱいでは編集途中に見えない)。
    let mut durations: Vec<i64> = rows.iter().map(|r| r.duration).collect();
    durations.sort_unstable();
    durations.dedup();
    assert!(durations.len() > 1, "全層の尺が同じ — 配置がずれていない");
}

#[test]
fn fixture_names_mix_lyric_and_material_vocabulary() {
    let shell = fixture();
    let rows = shell.timeline_rows();
    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();

    let lyric_like = names.iter().any(|n| n.contains("歌詞"));
    let material_like = names
        .iter()
        .any(|n| n.contains("ロゴ") || n.contains("ロール") || n.contains("映像"));
    assert!(lyric_like, "歌詞風の名前が1つも無い: {names:?}");
    assert!(material_like, "素材風の名前が1つも無い: {names:?}");

    // 名前は空でないこと(fixture が地の layer id 表示にフォールバックしていない)。
    assert!(
        names.iter().all(|n| !n.is_empty()),
        "名前の付いていない層がある: {names:?}"
    );
}

#[test]
fn fixture_has_three_markers() {
    let shell = fixture();
    let markers = shell.markers();
    assert_eq!(markers.len(), 3, "マーカーが3個でない: {markers:?}");
}

#[test]
fn fixture_has_one_selected_layer() {
    let shell = fixture();
    let rows = shell.timeline_rows();
    let selected: Vec<_> = rows.iter().filter(|r| r.selected).collect();
    assert_eq!(
        selected.len(),
        1,
        "選択済みの層がちょうど1本でない: {selected:?}"
    );
}

#[test]
fn fixture_playhead_sits_mid_composition() {
    let shell = fixture();
    let comp = shell.composition().expect("fixture は comp を持つ");
    let playhead = shell.session().playhead;
    assert!(
        playhead > 0 && playhead < comp.duration_frames,
        "playhead が中盤にない(playhead={playhead}, duration={})",
        comp.duration_frames
    );
    // 「中盤」の具体的な検分 — 尺の半分に一致する(fixture::build の設計値)。
    assert_eq!(playhead, comp.duration_frames / 2);
}

#[test]
fn fixture_status_band_carries_one_line() {
    let shell = fixture();
    let status = shell.status().expect("status 帯に文言が無い");
    assert!(!status.is_empty(), "status 帯の文言が空");
}

/// 数層に position/opacity キー — 直接 store を叩いて検分する(`Shell` は
/// `StoreView` を外へ出さないので、`fixture::build()` と同じ Intent 列を
/// `motolii_store` 直叩きで再現する。`timeline_pane` の第2波以降で `StoreView`
/// 経由の口が増えたら、そちらへ寄せてよい)。
#[test]
fn fixture_gives_some_layers_position_or_opacity_keyframes() {
    use motolii_store::{property, PropertyId};

    // `fixture::build()` が返す `Document` を直接使う — `Shell` は `StoreView` を
    // 外へ出さないので、track の有無を検分するにはここだけ store を直叩きする。
    let built = motolii_shell::fixture::build();
    let store = built.doc.view();

    let position = PropertyId::new(property::POSITION).unwrap();
    let opacity = PropertyId::new(property::OPACITY).unwrap();

    let mut layers_with_keys = 0;
    for id in store.layers() {
        let has_position = store
            .track(id, &position)
            .ok()
            .flatten()
            .map(|t| !t.keys().is_empty())
            .unwrap_or(false);
        let has_opacity = store
            .track(id, &opacity)
            .ok()
            .flatten()
            .map(|t| !t.keys().is_empty())
            .unwrap_or(false);
        if has_position || has_opacity {
            layers_with_keys += 1;
        }
    }
    assert!(
        layers_with_keys >= 2,
        "position/opacity キーを持つ層が2本未満(fixture の受入条件『数層』を満たさない)"
    );
}

// ---------------------------------------------------------------------------
// screenshot 器具
// ---------------------------------------------------------------------------

/// **1フレーム描いて PNG を書く**。トンマナ照合の機械化用の口そのもの。
#[test]
fn screenshot_writes_a_readable_png_sized_like_the_window() {
    let mut shell = fixture();
    let dir = motolii_testkit::tmp_dir("shell-screenshot");
    let path = dir.join("fixture.png");

    motolii_shell::screenshot::write_png(&mut shell, &path).expect("screenshot を書き出せない");

    let bytes = std::fs::read(&path).expect("PNG が書かれていない");
    assert!(
        bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "PNG シグネチャが無い"
    );

    let image = image::open(&path)
        .expect("書いた PNG を読み直せない")
        .to_rgba8();
    assert!(
        image.width() >= 1200,
        "窓幅相当の広さが無い: {}",
        image.width()
    );
    assert!(
        image.height() > 200,
        "5帯(header/stage/timeline/transport/status)分の高さが無い"
    );
}

/// 色トンマナの照合: Stage letterbox は `surface_app`(neutral dark、D8)。
/// 四隅は Stage の実素材が絶対に届かない位置なので、letterbox 色そのものを拾える。
#[test]
fn screenshot_top_left_corner_is_the_app_surface_color() {
    let mut shell = fixture();
    let image = motolii_shell::screenshot::render(&mut shell);
    let colors = shell.tokens().colors;
    let expected = to_u8(colors.surface_app);
    let pixel = image.get_pixel(0, 0);
    assert_eq!(
        [pixel[0], pixel[1], pixel[2]],
        expected,
        "左上の色が surface_app トークンと一致しない(raw値で塗られている疑い)"
    );
}

fn to_u8(color: iced::Color) -> [u8; 3] {
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [c(color.r), c(color.g), c(color.b)]
}

/// **本命(red→green の柵)**: `--screenshot` 器具に Inspector 領域が無かった穴
/// (発注書 OUTCOME)。旧実装(このテスト追加前の `screenshot.rs`)へ当てると
/// canvas 幅が Stage/Timeline 列(1600px)止まりで Inspector の x 座標
/// (`inspector_region_x`)が画像の外に出るため、走査ループが1度も回らず
/// 両フラグが偽のまま fail する。
///
/// 実装後は `inspector_pane.rs` と同じ tokens(`surface_panel`=pane 地・
/// `surface_app`=値セルの窪み・`border_hairline_weak`/`border_default`=区切り線)で
/// 描いた領域から、(a) hairline 色の画素・(b) 値セルの窪み(`surface_app`、
/// pane 地の `surface_panel` とは異なる面色)の両方を実測する。
#[test]
fn screenshot_draws_the_inspector_region_with_hairlines_and_value_cell_insets() {
    let mut shell = fixture();
    let dims = shell.dims();
    let colors = shell.tokens().colors;
    let image = motolii_shell::screenshot::render(&mut shell);

    let x0 = motolii_shell::screenshot::inspector_region_x(dims).round() as u32;
    let y0 = motolii_shell::screenshot::inspector_region_top(dims).round() as u32;
    let width = dims.inspector_panel_width.round() as u32;
    let height = motolii_shell::screenshot::inspector_region_height(&shell).round() as u32;

    assert!(
        x0 + width <= image.width(),
        "Inspector 領域が画像の外にはみ出している(canvas 幅を広げ忘れ): x0={x0} width={width} image.width={}",
        image.width()
    );
    assert!(
        y0 + height <= image.height(),
        "Inspector 領域が画像の下にはみ出している: y0={y0} height={height} image.height={}",
        image.height()
    );

    let panel = to_u8(colors.surface_panel);
    let inset = to_u8(colors.surface_app);
    let raised = to_u8(colors.surface_raised);

    let mut found_panel = false;
    let mut found_inset = false;
    let mut found_hairline = false;

    for y in y0..(y0 + height) {
        for x in x0..(x0 + width) {
            let pixel = image.get_pixel(x, y);
            let rgb = [pixel[0], pixel[1], pixel[2]];
            if rgb == panel {
                found_panel = true;
            } else if rgb == inset {
                found_inset = true;
            } else if rgb != raised {
                // panel/inset/raised のどれでもない画素 = hairline(不透明
                // `border_default` か、`surface_panel` へ blend した
                // `border_hairline_weak` のどちらか)。
                found_hairline = true;
            }
        }
    }

    assert!(found_panel, "Inspector pane 地の色(surface_panel)の画素が無い");
    assert!(
        found_inset,
        "値セルの窪み(surface_app、pane 地と異なる面色)の画素が無い"
    );
    assert!(found_hairline, "hairline 色の画素が無い(区切り線が描かれていない)");
}
