//! [`Engine`] のレイヤー組み立て・render 経路 — `render_with_camera_override`
//! (`render_frame`/`render_frame_without_background`/`render_frame_with_view_camera`
//! 共通実体)・`layers_from_resolved`(zero-copy 経路)・`render_resolved_to_texture*`/
//! `render_frame_to_texture`/`apply_matte`。`next/engine/motolii-engine/src/lib.rs`
//! から移送(SP-7、2026-08-23、中身は変えていない——移送のみ)。

use std::collections::{HashMap, HashSet};

use motolii_compositor::{Layer, LayerPlacement, LayerWithPasses};
use motolii_core::{CompSpec, ResolvedCamera};
use motolii_store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};

use crate::translate::{
    to_u8_rgba, translate_blend_mode, translate_effect_passes, translate_matte_mode,
};
use crate::{Engine, EngineError, BACKGROUND_ORDER};

impl Engine {
    pub(crate) fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        // A05隔離(モジュール doc/`Self::layer_failures` 参照)——このフレームで
        // 新しく隔離した理由だけを持つ。前フレームの理由を持ち越さない(Q0)。
        self.layer_failures.clear();
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        // カメラも comp と同じく Document が持つ(裁定113/115)。preview/export が
        // 違うカメラを渡せないよう、ここでも引数ではなく `view` から読む
        // (裁定40 が comp について立てた規律と同じ形)。**観測視点だけが例外**
        // (`camera_override`、裁定157) — Document を一切読まず渡された値をそのまま使う。
        let camera = match camera_override {
            Some(camera) => camera,
            None => view
                .resolve_camera(t)
                .map_err(|e| EngineError::Store(e.to_string()))?,
        };
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        // `LayerWithPasses` で包んで `Compositor::render_with_effects` へ渡す(裁定153 S3、
        // S2 の注意書きどおり)。`passes` が空な layer は `render_with_effects` 内部で
        // オフスクリーンを一切作らず元の texture をそのまま使う従来コストの分岐を通るので
        // (`motolii-compositor` のモジュール doc/`tests/effects.rs` 参照)、effect を
        // 持たない layer(今のところ背景 layer を含め全部——`translate_effect_passes` は
        // 2026-08-21 時点で常に空を返す)はここを経由しても速度もアロケーションも変わらない。
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        if include_background {
            // comp の背景色(`Composition::background`、利用者要望: 黒だと気分が上がらない)。
            // **`motolii-compositor` の clear 色は変えない**(compositor は書き込み禁止の
            // 並列レーンが触っている最中)。代わりに comp 全域を覆う不透明の layer を
            // どの実 layer よりも奥(`order = BACKGROUND_ORDER`、定数の doc 参照 —
            // `i16::MIN` は depth_offset の shader 側スケールで外周1px を欠落させる
            // ので使わない)に足す — pinned layer(裁定113、カメラの pan/zoom を受けず
            // 画面に張り付く機構)を流用すれば、camera がどこを向いていても render
            // target をちょうど覆う「クリア色」として働く。
            // 既定値([0,0,0,1] 不透明黒)は旧 clear 色と同じ見た目になるので、
            // 既存テストの期待画素は変わらない(合成器の実測: `TRANSPARENT` clear は
            // 読み戻すと不透明黒になる — `motolii-compositor` の
            // `default_camera_all_z0_matches_orthographic_pixel_mapping` 参照)。
            // export は必ずこの分岐を通る([`Self::render_frame`] からしか
            // `include_background = false` は選ばれない)ので、背景も書き出しに乗る。
            let (background_texture, _) = self.texture_for(
                &LayerSource::Solid {
                    rgba: to_u8_rgba(composition.background),
                    // 1x1 で足りる — 単色は quad の `size` で comp 全域まで引き伸ばすので、
                    // texture 自体の解像度は意味を持たない。
                    width: 1,
                    height: 1,
                },
                0,
            )?;
            // 背景 layer には pass を積まない(S3 EXACT TARGET 3) — 背景は engine が
            // ここで直接組み立てる単色 pinned layer であって `ResolvedLayer` を経由しない
            // ので、そもそも effect スタックを持ち得ない。`passes: vec![]` で明示する。
            layers.push(LayerWithPasses {
                layer: Layer {
                    texture: background_texture.expect("LayerSource::Solid は常に texture を返す"),
                    size: [comp.width as f32, comp.height as f32],
                    placement: LayerPlacement {
                        order: BACKGROUND_ORDER,
                        ..Default::default()
                    },
                    pinned: true,
                    blend_mode: motolii_compositor::BlendMode::Normal,
                },
                passes: vec![],
            });
        }

