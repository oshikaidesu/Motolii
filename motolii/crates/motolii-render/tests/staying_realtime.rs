//! 毎コマの道が「時間の読めない呼び出し」をしていないか。**測る前に落ちる。**
//!
//! 定規は外の物 — LLVM 20 の RealtimeSanitizer(`rtsan-standalone` 0.3、prebuilt な
//! compiler-rt を link)。`ScopedSanitizeRealtime` の間に `malloc` / `free` /
//! `pthread_mutex_lock` / I/O などが走ると、その場で stderr に stack を吐く。
//! 呼ぶ側で囲むので、engine 側の file は 1 行も触らない — call tree 全部が対象。
//!
//! 既定は**何もしない**(build.rs が `RTSAN_ENABLE` を見て初めて runtime を繋ぐ)ので、
//! 普通の `cargo test` では 1 コマ描くだけの煙試験になる。囲いを効かせるには:
//!
//! ```text
//! RTSAN_ENABLE=1 cargo test -p motolii-render --test staying_realtime
//! # 全部並べる(最初の 1 件で止めない):
//! RTSAN_ENABLE=1 RTSAN_OPTIONS=halt_on_error=false \
//!   cargo test -p motolii-render --test staying_realtime -- --nocapture
//! # 依存の雑音を抑える:
//! RTSAN_OPTIONS=suppressions=$PWD/motolii/reference/rtsan-suppressions.txt
//! ```

use motolii_doc::store::{
    rect_shape, Composition, EffectId, EffectInstance, Fps, Interp, Keyframe, KeyframeTrack,
    LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value, property,
};
use motolii_edit::{Document, Intent};
use motolii_render::engine::Engine;

const FRAMES: i64 = 12;

fn fps() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

/// 絵になる物を一通り: 動く四角が 3 枚、うち 1 枚に同梱の効果。
fn document() -> Document {
    let mut doc = Document::new().with_programs(motolii_render::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 480,
        fps: fps(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    for id in 1..=3u64 {
        let layer = LayerId(id);
        let track = KeyframeTrack::try_from_keys(vec![
            Keyframe { t: RationalTime::ZERO, value: Value::Vec2([0.0, 0.0]), interp: Interp::Linear, spatial: None },
            Keyframe { t: RationalTime::try_new(1, 1).unwrap(), value: Value::Vec2([200.0, 120.0]), interp: Interp::Linear, spatial: None },
        ])
        .unwrap();
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [120.0, 80.0])] },
            Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track },
        ])
        .unwrap();
    }
    doc.apply(Intent::SetEffects {
        layer: LayerId(2),
        effects: vec![EffectInstance { id: EffectId(1), plugin_id: "motolii.gain".into() }],
    })
    .unwrap();
    doc
}

/// Stage が毎コマ通る道(readback 無しの preview)を囲む。
#[test]
fn a_preview_frame_makes_no_unpredictable_call() {
    let doc = document();
    let mut engine = Engine::new().unwrap();
    engine.set_realtime(true);
    let device = engine.gpu_device().clone();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("staying-realtime"),
        size: wgpu::Extent3d { width: 640, height: 480, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: motolii_render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    // 温め: 初回の shader compile・texture 確保は毎コマの道ではない。囲いの外で済ます。
    let warm = RationalTime::try_from_frame(0, fps()).unwrap();
    engine.render_frame_into(&doc.view(), warm, &target).unwrap();
    rtsan_standalone::ensure_initialized();

    for frame in 1..FRAMES {
        let t = RationalTime::try_from_frame(frame, fps()).unwrap();
        let view = doc.view();
        {
            let _realtime = rtsan_standalone::ScopedSanitizeRealtime::default();
            engine.render_frame_into(&view, t, &target).unwrap();
        }
    }
}
