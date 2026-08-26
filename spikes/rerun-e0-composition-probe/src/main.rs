//! E0 probe — Rerun を「空間合成のメイン基盤」に据えられるかの3点実測。
//!
//! 測るのは以下の3つで、いずれも `SpatialStage`(= Spatial Viewer の wrapper)を通す。
//! direct な `re_renderer` scene も第二 runtime も作らない(2026-08-11裁定)。
//!
//! - (a) offscreen: OS 窓なしで `SpatialStage` のシーンを texture へ描き、pixel を読めるか
//! - (b) camera:    定義済みカメラをプログラムから注入し、既知配置の矩形が期待座標へ写るか
//! - (c) occlusion: 画素 alpha を持つ2枚を前後に置いて、前が後を遮蔽し alpha 部は透けるか
//!
//! oracle は先に書いてある。埋めるのは実測値だけで、通らなかったものは通らなかったと出す。
//!
//! 使い方: `cargo run -p rerun-e0-composition-probe -- <出力ディレクトリ>`

mod harness;
mod oracle;
mod scene;

use std::path::{Path, PathBuf};

use harness::Offscreen;
use oracle::{Frame, Hue};
use re_log_types::ApplicationId;
use re_view_spatial::{SpatialStage, StageCamera};

const TARGET_W: u32 = 640;
const TARGET_H: u32 = 480;
/// bounding box の平滑化とカメラ補間が落ち着くまで回す。
/// 決定性のため実時間ではなく固定 dt で数える(`harness::Offscreen::frame`)。
const WARMUP_FRAMES: u32 = 90;

/// 注入したカメラは補間を挟まずに効くので、落ち着かせる必要が無い。
/// それでも1フレームでは「たまたま」を排除できないので2フレーム回す。
const SETTLE_FRAMES: u32 = 2;

/// document camera の垂直画角(60°)。
///
/// レイヤーの縦 1.0 がちょうど画枠の縦を満たす距離を、この画角から逆算する。
/// ビューポート(640x480)もレイヤー(320x240)も 4:3 なので、横もちょうど収まる。
const DOC_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;

/// レイヤー面の z。`copy_gpu_image` が自分の path へ書く値である。
const LAYER_PLANE_Z: f32 = -0.01;

/// (c) の前後差。
///
/// `plane_clustering.rs:168` が平面距離を 1.0e-3 刻みで量子化するので、
/// 0.05 は「同一平面クラスタ」から確実に外れる。一方で既定カメラは
/// レイヤーの真上から見下ろすため、この程度なら視差で円板が矩形の外へはみ出さない。
/// レイヤー面は XY 平面(`grid_map.rs:290-293`)で、z が面の法線方向である。
const LAYER_Z_STEP: f32 = 0.05;

fn main() -> std::process::ExitCode {
    let Some(out_dir) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: rerun-e0-composition-probe <output-dir>");
        return std::process::ExitCode::from(2);
    };

    let mut failures = Vec::new();
    for (name, result) in [
        ("(a) offscreen", probe_offscreen(&out_dir)),
        ("(b) camera", probe_camera(&out_dir)),
        ("(c) occlusion", probe_occlusion(&out_dir)),
    ] {
        match result {
            Ok(report) => println!("\n=== {name}: PASS ===\n{report}"),
            Err(report) => {
                println!("\n=== {name}: FAIL ===\n{report}");
                failures.push(name);
            }
        }
    }

    if failures.is_empty() {
        println!("\nall three E0 checks passed");
        std::process::ExitCode::SUCCESS
    } else {
        println!("\nfailed: {}", failures.join(", "));
        std::process::ExitCode::FAILURE
    }
}

