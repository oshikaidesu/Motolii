use re_renderer::renderer::{
    ColormappedTexture, RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ScreenshotProcessor, ViewBuilderId};

use crate::*;

impl Compositor {
    /// 試験専用の introspection。`effect_scratch` が実際に**新規生成**した
    /// (プール再利用ではない)texture の総数。`RenderTiming` が時間の内訳を
    /// 隠さないのと同じ規律で、資源生成も隠さない。
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    /// 試験専用の introspection。[`Self::accumulate_sequential`] が `queue.submit`
    /// を呼んだ累計回数(run-batching の oracle、`effect_passes_created_textures`
    /// と同じ「資源生成/同期点も隠さない」規律)。
    pub fn sequential_submits(&self) -> u64 {
        self.sequential_submits
    }

    /// [`Self::render_sequential`]/[`Self::render_with_effects`]/[`Self::render_to_texture`]
    /// が共有する、逐次合成の核(裁定161 BL1b が確立した main_target アクセサ経路を、
    /// BL3で「Normal/Add の固定式高速路」と「分離可能 blend の2段パス」の分岐へ拡張、
    /// その後 run-batching で BL3 merge(`118cdbf4`)の構造退行を根治)。
    ///
    /// layer を順に accumulator へ焼き込み、直前までの accumulator の裏付け
    /// ([`AccumulatorBacking`]、fork pool の reclaim から守る Arc か、自前 scratch か)を
    /// 返す——CPU 読み戻しをするか([`Self::finalize_readback`])・GPU texture のまま
    /// 返すか([`Self::finalize_texture`])は呼び手が選ぶ(この関数自体はどちらもしない)。
    ///
    /// ## run-batching(Normal/Add、`two_texture_pass_mode_index` が `None`)
    ///
    /// 2枚読みパスを要る blend(分離可能11種+非分離4種、BL4)の layer の出現点
    /// **だけ**で「run」を切る——連続する Normal/Add
    /// の layer は1つの `ViewBuilder` へ「background rect(run に直前 accumulator が
    /// 有る時だけ) + run 内の全 layer rect」を深度順に積み、1回の submit+poll で
    /// まとめて描く(旧 `render_sequential` 本体は「layer 1枚 = background rect 1枚 +
    /// layer rect 1枚」だったのを、「layer N枚 = background rect 高々1枚 + layer rect
    /// N枚」へ拡張しただけ——rect を1つの `ViewBuilder` に積む/`RectangleDrawData` へ
    /// 複数 rect を渡す仕組み自体は元から複数枚対応だった、`render_with_timing` が
    /// 全 layer をこの形で描いているのと同じ)。分離可能 blend が1つも無い合成(= 全 run
    /// が1本)なら submit は1回だけになり、旧単一パスのゼロコピー経路(裁定171)と
    /// 構造的に同等になる(`tests/run_batching.rs` の oracle 参照)。
    ///
    /// rect は同じ main_target の中で描かれるので、一括経路(`render_with_timing`)が
    /// N layer を1つの main_target の中で順に混ぜる時と**layer あたりの quantize 回数が
    /// 同じ**になり、バイト一致する(裁定161、`tests/sequential.rs` の overlap fixture が
    /// 縛る——run-batching は「何回に分けて submit するか」だけを変え、各 layer が
    /// 混ざる際の中間 quantize 回数は変えない)。
    ///
    /// `background_rect` の `depth_offset` は「run **先頭** layer の depth_offset より
    /// 1小さい値」(`background_rect` doc「極端値を使わない」節、過去に `i16::MIN` で
    /// 外周1px欠落を引いた実測が2度ある)。run 内の後続 layer は必ずそれより大きい
    /// depth_offset を持つ(呼び手が `placement.order` を単調増加のまま渡す——
    /// `render_sequential`/`render_with_effects`/`render_to_texture` のどの入口でも
    /// layer の並び順=order の並び順)ので、run 先頭の1回で sort 順は保たれる。
    ///
    /// ## 2枚読みパス(`Some`、分離可能11種 Multiply〜Exclusion + 非分離4種
    /// Hue〜Luminosity、BL4)
    ///
    /// 固定式では表現できない(`crate` module doc「分離可能 blend」「非分離4種」節)
    /// ので2段(この layer 単体で1つの run、run-batching の対象外——**変更なし**、
    /// 分離可能/非分離のどちらでも同じ2段構造——違うのは `mode_index` の範囲だけ):
    /// 1. layer 単体を(dst 無しで)自分の main_target へ描く——「layer 単体を transparent
    ///    へ premultiplied-over した canvas」を得る。
    /// 2. 直前までの accumulator が有れば、[`blend::SeparableBlendPipelines`] で
    ///    2枚読み混ぜて新しい accumulator を作る(`blend` モジュール doc の一般合成式、
    ///    `B(Cb,Cs)` の中身だけが mode で変わる)。
    ///    **無ければ**(1枚目)layer 単体の描画結果がそのまま新しい accumulator になる
    ///    ——`blend` モジュール doc が導出するとおり `αb=0` では `Co = αs·Cs` と数学的に
    ///    一致するので、混ぜる処理自体を省いてよい。
    ///
    /// ## fork pool の罠(`AccumulatorBacking::Fork` にのみ効く)
    ///
    /// fork の `GpuTexturePool::begin_frame` は、通常(import ではない)texture の参照が
    /// 尽きると reclaim 時に `res.texture.destroy()` を**明示的に**呼ぶ——import は
    /// 「別の `wgpu::Texture` clone」を作るだけで、元の pool エントリの生存とは独立に
    /// 守ってくれない。そのため `ViewBuilder::main_target().clone()` で `GpuTexture`
    /// (Arc)を明示的に握り続け、次の run/layer の背景として使い終わる(= 次の submit を
    /// poll で待ち終える)まで手放さない([`AccumulatorBacking::Fork`] 参照)。
    /// [`AccumulatorBacking::Scratch`](blend pass の出力)はこのプールに属さない
    /// 素の `wgpu::Texture` なので、この罠は無関係(Rust の所有権だけで足りる)。
    pub(crate) fn accumulate_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
    ) -> Result<Option<(AccumulatorBacking, GpuTexture2D)>, CompositorError> {
        // run-batching は「inputs の並び=重ね順=depth_offset 非減少」に依存する
        // (run の background rect を `run[0].depth_offset - 1` に敷く前提と、逐次
        // 累積の順序そのもの)。現状の唯一の発生源は `order: id.0 as i16`
        // (BACKGROUND_ORDER doc 参照)なので常に成立するが、store の `SetOrder` が
        // UI へ配線された時に黙って崩れないよう、ここで縛る。
        debug_assert!(
            inputs
                .windows(2)
                .all(|w| w[0].depth_offset <= w[1].depth_offset),
            "accumulate_sequential: inputs は depth_offset 非減少(=重ね順)で渡すこと"
        );
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut background: Option<(AccumulatorBacking, GpuTexture2D)> = None;

        let mut idx = 0;
        while idx < inputs.len() {
            let input = &inputs[idx];

            if let Some(mode_index) = two_texture_pass_mode_index(input.blend_mode) {
                self.ctx.begin_frame();

                let (transform, z) = if input.pinned {
                    (pinned_cancel * input.transform, 0.0)
                } else {
                    (input.transform, input.z)
                };

                // --- 分離可能 blend: layer 単体をまず自分の main_target へ描く ---
                let solo_rect = TexturedRect {
                    top_left_corner_position: to_point3(
                        transform.transform_point2(input.local_min),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(input.local_size.x, 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, input.local_size.y)),
                    ),
                    colormapped_texture: ColormappedTexture::from_unorm_rgba(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            input.opacity,
                        ),
                        depth_offset: 0,
                        ..Default::default()
                    },
                };
                let draw_data = RectangleDrawData::new(&self.ctx, &[solo_rect])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let mut solo_view_builder = ViewBuilder::new(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-sequential-solo",
                        comp,
                        view_from_world,
                        projection,
                    ),
                    ViewBuilderId::new(self.next_readback),
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;

                solo_view_builder.queue_draw(&self.ctx, draw_data);
                let command_buffer = solo_view_builder
                    .draw(&self.ctx, Rgba::TRANSPARENT)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;

                self.ctx.before_submit();
                self.ctx.queue.submit([command_buffer]);
                self.sequential_submits += 1;
                self.ctx.begin_frame();
                self.ctx
                    .device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;

                let layer_canvas: GpuTexture = solo_view_builder.main_target().clone();

                match background.take() {
                    None => {
                        // 混ぜる相手が無い(1枚目)——layer 単体の結果がそのまま新しい
                        // accumulator になる(関数 doc「αb=0」節)。
                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &layer_canvas.texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((AccumulatorBacking::Fork(layer_canvas), imported));
                    }
                    Some((backing, _)) => {
                        let dst_view = backing.texture().create_view(&Default::default());
                        let src_view = layer_canvas.default_view.clone();
                        let out_texture =
                            self.create_blend_scratch_texture(comp.width, comp.height);
                        let out_view = out_texture.create_view(&Default::default());

                        let mut encoder = self.ctx.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("motolii-compositor-blend-pass-encoder"),
                            },
                        );
                        self.blend_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            &mut encoder,
                            &dst_view,
                            &src_view,
                            &out_view,
                            mode_index,
                        );
                        self.ctx.before_submit();
                        self.ctx.queue.submit([encoder.finish()]);
                        self.sequential_submits += 1;
                        self.ctx.begin_frame();
                        self.ctx
                            .device
                            .poll(wgpu::PollType::wait_indefinitely())
                            .map_err(|e| CompositorError::Draw(e.to_string()))?;

                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((AccumulatorBacking::Scratch(out_texture), imported));
                        // `backing`(直前の accumulator)はここで drop される——直前の
                        // poll で GPU 読み取りは完了済みなので安全(fork pool の罠は
                        // 関数 doc 参照、`Scratch` ならそもそも罠が無い)。
                    }
                }

                idx += 1;
                continue;
            }

            // --- run-batching(Normal/Add の連続区間): 分離可能 blend の出現点だけで
            //     run を切る。run 内は「background rect(直前 accumulator が有る時
            //     だけ、run 先頭 layer 基準の depth_offset)+ run 内全 layer rect」を
            //     同じ `ViewBuilder` へ積み、1回の submit+poll でまとめて描く
            //     (関数 doc「run-batching」節、旧 `render_sequential` は「layer 1枚に
            //     つき submit 1回」だったのをここで束ねる)。
            self.ctx.begin_frame();

            let run_start = idx;
            while idx < inputs.len() && two_texture_pass_mode_index(inputs[idx].blend_mode).is_none() {
                idx += 1;
            }
            let run = &inputs[run_start..idx];

            let mut rects: Vec<TexturedRect> = Vec::with_capacity(run.len() + 1);
            if let Some((_, imported)) = &background {
                // 「run 先頭の layer より1小さい」だけ——`background_rect` doc の
                // 「極端値を使わない」節参照(過去に `i16::MIN` で外周1px欠落を
                // 引いた)。run 内の後続 layer は必ずそれより大きい depth_offset を
                // 持つ(関数 doc「run-batching」節)ので、この1回で sort 順は保たれる。
                rects.push(background_rect(
                    comp,
                    pinned_cancel,
                    imported.clone(),
                    run[0].depth_offset.saturating_sub(1),
                ));
            }

            for input in run {
                let (transform, z) = if input.pinned {
                    (pinned_cancel * input.transform, 0.0)
                } else {
                    (input.transform, input.z)
                };
                let a = match input.blend_mode {
                    BlendMode::Normal => input.opacity,
                    BlendMode::Add => 0.0,
                    // `two_texture_pass_mode_index` が `None` を返した mode(Normal/Add)
                    // のみ run に入る——他の全 variant は上の `if let Some(...)` 側で
                    // 個別処理され、run を切る側になる。
                    _ => unreachable!(
                        "two_texture_pass_mode_index が None を返した blend_mode のみ run に入る"
                    ),
                };
                rects.push(TexturedRect {
                    top_left_corner_position: to_point3(
                        transform.transform_point2(input.local_min),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(input.local_size.x, 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, input.local_size.y)),
                    ),
                    colormapped_texture: ColormappedTexture::from_unorm_rgba(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            a,
                        ),
                        depth_offset: input.depth_offset,
                        ..Default::default()
                    },
                });
            }

            let draw_data = RectangleDrawData::new(&self.ctx, &rects)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

            let mut view_builder = ViewBuilder::new(
                &self.ctx,
                sequential_target_config(
                    "motolii-comp-sequential-run",
                    comp,
                    view_from_world,
                    projection,
                ),
                ViewBuilderId::new(self.next_readback),
            )
            .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;

            view_builder.queue_draw(&self.ctx, draw_data);
            let command_buffer = view_builder
                .draw(&self.ctx, Rgba::TRANSPARENT)
                .map_err(|e| CompositorError::Draw(e.to_string()))?;

            self.ctx.before_submit();
            self.ctx.queue.submit([command_buffer]);
            self.sequential_submits += 1;
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;

            let held: GpuTexture = view_builder.main_target().clone();
            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &held.texture)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;
            background = Some((AccumulatorBacking::Fork(held), imported));
        }

        Ok(background)
    }

    /// 2枚読みパスの出力先。分離可能/非分離 blend([`Self::accumulate_sequential`])と
    /// track matte([`Self::matte_layer`]、BL4)が共有する——どちらも「canvas サイズの
    /// premultiplied Rgba8UnormSrgb を1枚作って結果を書く」という同じ要求なので、
    /// 名前(`blend`)は歴史的だが挙動はどちらの呼び手にも過不足ない。fork の
    /// texture pool(`effect_scratch`)には**属さない**普通の `wgpu::Texture`——
    /// [`Self::accumulate_sequential`] doc の「fork pool の罠」が無いので、素の
    /// Rust 所有権(drop で破棄)だけで足りる。
    fn create_blend_scratch_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-compositor-blend-output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: blend::SEPARABLE_BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    /// [`Self::accumulate_sequential`]の結果を**1回だけ** screenshot 経由で CPU 読み
    /// 戻す(旧 `render_sequential`「最終変換は1回だけ」節と同じ理由——分離可能 blend
    /// パスは新しい gamma 変換を持ち込まない、8bit sRGB の unmultiply→encode→
    /// re-multiply は依然としてここ1箇所だけ、`composite.wgsl` は無改造のまま)。
    pub(crate) fn finalize_readback(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            // ここは常に単独 rect(sort 順の懸念は無い)——`background_rect` doc の
            // 「極端値を使わない」節に沿って、小さい定数値を渡す。
            final_rects.push(background_rect(comp, pinned_cancel, imported.clone(), -1));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            sequential_target_config(
                "motolii-comp-sequential-finalize",
                comp,
                view_from_world,
                projection,
            ),
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        final_view_builder.queue_draw(&self.ctx, final_draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        final_view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = final_view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);

        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        out.ok_or(CompositorError::ReadbackMissing)
    }

    /// [`Self::accumulate_sequential`]の結果を CPU 読み戻しせずそのまま返す
    /// (`Self::render_to_texture` 専用)。**`background_rect` を経由する追加
    /// `ViewBuilder` を挟まない**——accumulator の main_target 自身が既に最終画な
    /// ので、screenshot 経路(`composite.wgsl` のガンマ round-trip)は不要(旧
    /// `render_to_texture` が単一 `ViewBuilder` の `main_target()` を直接返して
    /// いたのと同じ理由)。`background` が `None`(layers が空)の時だけ、何も
    /// 描かない transparent な `ViewBuilder` を1つ作って返す(旧実装の空 layers
    /// 挙動を保つ)。
    pub(crate) fn finalize_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        match background {
            Some((backing, _imported)) => {
                let texture = backing.texture().clone();
                let view = texture.create_view(&Default::default());
                Ok((texture, view))
            }
            None => {
                let projection = motolii_core::camera_projection(comp, camera);
                let view_from_world = macaw::IsoTransform::from_rotation_translation(
                    projection.rotation,
                    -(projection.rotation * projection.eye),
                );
                let draw_data = RectangleDrawData::new(&self.ctx, &[])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;
                let mut view_builder = ViewBuilder::new(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-zero-copy-empty",
                        comp,
                        view_from_world,
                        projection,
                    ),
                    ViewBuilderId::new(self.next_readback),
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                view_builder.queue_draw(&self.ctx, draw_data);
                let command_buffer = view_builder
                    .draw(&self.ctx, Rgba::TRANSPARENT)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;
                self.ctx.before_submit();
                self.ctx.queue.submit([command_buffer]);

                let target = view_builder.main_target();
                let texture = target.texture.clone();
                let view = target.default_view.clone();
                drop(view_builder);
                Ok((texture, view))
            }
        }
    }

    /// **逐次合成(accumulator)経路の薄い入口**(裁定161 BL1b、BL3 で分離可能 blend
    /// 対応へ拡張、2026-08-22)。実装は [`Self::accumulate_sequential`]+
    /// [`Self::finalize_readback`](両方とも [`Self::render_with_effects`]/
    /// [`Self::render_to_texture`] と共有) —— この関数自体は「`Layer` を
    /// [`SequentialInput`] へ詰め替えるだけ」。
    ///
    /// [`Self::render`]/[`Self::render_with_timing`]/[`Self::render_with_effects`] は
    /// **無改造のまま**——この関数は並設した入口で、既存3つの呼び出し元へ
    /// 一切波及しない。`tests/sequential.rs` の overlap fixture(Normal/Add の
    /// バイト一致)は裁定161 のまま維持——固定式高速路は無改造。
    pub fn render_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .map(|layer| SequentialInput {
                texture: &layer.texture,
                local_min: glam::Vec2::ZERO,
                local_size: glam::Vec2::new(layer.size[0], layer.size[1]),
                transform: layer.placement.transform,
                z: layer.placement.z,
                pinned: layer.pinned,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        self.finalize_readback(comp, camera, background)
    }

    /// [`Self::matte_layer`]/[`Self::accumulate_sequential`]の分離可能 blend「solo」
    /// パス(旧実装、`SequentialInput` 経由)が使っているのと**同型**の「1 layer を
    /// canvas 全体の premultiplied texture へ描く」処理を、`Layer` から直接組み立てる
    /// 独立コピーとして持つ。
    ///
    /// **`accumulate_sequential` を改造して共有しない**——あちら側は `blend_mode` に
    /// 応じて run を切る/切らないの分岐そのものであり、`sequential_submits` oracle
    /// (`tests/run_batching.rs`)が「`queue.submit` の増分」を厳密に数えている。
    /// track matte はどの `blend_mode` の layer にも適用できる必要がある(matte は
    /// 「下の layer との混ざり方」より**前**の段階で消費される——`blend_mode` 自体は
    /// 読まない)ため、`accumulate_sequential` の分岐条件に matte を混ぜ込むと
    /// oracle の数え方まで変える改造になってしまう。BL3 の run-batching 修理を壊さない
    /// ため、コード量は小さい(30行程度)ので複製する側を選んだ。
    fn render_layer_to_canvas(
        &mut self,
        comp: CompSpec,
        projection: motolii_core::CameraProjection,
        view_from_world: macaw::IsoTransform,
        pinned_cancel: glam::Affine2,
        layer: &Layer,
        label: &'static str,
    ) -> Result<GpuTexture, CompositorError> {
        self.ctx.begin_frame();

        let (transform, z) = if layer.pinned {
            (pinned_cancel * layer.placement.transform, 0.0)
        } else {
            (layer.placement.transform, layer.placement.z)
        };

        // 板の中心を軸にして傾ける。四隅は傾き後の辺ベクトルから組み直す。
        let tilt = crate::tilt(layer.placement.rotation_x, layer.placement.rotation_y);
        let u = tilt * to_vector3(transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)));
        let v = tilt * to_vector3(transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])));
        let center = to_point3(
            transform.transform_point2(glam::Vec2::new(layer.size[0], layer.size[1]) * 0.5),
            z,
        );

        let rect = TexturedRect {
            top_left_corner_position: center - (u + v) * 0.5,
            extent_u: u,
            extent_v: v,
            colormapped_texture: ColormappedTexture::from_unorm_rgba(layer.texture.clone()),
            options: RectangleOptions {
                multiplicative_tint: Rgba::from_rgba_premultiplied(
                    layer.placement.opacity,
                    layer.placement.opacity,
                    layer.placement.opacity,
                    layer.placement.opacity,
                ),
                depth_offset: 0,
                ..Default::default()
            },
        };
        let draw_data = RectangleDrawData::new(&self.ctx, &[rect])
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            sequential_target_config(label, comp, view_from_world, projection),
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        Ok(view_builder.main_target().clone())
    }

    /// **track matte 適用パス**(BL4)。`layer`(matte を持つ本体)と `matte_source`
    /// (直上の matte 元、AE 型)をそれぞれ[`Self::render_layer_to_canvas`]で canvas
    /// 全体の premultiplied texture へ描き(camera/transform/pinned/opacity は両方
    /// 個別に反映される——matte 元が本体と違う位置/大きさに置かれていても、canvas
    /// 空間で正しく整列した2枚として揃う)、[`matte::MattePipelines`]で
    /// 「`layer` の premultiplied 値 × `matte_source` から導いた coverage」を計算する
    /// (係数の出典・4モードの式は `matte` モジュール doc 参照)。
    ///
    /// 返り値は**すでに canvas 全体に正しく配置し終えた1枚の `Layer`**
    /// (`pinned: true`・`size = comp`・`transform = IDENTITY`——[`background_rect`]が
    /// 逐次 accumulator を「画面に張り付く full-canvas 板」として折り返しているのと
    /// 同じ考え方)。呼び出し側はこれを他の layer と同様に
    /// [`Self::render_sequential`]/[`Self::render_with_effects`]/[`Self::render_to_texture`]
    /// の `layers` へそのまま混ぜてよい——`blend_mode` は `layer` のものをそのまま
    /// 引き継ぐので、matte 消費後もその layer 自身の(下の layer との)混ざり方は
    /// 変わらない。
    ///
    /// **layer 単位で solo 描画2回+combine 1回 = 3 submit**(`render_layer_to_canvas`
    /// が2回、内部の `matte_pipelines.record` 提出が1回)。`accumulate_sequential` の
    /// `sequential_submits` オラクルには乗らない(あのカウンタは
    /// `accumulate_sequential` 内の submit だけを数える契約、関数 doc 参照)——
    /// コストを隠さない旨をここに明記する。
    pub fn matte_layer(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: &Layer,
        matte_source: &Layer,
        mode: MatteMode,
    ) -> Result<Layer, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let layer_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            pinned_cancel,
            layer,
            "motolii-comp-matte-layer",
        )?;
        let matte_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            pinned_cancel,
            matte_source,
            "motolii-comp-matte-source",
        )?;

        let layer_view = layer_canvas.default_view.clone();
        let matte_view = matte_canvas.default_view.clone();
        let out_texture = self.create_blend_scratch_texture(comp.width, comp.height);
        let out_view = out_texture.create_view(&Default::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-compositor-matte-pass-encoder"),
            });
        self.matte_pipelines.record(
            &self.ctx.device,
            &self.ctx.queue,
            &mut encoder,
            &layer_view,
            &matte_view,
            &out_view,
            matte::matte_mode_index(mode),
        );
        self.ctx.before_submit();
        self.ctx.queue.submit([encoder.finish()]);
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.next_effect_key += 1;
        let key = self.next_effect_key;
        let imported = self
            .ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
            .map_err(|e| CompositorError::Effect(e.to_string()))?;

        Ok(Layer {
            texture: imported,
            size: [comp.width as f32, comp.height as f32],
            placement: LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                opacity: 1.0,
                order: layer.placement.order,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
            },
            pinned: true,
            blend_mode: layer.blend_mode,
        })
    }
}
