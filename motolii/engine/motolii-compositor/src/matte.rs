//! track matte(BL4、AE 型 — 直上レイヤーを alpha/luma マットとして使う、
//! `motolii_store::Matte`/`MatteMode` の4値と1対1)。
//!
//! [`blend`](crate::blend) と同じ手口(fork を触らず crate 内へ新規 WGSL パイプラインを
//! 1本足す)——`layer`(matte を持つ本体)と `matte_source`(直上の matte 元)を
//! それぞれ canvas 全体の premultiplied texture として描き終えた2枚を読み、
//! `layer` の値を `matte_source` から導いた **coverage**(0〜1のスカラー)で
//! スケールする(`crate::Compositor::matte_layer` が2枚の描画+この pass の呼び出しを
//! 担う)。
//!
//! ## 4モードの式
//!
//! 入力の2枚 (`layer_tex`/`matte_tex`) はどちらも [`crate::blend`] と同じ
//! premultiplied Rgba8UnormSrgb(`textureLoad` で GPU が自動 sRGB decode する)。
//!
//! - **Alpha**: `coverage = matte.a`(matte 元の alpha をそのまま使う——alpha は
//!   premultiplication の影響を受けない)。
//! - **InvertedAlpha**: `coverage = 1 - matte.a`。
//! - **Luma**: `coverage = dot(matte.rgb, LUMA_WEIGHTS)`(matte 元の
//!   **premultiplied** RGB に直接輝度係数を掛ける——unpremultiply しない)。
//!   `matte.rgb = straight_rgb * matte.a` なので、線形性から
//!   `dot(matte.rgb, w) = dot(straight_rgb, w) * matte.a` と数学的に一致する
//!   ——「straight 輝度 × alpha」を求めたいときの標準的な近道であり、AE/Lottie 系
//!   実装でも同じ簡約が使われる(alpha=0 の画素は自動的に coverage=0 になる —
//!   透明領域は「見せない」が自然に成立する)。
//! - **InvertedLuma**: `coverage = 1 - dot(matte.rgb, LUMA_WEIGHTS)`。
//!   **既知の癖**(AE のドキュメント化された挙動と一致): alpha=0 の画素は
//!   premultiplied RGB が `(0,0,0)` になるので Luma coverage は 0 になり、
//!   InvertedLuma では逆に coverage が 1 になる——「透明な matte 領域が
//!   invert luma matte では逆に完全露出になる」という AE ユーザーに広く知られた
//!   挙動をそのまま再現する(黙って別の定義へ寄せない)。
//!
//! `LUMA_WEIGHTS`(Rec.709: 0.2126/0.7152/0.0722)は sRGB と同じ係数——ただし
//! **AE の track matte luma の正確な係数は非公開**なので、業界標準の輝度係数を
//! 明示的に選んだという設計判断であることを記す(発明ではなく選択。
//! [`crate::blend`] の非分離 blend `Lum()`(W3C 3.7節の 0.3/0.59/0.11、NTSC 系)とは
//! **別の式・別出典**——混同しないこと)。
//!
//! ## 出力の alpha 規約(premultiplied ではなく straight)
//!
//! matte を消費した結果は「straight 色は変えず、alpha だけ coverage で縮める」
//! ——`layer`(premultiplied)から straight 色を復元し(`straight = layer.rgb /
//! layer.a`)、`out = vec4(straight, layer.a · coverage)` を書く。
//!
//! **なぜ premultiplied のまま返さないか**: [`crate::Compositor::matte_layer`] は
//! 結果を普通の `Layer` として返し、呼び手はそれを他の `Layer` と同じ配列へ混ぜて
//! [`crate::Compositor::render_sequential`] 等へ渡す——**この一般経路は
//! `ColormappedTexture::from_unorm_rgba` の既定(`TextureAlpha::SeparateAlpha`、
//! 「まだ alpha を掛けていない straight color」という前提で使用時に掛け直す)を
//! 前提に `Layer.texture` を読む**(`crate` の `background_rect` が「accumulator は
//! 既に premultiplied なのでこの既定は使えない」と明示的に迂回しているのが、
//! この既定の存在そのものの証拠)。ここで premultiplied のまま返すと、一般経路が
//! **もう一度** alpha を掛けてしまい(`straight·(α·c)` のつもりが `(α·c)·(α·c)` に
//! なる二重適用)、coverage が実際より暗く/透明になる事故になる——**数式としての
//! matte 適用は premultiplied 表現(rgb/alpha を同じスカラーで縮める)と straight
//! 表現(rgb 不変・alpha だけ縮める)は数学的に同じ結果**(`layer = straight·α` を
//! 代入すれば一致することが確かめられる)なので、ここでは「一般経路が期待する方の
//! 表現で書く」という表現形式の選択であって、別の計算をしているわけではない。
//!
//! ## gamma 空間
//!
//! [`crate::blend`] と同じ前提——入力2枚(`layer_tex`/`matte_tex`)・出力とも
//! `Rgba8UnormSrgb`、GPU が自動で sRGB decode/encode する。ソフトウェアで2重に
//! 変換しない。

