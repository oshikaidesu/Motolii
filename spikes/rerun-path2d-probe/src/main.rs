use motolii_doc::pathgeom::Point;
use rerun::external::re_sdk_types::{self, View as _};
use rerun::external::{re_log, re_log_channel, re_viewer, tokio};

mod path2d;
mod path_archetype;
mod path_renderer;
mod path_visualizer;

use path_archetype::Path2DFill;
use path_visualizer::Path2DVisualizer;
use path2d::{FillContribution, ShapeRecipe};

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
            center: Point { x: -0.15, y: 0.0 },
            size: Point { x: 1.2, y: 0.8 },
        }
        .lower(),
        color: [1.0, 0.2, 0.4, 0.80],
        draw_order: 0.0,
    };
    let circle = FillContribution {
        path: ShapeRecipe::Circle {
            center: Point { x: 0.25, y: 0.08 },
            radius: 0.48,
        }
        .lower(),
        color: [0.1, 0.6, 1.0, 0.65],
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
    recording.log_static(
        "shapes/view_bounds",
        &rerun::Points2D::new([[-0.9, -0.65], [0.9, 0.65]])
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