/// (a) 窓なしで描いて読み出せるか。
///
/// oracle: 4象限すべてが十分な画素数で出ること。clear color だけの絵や、
/// 「レイヤーが描かれていない絵」はここで落ちる。
fn probe_offscreen(out_dir: &Path) -> Result<String, String> {
    let mut offscreen = Offscreen::new(TARGET_W, TARGET_H)?;
    let mut stage = new_stage("e0-offscreen")?;

    let texture = scene::upload(
        offscreen.device(),
        offscreen.queue(),
        "quadrants",
        &scene::quadrants(),
    );
    stage
        .copy_gpu_image(offscreen.render_ctx(), "stack/main/image", &texture)
        .map_err(|error| format!("copy_gpu_image: {error}"))?;
    scene::place_parent(&mut stage, "stack/main", 0.0)?;
    scene::set_draw_order(&mut stage, "stack/main/image", 0.0)?;

    for _ in 0..WARMUP_FRAMES {
        offscreen.frame(&mut stage)?;
    }
    let frame = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
    let png = out_dir.join("e0-a-offscreen.png");
    frame.write_png(&png)?;

    let counts = frame.hue_histogram();
    let mut report = format!(
        "PNG: {}\nfnv1a: {:016x}\nhue histogram: red={} green={} blue={} yellow={} dark={} other={}\n",
        png.display(),
        frame.digest(),
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
        counts[5],
    );

    let minimum = 2_000;
    for (hue, count) in [
        (Hue::Red, counts[0]),
        (Hue::Green, counts[1]),
        (Hue::Blue, counts[2]),
        (Hue::Yellow, counts[3]),
    ] {
        if count < minimum {
            return Err(format!(
                "{report}only {count} {hue} pixels (< {minimum}) — the layer was not drawn offscreen"
            ));
        }
    }
    report.push_str("all four quadrants are present: the scene really rendered without a window\n");
    Ok(report)
}