        // BL4/切片3: `matte.layer` を突き合わせるための索引(`ResolvedLayer.id` が
        // 運ばれるようになったので作れる、`EngineError::UnsupportedMatte` の doc 参照)。
        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
        // matte 元として指名されている layer 全部。**通常描画リストからは除外する**
        // (AE/Lottie の track matte 意味論 — matte 元自身はもう1枚の可視 layer として
        // 重ならない。「手前の兄弟に暗黙に効く」ではなく `matte.layer` に指名された
        // layer だけが対象なので、除外も名指しの集合で行う)。
        let matte_sources: HashSet<LayerId> = resolved
            .iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        for layer in &resolved {
            if matte_sources.contains(&layer.id) {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            // store→compositor の effect 語彙変換(裁定153 S3、EXACT TARGET 1)。
            // texture のアップロードより前に計算しても副作用は無い(純関数)ので、
            // 上の blend 判定と同じ「早く判定する」並びに合わせてここに置く。
            let passes = translate_effect_passes(&layer.effects);

            let (texture, natural) = self.texture_for_layer(view, layer, t, comp)?;
            let Some(texture) = texture else {
                // 素材の外の時刻、または text layer に描く物が無い。この layer は
                // 今フレームに居ない。
                continue;
            };
            let built = Layer {
                texture,
                size: layer_size(layer, natural),
                // **置き方はそのまま持ち回る** — 並べ直すとそこが翻訳層になる。
                placement: layer.placement,
                pinned: layer.pinned,
                blend_mode,
            };

            let final_layer = match layer.matte {
                None => built,
                Some(matte) => {
                    let Some(source) = by_id.get(&matte.layer).copied() else {
                        // マット元がこの時刻の `resolved_layers()` に居ない(削除された、
                        // または timing の外)。matte を適用できないので本体も描かない
                        // (AE: track matte 対象はマット元が消えると一緒に消える —
                        // 黙って matte 抜きの絵を出すと「勝手に露出した」ことになる)。
                        continue;
                    };
                    let (source_texture, source_natural) =
                        self.texture_for_layer(view, source, t, comp)?;
                    let Some(source_texture) = source_texture else {
                        // マット元自身がこの時刻に絵を持たない(素材の外、text なら
                        // 空文字列等)——同じ理由で本体も描かない。
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let source_layer = Layer {
                        texture: source_texture,
                        size: layer_size(source, source_natural),
                        placement: source.placement,
                        pinned: source.pinned,
                        blend_mode: source_blend,
                    };
                    self.apply_matte(comp, camera, &built, &source_layer, matte.mode)?
                }
            };

            layers.push(LayerWithPasses {
                layer: final_layer,
                passes,
            });
        }

        Ok(self.compositor.render_with_effects(comp, camera, &layers)?)
    }

    /// **裁定171 v2(M4)**: [`Self::render_with_camera_override`]の層構築
    /// (`include_background`/`for layer in resolved`のループ、上記)を
    /// **そのまま複製**した private ヘルパー。既存の
    /// [`Self::render_with_camera_override`](延いては
    /// [`Self::render_frame`]/[`Self::render_frame_without_background`]/
    /// [`Self::render_frame_with_view_camera`])は1行も触っていない
    /// (supervisor 裁定「additive のみ」)——複製の理由は、この新しいヘルパーが
    /// `&StoreView<'_>` を取らず**既に resolve 済みの所有データ**
    /// (`comp`/`background`/`camera`/`resolved: &[ResolvedLayer]`)を取ることだけ
    /// が違うため([`Self::render_resolved_to_texture`]のモジュール doc 参照 —
    /// `motolii-shell` の presenter `Primitive` は `Document`(非 `Clone`、
    /// `re_entity_db::EntityDb` が `testing` feature 外では `Clone` を持たない)
    /// を共有できないので、`StoreView` を後から作り直せない)。
    ///
    /// **2026-08-22(ゼロコピー経路にも matte とテキストを通す発注)で matte と
    /// テキストの結線を追加**——`render_with_camera_override` が持つ `by_id`/
    /// `matte_sources`/`apply_matte` の3点セットをここへも複製したので、
    /// `camera`(`apply_matte` が要る)を新しく引数に足した。テキストは
    /// `&StoreView` を持たずに `TextDocument` へ辿り着けない
    /// (`LayerSource::Text` が中身を持たない unit variant である理由と同じ)ので、
    /// 呼び出し側が**先に resolve 済みの `TextDocument` を集めておく**設計にした
    /// (`text_documents: &HashMap<LayerId, TextDocument>`、呼び手は
    /// [`Self::render_frame_to_texture`]/`motolii-shell` の
    /// `Shell::build_preview_snapshot` —— どちらも `resolved_layers()` を呼んだ
    /// その場に `StoreView` があるので、そこで `text_document(id)` を引いて
    /// 添えるだけでよい)。`t`(`RationalTime`)も同じ理由で新規引数——
    /// Hold 評価(`content.eval(t)`)とキャッシュ鍵(`TextCacheKey`)の両方に要る。
    /// この関数自体は今も `Document`/`StoreView` を一切知らない
    /// (受け取るのは所有データの束だけ)。
    ///
    /// **`shape_documents` は2026-08-22(シェイプが画に出るようにする発注)で
    /// 新設**——`text_documents` と同型(`Self::texture_for_resolved`/
    /// `collect_shape_documents` の doc 参照)。
    fn layers_from_resolved(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        // A05隔離、`render_with_camera_override` と同じ規律(モジュール doc 参照)。
        self.layer_failures.clear();
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        let (background_texture, _) = self.texture_for(
            &LayerSource::Solid {
                rgba: to_u8_rgba(background),
                width: 1,
                height: 1,
            },
            0,
        )?;
        layers.push(LayerWithPasses {
            layer: Layer {
                texture: background_texture.expect("LayerSource::Solid は常に texture を返す"),
                size: [comp.width as f32, comp.height as f32],
                placement: LayerPlacement {
                    order: BACKGROUND_ORDER,
                    ..Default::default()
                },
                pinned: true,
                blend_mode: motolii_compositor::BlendMode::Normal,
            },
            passes: vec![],
        });

        // `render_with_camera_override` と同型の matte 索引(モジュール doc 参照)。
        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
        let matte_sources: HashSet<LayerId> = resolved
            .iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        for layer in resolved {
            if matte_sources.contains(&layer.id) {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let passes = translate_effect_passes(&layer.effects);

            let (texture, natural) =
                self.texture_for_resolved(layer, text_documents, shape_documents, t, comp)?;
            let Some(texture) = texture else {
                continue;
            };
            let built = Layer {
                texture,
                size: layer_size(layer, natural),
                placement: layer.placement,
                pinned: layer.pinned,
                blend_mode,
            };

            let final_layer = match layer.matte {
                None => built,
                Some(matte) => {
                    let Some(source) = by_id.get(&matte.layer).copied() else {
                        continue;
                    };
                    let (source_texture, source_natural) = self.texture_for_resolved(
                        source,
                        text_documents,
                        shape_documents,
                        t,
                        comp,
                    )?;
                    let Some(source_texture) = source_texture else {
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let source_layer = Layer {
                        texture: source_texture,
                        size: layer_size(source, source_natural),
                        placement: source.placement,
                        pinned: source.pinned,
                        blend_mode: source_blend,
                    };
                    self.apply_matte(comp, camera, &built, &source_layer, matte.mode)?
                }
            };

            layers.push(LayerWithPasses {
                layer: final_layer,
                passes,
            });
        }

        Ok(layers)
    }

    /// **裁定171 v2(M4)— zero-copy GPU 出力、resolve 済みスナップショット版**。
    /// CPU readback を一切しない([`motolii_compositor::Compositor::render_to_texture`]
    /// をそのまま呼ぶ)。`motolii-shell` の presenter `Primitive::prepare` は
    /// `Document`(非 `Clone`)を共有できないので、`Shell::refresh_frame` が
    /// **世代が変わった時だけ**(裁定171 v2 EXACT TARGET 2 の世代ゲート)
    /// `StoreView::resolved_layers`/`resolve_camera`/`composition` から
    /// 抜き出した所有データのスナップショットをここへ渡す設計 — この関数自体は
    /// `Document`/`StoreView` を一切知らない(層は既に resolve 済み)。
    ///
    /// `include_background` は常に `true` 固定(`Self::render_frame` と同じ
    /// 「唯一の評価経路は背景込み」の規律——export 専用の
    /// `render_frame_without_background` 相当は zero-copy 経路にはまだ無い、
    /// NON-GOALS「市松の GPU 化」の裏返し。市松 ON は
    /// `motolii-shell` 側が CPU フォールバック経路(`render_frame_without_background`)
    /// へ切り替える、裁定171 v2 §0-6)。
    ///
    /// **`t`/`text_documents` は2026-08-22(ゼロコピー経路にも matte とテキストを
    /// 通す発注)で新設**——[`Self::layers_from_resolved`]がテキストの Hold 評価
    /// (`t`)と `TextDocument` 本体(`text_documents`)を要るようになったのに
    /// 合わせた素通しの追加引数(`camera` は元々あった)。呼び出し側
    /// ([`Self::render_frame_to_texture`]/`motolii-shell` の
    /// `Shell::build_preview_snapshot`)はどちらも `resolved_layers(t)` を呼んだ
    /// その場で `StoreView` を持っているので、`text_document(id)` を添えるだけで
    /// 済む——この関数自体は相変わらず `Document`/`StoreView` を知らない。
    ///
    /// **公開シグネチャは無改造のまま固定**(2026-08-22、シェイプが画に出るように
    /// する発注)——`motolii-shell` の presenter Pipeline が直接呼ぶ口であり、
    /// この発注の EXACT TARGET は shell を触らない(別レーンが `create_from_card`
    /// を施工中)。中身は空の `shape_documents` を添えて
    /// [`Self::render_resolved_to_texture_with_shapes`]へ委譲するだけの後方互換
    /// ラッパーへ変わった——shell 経由の zero-copy 経路は今まで通り shape を
    /// 描かない(shell の `create_from_card` がまだ `Intent::SetShapes` を呼ばない
    /// ので、実際に空の shape しか存在しない今の実態とも一致する)。zero-copy 経路で
    /// 実際に shape を描かせたい呼び手は
    /// [`Self::render_frame_to_texture`](`&StoreView` 版、shape_documents を自動収集)
    /// か、[`Self::render_resolved_to_texture_with_shapes`]を直接呼ぶ。
    pub fn render_resolved_to_texture(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_resolved_to_texture_with_shapes(
            comp,
            background,
            camera,
            t,
            resolved,
            text_documents,
            &HashMap::new(),
        )
    }

    /// [`Self::render_resolved_to_texture`]の拡張版(2026-08-22、シェイプが画に
    /// 出るようにする発注)——`shape_documents: &HashMap<LayerId, Vec<ShapeNode>>`
    /// を追加で受け取る点だけが違う(`text_documents` と同型、
    /// `collect_shape_documents` 参照)。**公開 API を1本増やしただけ**——
    /// `render_resolved_to_texture` の既存呼び手(shell)の呼び出しは1文字も
    /// 変える必要が無い。
    pub fn render_resolved_to_texture_with_shapes(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers = self.layers_from_resolved(
            comp,
            background,
            camera,
            t,
            resolved,
            text_documents,
            shape_documents,
        )?;
        Ok(self.compositor.render_to_texture(comp, camera, &layers)?)
    }

    /// [`Self::render_resolved_to_texture`]の `&StoreView<'_>` 版
    /// (`Self::render_frame`の"resolve してから渡す"部分をここでもやるだけの
    /// 薄いラッパー)。`view`/`t` から `comp`/`background`/`camera`/`resolved` を
    /// 抜き出して委譲する——`motolii-shell` の presenter は(上記の理由で)
    /// これを直接呼べない(`StoreView` を保持できない)ので、こちらは主に
    /// この crate 自身のテスト・「将来 Document を直接持てる呼び手」向けの
    /// 対称な入口として用意する。[`Self::render_frame`]/
    /// [`Self::render_with_camera_override`]は無改造 — 独立した新規メソッド。
    ///
    /// **2026-08-22 でテキストも集める**——`resolved` の中から `LayerSource::Text`
    /// の layer だけ `view.text_document(id)` を引いて `text_documents` へ積む
    /// ([`collect_text_documents`] 参照)。matte 元が text layer である場合も
    /// (`layers_from_resolved` が `by_id` 越しにその layer を texture 化する時)
    /// 同じ map から引けるよう、`matte_sources`/対象を区別せず `resolved` 全体を
    /// 走査する。
    ///
    /// **shape も同じ形で集める**(2026-08-22、シェイプが画に出るようにする発注)
    /// ——[`collect_shape_documents`]参照。`render_resolved_to_texture`(shell が
    /// 直接呼ぶ、公開シグネチャ固定)ではなく
    /// [`Self::render_resolved_to_texture_with_shapes`]へ渡すので、この
    /// `&StoreView` 経由のゼロコピー入口は shape も実際に描く。
    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let camera = view
            .resolve_camera(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved)?;
        self.render_resolved_to_texture_with_shapes(
            comp,
            composition.background,
            camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )
    }

    /// **BL4 track matte 消費**。`target`(matte を持つ本体、既に texture が乗った
    /// [`Layer`])を `matte_source`(直上の matte 元、同じく既に texture が乗った
    /// [`Layer`])と `mode` で合成し、「絵から除外しつつマットとして消費し終えた
    /// 1枚の `Layer`」を返す(`translate_matte_mode` で語彙を写した上で
    /// `motolii_compositor::Compositor::matte_layer` へそのまま委譲するだけの薄い
    /// ラッパー)。
    ///
    /// **[`Self::render_frame`]/[`Self::render_frame_without_background`]/
    /// [`Self::render_frame_with_view_camera`] から `render_with_camera_override` 経由で
    /// 自動的に呼ばれる**(2026-08-22、テキスト+matte 結線)——`by_id`/`matte_sources`
    /// で matte 元を突き合わせて除外した上でここへ委譲する(`EngineError::UnsupportedMatte`
    /// の doc 参照)。この関数自体は今も「呼び出し元が `target`/`matte_source` を正しく
    /// 対応付けて渡す」薄いラッパーのまま——対応付けの責務はループ側にある。
    pub fn apply_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        target: &Layer,
        matte_source: &Layer,
        mode: motolii_store::MatteMode,
    ) -> Result<Layer, EngineError> {
        Ok(self.compositor.matte_layer(
            comp,
            camera,
            target,
            matte_source,
            translate_matte_mode(mode),
        )?)
    }
}

fn collect_text_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    t: RationalTime,
) -> Result<HashMap<LayerId, TextDocument>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Text {
            // **`resolved_text_document`**(A-1b)——`text_texture_for` と同じ理由、
            // `t` を新規引数に足した(このゼロコピー経路もそれまでは track を
            // 一切見ていなかった)。
            if let Some(document) = view
                .resolved_text_document(layer.id, t)
                .map_err(|e| EngineError::Store(e.to_string()))?
            {
                documents.insert(layer.id, document);
            }
        }
    }
    Ok(documents)
}

