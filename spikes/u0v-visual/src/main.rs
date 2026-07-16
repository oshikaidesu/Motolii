//! U0V visual spike — v2 mock shell in Slint + semantic tokens.

use std::path::PathBuf;

use slint::{ComponentHandle, Global};
use u0v_visual::{
    apply_theme_tokens, index_from_theme_id, load_theme_pref, load_theme_safe, manifest_dir,
    save_theme_pref, set_locale, theme_id_from_index, AppState, MainWindow, ThemeId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(dir) = std::env::var("U0V_EVIDENCE_DIR") {
        u0v_visual::structural_timeline_evidence(PathBuf::from(dir).as_path())?;
        if std::env::var_os("U0V_EVIDENCE_ONLY").is_some() {
            std::process::exit(0);
        }
    }

    let (gpu, parts) = motolii_gpu::GpuCtx::new_for_ui()?;
    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Manual {
            instance: parts.instance,
            adapter: parts.adapter,
            device: parts.device,
            queue: parts.queue,
        })
        .select()?;

    let pref_path = std::env::temp_dir().join("motolii-u0v-theme.json");
    let initial_theme = load_theme_pref(&pref_path).unwrap_or(ThemeId::MotoliiDark);
    let (theme, diag) = load_theme_safe(&manifest_dir(), initial_theme);

    let app = MainWindow::new()?;
    let state = AppState::get(&app);
    apply_theme_tokens(&app, &theme);
    state.set_theme_index(index_from_theme_id(initial_theme));
    if let Some(d) = diag {
        state.set_theme_diagnostic(d.into());
    }

    let timeline_tex = u0v_visual::render_timeline_for_theme(&gpu, &theme)?;
    if let Ok(img) = slint::Image::try_from(timeline_tex) {
        state.set_timeline_texture(img);
    }

    let preview = solid_preview_texture(&gpu, &theme)?;
    if let Ok(img) = slint::Image::try_from(preview) {
        state.set_preview_texture(img);
    }

    let app_weak = app.as_weak();
    let manifest = manifest_dir();
    state.on_theme_changed(move |idx| {
        let Some(app) = app_weak.upgrade() else { return };
        let state = AppState::get(&app);
        let id = theme_id_from_index(idx);
        let (theme, diag) = load_theme_safe(&manifest, id);
        apply_theme_tokens(&app, &theme);
        let _ = save_theme_pref(&pref_path, id);
        state.set_theme_diagnostic(diag.unwrap_or_default().into());
        if let Ok(g) = motolii_gpu::GpuCtx::new_for_ui().map(|(g, _)| g) {
            if let Ok(tex) = u0v_visual::render_timeline_for_theme(&g, &theme) {
                if let Ok(img) = slint::Image::try_from(tex) {
                    state.set_timeline_texture(img);
                }
            }
        }
    });

    let app_weak2 = app.as_weak();
    state.on_locale_changed(move |locale| {
        set_locale(locale.as_str());
        if let Some(app) = app_weak2.upgrade() {
            AppState::get(&app).set_locale(locale);
        }
    });

    set_locale("ja");
    app.run()?;
    Ok(())
}

fn solid_preview_texture(
    gpu: &motolii_gpu::GpuCtx,
    theme: &u0v_visual::ThemeTokens,
) -> Result<wgpu::Texture, String> {
    use u0v_visual::ResolvedToken;
    let bg = match theme.tokens.get("color.surface.vp-surround") {
        Some(ResolvedToken::Color(c)) => wgpu::Color {
            r: c.r as f64 / 255.0,
            g: c.g as f64 / 255.0,
            b: c.b as f64 / 255.0,
            a: c.a as f64,
        },
        _ => wgpu::Color {
            r: 0.08,
            g: 0.08,
            b: 0.08,
            a: 1.0,
        },
    };
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("u0v-preview"),
        size: wgpu::Extent3d {
            width: 640,
            height: 360,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(bg),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    gpu.queue.submit(Some(encoder.finish()));
    Ok(texture)
}