/// (b) カメラをプログラムから決められるか、そして既知配置が期待座標へ写るか。
///
/// `SpatialStage::set_camera` へ document camera 相当を渡し、**こちらが独立に
/// 計算した期待座標**とレンダリング結果を突き合わせる。期待 pixel は画角と距離から
/// 直接出しており、Rerun の `Eye` や `ui_from_world` を経由しない。よって上流の
/// カメラ実装が変われば、この照合が落ちて教えてくれる。
///
/// oracle は5段。
/// 1. 注入で絵が変わること
/// 2. `last_eye()` が注入した姿勢をそのまま返すこと
/// 3. レイヤーが画枠ちょうどに写ること(四隅の色)
/// 4. レイヤー内の格子点が期待座標の色と一致すること
/// 5. `reset_view()` で Rerun 既定の画へ戻ること
fn probe_camera(out_dir: &Path) -> Result<String, String> {
    let mut offscreen = Offscreen::new(TARGET_W, TARGET_H)?;
    let mut stage = new_stage("e0-camera")?;

    let texture = scene::upload(
        offscreen.device(),
        offscreen.queue(),
        "quadrants",
        &scene::quadrants(),
    );
    stage
        .copy_gpu_image(offscreen.render_ctx(), "stack/main/image", &texture)
        .map_err(|error| format!("copy_gpu_image: {error}"))?;
    scene::place_parent(&mut stage, "stack/main", 0.0)?;
    scene::set_draw_order(&mut stage, "stack/main/image", 0.0)?;

    for _ in 0..WARMUP_FRAMES {
        offscreen.frame(&mut stage)?;
    }
    let before = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
    before.write_png(&out_dir.join("e0-b-default-camera.png"))?;

    let eye = stage
        .last_eye()
        .ok_or_else(|| "SpatialStage::last_eye() is None after warmup".to_owned())?;
    let eye_pos = eye.pos_in_world();
    let eye_fwd = eye.forward_in_world();
    let mut report = format!(
        "PNG(default camera): {}\nfnv1a: {:016x}\n\
         eye.pos = ({:.4}, {:.4}, {:.4})  fwd = ({:.4}, {:.4}, {:.4})  fov_y = {:?}\n",
        out_dir.join("e0-b-default-camera.png").display(),
        before.digest(),
        eye_pos.x,
        eye_pos.y,
        eye_pos.z,
        eye_fwd.x,
        eye_fwd.y,
        eye_fwd.z,
        eye.fov_y,
    );

    let mut failures = Vec::new();

    // --- oracle 1: 注入したカメラで絵が変わること ---
    //
    // レイヤーの半分の高さ 0.5 が画枠の半分をちょうど満たす距離を、画角から逆算する。
    // これで「document が画枠に一致する」状態が作れ、期待座標が解析的に決まる。
    let distance = 0.5 / (DOC_FOV_Y * 0.5).tan();
    let camera_z = LAYER_PLANE_Z + distance;
    let camera = StageCamera::new(
        [0.0, 0.0, camera_z],
        [0.0, 0.0, LAYER_PLANE_Z],
        [0.0, 1.0, 0.0],
    )
    .with_fov_y_radians(DOC_FOV_Y);
    stage.set_camera(camera);

    for _ in 0..SETTLE_FRAMES {
        offscreen.frame(&mut stage)?;
    }
    let shot = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
    let shot_png = out_dir.join("e0-b-document-camera.png");
    shot.write_png(&shot_png)?;

    report.push_str(&format!(
        "\ninjected document camera: pos = (0, 0, {camera_z:.4}), look_target = (0, 0, {LAYER_PLANE_Z}), \
         fov_y = {DOC_FOV_Y:.4} rad\nPNG(document camera): {}\nfnv1a: {:016x} ({})\n",
        shot_png.display(),
        shot.digest(),
        if shot.digest() == before.digest() {
            "IDENTICAL to the default camera — nothing moved"
        } else {
            "changed"
        },
    ));
    if shot.digest() == before.digest() {
        failures.push("set_camera did not change the picture".to_owned());
    }

    // --- oracle 2: last_eye() が注入した姿勢をそのまま返すこと ---
    match stage.last_eye() {
        Some(eye) => {
            let pos = eye.pos_in_world();
            let fwd = eye.forward_in_world();
            report.push_str(&format!(
                "  last_eye() readback: pos = ({:.4}, {:.4}, {:.4}), fwd = ({:.4}, {:.4}, {:.4}), fov_y = {:?}\n",
                pos.x, pos.y, pos.z, fwd.x, fwd.y, fwd.z, eye.fov_y,
            ));
            if (pos - glam::vec3(0.0, 0.0, camera_z)).length() > 1.0e-4 {
                failures.push(format!(
                    "last_eye() position ({pos:?}) is not the injected one"
                ));
            }
            if (fwd - glam::vec3(0.0, 0.0, -1.0)).length() > 1.0e-4 {
                failures.push(format!("last_eye() forward ({fwd:?}) is not -Z"));
            }
        }
        None => failures.push("last_eye() is None after set_camera".to_owned()),
    }

    // --- oracle 3: レイヤーが画枠ちょうどに写ること ---
    //
    // 四隅の内側が各象限の色なら、画枠と document がぴったり重なっている。
    // どこかに Rerun の world grid や背景が見えていれば、ここで落ちる。
    const INSET: i32 = 6;
    let corners: [(&str, i32, i32, Hue); 4] = [
        ("top-left", INSET, INSET, Hue::Red),
        ("top-right", TARGET_W as i32 - 1 - INSET, INSET, Hue::Green),
        ("bottom-left", INSET, TARGET_H as i32 - 1 - INSET, Hue::Blue),
        (
            "bottom-right",
            TARGET_W as i32 - 1 - INSET,
            TARGET_H as i32 - 1 - INSET,
            Hue::Yellow,
        ),
    ];
    for (name, x, y, expected) in corners {
        let observed = shot.hue_at(x, y);
        report.push_str(&format!(
            "  frame corner {name} ({x}, {y}): {observed} (expected {expected})\n"
        ));
        if observed != expected {
            failures.push(format!(
                "frame corner {name} is {observed}, expected {expected} — the document does not fill the frame"
            ));
        }
    }

    // --- oracle 4: 格子点が期待座標の色と一致すること ---
    //
    // 期待 pixel は「レイヤーが画枠を満たす」ことだけから出す。Rerun の
    // `ui_from_world` は通さないので、これは描画に対する独立な照合である。
    //
    // 画像の行0は world の +y 側(`grid_map.rs:290-293` の extent_v = -Y)。
    // 左上/右上/左下/右下 = 赤/緑/青/黄。
    const STEPS: u32 = 24;
    // 象限境界と矩形の縁から離す幅。ここを跨ぐ点は「どちらの色でも正しい」ので測らない。
    const MARGIN: f32 = 0.06;
    let half_w = 0.5 * scene::LAYER_ASPECT;
    let quadrants: [(&str, f32, f32, Hue); 4] = [
        ("image top-left", -1.0, 1.0, Hue::Red),
        ("image top-right", 1.0, 1.0, Hue::Green),
        ("image bottom-left", -1.0, -1.0, Hue::Blue),
        ("image bottom-right", 1.0, -1.0, Hue::Yellow),
    ];
    for (name, sign_x, sign_y, expected) in quadrants {
        let (mut checked, mut wrong) = (0u32, 0u32);
        let mut first_wrong = None;
        for i in 0..STEPS {
            for j in 0..STEPS {
                let tx = MARGIN + (half_w - 2.0 * MARGIN) * (i as f32 / (STEPS - 1) as f32);
                let ty = MARGIN + (0.5 - 2.0 * MARGIN) * (j as f32 / (STEPS - 1) as f32);
                let (world_x, world_y) = (sign_x * tx, sign_y * ty);
                // document が画枠に一致しているので、写像はこの1行で決まる。
                let px = (world_x + half_w) / (2.0 * half_w) * TARGET_W as f32;
                let py = (0.5 - world_y) * TARGET_H as f32;
                let (x, y) = (px.round() as i32, py.round() as i32);
                if x < 0 || y < 0 || x >= TARGET_W as i32 || y >= TARGET_H as i32 {
                    wrong += 1;
                    first_wrong.get_or_insert(format!(
                        "world ({world_x:.4}, {world_y:.4}) -> expected pixel ({x}, {y}) is outside the frame"
                    ));
                    continue;
                }
                checked += 1;
                let observed = shot.hue_at(x, y);
                if observed != expected {
                    wrong += 1;
                    first_wrong.get_or_insert(format!(
                        "world ({world_x:.4}, {world_y:.4}) -> pixel ({x}, {y}) = {observed} {:?}",
                        shot.rgb(x, y)
                    ));
                }
            }
        }
        report.push_str(&format!(
            "  {name} ({expected}): {checked} samples landed on their expected pixel, {wrong} wrong{}\n",
            first_wrong
                .as_ref()
                .map_or_else(String::new, |detail| format!(" (first: {detail})")),
        ));
        if wrong > 0 {
            failures.push(format!(
                "{name}: {wrong} samples missed the expected mapping"
            ));
        }
    }

    // --- oracle 5: reset_view() で Rerun 既定の画へ戻ること ---
    stage.reset_view();
    for _ in 0..WARMUP_FRAMES {
        offscreen.frame(&mut stage)?;
    }
    let after_reset = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
    after_reset.write_png(&out_dir.join("e0-b-after-reset-view.png"))?;
    report.push_str(&format!(
        "\nreset_view(): fnv1a {:016x} ({} the injected camera, {} the default camera)\n",
        after_reset.digest(),
        if after_reset.digest() == shot.digest() {
            "SAME as"
        } else {
            "differs from"
        },
        if after_reset.digest() == before.digest() {
            "back to"
        } else {
            "differs from"
        },
    ));
    if after_reset.digest() == shot.digest() {
        failures.push("reset_view() left the injected camera in place".to_owned());
    }
    if after_reset.digest() != before.digest() {
        failures.push("reset_view() did not restore Rerun's own framing".to_owned());
    }

    if failures.is_empty() {
        report.push_str(
            "the injected document camera lands the layer exactly on the frame, \
             and reset_view() gives Rerun's framing back\n",
        );
        Ok(report)
    } else {
        Err(format!("{report}\n{}\n", failures.join("\n")))
    }
}

