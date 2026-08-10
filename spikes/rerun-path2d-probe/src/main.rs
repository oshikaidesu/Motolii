use motolii_doc::{
    PointType,
    pathgeom::{Point, ResolvedPathOp, apply},
};
use rerun::external::re_sdk_types::{self, View as _};
use rerun::external::{re_log, re_log_channel, re_viewer, tokio};

mod path2d;
mod path_archetype;
mod path_renderer;
mod path_visualizer;

use path_archetype::Path2DFill;
use path_visualizer::Path2DVisualizer;
use path2d::{FillContribution, PlanarPaint, ShapeRecipe, sample_outline};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_thread = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();
    re_log::setup_logging();
    let recording = builtin_recording()?;
    re_viewer::run_native_app(
        main_thread,
        Box::new(move |cc| {
            let mut app = re_viewer::App::new(
                main_thread,
                re_viewer::build_info(),
                re_viewer::AppEnvironment::Custom("Motolii Path2D probe".to_owned()),
                re_viewer::StartupOptions::default(),
                cc,
                None,
                re_viewer::AsyncRuntimeHandle::from_current_tokio_runtime_or_wasmbindgen()
                    .expect("tokio runtime"),
            );
            app.add_log_receiver(recording);
            app.extend_view_class(
                re_sdk_types::blueprint::views::Spatial2DView::identifier(),
                |registrator| registrator.register_visualizer::<Path2DVisualizer>(),
            )?;
            Ok(Box::new(app))
        }),
        None,
    )?;
    Ok(())
}

fn builtin_recording() -> Result<re_log_channel::LogReceiver, rerun::RecordingStreamError> {
    let (recording, memory) =
        rerun::RecordingStreamBuilder::new("motolii_path2d_z0_overlap").memory()?;

    let rect = FillContribution {
        path: ShapeRecipe::Rect {
            center: Point { x: -0.15, y: 0.22 },
            size: Point { x: 1.2, y: 0.8 },
        }
        .lower(),
        paint: PlanarPaint::solid([1.0, 0.2, 0.4, 0.80]),
        draw_order: 0.0,
    };
    let circle = FillContribution {
        path: ShapeRecipe::Circle {
            center: Point { x: 0.25, y: 0.30 },
            radius: 0.48,
        }
        .lower(),
        paint: PlanarPaint {
            start: [-0.25, -0.18],
            end: [0.72, 0.78],
            start_color: [0.08, 0.32, 1.0, 0.82],
            end_color: [1.0, 0.25, 0.52, 0.82],
        },
        draw_order: 1.0,
    };

    recording.log_static(
        "shapes/rect",
        &Path2DFill::new(rect.encode().expect("valid rect Path2D")),
    )?;
    recording.log_static(
        "shapes/circle",
        &Path2DFill::new(circle.encode().expect("valid circle Path2D")),
    )?;

    let source = ShapeRecipe::Circle {
        center: Point { x: -0.65, y: -0.78 },
        radius: 0.28,
    }
    .lower();
    let pucker = apply(
        &ShapeRecipe::Circle {
            center: Point { x: 0.0, y: -0.78 },
            radius: 0.28,
        }
        .lower(),
        &ResolvedPathOp::PuckerBloat { amount: 0.55 },
        0.0,
    )
    .expect("valid Lottie pucker/bloat path");
    let burst = apply(
        &ShapeRecipe::Circle {
            center: Point { x: 0.65, y: -0.78 },
            radius: 0.28,
        }
        .lower(),
        &ResolvedPathOp::ZigZag {
            amount: 0.09,
            ridges: 2.0,
            point_type: PointType::Corner,
        },
        0.0,
    )
    .expect("valid zig-zag burst path");

    for (entity, path, color) in [
        (
            "effects/source",
            &source,
            rerun::Color::from_rgb(180, 190, 205),
        ),
        (
            "effects/pucker_bloat",
            &pucker,
            rerun::Color::from_rgb(255, 100, 160),
        ),
        (
            "effects/zig_zag_burst",
            &burst,
            rerun::Color::from_rgb(255, 190, 50),
        ),
    ] {
        recording.log_static(
            entity,
            &rerun::LineStrips2D::new([sample_outline(path).expect("closed effect Path2D")])
                .with_colors([color])
                .with_radii([rerun::Radius::new_ui_points(2.5)]),
        )?;
    }
    recording.log_static(
        "effects/labels",
        &rerun::Points2D::new([[-0.65, -0.38], [0.0, -0.38], [0.65, -0.38]])
            .with_labels(["Source", "Pucker / Bloat", "Zig Zag burst"])
            .with_radii([0.0]),
    )?;
    recording.log_static(
        "shapes/view_bounds",
        &rerun::Points2D::new([[-1.0, -1.25], [1.0, 0.9]])
            .with_radii([0.0])
            .with_colors([rerun::Color::from_unmultiplied_rgba(0, 0, 0, 0)]),
    )?;

    let (tx, rx) = re_log_channel::log_channel(re_log_channel::LogSource::Sdk);
    recording.flush_blocking().ok();
    for message in memory.take() {
        tx.send(re_log_channel::DataSourceMessage::LogMsg(message))
            .expect("forward builtin recording");
    }
    Ok(rx)
}
