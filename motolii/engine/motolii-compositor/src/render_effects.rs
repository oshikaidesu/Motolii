
use crate::*;

impl Compositor {
    /// [`Self::render_with_effects`]/[`Self::render_to_texture`]/
    /// `Compositor::render_into`(`presentable.rs`、共有面へ直接書く口)が共有する
    /// step 1 の核(2026-08-28 一本化)——layer ごとに `passes` を適用した「合成へ
    /// 渡す実効 texture」を作る。3箇所に同じ手順を複製すると `render_into` が
    /// effect を無視していたのに誰も気づけなかった穴が再発するので、ここへ集めた。
    ///
    /// 返す `checked_out` は呼び手が**自分の合成が完全に終わってから**
    /// `self.effect_scratch.release` へ返すこと——scratch は合成の入力
    /// (`effective_textures`)が指す実体なので、合成の最中に手放すと壊れる。
    pub(crate) fn effective_layer_textures(
        &mut self,
        layers: &[LayerWithPasses],
    ) -> Result<
        (
            Vec<GpuTexture2D>,
            Vec<u32>,
            Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)>,
        ),
        CompositorError,
    > {
        let mut effective_textures: Vec<GpuTexture2D> = Vec::with_capacity(layers.len());
        let mut effective_paddings: Vec<u32> = Vec::with_capacity(layers.len());
        let mut checked_out: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)> = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
            if lwp.passes.is_empty() {
                effective_textures.push(lwp.layer.texture.clone());
                effective_paddings.push(0);
                continue;
            }

            let [width, height] = lwp.layer.texture.width_height();
            let padding = lwp
                .passes
                .iter()
                .map(EffectPass::padding)
                .max()
                .unwrap_or(0);
            let padded_width = width + 2 * padding;
            let padded_height = height + 2 * padding;

            // `EffectPass::intermediate_format` の一般化(module doc「`render_effects.rs`
            // 側は「Glow かどうか」を名指さない」)——`Isf` を足す前はここが
            // `has_glow` という Glow 専用の bool だった。
            let format = lwp
                .passes
                .iter()
                .find_map(EffectPass::intermediate_format)
                .unwrap_or_else(|| lwp.layer.texture.format());

            let src_handle = lwp.layer.texture.handle();
            let src = self
                .ctx
                .gpu_resources
                .textures
                .get_from_handle(src_handle)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            let scratch =
                self.effect_scratch
                    .acquire(&self.ctx.device, padded_width, padded_height, format);

            let encoder = copy_encoder.get_or_insert_with(|| {
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-effective-layer-textures"),
                    })
            });

            for pass in &lwp.passes {
                match pass {
                    EffectPass::Identity => {
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &scratch,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    EffectPass::Glow {
                        threshold,
                        intensity,
                        radius,
                    } => {
                        let padded_source = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            lwp.layer.texture.format(),
                        );
                        let padded_source_view = padded_source.create_view(&Default::default());
                        {
                            let _clear_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some(
                                        "motolii-compositor-glow-padded-source-clear",
                                    ),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &padded_source_view,
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
                        }
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &padded_source,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );

                        let bloom = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let blur_ping = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let bloom_view = bloom.create_view(&Default::default());
                        let blur_ping_view = blur_ping.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());

                        self.glow_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            encoder,
                            &padded_source_view,
                            &bloom_view,
                            &blur_ping_view,
                            &dst_view,
                            *threshold,
                            *intensity,
                            *radius,
                        );

                        checked_out.push((
                            padded_width,
                            padded_height,
                            lwp.layer.texture.format(),
                            padded_source,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            bloom,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            blur_ping,
                        ));
                    }
                    EffectPass::Isf { params } => {
                        // padding=0 なので padded_width/height == width/height
                        // (`EffectPass::padding` 参照)——Glow と違い、layer 自身の
                        // texture を直接読める(前段用の padded コピーが要らない)。
                        // これが「単一パスは Glow の5パス構造を持ち込まなくて良い」
                        // ことの直接の帰結(発注書の狙いそのもの)。
                        let src_view = src.texture.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());
                        self.isf_bloom.record(
                            &self.ctx,
                            encoder,
                            &src_view,
                            &dst_view,
                            params,
                            [width as f32, height as f32],
                        );
                    }
                    EffectPass::Gradient => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.wgsl_gradient.record(&self.ctx, encoder, &dst_view);
                    }
                    EffectPass::TriLed => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.wgsl_tri_led.record(&self.ctx, encoder, &dst_view);
                    }
                }
            }

            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &scratch)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            effective_textures.push(imported);
            effective_paddings.push(padding);
            checked_out.push((padded_width, padded_height, format, scratch));
        }

        if let Some(encoder) = copy_encoder.take() {
            self.ctx.before_submit();
            self.ctx.queue.submit([encoder.finish()]);
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;
        }

        Ok((effective_textures, effective_paddings, checked_out))
    }

    /// **layer 単位オフスクリーンパスの入口**(裁定153 S2、2026-08-21)。
    ///
    /// [`Self::render`]/[`Self::render_with_timing`] は**無改造のまま** — `motolii-engine`
    /// は今もそちらへ裸の `Layer` を渡しており(並走レーン、この crate の外)、
    /// この関数を新設するだけならその経路を一切変えない。effect を持たせたい呼び手は
    /// [`Layer`] を [`LayerWithPasses`] で包んでここへ渡す。
    ///
    /// **分岐はここ、layer 1枚ごと**: `passes` が空なら元の `layer.texture` を
    /// そのまま合成へ渡す(オフスクリーンを一切作らない — コスト増ゼロ)。非空なら
    /// [`effects::EffectScratch`] から中間 texture を借り、`passes` を順に適用してから
    /// その結果を合成へ渡す。texture(と、将来 pipeline が増えた時のそれ)は
    /// `Compositor` が所有し**フレームをまたいで再利用**する(毎フレーム作り直さない
    /// — `effects` モジュール doc の M5 proof 参照)。
    ///
    /// **第二 render パス禁止(裁定15/18)との関係**: `Compositor::render`/`render_frame`
    /// の呼び出し回数はこの関数を通っても増えない。増えるのは同じ `RenderContext`・
    /// 同じ `queue.submit` 呼び出しへ同乗する追加の `copy_texture_to_texture` コマンドで
    /// あって、別の合成器や別の描画エントリではない — `render_frame_without_background`
    /// (裁定141)が「第二経路ではなく同一合成器への入力差分」と整理したのと同じ論法。
    pub fn render_with_effects(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<Vec<u8>, CompositorError> {
        // 1) layer ごとに「合成へ渡す実効 texture」を決める(`Self::effective_layer_textures`
        //    が [`Self::render_to_texture`]/`Compositor::render_into` と共有する核)。
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        // 2) 通常合成——逐次 accumulator 経路([`Self::accumulate_sequential`]、BL3で
        //    分離可能 blend 対応へ拡張、`crate` module doc「分離可能 blend」節)。
        //    使う texture だけが「元の layer.texture」から「上で決めた実効 texture」に
        //    変わる(padding 込みの quad 拡張は `SequentialInput::local_min`/
        //    `local_size` へそのまま持ち込む——`render_with_timing` 旧経路と同じ計算)。
        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .zip(effective_textures.iter())
            .zip(effective_paddings.iter())
            .map(|((lwp, texture), &padding)| {
                let layer = &lwp.layer;
                // **既知の穴の根治**: pass が出力を拡張した分(`padding`、texel、
                // `EffectPass::padding` 参照)だけ quad を local 空間で広げる——
                // `LayerPlacement::transform` 自体は一切変えず、この rect の
                // 組み立てだけが「実 texture が layer 実寸より大きい」事実を吸収
                // する(transform は affine なので、広げた local 矩形にそのまま
                // 掛ければ回転/拡大/skew があっても正しく追従する)。padding=0
                // (pass 無し/Identity のみ)なら local_min=(0,0)・local_size=
                // layer.size のまま、従来と完全に同じ幾何になる。
                let pad = padding as f32;
                SequentialInput {
                    texture,
                    local_min: glam::Vec2::new(-pad, -pad),
                    local_size: glam::Vec2::new(
                        layer.size[0] + 2.0 * pad,
                        layer.size[1] + 2.0 * pad,
                    ),
                    transform: layer.placement.transform,
                    z: layer.placement.z,
                    rotation_x: layer.placement.rotation_x,
                    rotation_y: layer.placement.rotation_y,
                    pinned: layer.pinned,
                    opacity: layer.placement.opacity,
                    depth_offset: layer.placement.order,
                    blend_mode: layer.blend_mode,
                }
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        let frame = self.finalize_readback(comp, camera, background)?;

        // GPU が読み終わった後なので、scratch をプールへ返して次フレームで使い回す
        // (毎フレーム作り直さない)。
        for (width, height, format, texture) in checked_out {
            self.effect_scratch.release(width, height, format, texture);
        }

        Ok(frame)
    }

    /// **裁定171 v2(M4)— zero-copy GPU 出力**。[`Self::render_with_effects`]と
    /// **同じ層構築・同じ合成**を通すが、CPU 読み戻し(`ScreenshotProcessor`)を
    /// 一切しない——呼び出し側(embedder、`motolii-shell` の presenter Pipeline)が
    /// 同じ `queue` へ後続の描画コマンドを積める GPU texture をそのまま返す。
    ///
    /// **既存4メソッド([`Self::render`]/[`Self::render_with_timing`]/
    /// [`Self::render_with_effects`]/[`Self::render_sequential`])は無改造** ——
    /// これは並設した新しい入口で、上の4つのどれの呼び出し元にも一切波及しない
    /// (裁定171 M4 supervisor 裁定「additive のみ」)。
    ///
    /// ## 順序保証(fence 不要)
    ///
    /// このメソッドは `self.ctx.queue.submit(...)` までで止まる——`device.poll`
    /// もしない。呼び出し側が返された texture を**同じ `queue`** 上の後続コマンド
    /// (例: iced の render pass でこの texture を sample する)で使う限り、GPU は
    /// submission 順に実行するので「書いてから読む」が構造的に成立する
    /// (裁定171 v2 §0-5)。
    ///
    /// ## main_target の生存期間(fork の doc がそのまま保証を与える)
    ///
    /// fork の `GpuTexture`(`Arc<DynamicResource<..>>`)は「全参照が drop されたら
    /// **次のフレーム**で回収対象になる」(`wgpu_resources/texture_pool.rs` の doc
    /// comment)。ここで返す `wgpu::Texture`/`wgpu::TextureView` は
    /// `view_builder`(このメソッドを抜けると drop される)が最後に握っていた
    /// `GpuTexture` の中身の clone であって、次に `self.ctx.begin_frame()` が
    /// 呼ばれる(= 次にこのメソッドか他の render系メソッドが呼ばれる)まで
    /// 回収は起きない。呼び出し側がこのメソッドを「内容が変わった時だけ」
    /// (世代ゲート)呼ぶ設計である限り、前回の texture は次回このメソッドを
    /// 呼ぶ**その時**まで有効——ちょうど呼び出し側が新しい texture へ差し替える
    /// タイミングと一致するので、古い方が消えても実害が無い。
    ///
    /// ## effect pass の scratch(2026-08-22 修理 — RB 調査発見3番)
    ///
    /// [`Self::render_with_effects`]の scratch 解放(`effect_scratch.release`)は
    /// 「GPU が読み終わってから」が前提(`effects::EffectScratch::release` の doc
    /// 参照)——ただしそれは readback 経路が **CPU 側**でピクセルを読むために
    /// `device.poll` を必要としているからであって(`ScreenshotProcessor` が
    /// mapped buffer を読む)、poll 自体が release の安全条件ではない。
    /// release が本当に必要としているのは「次の `acquire` がこの texture を
    /// 新しい書き込み先として GPU へ積む時点で、前の利用(読み/書き)が GPU 上で
    /// 先に完了していること」——これは **GPU 側の順序**の話であり、CPU が
    /// 結果を見る/見ないとは独立している。
    ///
    /// このメソッドは CPU 読み戻しをしないので `device.poll` はしないが、
    /// scratch の再利用は「同一 queue の submission 順で書いてから読む」
    /// (上の「順序保証」節、裁定171 v2 §0-5 と同じ論法)で GPU 側の順序として
    /// 成立する: `Compositor` は `&mut self` でしか呼べない(Rust の排他借用が
    /// 並行呼び出しを構造的に禁止する)ので、`render_to_texture`/
    /// `render_with_effects` の呼び出しは常に CPU 上で直列——scratch を
    /// 使い回す2回目の `acquire`+書き込みコマンドは、必ず1回目の
    /// `queue.submit` より**後**の `queue.submit` に載る。同一 `wgpu::Queue`
    /// への複数回の submit は提出順に実行される(fork にも自前にも明示の
    /// fence/semaphore は要らない——wgpu がリソースの生存/使用状態を
    /// 内部で追跡し、同じ `wgpu::Texture` に対する後続コマンドの前に必要な
    /// バリアを自動で挿む。これは「前フレームで書いたテクスチャを次フレームで
    /// 読む/書く」という wgpu の標準的な用法そのもので、CPU 側の poll は
    /// 元来 CPU が結果を読みたい時にしか要らない)。
    ///
    /// したがって: `acquire` した scratch は、この関数が `queue.submit` を
    /// 終えた**直後**(readback 経路と同じタイミング、ただし poll を挟まない)
    /// に `effect_scratch.release` へ返す——`render_with_effects` と同じ
    /// プールを共有し、次回このメソッド(または `render_with_effects`)が
    /// 呼ばれた時に再利用される。GPU がまだ前フレームのコマンドを実行中でも、
    /// 次の書き込みは同一 queue の後続 submission として積まれるだけなので
    /// 破綻しない(fence 不要、上の「順序保証」節と同一の根拠)。
    pub fn render_to_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        // 1) layer ごとに「合成へ渡す実効 texture」を決める
        //    (`Self::effective_layer_textures` が [`Self::render_with_effects`]/
        //    `Compositor::render_into` と共有する核)。
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        // 2) 通常合成。[`Self::render_with_effects`] の step 2 と同じ
        //    `accumulate_sequential` 経由の組み立て(`crate` module doc「分離可能
        //    blend」節)。
        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .zip(effective_textures.iter())
            .zip(effective_paddings.iter())
            .map(|((lwp, texture), &padding)| {
                let layer = &lwp.layer;
                let pad = padding as f32;
                SequentialInput {
                    texture,
                    local_min: glam::Vec2::new(-pad, -pad),
                    local_size: glam::Vec2::new(
                        layer.size[0] + 2.0 * pad,
                        layer.size[1] + 2.0 * pad,
                    ),
                    transform: layer.placement.transform,
                    z: layer.placement.z,
                    rotation_x: layer.placement.rotation_x,
                    rotation_y: layer.placement.rotation_y,
                    pinned: layer.pinned,
                    opacity: layer.placement.opacity,
                    depth_offset: layer.placement.order,
                    blend_mode: layer.blend_mode,
                }
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        let (texture, view) = self.finalize_texture(comp, camera, background)?;

        // scratch をプールへ返す(2026-08-22 修理、RB 調査発見3番)。
        // [`Self::finalize_texture`] が最終 submit を済ませた後なので、同一 queue の
        // submission 順の論証(モジュール doc「effect pass の scratch」——次の
        // acquire+書き込みは必ず今回の submit より後に積まれる)がそのまま効く。
        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }

        Ok((texture, view))
    }

}
