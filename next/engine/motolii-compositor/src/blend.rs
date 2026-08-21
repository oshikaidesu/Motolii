//! 分離可能(per-channel)blend 11 モード(BL3、裁定161 fork accessor の main_target
//! 経路を使う2番目の実装、`effects::glow` と同じ「`motolii-compositor` 内へ新規 WGSL
//! パイプラインを足す」手口——fork は無改造)。
//!
//! ## なぜ固定式(`RectangleOptions::multiplicative_tint`)で表現できないか
//!
//! [`crate::BlendMode::Normal`]/[`crate::BlendMode::Add`] は `out = src*Fa + dst*Fb`
//! (係数だけが変わる Porter-Duff)なので `multiplicative_tint` 1個で足りたが、
//! Multiply 以降は `out` が `src` と `dst` の**積・比・条件分岐**を要る——
//! 固定機能 blend の「係数×src + 係数×dst」の外にある(`crate` module doc 旧節参照)。
//! ここでは2枚の texture(dst=直前までの accumulator・src=layer 単体を描いた
//! canvas)を読む fullscreen パスを1本足して解決する。
//!
//! ## 数式の出典: W3C Compositing and Blending Level 1
//! (<https://www.w3.org/TR/compositing-1/>)の separable blend 関数(3.6節)+
//! 一般合成式(3.5節)をそのまま実装する。straight(非 premultiplied)色 `Cb`(backdrop)・
//! `Cs`(source)・alpha `αb`/`αs` について:
//!
//! ```text
//! B(Cb, Cs)  ... モードごとの per-channel 関数(下記 WGSL 参照)
//! Co = αs·(1-αb)·Cs + αs·αb·B(Cb,Cs) + (1-αs)·αb·Cb   (premultiplied 出力)
//! αo = αs + αb·(1-αs)
//! ```
//!
//! `αb=0`(まだ何も accumulator に無い)時は右辺の後ろ2項が消え `Co = αs·Cs` —
//! 「1枚目は普通に描くだけ」という [`crate::Compositor::render_sequential`] 既存の
//! 早期分岐と数学的に一致する(このモジュールは `background` が `Some` の時にしか
//! 呼ばれない設計、呼び出し側 `crate::lib` 参照)。
//!
//! ## Soft Light の分岐について
//!
//! Photoshop の SoftLight とは **式が異なる**(W3C 版は `D(x)` に3次多項式+平方根の
//! 滑らかな接続を使うが、Photoshop 版は `sqrt` のみ・分岐点も違う)。ここでは
//! 発注書どおり **W3C 版を正**とする——Lottie/AE のドキュメント化された挙動が
//! この式系列(CSS `mix-blend-mode`/SVG compositing と同じ出典)に従うため。
//!
//! ## gamma 空間
//!
//! 入出力とも `ViewBuilder::MAIN_TARGET_COLOR_FORMAT`(`Rgba8UnormSrgb`)——
//! `textureLoad`/render target への書き込みとも GPU が自動で sRGB decode/encode する
//! (`crate` module doc の「色空間の注意」節と同じ前提)。ソフトウェアで2重に
//! 変換しない。

/// `textureLoad`/書き込みの出入力が乗る format。呼び出し側([`crate`]の
/// `accumulate_sequential`)が新規に確保する scratch texture もこれに揃える——
/// 直前までの accumulator([`re_renderer::view_builder::ViewBuilder::main_target`])と
/// 同じ format でないと、次のイテレーションで再び `textureLoad` した時に
/// 暗黙の decode が起きたり起きなかったりして数値が壊れる。
pub(crate) const SEPARABLE_BLEND_TARGET_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8UnormSrgb;

pub(crate) struct SeparableBlendPipelines {
    pipeline: wgpu::RenderPipeline,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl SeparableBlendPipelines {
    /// **初回生成して以後使い回す**(`effects::GlowPipelines::new` と同じ規律 —
    /// `Compositor::with_device` が1回だけ呼ぶ)。
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-blend-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-blend-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-blend-params-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-compositor-blend-pipeline-layout"),
            bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-compositor-blend-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("blend_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: SEPARABLE_BLEND_TARGET_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            two_texture_layout,
            params_layout,
        }
    }