/// [`Engine::render_frame_to_texture`]専用。`resolved` の中から `LayerSource::Shape`
/// の layer だけ `view.shapes(id)` を引いて集める(2026-08-22、シェイプが画に
/// 出るようにする発注)——[`collect_text_documents`]と同型。`StoreView::shapes`
/// は「無ければ空 `Vec`」を返す(`text_document` の「無ければ `None`」とは違う形、
/// `motolii-store` の `view.rs` 参照)ので、ここも空配列をそのまま積む——
/// [`Engine::shape_texture_from_shapes`]が空配列を「描く物が無い」として扱う。
fn collect_shape_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = view
                .shapes(layer.id)
                .map_err(|e| EngineError::Store(e.to_string()))?;
            documents.insert(layer.id, shapes);
        }
    }
    Ok(documents)
}

/// `declared_size`(Document 側の指定)が無ければ(`<=0`)`natural`(実寸/素材由来)で埋める。
/// **大きさは transform の scale で動く**ので、ここは「板のローカル矩形」を決めている
/// だけ(裁定59)。`render_with_camera_override` の本体・matte 元の両方で使う共通式
/// (以前は2箇所に手書きで複製されていた)。
fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 {
            layer.declared_size[0]
        } else {
            natural[0]
        },
        if layer.declared_size[1] > 0.0 {
            layer.declared_size[1]
        } else {
            natural[1]
        },
    ]
}
