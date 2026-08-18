//! E0 と同じシーン・同じカメラ・同じ期待色 oracle を1か所にまとめる。
//!
//! preflight(窓なし)と gui(iced 窓内)の両方から呼ぶ。**同じ oracle を通すこと**が
//! 「iced の中の絵が E0 の絵と同じ物である」という主張の根拠なので、判定式は共有する。

use re_log_types::ApplicationId;
use re_view_spatial::{SpatialStage, StageCamera};

use crate::harness::Offscreen;
use crate::oracle::{Frame, Hue};
use crate::scene;

/// document camera の垂直画角(60°)。E0 と同じ。
pub const DOC_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;

/// レイヤー面の z。`copy_gpu_image` が自分の path へ書く値。
pub const LAYER_PLANE_Z: f32 = -0.01;

/// bounding box の平滑化とカメラ補間が落ち着くまで回す数(E0 と同じ)。
pub const WARMUP_FRAMES: u32 = 90;

/// 注入したカメラは補間を挟まないので落ち着かせる必要が無い。
/// それでも1フレームでは「たまたま」を排除できないので2フレーム回す。
pub const SETTLE_FRAMES: u32 = 2;

pub fn new_stage(name: &str) -> Result<SpatialStage, String> {
    SpatialStage::new(ApplicationId::from(name))
        .map_err(|error| format!("create SpatialStage: {error}"))
}

/// 4象限テクスチャを stage に置く。E0 の (a)/(b) と同じ構成。
pub fn install_quadrants(offscreen: &mut Offscreen, stage: &mut SpatialStage) -> Result<(), String> {
    let texture = scene::upload(
        offscreen.device(),
        offscreen.queue(),
        "quadrants",
        &scene::quadrants(),
    );
    stage
        .copy_gpu_image(offscreen.render_ctx(), "stack/main/image", &texture)
        .map_err(|error| format!("copy_gpu_image: {error}"))?;
    scene::place_parent(stage, "stack/main", 0.0)?;
    scene::set_draw_order(stage, "stack/main/image", 0.0)?;
    Ok(())
}

/// レイヤーが画枠ちょうどを満たす document camera。
///
/// `pull_back` は 1.0 で「ぴったり」。1.0 より大きいと引いた絵になり、四隅に
/// Rerun の背景が見える。(c) で「クリックが本当にカメラを動かしたか」を
/// 四隅の色の変化で言うために使う。
pub fn document_camera(pull_back: f32) -> StageCamera {
    let distance = 0.5 / (DOC_FOV_Y * 0.5).tan() * pull_back;
    StageCamera::new(
        [0.0, 0.0, LAYER_PLANE_Z + distance],
        [0.0, 0.0, LAYER_PLANE_Z],
        [0.0, 1.0, 0.0],
    )
    .with_fov_y_radians(DOC_FOV_Y)
}

/// E0 の (b) oracle 3: 四隅の内側が各象限の色なら、画枠と document がぴったり重なっている。
///
/// E0 と同じ INSET・同じ期待色。落ちた項目を文字列で返す(空なら合格)。
pub fn check_frame_corners(frame: &Frame) -> (String, Vec<String>) {
    const INSET: i32 = 6;
    let width = frame.width as i32;
    let height = frame.height as i32;
    let corners: [(&str, i32, i32, Hue); 4] = [
        ("top-left", INSET, INSET, Hue::Red),
        ("top-right", width - 1 - INSET, INSET, Hue::Green),
        ("bottom-left", INSET, height - 1 - INSET, Hue::Blue),
        ("bottom-right", width - 1 - INSET, height - 1 - INSET, Hue::Yellow),
    ];

    let mut report = String::new();
    let mut failures = Vec::new();
    for (name, x, y, expected) in corners {
        let observed = frame.hue_at(x, y);
        report.push_str(&format!(
            "  frame corner {name} ({x}, {y}): {observed} {:?} (expected {expected})\n",
            frame.rgb(x, y)
        ));
        if observed != expected {
            failures.push(format!(
                "frame corner {name} is {observed}, expected {expected}"
            ));
        }
    }
    (report, failures)
}

/// E0 の (a) oracle: 4象限すべてが十分な画素数で出ていること。
///
/// `minimum` は面積に対する割合で決める。E0 は 640x480 に対して 2000 画素だったので、
/// 同じ密度(全体の約 0.65%)を使う。iced の shader widget は窓より小さいので絶対数では比べられない。
pub fn check_all_quadrants_present(frame: &Frame) -> (String, Vec<String>) {
    let counts = frame.hue_histogram();
    let total = frame.width * frame.height;
    let minimum = (total as f64 * 0.0065) as u32;
    let mut report = format!(
        "  hue histogram: {} (minimum per quadrant = {minimum} of {total})\n",
        crate::oracle::histogram_line(counts)
    );
    let mut failures = Vec::new();
    for (hue, count) in [
        (Hue::Red, counts[0]),
        (Hue::Green, counts[1]),
        (Hue::Blue, counts[2]),
        (Hue::Yellow, counts[3]),
    ] {
        if count < minimum {
            failures.push(format!("only {count} {hue} pixels (< {minimum})"));
        }
    }
    if failures.is_empty() {
        report.push_str("  all four quadrants are present\n");
    }
    (report, failures)
}