    /// 1パスを `encoder` へ積む。`dst_view`=直前までの accumulator(backdrop)・
    /// `src_view`=layer 単体を描いた canvas(source)・`out_view`=結果の書き込み先
    /// (呼び手が確保した [`SEPARABLE_BLEND_TARGET_FORMAT`] の texture)。`mode` は
    /// [`crate::separable_mode_index`] が返す index(WGSL 側の `switch` と対応)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        dst_view: &wgpu::TextureView,
        src_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        mode: u32,
    ) {
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-blend-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&mode.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-blend-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-blend-texture-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(dst_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(src_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-blend-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: out_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &params_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

/// `effects::glow` の `SHADER` と同じ形(fullscreen triangle `vs_main` + 1 fragment)。
/// `params.mode` の値は [`crate::separable_mode_index`] の返り値と1対1(0=Multiply〜
/// 10=Exclusion、モジュール doc「数式の出典」参照)。
const SHADER: &str = r#"
struct BlendParams {
  mode: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var dst_tex: texture_2d<f32>;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: BlendParams;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

fn blend_channel(mode: u32, cb: f32, cs: f32) -> f32 {
  if (mode == 0u) {
    // Multiply
    return cs * cb;
  }
  if (mode == 1u) {
    // Screen
    return cs + cb - cs * cb;
  }
  if (mode == 2u) {
    // Overlay = HardLight(Cb, Cs) との式は同じで backdrop/source の役が入れ替わる
    if (cb <= 0.5) {
      return 2.0 * cs * cb;
    }
    return 1.0 - 2.0 * (1.0 - cs) * (1.0 - cb);
  }
  if (mode == 3u) {
    // Darken
    return min(cb, cs);
  }
  if (mode == 4u) {
    // Lighten
    return max(cb, cs);
  }
  if (mode == 5u) {
    // ColorDodge
    if (cb <= 0.0) {
      return 0.0;
    }
    if (cs >= 1.0) {
      return 1.0;
    }
    return min(1.0, cb / (1.0 - cs));
  }
  if (mode == 6u) {
    // ColorBurn
    if (cb >= 1.0) {
      return 1.0;
    }
    if (cs <= 0.0) {
      return 0.0;
    }
    return 1.0 - min(1.0, (1.0 - cb) / cs);
  }
  if (mode == 7u) {
    // HardLight
    if (cs <= 0.5) {
      return 2.0 * cs * cb;
    }
    return 1.0 - 2.0 * (1.0 - cs) * (1.0 - cb);
  }
  if (mode == 8u) {
    // SoftLight(W3C 版、モジュール doc「Soft Light の分岐について」参照)
    if (cs <= 0.5) {
      return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    var d: f32;
    if (cb <= 0.25) {
      d = ((16.0 * cb - 12.0) * cb + 4.0) * cb;
    } else {
      d = sqrt(cb);
    }
    return cb + (2.0 * cs - 1.0) * (d - cb);
  }
  if (mode == 9u) {
    // Difference
    return abs(cb - cs);
  }
  // Exclusion(mode == 10u、default もここへ落ちる)
  return cs + cb - 2.0 * cs * cb;
}

@fragment
fn blend_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let dst = textureLoad(dst_tex, vec2<i32>(p.xy), 0);
  let src = textureLoad(src_tex, vec2<i32>(p.xy), 0);

  let alpha_b = dst.a;
  let alpha_s = src.a;

  var cb = vec3<f32>(0.0);
  if (alpha_b > 0.0) {
    cb = dst.rgb / alpha_b;
  }
  var cs = vec3<f32>(0.0);
  if (alpha_s > 0.0) {
    cs = src.rgb / alpha_s;
  }

  let b = vec3<f32>(
    blend_channel(params.mode, cb.r, cs.r),
    blend_channel(params.mode, cb.g, cs.g),
    blend_channel(params.mode, cb.b, cs.b),
  );

  // W3C 一般合成式(モジュール doc 参照)。premultiplied 出力。
  let out_rgb = alpha_s * (1.0 - alpha_b) * cs + alpha_s * alpha_b * b + (1.0 - alpha_s) * alpha_b * cb;
  let out_a = alpha_s + alpha_b * (1.0 - alpha_s);

  return clamp(vec4<f32>(out_rgb, out_a), vec4<f32>(0.0), vec4<f32>(1.0));
}
"#;