/// (c) 前後関係が遮蔽として成立するか。
///
/// oracle: 不透明な赤い矩形と、中央だけ不透明で外側 alpha=0 の緑の円板を前後に置く。
/// - 円板が手前 → 緑が見え、その周囲では赤が透けて見える
/// - 円板が奥   → 緑は**1画素も**出ない(不透明な赤に完全に隠される)
///
/// 2026-08-17の実測(§5)は「透明フェーズは深度を書かない」を報告していた。
/// 深度を書かないことが遮蔽の失敗を意味するなら、奥側の緑が漏れる形で出る。
fn probe_occlusion(out_dir: &Path) -> Result<String, String> {
    let mut report = String::new();
    let mut verdicts = Vec::new();

    for (label, disc_z, solid_z) in [
        ("disc-in-front", LAYER_Z_STEP, 0.0),
        ("disc-behind", 0.0, LAYER_Z_STEP),
    ] {
        let mut offscreen = Offscreen::new(TARGET_W, TARGET_H)?;
        let mut stage = new_stage("e0-occlusion")?;

        let solid = scene::upload(
            offscreen.device(),
            offscreen.queue(),
            "solid red",
            &scene::solid([230, 30, 30]),
        );
        let disc = scene::upload(
            offscreen.device(),
            offscreen.queue(),
            "green disc with transparent gutter",
            &scene::center_disc([30, 230, 30], 0.30),
        );
        stage
            .copy_gpu_image(offscreen.render_ctx(), "stack/solid/image", &solid)
            .map_err(|error| format!("copy_gpu_image(solid): {error}"))?;
        stage
            .copy_gpu_image(offscreen.render_ctx(), "stack/disc/image", &disc)
            .map_err(|error| format!("copy_gpu_image(disc): {error}"))?;
        scene::place_parent(&mut stage, "stack/solid", solid_z)?;
        scene::place_parent(&mut stage, "stack/disc", disc_z)?;
        // draw order は同一平面のときだけ効く鍵。ここでは z が違うので、
        // 効いてしまうと「距離ソートではなく draw order で決まっている」ことになる。
        // わざと「奥のレイヤーに大きい draw order」を与えて取り違えを検出する。
        scene::set_draw_order(&mut stage, "stack/solid/image", 0.0)?;
        scene::set_draw_order(&mut stage, "stack/disc/image", 10.0)?;

        for _ in 0..WARMUP_FRAMES {
            offscreen.frame(&mut stage)?;
        }
        let frame = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
        let png = out_dir.join(format!("e0-c-{label}.png"));
        frame.write_png(&png)?;

        let eye = stage
            .last_eye()
            .ok_or_else(|| "SpatialStage::last_eye() is None after warmup".to_owned())?;
        let eye_pos = eye.pos_in_world();
        let disc_distance = (eye_pos - glam::vec3(0.0, 0.0, disc_z - 0.01)).length();
        let solid_distance = (eye_pos - glam::vec3(0.0, 0.0, solid_z - 0.01)).length();
        let disc_is_nearer = disc_distance < solid_distance;

        let counts = frame.hue_histogram();
        report.push_str(&format!(
            "{label}: PNG {}\n  fnv1a {:016x}\n  eye.pos = ({:.4}, {:.4}, {:.4}); \
             disc z={disc_z} d={disc_distance:.4}, solid z={solid_z} d={solid_distance:.4} \
             -> disc is {}\n  hue histogram: red={} green={} dark={} other={}\n",
            png.display(),
            frame.digest(),
            eye_pos.x,
            eye_pos.y,
            eye_pos.z,
            if disc_is_nearer { "NEARER" } else { "FARTHER" },
            counts[0],
            counts[1],
            counts[4],
            counts[5],
        ));

        if counts[0] < 20_000 {
            return Err(format!(
                "{report}only {} red pixels — the opaque backdrop is missing, the scene is wrong\n",
                counts[0]
            ));
        }

        if disc_is_nearer {
            if counts[1] < 2_000 {
                verdicts.push(format!(
                    "{label}: the nearer disc is not visible ({} green pixels)",
                    counts[1]
                ));
            } else {
                report.push_str("  nearer disc composites over the backdrop: OK\n");
            }
        } else if counts[1] > 0 {
            verdicts.push(format!(
                "{label}: {} green pixels leaked through an opaque nearer layer — no occlusion",
                counts[1]
            ));
        } else {
            report.push_str("  farther disc is fully occluded by the opaque nearer layer: OK\n");
        }
    }

    if verdicts.is_empty() {
        report.push_str(
            "both orderings behave: the nearer layer wins and the transparent gutter lets the \
             farther one through\n",
        );
        Ok(report)
    } else {
        Err(format!("{report}{}\n", verdicts.join("\n")))
    }
}

