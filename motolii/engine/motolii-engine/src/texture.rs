//! [`Engine`] の texture 取得経路 — `texture_for`(素材/テキスト/シェイプの取得)と
//! そのキャッシュ鍵(`TextCacheKey`/`ShapeCacheKey`)。`next/engine/motolii-engine/src/lib.rs`
//! から移送(SP-7、2026-08-23、中身は変えていない——移送のみ)。呼び手は
//! `crate::render`(`render_with_camera_override`/`layers_from_resolved`)の
//! `texture_for_layer` 呼び出し。

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use motolii_compositor::GpuTexture2D;
use motolii_core::CompSpec;
use motolii_media::{load_point_cloud, probe};
use motolii_store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};

use crate::render::layer_size;
use crate::{shape, text, Engine, EngineError};

/// 層ごとに独立した復号ストリーム。**同じ動画を別の時刻で使う層が同じ id を
/// 共有すると、1つのデコーダが毎フレーム別々の時刻を要求されてシークで詰まる。**
fn layer_stream_id(layer: LayerId, path: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layer.0.hash(&mut hasher);
    path.hash(&mut hasher);
    hasher.finish()
}

impl Engine {
    /// `layer.source` を texture 化する共通口。**`Text` だけ特別扱い** —
    /// `texture_for`(下記)は `&LayerSource`/`source_frame` しか受け取らず、
    /// `LayerSource::Text` はコンテンツを持たない印(unit variant)なのでどの layer の
    /// `TextDocument` を読むべきか `texture_for` の中だけでは決まらない
    /// ([`text_texture_for`](Self::text_texture_for) の doc 参照)——それ以外の
    /// variant は今まで通り `texture_for` へそのまま委譲する。
    ///
    /// `render_with_camera_override` のループが本体・matte 元の両方でこれを呼ぶ
    /// (`matte.layer` を突き合わせた後の matte 元も、本体と同じ経路で texture 化する)。
    pub(crate) fn texture_for_layer(
        &mut self,
        view: &StoreView<'_>,
        layer: &ResolvedLayer,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if layer.source == LayerSource::Text {
            self.text_texture_for(view, layer.id, t, comp)
        } else if layer.source == LayerSource::Shape {
            self.shape_texture_for(view, layer.id, comp)
        } else if let LayerSource::PointCloud { path, .. } = &layer.source {
            self.point_cloud_texture_for(path, comp)
        } else if let LayerSource::Media { path, .. } = &layer.source {
            let path = path.clone();
            self.media_texture_for(&path, layer.source_frame, layer.id)
        } else {
            self.texture_for(&layer.source, layer.source_frame)
        }
    }

    /// 板のローカル矩形サイズ(`declared_size`と実寸`natural`を`layer_size`で
    /// 突き合わせた `[w, h]`)。**キャッシュに焼けている texture の実寸だけを見る**
    /// ——ここで新たに rasterize/upload はしない。cache miss は `None`
    /// (通常描画が次に回ればキャッシュへ入り、選択枠が1フレーム遅れるだけ)。
    /// `front` が layer の種類(`LayerSource`)で分岐しないための唯一の口。
    pub fn selected_layer_size(
        &self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        let composition = view.composition().ok().flatten()?;
        let comp = composition.spec();
        let resolved = view.resolved_layers(t).ok()?;
        let layer = resolved.iter().find(|l| l.id == layer_id)?;

        let natural = match &layer.source {
            LayerSource::Text => {
                let document = view.resolved_text_document(layer_id, t).ok().flatten()?;
                let key = TextCacheKey::new(layer_id, &document, t, comp.width, comp.height);
                self.text_textures.get(&key)?.width_height().map(|v| v as f32)
            }
            LayerSource::Shape => {
                let shapes = view.shapes(layer_id).ok()?;
                let canvas = motolii_vector::Canvas::centered(comp.width, comp.height);
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                self.shape_textures.get(&key)?.width_height().map(|v| v as f32)
            }
            LayerSource::PointCloud { .. } | LayerSource::Null | LayerSource::Group => {
                [comp.width as f32, comp.height as f32]
            }
            LayerSource::Solid { width, height, .. } => [*width as f32, *height as f32],
            LayerSource::Media { path, .. } => {
                let info = self.probes.get(path)?;
                [info.width as f32, info.height as f32]
            }
        };
        Some(layer_size(layer, natural))
    }

