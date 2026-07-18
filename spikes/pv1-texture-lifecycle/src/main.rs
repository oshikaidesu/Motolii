//! PV-1 spike GUI: Manual共有device + 保持texture + worker/content tick。
//! UI thread は render しない。人間審判は release build で実施。

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use motolii_gpu::GpuCtx;
use pv1_texture_lifecycle::{
    converge_mailbox_slot_poison, decide_ui_tick_status, format_status_snapshot, LatestSlot,
    LifecycleEngine, LifecycleError, LifecycleEvent, LifecycleState, Pv1Manifest, ResourceCounters,
    SlotTake, StatusSnapshot, UiTickStatusDecision, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};
use slint::wgpu_29::wgpu;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

slint::slint! {
    export component Pv1Window inherits Window {
        title: "PV-1 texture lifecycle spike";
        preferred-width: 800px;
        preferred-height: 520px;

        in-out property <image> preview-texture;
        in-out property <string> status-text: "booting";
        callback hide-requested();
        callback show-requested();
        callback minimize-requested();
        callback restore-requested();
        callback regenerate-requested();
        callback resize-requested(int, int);

        VerticalLayout {
            spacing: 8px;
            padding: 8px;
            Text { text: "PV-1 — 保持texture lifecycle (GPU Clear / resize時のみ再create)"; font-size: 14px; }
            Text { text: root.status-text; font-size: 12px; color: #a0a0b0; }
            Image {
                source: root.preview-texture;
                min-height: 360px;
                image-fit: contain;
            }
            HorizontalLayout {
                spacing: 8px;
                TouchArea {
                    width: 64px;
                    height: 28px;
                    clicked => { root.hide-requested(); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "Hide"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 64px; height: 28px;
                    clicked => { root.show-requested(); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "Show"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 80px; height: 28px;
                    clicked => { root.minimize-requested(); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "Minimize"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 72px; height: 28px;
                    clicked => { root.restore-requested(); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "Restore"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 96px; height: 28px;
                    clicked => { root.regenerate-requested(); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "Regenerate"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 120px; height: 28px;
                    clicked => { root.resize-requested(480, 270); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "480x270"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
                TouchArea {
                    width: 120px; height: 28px;
                    clicked => { root.resize-requested(640, 360); }
                    Rectangle { background: #3a3a48; border-radius: 4px;
                        Text { text: "640x360"; color: white; horizontal-alignment: center; vertical-alignment: center; }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct TextureEnvelope {
    generation: u64,
    texture: wgpu::Texture,
}

#[derive(Debug)]
enum UiCommand {
    Hide,
    Show,
    Minimize,
    Restore,
    Regenerate,
    Resize { width: u32, height: u32 },
    DisplayBound { generation: u64 },
    DisplayBindFailed { generation: u64 },
    SlotPoisoned,
    Shutdown,
}

struct WorkerShutdownSnapshot {
    state: LifecycleState,
    counters: ResourceCounters,
}

fn unix_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn write_manifest_skeleton(
    out_dir: &std::path::Path,
    counters: ResourceCounters,
    state: LifecycleState,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out_dir)?;
    let path = out_dir.join("manifest-skeleton.json");
    let mut manifest = match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str::<Pv1Manifest>(&contents)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Pv1Manifest::skeleton_template(),
        Err(e) => return Err(e.into()),
    };
    manifest.record_run(unix_timestamp(), counters, state);
    std::fs::write(&path, serde_json::to_string_pretty(&manifest)?)?;
    eprintln!("pv1 evidence: wrote {}", path.display());
    Ok(())
}

fn publish_status(
    engine: &mut LifecycleEngine,
    status_slot: &LatestSlot<StatusSnapshot>,
    last_error: Option<LifecycleError>,
) {
    // converge_mailbox_slot_poison 済みの fatal snapshot（SlotPoisoned）を None 上書きしない
    if engine.state() == LifecycleState::Failed && last_error.is_none() {
        return;
    }
    let snapshot = StatusSnapshot {
        state: engine.state(),
        counters: engine.counters(),
        last_error,
    };
    if let Err(LifecycleError::SlotPoisoned) = status_slot.replace(snapshot) {
        converge_mailbox_slot_poison(engine, status_slot);
    }
}

fn handle_slot_poison(engine: &mut LifecycleEngine, status_slot: &LatestSlot<StatusSnapshot>) {
    converge_mailbox_slot_poison(engine, status_slot);
}

fn publish_texture(
    engine: &mut LifecycleEngine,
    texture_slot: &LatestSlot<TextureEnvelope>,
    status_slot: &LatestSlot<StatusSnapshot>,
    generation: u64,
    texture: wgpu::Texture,
) {
    if let Err(LifecycleError::SlotPoisoned) = texture_slot.replace(TextureEnvelope {
        generation,
        texture,
    }) {
        handle_slot_poison(engine, status_slot);
    }
}

fn send_ui_command(
    tx: &mpsc::Sender<UiCommand>,
    cmd: UiCommand,
    app: &Pv1Window,
    command_channel_failed: &Cell<bool>,
) {
    if tx.send(cmd).is_err() {
        command_channel_failed.set(true);
        app.set_status_text(format!("FATAL: {}", LifecycleError::CommandChannelClosed).into());
    }
}

fn spawn_worker(
    gpu: Arc<GpuCtx>,
    cmd_rx: mpsc::Receiver<UiCommand>,
    texture_slot: Arc<LatestSlot<TextureEnvelope>>,
    status_slot: Arc<LatestSlot<StatusSnapshot>>,
) -> JoinHandle<WorkerShutdownSnapshot> {
    std::thread::spawn(move || {
        let mut engine = match LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT) {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("pv1 worker: LifecycleEngine init failed: {e}");
                return WorkerShutdownSnapshot {
                    state: LifecycleState::Failed,
                    counters: ResourceCounters::default(),
                };
            }
        };

        engine.boot_to_ready();
        if let Ok(texture) = engine.handoff_display_texture() {
            if let Some(generation) = engine.texture_generation() {
                publish_texture(
                    &mut engine,
                    &texture_slot,
                    &status_slot,
                    generation,
                    texture,
                );
            }
        }
        publish_status(&mut engine, &status_slot, None);

        let tick = Duration::from_millis(33);
        loop {
            if engine.state() == LifecycleState::Failed {
                std::thread::sleep(tick);
                while let Ok(cmd) = cmd_rx.try_recv() {
                    if matches!(cmd, UiCommand::Shutdown) {
                        return WorkerShutdownSnapshot {
                            state: engine.state(),
                            counters: engine.counters(),
                        };
                    }
                }
                continue;
            }

            let mut had_command = false;
            while let Ok(cmd) = cmd_rx.try_recv() {
                had_command = true;
                match cmd {
                    UiCommand::Shutdown => {
                        return WorkerShutdownSnapshot {
                            state: engine.state(),
                            counters: engine.counters(),
                        };
                    }
                    UiCommand::SlotPoisoned => {
                        handle_slot_poison(&mut engine, &status_slot);
                    }
                    UiCommand::Hide => match engine.apply_event(&gpu, LifecycleEvent::Hide) {
                        Ok(_) => {
                            publish_status(&mut engine, &status_slot, None);
                        }
                        Err(e) => {
                            publish_status(&mut engine, &status_slot, Some(e));
                        }
                    },
                    UiCommand::Show => match engine.apply_event(&gpu, LifecycleEvent::Show) {
                        Ok(_) => {
                            publish_status(&mut engine, &status_slot, None);
                        }
                        Err(e) => {
                            publish_status(&mut engine, &status_slot, Some(e));
                        }
                    },
                    UiCommand::Minimize => {
                        match engine.apply_event(&gpu, LifecycleEvent::Minimize) {
                            Ok(_) => {
                                publish_status(&mut engine, &status_slot, None);
                            }
                            Err(e) => {
                                publish_status(&mut engine, &status_slot, Some(e));
                            }
                        }
                    }
                    UiCommand::Restore => match engine.apply_event(&gpu, LifecycleEvent::Restore) {
                        Ok(_) => {
                            publish_status(&mut engine, &status_slot, None);
                        }
                        Err(e) => {
                            publish_status(&mut engine, &status_slot, Some(e));
                        }
                    },
                    UiCommand::Regenerate => {
                        match engine.apply_event(&gpu, LifecycleEvent::Regenerate) {
                            Ok(outcome) => {
                                if outcome.needs_image_rebind {
                                    if let Ok(texture) = engine.handoff_display_texture() {
                                        if let Some(generation) = engine.texture_generation() {
                                            publish_texture(
                                                &mut engine,
                                                &texture_slot,
                                                &status_slot,
                                                generation,
                                                texture,
                                            );
                                        }
                                    }
                                }
                                publish_status(&mut engine, &status_slot, None);
                            }
                            Err(e) => {
                                publish_status(&mut engine, &status_slot, Some(e));
                            }
                        }
                    }
                    UiCommand::Resize { width, height } => {
                        match engine.apply_event(&gpu, LifecycleEvent::Resize { width, height }) {
                            Ok(outcome) => {
                                if outcome.needs_image_rebind {
                                    if let Ok(texture) = engine.handoff_display_texture() {
                                        if let Some(generation) = engine.texture_generation() {
                                            publish_texture(
                                                &mut engine,
                                                &texture_slot,
                                                &status_slot,
                                                generation,
                                                texture,
                                            );
                                        }
                                    }
                                }
                                publish_status(&mut engine, &status_slot, None);
                            }
                            Err(e) => {
                                publish_status(&mut engine, &status_slot, Some(e));
                            }
                        }
                    }
                    UiCommand::DisplayBound { generation } => {
                        if let Err(e) = engine.record_display_bound(generation) {
                            publish_status(&mut engine, &status_slot, Some(e));
                        } else {
                            publish_status(&mut engine, &status_slot, None);
                        }
                    }
                    UiCommand::DisplayBindFailed { generation } => {
                        engine.record_display_bind_failed(generation);
                        publish_status(
                            &mut engine,
                            &status_slot,
                            Some(LifecycleError::ImageBindFailed),
                        );
                    }
                }
            }

            if engine.state() != LifecycleState::Failed {
                match engine.apply_event(&gpu, LifecycleEvent::ContentTick) {
                    Ok(_) => {
                        if had_command {
                            publish_status(&mut engine, &status_slot, None);
                        }
                    }
                    Err(e) => {
                        publish_status(&mut engine, &status_slot, Some(e));
                    }
                }
            }
            std::thread::sleep(tick);
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    if let Some(info) = &gpu.adapter_info {
        eprintln!("pv1 adapter: {} ({:?})", info.name, info.backend);
    }

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Manual {
            instance: parts.instance,
            adapter: parts.adapter,
            device: parts.device,
            queue: parts.queue,
        })
        .select()?;

    let app = Pv1Window::new()?;
    let window = app.window();
    window.set_size(slint::PhysicalSize::new(800, 520));

    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCommand>();
    let texture_slot: Arc<LatestSlot<TextureEnvelope>> = Arc::new(LatestSlot::new());
    let status_slot: Arc<LatestSlot<StatusSnapshot>> = Arc::new(LatestSlot::new());

    let gpu = Arc::new(gpu);
    let worker_handle = spawn_worker(
        Arc::clone(&gpu),
        cmd_rx,
        Arc::clone(&texture_slot),
        Arc::clone(&status_slot),
    );

    let app_weak = app.as_weak();
    let cmd_tx_timer = cmd_tx.clone();
    let texture_slot_timer = Arc::clone(&texture_slot);
    let status_slot_timer = Arc::clone(&status_slot);
    let last_window_size: Cell<Option<(u32, u32)>> = Cell::new(None);
    let command_channel_failed = Rc::new(Cell::new(false));
    let command_channel_failed_timer = Rc::clone(&command_channel_failed);

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };

            let size = app.window().size();
            let observed = (size.width, size.height);
            let prev = last_window_size.get();
            if prev != Some(observed) {
                last_window_size.set(Some(observed));
                if prev.is_some() {
                    send_ui_command(
                        &cmd_tx_timer,
                        UiCommand::Resize {
                            width: observed.0,
                            height: observed.1,
                        },
                        &app,
                        &command_channel_failed_timer,
                    );
                }
            }

            let texture_take = texture_slot_timer.try_take();
            let status_take = status_slot_timer.try_take();

            let texture_poisoned = matches!(texture_take, SlotTake::Poisoned);
            let status_poisoned = matches!(status_take, SlotTake::Poisoned);

            if let SlotTake::Item(envelope) = texture_take {
                let generation = envelope.generation;
                match slint::Image::try_from(envelope.texture) {
                    Ok(img) => {
                        app.set_preview_texture(img);
                        send_ui_command(
                            &cmd_tx_timer,
                            UiCommand::DisplayBound { generation },
                            &app,
                            &command_channel_failed_timer,
                        );
                    }
                    Err(_) => {
                        send_ui_command(
                            &cmd_tx_timer,
                            UiCommand::DisplayBindFailed { generation },
                            &app,
                            &command_channel_failed_timer,
                        );
                    }
                }
            }

            if texture_poisoned || status_poisoned {
                send_ui_command(
                    &cmd_tx_timer,
                    UiCommand::SlotPoisoned,
                    &app,
                    &command_channel_failed_timer,
                );
            }

            let status_line = match status_take {
                SlotTake::Item(ref snapshot) => Some(format_status_snapshot(snapshot)),
                _ => None,
            };

            match decide_ui_tick_status(
                command_channel_failed_timer.get(),
                texture_poisoned,
                status_poisoned,
                status_line,
            ) {
                UiTickStatusDecision::KeepPrevious => {}
                UiTickStatusDecision::ShowStatus(line) => {
                    app.set_status_text(line.into());
                }
                UiTickStatusDecision::ShowFatal(err) => {
                    app.set_status_text(format!("FATAL: {err}").into());
                }
            }
        },
    );

    let cmd_hide = cmd_tx.clone();
    let app_weak_hide = app.as_weak();
    let command_channel_failed_hide = Rc::clone(&command_channel_failed);
    app.on_hide_requested(move || {
        if let Some(app) = app_weak_hide.upgrade() {
            send_ui_command(
                &cmd_hide,
                UiCommand::Hide,
                &app,
                &command_channel_failed_hide,
            );
            let _ = app.window().hide();
            let cmd = cmd_hide.clone();
            let weak = app_weak_hide.clone();
            let command_channel_failed = Rc::clone(&command_channel_failed_hide);
            slint::Timer::single_shot(Duration::from_millis(400), move || {
                if let Some(w) = weak.upgrade() {
                    let _ = w.window().show();
                    send_ui_command(&cmd, UiCommand::Show, &w, &command_channel_failed);
                }
            });
        }
    });

    let cmd_show = cmd_tx.clone();
    let app_weak_show = app.as_weak();
    let command_channel_failed_show = Rc::clone(&command_channel_failed);
    app.on_show_requested(move || {
        if let Some(app) = app_weak_show.upgrade() {
            send_ui_command(
                &cmd_show,
                UiCommand::Show,
                &app,
                &command_channel_failed_show,
            );
            let _ = app.window().show();
        }
    });

    let cmd_min = cmd_tx.clone();
    let app_weak_minimize = app.as_weak();
    let command_channel_failed_minimize = Rc::clone(&command_channel_failed);
    app.on_minimize_requested(move || {
        if let Some(app) = app_weak_minimize.upgrade() {
            send_ui_command(
                &cmd_min,
                UiCommand::Minimize,
                &app,
                &command_channel_failed_minimize,
            );
            app.window().set_minimized(true);
            let cmd = cmd_min.clone();
            let weak = app_weak_minimize.clone();
            let command_channel_failed = Rc::clone(&command_channel_failed_minimize);
            slint::Timer::single_shot(Duration::from_millis(400), move || {
                if let Some(w) = weak.upgrade() {
                    w.window().set_minimized(false);
                    send_ui_command(&cmd, UiCommand::Restore, &w, &command_channel_failed);
                }
            });
        }
    });

    let cmd_restore = cmd_tx.clone();
    let app_weak_restore = app.as_weak();
    let command_channel_failed_restore = Rc::clone(&command_channel_failed);
    app.on_restore_requested(move || {
        if let Some(app) = app_weak_restore.upgrade() {
            send_ui_command(
                &cmd_restore,
                UiCommand::Restore,
                &app,
                &command_channel_failed_restore,
            );
            app.window().set_minimized(false);
        }
    });

    let cmd_regen = cmd_tx.clone();
    let app_weak_regen = app.as_weak();
    let command_channel_failed_regen = Rc::clone(&command_channel_failed);
    app.on_regenerate_requested(move || {
        if let Some(app) = app_weak_regen.upgrade() {
            send_ui_command(
                &cmd_regen,
                UiCommand::Regenerate,
                &app,
                &command_channel_failed_regen,
            );
        }
    });

    let cmd_resize = cmd_tx.clone();
    let app_weak_resize = app.as_weak();
    let command_channel_failed_resize = Rc::clone(&command_channel_failed);
    app.on_resize_requested(move |w, h| {
        if let Some(app) = app_weak_resize.upgrade() {
            send_ui_command(
                &cmd_resize,
                UiCommand::Resize {
                    width: w as u32,
                    height: h as u32,
                },
                &app,
                &command_channel_failed_resize,
            );
        }
    });

    app.window().on_close_requested(|| {
        if let Err(e) = slint::quit_event_loop() {
            eprintln!("pv1: event loop quit failed: {e}");
        }
        slint::CloseRequestResponse::HideWindow
    });
    app.show()?;
    slint::run_event_loop_until_quit()?;
    cmd_tx.send(UiCommand::Shutdown)?;
    let snapshot = worker_handle
        .join()
        .map_err(|_| "pv1 worker thread panicked during join")?;

    if let Ok(dir) = std::env::var("PV1_EVIDENCE_DIR") {
        write_manifest_skeleton(
            PathBuf::from(&dir).as_path(),
            snapshot.counters,
            snapshot.state,
        )?;
    }

    Ok(())
}
