//! 内蔵 vism 第1号: Glow(裁定153 S4、2026-08-21)。
//!
//! 移植元: `spikes/m5-known-implementation/M5-R0/src/glow.rs`(bright-pass→水平blur→
//! 垂直blur→加算合成 の5パス、FP16 中間、Host 所有 texture/pipeline を使い回す構成)。
//! **shader の数式・pass 構成は proof のまま移植** — 剥がしたのは fixture 依存だけ:
//!
//! 1. adapter/device を自前で起こさない(proof の `GlowFixture::new()` の instance 起こし
//!    を削除)。`Compositor::headless()` が持つ device をそのまま使う。
//! 2. `source_fs`(画面中央に固定矩形を焼くダミー)を削除。source は
//!    「effect が付いた layer 自身の描画結果」(`src.default_view`、呼び手が持つ
//!    実 texture のビュー)をそのまま bright-pass の入力にする — 中間コピーを作らない。
//! 3. 固定 32×32 を削除。texture 生成は呼び手(`Compositor::render_with_effects`)が
//!    渡す `width`/`height`(comp 実寸 or layer 実寸)から動的に行う。
//! 4. readback(CPU 転送)を削除。最終 `output` は次の合成ステップへ GPU 上のまま渡す
//!    (readback は export 時の最終フレーム1回だけ、既存の
//!    `Compositor::render_with_timing` の経路)。
//! 5. `threshold`(bright-pass 閾値)・`intensity`(composite の減衰率)・`radius`
//!    (blur タップ間隔)をハードコードから uniform buffer 経由の param にした
//!    (`GlowParams`、group(1) binding(0))。**5-tap の重み自体は proof のまま固定**
//!    (`[0.0625, 0.25, 0.375, 0.25, 0.0625]`) — `radius` は5-tapの間隔(texel)を
//!    スケールするだけで、tap数は増減しない(縫い目調査4節「未決」への回答:
//!    5-tap 固定・間隔だけ可変)。`radius = 1.0` が proof の固定オフセット
//!    (1texel・2texel)と厳密に一致する既定値。
//!
//! **中間 texture は常に `Rgba16Float`**(`GLOW_INTERMEDIATE_FORMAT`)。layer 本体の
//! 元 format(`Rgba8Unorm` 等)とは独立 — bright-pass の閾値判定・加算合成が 1.0 を
//! 超える値を扱える必要があるため(HDR ヘッドルーム、proof と同じ前提)。
//! **新しい互換層は作らない**: `Rgba16Float` が render target として使えない環境の
//! 扱いは proof と同じ(`device.create_texture`/`create_render_pipeline` が失敗すれば
//! そのまま呼び手(`Compositor::render_with_effects`)の `Result` へ伝播する)。
//!
//! pipeline は `Compositor` 所有・**初回生成して以後使い回す**([`GlowPipelines::new`]
//! を1回だけ呼ぶ、`Compositor::headless()` 参照)。bind group と param uniform buffer は
//! layer のサイズ/値ごとに変わるので、[`GlowPipelines::record`] の呼び出しごとに作る
//! (proof が texture/pipeline は使い回し・render() 呼び出しごとに command だけ積むのと
//! 同じ役割分担 — bind group は texture の実体(scratch pool から借りた物)が毎回
//! 変わり得るので使い回せない)。

/// Glow の中間 texture の format。layer 本体の format とは独立(module doc 参照)。
pub(crate) const GLOW_INTERMEDIATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(crate) struct GlowPipelines {
    bright: wgpu::RenderPipeline,
    blur_horizontal: wgpu::RenderPipeline,
    blur_vertical: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    one_texture_layout: wgpu::BindGroupLayout,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl GlowPipelines {
    /// **初回生成して以後使い回す**(module doc 参照)。texture サイズに依存しない
    /// (pipeline はサイズ非依存、bind group だけがサイズ依存)ので、layer ごと・
    /// フレームごとに作り直す理由が無い。
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-glow-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let one_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-glow-one-texture-layout"),
                entries: &[texture_entry(0)],
            });
        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-glow-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-glow-params-layout"),
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

        let one_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-compositor-glow-one-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&one_texture_layout), Some(&params_layout)],
                immediate_size: 0,
            });
        let two_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-compositor-glow-two-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
                immediate_size: 0,
            });

        let pipeline = |label: &'static str,
                         layout: &wgpu::PipelineLayout,
                         entry_point: &'static str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
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
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: GLOW_INTERMEDIATE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };

        let bright = pipeline(
            "motolii-compositor-glow-bright-pipeline",
            &one_texture_pipeline_layout,
            "bright_fs",
        );
        let blur_horizontal = pipeline(
            "motolii-compositor-glow-blur-horizontal-pipeline",
            &one_texture_pipeline_layout,
            "blur_horizontal_fs",
        );
        let blur_vertical = pipeline(
            "motolii-compositor-glow-blur-vertical-pipeline",
            &one_texture_pipeline_layout,
            "blur_vertical_fs",
        );
        let composite = pipeline(
            "motolii-compositor-glow-composite-pipeline",
            &two_texture_pipeline_layout,
            "composite_fs",
        );

        Self {
            bright,
            blur_horizontal,
            blur_vertical,
            composite,
            one_texture_layout,
            two_texture_layout,
            params_layout,
        }
    }

    /// 5パスを `encoder` へ積む(source→bright→blur-h→blur-v→composite、proof
    /// `glow.rs:230-264` の順序どおり)。`source_view` は effect が付いた layer 自身の
    /// 描画結果(呼び手所有、format 任意)。`bloom_view`/`blur_ping_view` は
    /// [`GLOW_INTERMEDIATE_FORMAT`] の scratch texture(呼び手が
    /// `EffectScratch::acquire` で確保済み)。`dst_view` も同じ format の scratch
    /// (呼び手が確保・後で通常合成へ渡す)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        blur_ping_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        threshold: f32,
        intensity: f32,
        radius: f32,
    ) {
        // uniform buffer は layer ごと・呼び出しごとに値が変わるので使い回さない
        // (pipeline/layout は使い回す、module doc の役割分担どおり)。
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-glow-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&threshold.to_le_bytes());
        bytes[4..8].copy_from_slice(&intensity.to_le_bytes());
        bytes[8..12].copy_from_slice(&radius.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-glow-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let one_texture_bind_group = |label: &'static str, view: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.one_texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                }],
            })
        };

        // bright-pass: 実 layer の描画結果を直接読む(proof の `source_fs` 相当は削除
        // 済み — module doc 2番)。
        let bright_bind = one_texture_bind_group("motolii-compositor-glow-bright-bind", source_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-bright-pass",
            bloom_view,
            &self.bright,
            &bright_bind,
            &params_bind_group,
        );

        let blur_h_bind = one_texture_bind_group("motolii-compositor-glow-blur-h-bind", bloom_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-blur-horizontal-pass",
            blur_ping_view,
            &self.blur_horizontal,
            &blur_h_bind,
            &params_bind_group,
        );

        let blur_v_bind =
            one_texture_bind_group("motolii-compositor-glow-blur-v-bind", blur_ping_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-blur-vertical-pass",
            bloom_view,
            &self.blur_vertical,
            &blur_v_bind,
            &params_bind_group,
        );

        let composite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-glow-composite-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_view),
                },
            ],
        });
        draw_pass(
            encoder,
            "motolii-compositor-glow-composite-pass",
            dst_view,
            &self.composite,
            &composite_bind,
            &params_bind_group,
        );
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