    /// [`Self::texture_for_layer`]の zero-copy 版(2026-08-22、ゼロコピー経路にも
    /// matte とテキストを通す発注)。`&StoreView` の代わりに、呼び出し側が
    /// 先に集めておいた `text_documents: &HashMap<LayerId, TextDocument>`
    /// (`collect_text_documents`/`Shell::build_preview_snapshot` 参照)から引く
    /// ことだけが違う——`Text` 以外の分岐は完全に同じ `texture_for` へ委譲する。
    ///
    /// **`shape_documents` は2026-08-22(シェイプが画に出るようにする発注)で新設**
    /// ——`text_documents` と同型(`collect_shape_documents` 参照)。`Shape` も
    /// 中身を運ばない unit variant なので、`ResolvedLayer` からは辿り着けない。
    pub(crate) fn texture_for_resolved(
        &mut self,
        layer: &ResolvedLayer,
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if layer.source == LayerSource::Text {
            self.text_texture_from_document(text_documents.get(&layer.id), layer.id, t, comp)
        } else if layer.source == LayerSource::Shape {
            let shapes = shape_documents
                .get(&layer.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            self.shape_texture_from_shapes(shapes, layer.id, comp)
        } else if let LayerSource::PointCloud { path, .. } = &layer.source {
            self.point_cloud_texture_for(path, comp)
        } else if let LayerSource::Media { path, .. } = &layer.source {
            let path = path.clone();
            self.media_texture_for(&path, layer.source_frame, layer.id)
        } else {
            self.texture_for(&layer.source, layer.source_frame)
        }
    }

    /// `LayerSource::Text` の texture 化(裁定190 切片3、`texture_for` の
    /// `LayerSource::Text` 枝の doc が示していた差し込み口そのもの)。
    ///
    /// **`view`/`layer_id` を追加で受け取る** — `ResolvedLayer` は自分の
    /// `TextDocument` を運ばない(`LayerSource::Text` が中身を持たない unit variant
    /// である理由と同じ、`motolii_store::LayerSource` のモジュール doc 参照)ので、
    /// `StoreView::text_document(layer_id)` で store から直接引く。
    ///
    /// **薄いラッパー**(2026-08-22 で `text_texture_from_document` へ実体を
    /// 移した)——`&StoreView` を持つ呼び手([`Self::texture_for_layer`]、
    /// `render_with_camera_override` 経路)専用の入口として残す。
    fn text_texture_for(
        &mut self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        // **`text_document` ではなく `resolved_text_document`**(A-1b、裁定214
        // 同日訂正版)——`text_style.*`/`text_justify` track を時刻 `t` で重ねた
        // 値を使う。`text_document` の生の静的値のままだと track を打っても画が
        // 変わらない(store 側は在るが合成器が未消費、というA-1が残した穴)。
        let document = view
            .resolved_text_document(layer_id, t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        self.text_texture_from_document(document.as_ref(), layer_id, t, comp)
    }

    /// [`Self::text_texture_for`]/[`Self::texture_for_resolved`]の共通実体
    /// (2026-08-22 で `text_texture_for` から抜き出した)。`document` が既に
    /// 引けているかどうかだけを呼び手に委ね、以降のラスタライズ/キャッシュ処理は
    /// `&StoreView` の有無に関わらず完全に同じ——ゼロコピー経路(`layers_from_resolved`)
    /// と `render_with_camera_override` が同じキャッシュ(`self.text_textures`)を
    /// 共有するので、Stage 表示(zero-copy)と export(CPU)が同じ内容の text layer を
    /// 同じフレームで描いても再ラスタライズは1回で済む。
    ///
    /// canvas は **comp 全域**(左上原点、`Canvas::centered` は使わない——`text.rs` の
    /// 単体試験がそうしているのと同じ座標系)を使う。テキストの組版はまだ folding-box
    /// (`declared_size`/anchor)を持たないので、layer の「板」を comp と同じ大きさに
    /// 固定し、実際の字面の位置は raster 内の画素そのもの(ペン位置)で決まる —
    /// これは裁定190 切片3の結線で選んだ最小の設計であって、`declared_size` で
    /// 好きな矩形へ描かせる(folding box)機能はこの発注の範囲外(次切片候補)。
    fn text_texture_from_document(
        &mut self,
        document: Option<&TextDocument>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let Some(document) = document else {
            // `SetTextDocument` がまだ一度も書かれていない — 描く物が無い
            // (`StoreView::text_document` の doc と同じ「無い」≠「壊れている」)。
            return Ok((None, [0.0, 0.0]));
        };

        let canvas = motolii_vector::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };

        let key = TextCacheKey::new(layer_id, document, t, canvas.width, canvas.height);
        if let Some(texture) = self.text_textures.get(&key) {
            return Ok((
                Some(texture.clone()),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let Some(raster) = text::rasterize_text_document(document, t, &canvas)? else {
            // style 表が空、または今の内容(Hold 評価後)が空文字列 — 描く物が無い
            // (エラーではない、`rasterize_text_document` の doc 参照)。どちらの分岐も
            // cosmic-text のフォント読み込みへ進む前の早期 return なので、キャッシュ
            // しなくても再計算コストは小さい。
            return Ok((None, [0.0, 0.0]));
        };

        let texture = self.compositor.upload_rgba(
            "text",
            &raster.premultiplied_rgba8,
            raster.width,
            raster.height,
        )?;
        self.text_textures.insert(key, texture.clone());
        Ok((Some(texture), [raster.width as f32, raster.height as f32]))
    }

    /// `LayerSource::Shape` の texture 化(発注「シェイプが画に出るようにする」、
    /// 2026-08-22)。`text_texture_for`/`text_texture_from_document` と同型の
    /// 分割——`&StoreView` を持つ呼び手([`Self::texture_for_layer`])専用の薄い
    /// 入口で、`view.shapes(layer_id)` を引いてから実体
    /// ([`Self::shape_texture_from_shapes`])へ渡すだけ。
    fn shape_texture_for(
        &mut self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let shapes = view
            .shapes(layer_id)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        self.shape_texture_from_shapes(&shapes, layer_id, comp)
    }

    /// [`Self::shape_texture_for`]/[`Self::texture_for_resolved`]の共通実体
    /// (`text_texture_from_document` と同型)。`shapes` が既に引けているかどうかを
    /// 呼び手に委ね、以降のラスタライズ/キャッシュ処理は `&StoreView` の有無に
    /// 関わらず完全に同じ——ゼロコピー経路(`layers_from_resolved`)と
    /// `render_with_camera_override` が同じキャッシュ(`self.shape_textures`)を
    /// 共有するので、Stage 表示(zero-copy)と export(CPU)が同じ内容の shape layer
    /// を同じフレームで描いても再ラスタライズは1回で済む。
    ///
    /// canvas は **`Canvas::centered`**(`shape.rs` module doc の「canvas は
    /// `Canvas::centered`」節参照 — text とは逆に、shape のパス源は局所原点
    /// `(0,0)` を中心に生成されるため)。テキストと同じく layer の「板」を comp と
    /// 同じ大きさに固定する(`declared_size`/folding box はこの発注の範囲外)。
    ///
    /// `t`(`RationalTime`)を取らない——`ShapeCacheKey`/`shape.rs` module doc の
    /// 「時刻を取らない理由」節参照(shape 自身は時間評価を持たず、動くとしたら
    /// layer の transform 側)。
    fn shape_texture_from_shapes(
        &mut self,
        shapes: &[ShapeNode],
        layer_id: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if shapes.is_empty() {
            // `SetShapes` がまだ一度も書かれていない、または空配列——描く物が無い
            // (エラーではない、`shape::rasterize_shapes` の doc と同じ扱い)。
            return Ok((None, [0.0, 0.0]));
        }

        let canvas = motolii_vector::Canvas::centered(comp.width, comp.height);

        let key = ShapeCacheKey::new(layer_id, shapes, canvas.width, canvas.height);
        if let Some(texture) = self.shape_textures.get(&key) {
            return Ok((
                Some(texture.clone()),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let Some(raster) = shape::rasterize_shapes(shapes, &canvas)? else {
            return Ok((None, [0.0, 0.0]));
        };

        let texture = self.compositor.upload_rgba(
            "shape",
            &raster.premultiplied_rgba8,
            raster.width,
            raster.height,
        )?;
        self.shape_textures.insert(key, texture.clone());
        Ok((Some(texture), [raster.width as f32, raster.height as f32]))
    }

    /// `LayerSource::PointCloud` の texture 化。`text_texture_from_document`/
    /// `shape_texture_from_shapes` と同型 — canvas は comp 全域に固定し、板が
    /// そのまま comp を覆う(3D点群を「comp を覆うレンダリング済みビュー」として
    /// 平面合成に混ぜる。板の上で 3D カメラを手で振れる機能はこの切片の非目標)。
    ///
    /// **A05 隔離**(`LayerSource::Media` 枝と同じ規律): 読み込み/GPU描画の失敗は
    /// この layer だけへ閉じる——`self.layer_failures` へ積み、呼び出し元へは
    /// `Err` を伝播しない(comp 全体の合成を道連れにしない)。
    fn point_cloud_texture_for(
        &mut self,
        path: &str,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let natural = [comp.width as f32, comp.height as f32];

        let data = match self.point_clouds.get(path) {
            Some(data) => Some(data.clone()),
            None => match self.failed_point_clouds.get(path) {
                Some(reason) => {
                    self.layer_failures.push(reason.clone());
                    None
                }
                None => match load_point_cloud(std::path::Path::new(path)) {
                    Ok(data) => {
                        self.point_clouds.insert(path.to_owned(), data.clone());
                        Some(data)
                    }
                    Err(err) => {
                        let reason = format!("点群を読めない: {path}: {err}");
                        self.failed_point_clouds
                            .insert(path.to_owned(), reason.clone());
                        self.layer_failures.push(reason);
                        None
                    }
                },
            },
        };
        let Some(data) = data else {
            return Ok((None, natural));
        };
        if data.positions.is_empty() {
            // ply は読めたが点が1つも無い——描く物が無い(Q0: 実データの無い状態を
            // あるように見せない、`rasterize_text_document`/`rasterize_shapes` の
            // 「空なら None」と同じ扱い)。
            return Ok((None, natural));
        }

        // **render 結果は (path, comp解像度) で鍵を作る**——`TextCacheKey`/
        // `ShapeCacheKey` が canvas 寸法を鍵へ含めるのと同じ理由(comp resize で
        // 古い解像度の texture を出さない)。点群自体は静的(時刻非依存)なので
        // `frames`(`(path, frame)`)と違いフレーム番号は要らない。
        let key = (path.to_owned(), comp.width, comp.height);
        if let Some(texture) = self.point_cloud_textures.get(&key) {
            return Ok((Some(texture.clone()), natural));
        }

        let texture = match self.compositor.render_point_cloud_to_texture(
            &data.positions,
            &data.colors,
            comp.width,
            comp.height,
        ) {
            Ok(texture) => texture,
            Err(err) => {
                self.layer_failures
                    .push(format!("点群を描けない: {path}: {err}"));
                return Ok((None, natural));
            }
        };
        self.point_cloud_textures.insert(key, texture.clone());
        Ok((Some(texture), natural))
    }


    fn media_texture_for(
        &mut self,
        path: &str,
        frame: i64,
        layer: LayerId,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
                // **A05隔離(`next/reference/axis/A05-missing.tsv`)**: この枝の
                // 直上コメント(旧版)は「ここで Err を返すとフレーム全体が出なく
                // なるので、この layer だけ落とす」と約束していたが、実装は
                // 「素材の外の時刻」(下の out-of-range 分岐)しか局所化しておらず、
                // probe/decode の実際の失敗(`probe(path)?`/`read_frame_at(...)?`)は
                // `?` でそのまま外へ伝播していた——`render_with_camera_override`/
                // `layers_from_resolved` のループはこの関数の `Err` を
                // `self.texture_for_layer(..)?`/`self.texture_for_resolved(..)?` で
                // 即座に上へ流すので、comp 全体の合成(他の正常な layer も含む)が
                // その瞬間出せなくなっていた(実測: `tests/media_layer_isolation.rs`
                // 修正前は `render_frame` 自体が `Err` で返り、健全な Solid layer も
                // 一緒に消えていた)。
                //
                // ここでは probe/decode の失敗を**この layer だけ**に閉じる——
                // 呼び出し元へは `Err` を返さず、下の out-of-range 分岐と同じ
                // `Ok((None, ..))` で「今フレームは描く物が無い」として扱う。
                // **フェイクのプレースホルダは描かない**(Q0: 実データの無い状態を
                // あるように見せない)——`None` は「透明」ではなく「この layer は
                // 今フレームに出さない」という texture_for 全体の既存語彙
                // (Text/Shape の空、out-of-range と同じ)にそのまま乗る。
                //
                // **黙って握りつぶさない**(Q3)——理由は `self.layer_failures`
                // (呼び出し1回ぶんの Vec<String>、[`Self::layer_failures`] 参照)へ
                // 積む。新しいエラー機構は発明しない(裁定215)——既存の
                // `MediaError`/`thiserror::Error` の `Display` をそのまま文字列化
                // するだけ。
                let info = match self.probes.get(path) {
                    Some(info) => Some(info.clone()),
                    None => match self.failed_probes.get(path) {
                        // probe が失敗する素材(削除済み・非対応コーデック等)は
                        // 再生中ずっと毎フレームこの枝を通る。キャッシュが無いと
                        // 壊れた素材1つで毎フレーム ffprobe プロセスを起動し続ける
                        // ことになる(`probes`成功キャッシュの doc と対称の理由)。
                        // フレームが変わるたびに理由は改めて報告する(Q3: 沈黙禁止)。
                        Some(reason) => {
                            self.layer_failures.push(reason.clone());
                            None
                        }
                        None => match probe(path) {
                            Ok(info) => {
                                self.probes.insert(path.to_owned(), info.clone());
                                Some(info)
                            }
                            Err(err) => {
                                let reason =
                                    format!("素材を読めない(probe失敗): {path}: {err}");
                                self.failed_probes.insert(path.to_owned(), reason.clone());
                                self.layer_failures.push(reason);
                                None
                            }
                        },
                    },
                };
                let Some(info) = info else {
                    // probe 自体ができていない ⇒ natural size も分からない
                    // (Text/Shape の「描く物が無い」と同じ [0.0, 0.0])。
                    return Ok((None, [0.0, 0.0]));
                };
                let natural = [info.width as f32, info.height as f32];

                // **時間の計算はしない**。comp 時刻 → 素材フレームの写像は Document が
                // 持つ(`LayerTiming::source_frame`)。engine が別の写像を持つと
                // 時刻の正本が2本になる(2026-08-20 の敵対的レビューで一度やった失敗)。
                

                // 素材の外は描かない(フリーズフレーム禁止、M4)。ここで Err を返すと
                // フレーム全体が出なくなるので、この layer だけ落とす。
                let last_frame = info.nb_frames.map(|n| n - 1);
                if frame < 0 || last_frame.is_some_and(|last| frame > last) {
                    return Ok((None, natural));
                }

                if !self.videos.contains_key(path) {
                    let bytes = match std::fs::read(path) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            self.layer_failures
                                .push(format!("素材を読めない(read失敗): {path}: {err}"));
                            return Ok((None, natural));
                        }
                    };
                    let descr = match re_video::VideoDataDescription::load_from_bytes(
                        &bytes,
                        "video/mp4",
                        path,
                    ) {
                        Ok(descr) => descr,
                        Err(err) => {
                            self.layer_failures
                                .push(format!("動画を読めない(decode失敗): {path}: {err}"));
                            return Ok((None, natural));
                        }
                    };
                    let video = re_renderer::video::Video::load(
                        path.to_owned(),
                        descr,
                        re_video::DecodeSettings::default(),
                    );
                    self.videos.insert(path.to_owned(), (bytes, video));
                }
                let (bytes, video) = self.videos.get(path).expect("直前に insert した");

                let Some(timescale) = video.data_descr().timescale else {
                    self.layer_failures
                        .push(format!("動画にタイムスケールが無い: {path}"));
                    return Ok((None, natural));
                };
                let video_time = re_video::Time::from_secs(
                    frame as f64 / info.fps.as_f64(),
                    timescale,
                );
                let stream_id = re_video::player::VideoPlayerStreamId(layer_stream_id(layer, path));
                let source = re_video::player::VideoSliceSource(bytes);
                let output = video.frame_at(
                    self.compositor.render_context(),
                    stream_id,
                    video_time,
                    &source,
                );
                if let Some(err) = output.error {
                    self.layer_failures
                        .push(format!("フレームを読めない(decode失敗): {path} frame={frame}: {err}"));
                }
                match output.output.and_then(|frame_texture| frame_texture.texture) {
                    Some(texture) => {
                        self.video_last_texture.insert(stream_id.0, texture.clone());
                        Ok((Some(texture), natural))
                    }
                    // 非同期デコード中(まだ来ていない) — 画面を空にせず前フレームを保つ。
                    None => Ok((self.video_last_texture.get(&stream_id.0).cloned(), natural)),
                }
                }

    pub(crate) fn texture_for(
        &mut self,
        source: &LayerSource,
        source_frame: i64,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        match source {
            LayerSource::Solid {
                rgba,
                width,
                height,
            } => {
                let natural = [*width as f32, *height as f32];
                if let Some(texture) = self.textures.get(source) {
                    return Ok((Some(texture.clone()), natural));
                }
                let pixels: Vec<u8> = rgba
                    .iter()
                    .copied()
                    .cycle()
                    .take((width * height * 4) as usize)
                    .collect();
                let texture = self
                    .compositor
                    .upload_rgba("solid", &pixels, *width, *height)?;
                self.textures.insert(source.clone(), texture.clone());
                Ok((Some(texture), natural))
            }

            // **`Text`/`Shape` はここでは呼べない**(`&LayerSource`/`source_frame`
            // しか受け取らない口なので、それぞれ `TextDocument`/`Vec<ShapeNode>` へ
            // 辿り着けない)——`render_with_camera_override`/`layers_from_resolved`
            // はどちらも `Text`/`Shape` の場合ここへは来ず、専用の枝
            // ([`Self::text_texture_for`]/[`Self::shape_texture_for`]、
            // `Self::texture_for_layer`/`Self::texture_for_resolved` 経由)を呼ぶ——
            // ここに来る `Text`/`Shape` は `texture_for` を直接呼ぶ他の呼び手
            // (このモジュール内には現状無い)向けの安全側の既定値として残す
            // (2026-08-22、シェイプが画に出るようにする発注で `Shape` も `Text` と
            // 同じ扱いへ揃えた)。
            // **`PointCloud` もここでは呼べない**(comp 解像度が要る——`point_cloud_texture_for`
            // の doc 参照)。`Text`/`Shape` と同じ理由・同じ安全側の既定値。
            LayerSource::Text
            | LayerSource::Shape
            | LayerSource::PointCloud { .. }
            | LayerSource::Media { .. } => {
                Ok((None, [0.0, 0.0]))
            }
            // null layer は元々絵を持たず(裁定どおり)、`Group`(裁定173)も同じく
            // 絵を持たない——Group は「子を持てる」という印だけの layer で、合成は
            // 世界合成(`motolii-store::view::world_affine`)が親 transform として
            // 使うだけ、Group 自身のピクセルは無い。
            LayerSource::Null | LayerSource::Group => Ok((None, [0.0, 0.0])),
        }
    }
}

/// テキスト texture のキャッシュ鍵(裁定190 切片3、EXACT TARGET 1「キャッシュ鍵に
/// 何を使うか設計して doc に明記」への回答)。
///
/// # なぜ `textures: HashMap<LayerSource, GpuTexture2D>` を再利用しないか
///
/// `LayerSource` は `Eq + Hash` である必要がある(engine の texture cache キーである
/// ため、`motolii_store::LayerSource` のモジュール doc 参照)——その代償として
/// `Null`/`Shape`/`Text`/`Group` は中身を持たない unit variant になっている。
/// つまり **`LayerSource::Text` は全部の text layer で同じ値**であり、`textures` を
/// そのまま使うと2枚目の text layer を描いた瞬間に1枚目の texture が上書きされる
/// (comp に text layer が2枚以上あると事故る)。text 専用の鍵型がここで要る理由。
///
/// # 鍵の中身
///
/// - **`layer: LayerId`** — どの layer の texture か。内容が完全に一致する2枚の
///   text layer は理論上 texture を共有してもよいが、layer ごとに別枠にした方が
///   「この layer が編集された」という直感と cache のライフサイクル(layer が
///   消えても text_textures の他の entry は無関係)が一致する——`textures` が
///   `LayerSource` を鍵にして「内容が同じなら共有する」設計を選んでいるのとは
///   **意図的に非対称**(text は layer 単位、solid/media は内容単位)。
/// - **`canvas_width`/`canvas_height`** — ラスタライズ先の canvas 寸法
///   (= [`text_texture_for`](Engine::text_texture_for) 呼び出し時点の comp 解像度)。
///   comp resize で同じ文字でも画素配置(組版の基準点)が変わりうるので鍵に含める。
/// - **`content_snapshot`** — **時刻 `t` そのものではなく、`t` で評価した後の中身**
///   を JSON 化した文字列。
///
/// # なぜ `t`(`RationalTime`)を鍵に使わないか
///
/// `TextDocument::content` は Hold 評価(裁定92 — 動くのは中身の文字列だけで組版は
/// 静止する)。同じキーフレーム区間内の異なる `t` は**同じ絵になる**——`t` を鍵に
/// 使うと「1フレームごとに別キャッシュ行」になり、静止テキストが毎フレーム
/// 再ラスタライズされてこのキャッシュを作る意味が消える。逆に「評価後の中身」を
/// 鍵にすれば、Hold 区間の全フレームが1つの cache hit に落ちる
/// (`text_layer_reuses_cached_texture_across_frames_within_a_hold_span` が固定する)。
///
/// # なぜ `TextDocument`/`Revision` をそのまま鍵にできないか
///
/// `TextDocument`(`motolii_store::TextDocument`)は `PartialEq` はあるが `Eq`/`Hash`
/// を derive できない(`TextDocumentStyle::fill: [f64; 4]` 等の浮動小数、`f64` は
/// `Eq`/`Hash` 非実装)。`motolii_store::Document::revision()`/`DisplayRevision` は
/// **Document 全体**の世代であって `ResolvedLayer` 単位の細粒度カウンタを持たない
/// (`motolii-store` 側リサーチで確認済み——`re_chunk_store::ChunkStoreGeneration` を
/// 私有フィールドに包むだけで `Hash` も無い)ので、そのままでは使えない。
/// `serde_json::to_string`(`TextDocument` 内の全型は `motolii-store` 側で既に
/// `Serialize` を derive 済み)で `Eq + Hash` を持つ `String` へ落とすのが最小の
/// 追加コストで正しい鍵を作る道——`serde_json` は `next/Cargo.toml` の既存 workspace
/// 依存(`motolii-store` が内部で使っている物と同じ)をこの crate へ配線するだけで、
/// 新規の外部依存は増やさない(`Cargo.toml` 参照)。
///
/// # 上限を持たない理由
///
/// `frames`(実素材フレーム)と違う判断。`frames` に上限が要る理由は「秒間30枚 ×
/// 分単位の書き出しで数千キーが生まれる」実測(`Engine::frames` の doc 参照)だが、
/// text layer の内容はキーフレームの数だけしか値を持たない(編集者が打つ Hold
/// キーフレームは通常フレーム数のオーダーにならない)——増えて問題になったら
/// `frames` と同じ FIFO を足せばよい(先に最適化すると決まっていない形に合わせた
/// 形になる、という既存文化と同じ判断)。
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextCacheKey {
    layer: LayerId,
    canvas_width: u32,
    canvas_height: u32,
    content_snapshot: String,
}

impl TextCacheKey {
    /// `document`/`t` から「この layer の絵を決める入力」を JSON 化した文字列へ畳む。
    /// `t` そのものは鍵に含めない(型 doc の「なぜ `t` を鍵に使わないか」節参照)。
    fn new(
        layer: LayerId,
        document: &TextDocument,
        t: RationalTime,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        // Hold 評価(`rasterize_text_document` が読むのと同じ値)。
        let content = document.content.eval(t);
        // シリアライズ失敗は実質あり得ない(NaN も循環参照も持たない構造体の組み合わせ)
        // ——万一失敗しても cache miss 扱いになるだけで安全側(絵は壊れない、毎回
        // 描き直すだけ)なので `unwrap_or_default` で空文字列へ倒す。
        let content_snapshot = serde_json::to_string(&(
            &content,
            document.justify,
            document.wrap_size,
            &document.styles,
        ))
        .unwrap_or_default();
        Self {
            layer,
            canvas_width,
            canvas_height,
            content_snapshot,
        }
    }
}

/// shape texture のキャッシュ鍵(発注「シェイプが画に出るようにする」、
/// 2026-08-22)。[`TextCacheKey`]と同じ考え方(`layer`+canvas 寸法+
/// content snapshot)——ただし **`t` を引数に取らない**。
///
/// # なぜ `textures: HashMap<LayerSource, GpuTexture2D>` を再利用しないか
///
/// [`TextCacheKey`]の doc と同じ理由: `LayerSource::Shape` は中身を持たない unit
/// variant なので、複数の shape layer がそのまま `textures` を使うと1つの鍵に
/// 衝突する。
///
/// # なぜ `t`(`RationalTime`)を引数にも鍵にも持たないか
///
/// [`TextCacheKey`]は「`t` は取るが鍵には含めない」(Hold 評価後の内容が鍵)だが、
/// `ShapeCacheKey` は**そもそも `t` を取らない**——`motolii_vector::Shape`/
/// `ShapeNode`(`StoreView::shapes` が返す `Layer:shapes` component の中身)は
/// `TextDocument::content`(`ContentTrack`、Hold 評価)のような時間評価の型を
/// 一切持たない静的な記述で、`StoreView::shapes(layer)` 自体が時刻を引数に取らない
/// (`motolii-store` の `view.rs` 参照)。つまり「評価後の形」は常に
/// `shapes()` の返り値そのものであり、`t` を混ぜても区別が増えるどころか
/// [`TextCacheKey`]が避けた落とし穴(毎フレーム別キャッシュ行になる)をそのまま
/// 踏む——`shape.rs` module doc の「時刻を取らない理由」節も参照。shape の頂点が
/// 時間で動く経路は、shape 自身の記述ではなく **layer の transform**
/// (`ResolvedLayer.placement`)側にあり、そちらは `layers_from_resolved`/
/// `render_with_camera_override` が(shape かどうかに関わらず)常に持ち回っている
/// 既存の経路がそのまま担う。
///
/// # 鍵の中身
///
/// - **`layer: LayerId`** — [`TextCacheKey`]と同じ理由(layer 単位、
///   `textures` が `LayerSource` を鍵にする「内容が同じなら共有する」設計とは
///   意図的に非対称)。
/// - **`canvas_width`/`canvas_height`** — ラスタライズ先の canvas 寸法
///   (= [`Engine::shape_texture_for`]/[`Engine::shape_texture_from_shapes`]呼び出し
///   時点の comp 解像度)。comp resize で同じ shape でも画素配置(`Canvas::centered`
///   の中心)が変わりうるので鍵に含める。
/// - **`content_snapshot`** — `shapes`(`&[ShapeNode]`)そのものを JSON 化した
///   文字列。[`ShapeNode`]/[`crate::text`]と同じく`motolii-vector`側で既に
///   `Serialize` を derive 済みなので、追加の型実装は要らない。
///
/// # 上限を持たない理由
///
/// [`TextCacheKey`]の doc「上限を持たない理由」節と同型 — shape layer の内容も
/// キーフレームの数だけしか値を持たない(このレーンでは shape 自身は時間評価すら
/// 持たないので、実質「layer の数だけ」)。増えて問題になったら `frames` と同じ
/// FIFO を足せばよい。
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ShapeCacheKey {
    layer: LayerId,
    canvas_width: u32,
    canvas_height: u32,
    content_snapshot: String,
}

impl ShapeCacheKey {
    /// `shapes` から「この layer の絵を決める入力」を JSON 化した文字列へ畳む。
    fn new(layer: LayerId, shapes: &[ShapeNode], canvas_width: u32, canvas_height: u32) -> Self {
        // シリアライズ失敗は実質あり得ない(`Shape`/`ShapeNode` 内の全フィールドは
        // 有限の f64/enum/String の組み合わせ)——万一失敗しても cache miss 扱いに
        // なるだけで安全側(絵は壊れない、毎回描き直すだけ)なので
        // `unwrap_or_default` で空文字列へ倒す([`TextCacheKey::new`] と同じ判断)。
        let content_snapshot = serde_json::to_string(shapes).unwrap_or_default();
        Self {
            layer,
            canvas_width,
            canvas_height,
            content_snapshot,
        }
    }
}
