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
//! ## 非分離4種(BL4、2026-08-22): Hue/Saturation/Color/Luminosity
//!
//! W3C Compositing and Blending Level 1、3.7節(Non-separable blend modes)の
//! `SetLum`/`ClipColor`/`Sat`/`SetSat` 擬似コードそのまま。分離可能11種と違い、
//! `B(Cb,Cs)` が RGB 3成分を**1単位**として扱う(per-channel な `blend_channel` には
//! 分解できない)——そのため WGSL 側は `vec3<f32>` を返す `nonseparable_blend` を
//! 別に持ち、`params.mode` が 11以上ならそちらへ分岐する(0〜10 は従来どおり
//! `blend_channel` を3回呼ぶ per-channel 経路のまま、無改造)。**2枚読みの土台
//! (bind group・params uniform・一般合成式)は分離可能と完全に共有する**——
//! [`crate::two_texture_pass_mode_index`] が両者を同じ2枚読みパスへ振り分ける。
//!
//! `Lum(C) = 0.3·Cr + 0.59·Cg + 0.11·Cb`(spec の係数そのまま——[`crate::matte`]の
//! Luma matte が使う Rec.709 係数とは**別の式・別出典**、混同しないこと)。
//!
//! `SetSat` の実装は spec の Cmax/Cmid/Cmin を明示的に並べ替える代わりに、
//! `(C - Cmin) * s / (Cmax - Cmin)` という等価な線形写像を使う(`Cmin`→0・`Cmax`→`s`・
//! `Cmid`→線形補間、という spec の3分岐をチャンネルごとに代入すると数学的に一致する
//! ことが確かめられる——各種実装(例: Skia の HSL blend)が使う標準的な簡約)。
//!
//! `ClipColor` は spec と同じく `L`/`n`/`x` を**元の色から1回だけ**計算し、2つの
//! if 分岐(`n<0`/`x>1`)は互いに独立にこの値を参照する(2つ目の分岐が1つ目の
//! 分岐後の色を読むことはあっても、`L`/`x` 自体は再計算しない——spec の擬似コードの
//! 逐次性をそのまま再現)。**既知の限界**: 色が完全に一様(r/g/b が全チャンネル同値)
//! かつその値が範囲外(`< 0` または `> 1`)という極端な入力では `l - n = 0`/
//! `x - l = 0` の0除算(`NaN`)が理論上あり得る——spec 自体がこのケースを特別扱いして
//! いないので、ここでも追加のガードは入れない(SoftLight の「発注書どおり W3C 版を正
//! とする」と同じ姿勢)。
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
/// `params.mode` の値は [`crate::two_texture_pass_mode_index`] の返り値と1対1
/// (0=Multiply〜10=Exclusion は [`crate::separable_mode_index`]、11=Hue〜14=Luminosity
/// は [`crate::nonseparable_mode_index`]、モジュール doc「数式の出典」「非分離4種」
/// 節参照)。
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

// --- 非分離4種(BL4、モジュール doc「非分離4種」節、W3C 3.7節そのまま) ---

fn lum(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.3, 0.59, 0.11));
}

// spec の擬似コードどおり、L/n/x は元の色から1回だけ求め、2つの if 分岐は
// (1つ目が適用済みかもしれない)`c` を読みつつも L/n/x 自体は再計算しない。
fn clip_color(c_in: vec3<f32>) -> vec3<f32> {
  let l = lum(c_in);
  let n = min(min(c_in.r, c_in.g), c_in.b);
  let x = max(max(c_in.r, c_in.g), c_in.b);
  var c = c_in;
  if (n < 0.0) {
    c = l + (c - l) * (l / (l - n));
  }
  if (x > 1.0) {
    c = l + (c - l) * ((1.0 - l) / (x - l));
  }
  return c;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
  let d = l - lum(c);
  return clip_color(c + vec3<f32>(d, d, d));
}

fn sat(c: vec3<f32>) -> f32 {
  return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b);
}

// spec の Cmax/Cmid/Cmin 並べ替えと数学的に等価な線形写像(モジュール doc「非分離
// 4種」節参照)。
fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
  let cmax = max(max(c.r, c.g), c.b);
  let cmin = min(min(c.r, c.g), c.b);
  if (cmax > cmin) {
    return (c - vec3<f32>(cmin)) * (s / (cmax - cmin));
  }
  return vec3<f32>(0.0, 0.0, 0.0);
}

fn nonseparable_blend(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
  if (mode == 11u) {
    // Hue
    return set_lum(set_sat(cs, sat(cb)), lum(cb));
  }
  if (mode == 12u) {
    // Saturation
    return set_lum(set_sat(cb, sat(cs)), lum(cb));
  }
  if (mode == 13u) {
    // Color
    return set_lum(cs, lum(cb));
  }
  // Luminosity(mode == 14u、default もここへ落ちる)
  return set_lum(cb, lum(cs));
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

  var b: vec3<f32>;
  if (params.mode < 11u) {
    b = vec3<f32>(
      blend_channel(params.mode, cb.r, cs.r),
      blend_channel(params.mode, cb.g, cs.g),
      blend_channel(params.mode, cb.b, cs.b),
    );
  } else {
    // 非分離4種(BL4)——RGB を1単位として扱う `B(Cb,Cs)`、per-channel には
    // 分解できない(モジュール doc「非分離4種」節)。
    b = nonseparable_blend(params.mode, cb, cs);
  }

  // W3C 一般合成式(モジュール doc 参照)。premultiplied 出力。
  let out_rgb = alpha_s * (1.0 - alpha_b) * cs + alpha_s * alpha_b * b + (1.0 - alpha_s) * alpha_b * cb;
  let out_a = alpha_s + alpha_b * (1.0 - alpha_s);

  return clamp(vec4<f32>(out_rgb, out_a), vec4<f32>(0.0), vec4<f32>(1.0));
}
"#;