pub(crate) const MATTE_TARGET_FORMAT: wgpu::TextureFormat = crate::blend::SEPARABLE_BLEND_TARGET_FORMAT;

/// track matte の重ね方(AE/Lottie の4値、`motolii_store::MatteMode` と1対1)。
/// **この crate は store の語彙を知らない**(`motolii-engine` が変換する、
/// `translate_blend_mode`/`translate_matte_mode` と同じ形)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

/// [`MatteMode`] の WGSL `params.mode` index(`SHADER` の分岐と1対1)。
///
/// **`_` を使わない**(全 variant を列挙)——`separable_mode_index` と同じ
/// fail-closed の形(将来 variant が増えたらここも更新を強制される)。
pub(crate) fn matte_mode_index(mode: MatteMode) -> u32 {
    match mode {
        MatteMode::Alpha => 0,
        MatteMode::InvertedAlpha => 1,
        MatteMode::Luma => 2,
        MatteMode::InvertedLuma => 3,
    }
}

pub(crate) struct MattePipelines {
    pipeline: wgpu::RenderPipeline,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl MattePipelines {
    /// **初回生成して以後使い回す**(`blend::SeparableBlendPipelines::new`/
    /// `effects::GlowPipelines::new` と同じ規律 — `Compositor::with_device` が
    /// 1回だけ呼ぶ)。
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-matte-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-matte-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-matte-params-layout"),
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
            label: Some("motolii-compositor-matte-pipeline-layout"),
            bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-compositor-matte-pipeline"),
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
                entry_point: Some("matte_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: MATTE_TARGET_FORMAT,
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

    /// 1パスを `encoder` へ積む。`layer_view`=matte を持つ本体(canvas 全体へ描き
    /// 終えた premultiplied texture)・`matte_view`=matte 元(同じく canvas 全体)・
    /// `out_view`=結果の書き込み先(呼び手が確保した [`MATTE_TARGET_FORMAT`] の
    /// texture)。`mode` は [`matte_mode_index`] が返す index。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        layer_view: &wgpu::TextureView,
        matte_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        mode: u32,
    ) {
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-matte-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&mode.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-matte-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-matte-texture-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(layer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(matte_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-matte-pass"),
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

/// `blend::SHADER` と同じ形(fullscreen triangle `vs_main` + 1 fragment)。
/// `params.mode` の値は [`matte_mode_index`] の返り値と1対1(モジュール doc
/// 「4モードの式」参照)。
const SHADER: &str = r#"
struct MatteParams {
  mode: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var layer_tex: texture_2d<f32>;
@group(0) @binding(1) var matte_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: MatteParams;

// Rec.709(sRGB と同じ係数)——モジュール doc「4モードの式」参照。
const LUMA_WEIGHTS: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

fn coverage(mode: u32, matte: vec4<f32>) -> f32 {
  if (mode == 0u) {
    // Alpha
    return matte.a;
  }
  if (mode == 1u) {
    // InvertedAlpha
    return 1.0 - matte.a;
  }
  if (mode == 2u) {
    // Luma(premultiplied rgb に直接——モジュール doc「4モードの式」参照)
    return dot(matte.rgb, LUMA_WEIGHTS);
  }
  // InvertedLuma(mode == 3u、default もここへ落ちる)
  return 1.0 - dot(matte.rgb, LUMA_WEIGHTS);
}

@fragment
fn matte_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let layer = textureLoad(layer_tex, vec2<i32>(p.xy), 0);
  let matte = textureLoad(matte_tex, vec2<i32>(p.xy), 0);

  let c = clamp(coverage(params.mode, matte), 0.0, 1.0);

  // 出力は straight(SeparateAlpha)——モジュール doc「出力の alpha 規約」参照。
  // `layer` は premultiplied(solo canvas)なので、まず straight 色へ戻してから
  // alpha だけ coverage で縮める(straight 色そのものは matte で変わらない)。
  var straight_rgb = vec3<f32>(0.0);
  if (layer.a > 0.0) {
    straight_rgb = layer.rgb / layer.a;
  }
  let out_a = clamp(layer.a * c, 0.0, 1.0);

  return vec4<f32>(clamp(straight_rgb, vec3<f32>(0.0), vec3<f32>(1.0)), out_a);
}
"#;