#[allow(clippy::too_many_arguments)]
fn draw_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
    params_bind_group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
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
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.set_bind_group(1, params_bind_group, &[]);
    pass.draw(0..3, 0..1);
}

/// proof(`spikes/m5-known-implementation/M5-R0/src/glow.rs:384-436`)からそのまま移植。
/// 変更点: `source_fs` を削除(module doc 2番)、`bright_fs`/`blur_at`/`composite_fs` が
/// ハードコード定数の代わりに `GlowParams` uniform(group(1) binding(0))を読む
/// (module doc 5番)。5-tap の重みと構造(横→縦の分離ガウシアン近似)は無改造。
const SHADER: &str = r#"
struct GlowParams {
  threshold: f32,
  intensity: f32,
  radius: f32,
  _pad: f32,
};

@group(0) @binding(0) var input_a: texture_2d<f32>;
@group(0) @binding(1) var input_b: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: GlowParams;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn bright_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let value = textureLoad(input_a, vec2<i32>(p.xy), 0);
  let luminance = dot(value.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let contribution = max(luminance - params.threshold, 0.0) / max(luminance, 0.000001);
  return vec4<f32>(value.rgb * contribution, value.a * contribution);
}

fn blur_at(p: vec2<i32>, direction: vec2<i32>) -> vec4<f32> {
  let size = vec2<i32>(textureDimensions(input_a));
  let lo = vec2<i32>(0);
  let hi = size - vec2<i32>(1);
  let step = max(i32(round(params.radius)), 1);
  let d1 = direction * step;
  let d2 = direction * step * 2;
  return textureLoad(input_a, clamp(p - d2, lo, hi), 0) * 0.0625
    + textureLoad(input_a, clamp(p - d1, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p, lo, hi), 0) * 0.375
    + textureLoad(input_a, clamp(p + d1, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p + d2, lo, hi), 0) * 0.0625;
}

@fragment
fn blur_horizontal_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  return blur_at(vec2<i32>(p.xy), vec2<i32>(1, 0));
}

@fragment
fn blur_vertical_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  return blur_at(vec2<i32>(p.xy), vec2<i32>(0, 1));
}

@fragment
fn composite_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let source = textureLoad(input_a, vec2<i32>(p.xy), 0);
  let glow = textureLoad(input_b, vec2<i32>(p.xy), 0) * params.intensity;
  // proof はここを clamp しない(readback が直接 half-float を読むので 1.0 超えを
  // そのまま観測して確かめている、`glow.rs` テスト参照)。製品側はこの結果が
  // 通常合成(`re_renderer::renderer::rectangle_fs.wgsl` の `decode_color`)へ
  // 戻る — そこは `decode_srgb` 有効時に `[0,1]` を外れた値を「壊れたデータ」と
  // 見なして `ERROR_RGBA`(マゼンタ)へ差し替える固定の検査を持っている
  // (一次確認: 上流 `shader/rectangle_fs.wgsl` `decode_color`、
  // `shader/types.wgsl` の `ERROR_RGBA = vec4f(1.0, 0.0, 1.0, 1.0)`)。
  // 最終的な表示は SDR 8bit なので 1.0 超えを保持する意味も無い — ここで
  // clamp して「1.0超えは白に飽和する」という、このリポジトリの他の加算合成
  // (`tests/compose.rs::add_blend_saturates_to_white_when_the_linear_sum_exceeds_one`)
  // と同じ規約に揃える(proof からの product 化差分、fixture 依存の剥がし漏れ
  // だったので実装中に追加で剥がした — 縫い目調査 5節には無かった発見)。
  return clamp(
    vec4<f32>(source.rgb + glow.rgb, source.a + glow.a * (1.0 - source.a)),
    vec4<f32>(0.0),
    vec4<f32>(1.0)
  );
}
"#;