fn new_stage(name: &str) -> Result<SpatialStage, String> {
    SpatialStage::new(ApplicationId::from(name))
        .map_err(|error| format!("create SpatialStage: {error}"))
}

#[cfg(test)]
mod tests {
    /// **fork の rev を上げたらここが落ちて教えてくれる**、というのがこのテストの役目。
    ///
    /// `SpatialStage::set_camera` で注入した document camera が、既知配置のレイヤーを
    /// 期待座標へ写すことを見る。期待値は画角と距離から我々の側で決めており、
    /// Rerun の `Eye` を通さない。窓は開かないし、実時間にも依存しない。
    ///
    /// GPU を使うので、adapter が取れない環境では skip する(CI 無しの前提)。
    #[test]
    fn injected_document_camera_maps_the_layer_onto_the_frame() {
        let out_dir = std::env::temp_dir().join("e0-camera-oracle");
        if let Err(error) = std::fs::create_dir_all(&out_dir) {
            panic!("could not create {}: {error}", out_dir.display());
        }

        match super::probe_camera(&out_dir) {
            Ok(report) => println!("{report}"),
            Err(report) if report.contains("select an adapter") => {
                eprintln!("skipped: no usable GPU adapter\n{report}");
            }
            Err(report) => panic!("injected camera oracle failed:\n{report}"),
        }
    }
}
