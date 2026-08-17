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
use re_view_spatial::SpatialStage;

const TARGET_W: u32 = 640;
const TARGET_H: u32 = 480;
/// bounding box の平滑化とカメラ補間が落ち着くまで回す。
/// 決定性のため実時間ではなく固定 dt で数える(`harness::Offscreen::frame`)。
const WARMUP_FRAMES: u32 = 90;

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
/// oracle は2段になっている。
/// 1. 投影の一致: `SpatialStage::last_eye()` が返すカメラで4象限の中心を投影し、
///    その pixel が本当にその象限の色であること。カメラモデルと実際の描画が一致する証拠。
/// 2. 注入: `focus_entity` / `reset_view` / `Pinhole` tracking のいずれかで
///    カメラが動くこと。
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
    // document camera 相当。Rerun のカメラ機構へ外注できるかを見るために立てておく。
    scene::place_pinhole_camera(&mut stage, "stack/camera", [0.0, 0.0, 2.5])?;

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

    // --- oracle 1: カメラモデルと描画の一致 ---
    //
    // 4隅を1点ずつ見るのでは、既定カメラの画角次第で点が画面外へ出る(実際に出た)。
    // 代わりに各象限を格子状に舐め、「画面内へ落ちた点はすべてその象限の色である」
    // を要求する。カメラが1つでも狂っていれば、境界をまたいだ点が別の色を返す。
    let ui_from_world = eye.ui_from_world(offscreen.ui_rect());
    let half_w = 0.5 * scene::LAYER_ASPECT;
    // 象限境界と矩形の縁から離す幅。ここを跨ぐ点は「どちらの色でも正しい」ので測らない。
    const MARGIN: f32 = 0.06;
    // 既定カメラの画角はレイヤー全体を収めない(手前側の角が画面下へ出る)。
    // 収まらない点は数えないので、どの象限も判定に足る数が残るよう密に舐める。
    const STEPS: u32 = 24;
    // 画像の行0は world の +y 側(`grid_map.rs:290-293` の extent_v = -Y)。
    // 左上/右上/左下/右下 = 赤/緑/青/黄。
    let quadrants: [(&str, f32, f32, Hue); 4] = [
        ("image top-left", -1.0, 1.0, Hue::Red),
        ("image top-right", 1.0, 1.0, Hue::Green),
        ("image bottom-left", -1.0, -1.0, Hue::Blue),
        ("image bottom-right", 1.0, -1.0, Hue::Yellow),
    ];
    let mut projection_failures = Vec::new();
    for (name, sign_x, sign_y, expected) in quadrants {
        let (mut on_screen, mut off_screen, mut wrong) = (0u32, 0u32, 0u32);
        let mut first_wrong = None;
        for i in 0..STEPS {
            for j in 0..STEPS {
                let tx = MARGIN
                    + (half_w - 2.0 * MARGIN) * (f32::from(i as u16) / f32::from(STEPS as u16 - 1));
                let ty = MARGIN
                    + (0.5 - 2.0 * MARGIN) * (f32::from(j as u16) / f32::from(STEPS as u16 - 1));
                // `copy_gpu_image` は自分の path へ z=-0.01 を書く。面はそこにある。
                let world = glam::vec3(sign_x * tx, sign_y * ty, -0.01);
                let ui = ui_from_world.project_point3(world);
                let (x, y) = (ui.x.round() as i32, ui.y.round() as i32);
                if x < 1 || y < 1 || x >= TARGET_W as i32 - 1 || y >= TARGET_H as i32 - 1 {
                    off_screen += 1;
                    continue;
                }
                on_screen += 1;
                let observed = before.hue_at(x, y);
                if observed != expected {
                    wrong += 1;
                    first_wrong.get_or_insert(format!(
                        "world ({:.4}, {:.4}) -> pixel ({x}, {y}) = {observed} {:?}",
                        world.x,
                        world.y,
                        before.rgb(x, y)
                    ));
                }
            }
        }
        report.push_str(&format!(
            "  {name} ({expected}): {on_screen} samples inside the viewport, {off_screen} outside, {wrong} wrong{}\n",
            first_wrong
                .as_ref()
                .map_or_else(String::new, |detail| format!(" (first: {detail})")),
        ));
        if on_screen < 30 {
            projection_failures.push(format!(
                "{name}: only {on_screen} samples landed inside the viewport — cannot judge"
            ));
        }
        if wrong > 0 {
            projection_failures.push(format!("{name}: {wrong} samples hit the wrong colour"));
        }
    }

    // --- oracle 2: カメラ注入 ---
    // `SpatialStage` の公開APIでカメラへ触れるのは focus_entity / reset_view の2つだけ。
    // Pinhole を立ててあるので、tracking へ切り替わるかもここで見る。
    let mut injection = Vec::new();
    for (label, action) in [
        ("focus_entity(layer)", 0u8),
        ("focus_entity(pinhole camera)", 1),
        ("reset_view()", 2),
    ] {
        match action {
            0 => stage.focus_entity("stack/main/image"),
            1 => stage.focus_entity("stack/camera"),
            _ => stage.reset_view(),
        }
        for _ in 0..WARMUP_FRAMES {
            offscreen.frame(&mut stage)?;
        }
        let after = Frame::new(offscreen.read_rgba()?, TARGET_W, TARGET_H);
        let after_eye = stage.last_eye();
        let moved = after.digest() != before.digest();
        let pos = after_eye.map(|eye| eye.pos_in_world());
        report.push_str(&format!(
            "  {label}: fnv1a {:016x} ({}), eye.pos = {}\n",
            after.digest(),
            if moved { "CHANGED" } else { "identical" },
            pos.map_or_else(
                || "None".to_owned(),
                |p| format!("({:.4}, {:.4}, {:.4})", p.x, p.y, p.z)
            ),
        ));
        after.write_png(&out_dir.join(format!("e0-b-after-{action}.png")))?;
        if moved {
            injection.push(label);
        }
    }

    if !projection_failures.is_empty() {
        return Err(format!(
            "{report}projection mismatch: {}\n",
            projection_failures.join("; ")
        ));
    }
    report.push_str(
        "camera model matches the rendered pixels: the projected corners land on their own quadrant\n",
    );

    if injection.is_empty() {
        return Err(format!(
            "{report}no public API moved the camera. \
             `focus_entity`/`reset_view` both end in `ViewProperty::save_blueprint_component`, \
             which sends `SystemCommand::AppendToStore`; `SpatialStage::process_system_commands` \
             drops it (spatial_stage.rs:154-175, `_ => {{}}` arm). \
             There is also no blueprint ingest: `ingest_chunk` only targets `recording_store_id` \
             (spatial_stage.rs:182-185).\n"
        ));
    }
    report.push_str(&format!("camera moved via: {}\n", injection.join(", ")));
    Ok(report)
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
