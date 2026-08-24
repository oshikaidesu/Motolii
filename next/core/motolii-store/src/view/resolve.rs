//! comp 時刻での意味解決 — `resolved_*`/`resolve*` 系。`view.rs` から移送
//! (裁定220 SP-3、中身は変えていない)。生の store 読み口は `view.rs`(親
//! モジュール)側に残る — ここはその上に乗る評価層のみ。`world_affine`/local
//! transform の部品は `transform.rs`(子モジュール)へさらに分けてある。

mod transform;

use std::collections::{HashMap, HashSet};

use motolii_core::RationalTime;
use motolii_eval::Value;

use crate::{
    property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer, ResolvedMask,
    StoreError, TextDocument,
};

#[cfg(test)]
use crate::Document;
use super::StoreView;
#[cfg(test)]
use transform::{reset_world_affine_compute_count, world_affine_compute_count};

impl<'a> StoreView<'a> {
    /// この comp 時刻でのカメラ(裁定113/115)。track が無い property は既定値になる
    /// (パン無し・zoom=1・roll=0)— 裁定20「キーを打っていない property は静止値」を
    /// カメラにもそのまま適用する。
    pub fn resolve_camera(&self, t: RationalTime) -> Result<motolii_core::ResolvedCamera, StoreError> {
        let center_property = PropertyId::camera(property::CAMERA_CENTER)?;
        let center = match self.camera_value_at(&center_property, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::CAMERA_CENTER
                )))
            }
            None => [0.0, 0.0],
        };

        let zoom_property = PropertyId::camera(property::CAMERA_ZOOM)?;
        let zoom = match self.camera_value_at(&zoom_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ZOOM
                )))
            }
            None => 1.0,
        };

        let roll_property = PropertyId::camera(property::CAMERA_ROLL)?;
        let roll_degrees = match self.camera_value_at(&roll_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ROLL
                )))
            }
            None => 0.0,
        };

        Ok(motolii_core::ResolvedCamera {
            center,
            zoom,
            roll_degrees,
        })
    }

    /// comp 時刻での text-layer の中身。**[`Self::text_document`] の静的値の上に
    /// `text_style.{id}.*`/`text_justify` の track(裁定214 同日訂正版で時間軸に
    /// 乗った)を重ねる**——[`Self::resolved_masks`](形状/不透明度の overlay)と
    /// 同じ形: 「track があればその値、無ければ静的値」を `value_at` 1本
    /// (overlay→track→slot→modulator の解決を re-implement しない、裁定215)経由で
    /// 読み、型が合わなければ黙って近似せず `Err` にする(`resolved_masks` と同じ
    /// 「壊れた Document を静かに握り潰さない」規約)。
    ///
    /// **静的値と track の正本はどちらか**: track が正本、`TextDocumentStyle`/
    /// `TextDocument::justify` の静的フィールドは「track が無い時の既定値」——
    /// 二重帳簿にしないため、書き口([`crate::Intent::SetTextDocument`])は今も
    /// 静的値だけを書く(track は Inspector の drag/Key 列が別途書く、write-set
    /// 外の shell 配線が要る枝は RETURN 参照)。**読み出し側はこのメソッドを使う**
    /// のが正——[`Self::text_document`] は「今も編集フォームに出す生の静的値」
    /// (drag 開始前の初期値・保存フォーマットそのもの)用に残す。
    ///
    /// 性能: `value_at`→`source_at_path` は revision 鍵の `track_cache`(裁定140)を
    /// 経由するので、毎フレーム JSON を再解析しない——track が無い(=既定値のまま)
    /// layer では `source_at_path` が `Ok(None)` を返すだけで、パース済み値の
    /// キャッシュ命中コストのみ。
    pub fn resolved_text_document(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<TextDocument>, StoreError> {
        let Some(mut document) = self.text_document(layer)? else {
            return Ok(None);
        };

        let justify_property = PropertyId::text_justify();
        match self.value_at(layer, &justify_property, t)? {
            Some(Value::Enum(v)) => {
                document.justify = crate::TextJustify::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "`text_justify` track に未知の enum 値が入っている: {v}"
                    ))
                })?;
            }
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "`text_justify` に enum でない値が入っている(track が壊れている): {other:?}"
                )))
            }
            None => {}
        }

        for style in &mut document.styles {
            let size_property = PropertyId::text_style_size(style.id);
            if let Some(value) = self.value_at(layer, &size_property, t)? {
                match value {
                    Value::F64(v) => style.size = v as f32,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.size に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let line_height_property = PropertyId::text_style_line_height(style.id);
            if let Some(value) = self.value_at(layer, &line_height_property, t)? {
                match value {
                    Value::F64(v) => style.line_height = Some(v as f32),
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.line_height に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let tracking_property = PropertyId::text_style_tracking(style.id);
            if let Some(value) = self.value_at(layer, &tracking_property, t)? {
                match value {
                    Value::F64(v) => style.tracking = v as f32,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.tracking に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let fill_property = PropertyId::text_style_fill_color(style.id);
            if let Some(value) = self.value_at(layer, &fill_property, t)? {
                match value {
                    Value::Color(c) => style.fill = c,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.fill_color に色でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let stroke_property = PropertyId::text_style_stroke_color(style.id);
            if let Some(value) = self.value_at(layer, &stroke_property, t)? {
                match value {
                    Value::Color(c) => style.stroke_color = Some(c),
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.stroke_color に色でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }
        }

        Ok(Some(document))
    }

    /// comp 時刻でのマスク。形状・不透明度・膨張を普通の property track から取る。
    ///
    /// **裁定214**: mode/inverted も track で上書きできる — track が無ければ静的
    /// [`Mask::mode`]/[`Mask::inverted`] が既定値(`resolved_blend_mode` と同じ
    /// overlay の形)。
    fn resolved_masks(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedMask>, StoreError> {
        let mut out = Vec::new();
        for mask in self.masks(layer)? {
            let mode = match self.value_at(layer, &PropertyId::mask_mode(mask.id), t)? {
                Some(Value::Enum(v)) => crate::MaskMode::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "マスク {} の mode track に未知の enum 値が入っている: {v}",
                        mask.id
                    ))
                })?,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の mode に enum でない値が入っている(track が壊れている): {other:?}",
                        mask.id
                    )))
                }
                None => mask.mode,
            };

            let inverted = match self.value_at(layer, &PropertyId::mask_inverted(mask.id), t)? {
                Some(Value::Bool(v)) => v,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の inverted に真偽でない値が入っている(track が壊れている): {other:?}",
                        mask.id
                    )))
                }
                None => mask.inverted,
            };

            let shape_property = PropertyId::mask_shape(mask.id);
            let shape = match self.value_at(layer, &shape_property, t)? {
                Some(Value::Path(path)) => path,
                // **黙って飛ばさない**。形状の無いマスクは壊れた Document であって、
                // 既定値で描くと利用者には「マスクが勝手に消えた」としか見えない。
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の形状にパスでない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => {
                    return Err(StoreError::Property(format!(
                        "マスク {} に形状が無い(`mask.{}.shape` が未設定)",
                        mask.id, mask.id
                    )))
                }
            };

            let opacity_property = PropertyId::mask_opacity(mask.id);
            // キーを打っていない property は静止値(裁定20)。既定は不透明。
            let opacity = match self.value_at(layer, &opacity_property, t)? {
                Some(Value::F64(v)) => v as f32,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の不透明度に数値でない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => 1.0,
            };

            let expansion_property = PropertyId::mask_expansion(mask.id);
            let expansion = match self.value_at(layer, &expansion_property, t)? {
                Some(Value::F64(v)) if v.is_finite() => v,
                Some(Value::F64(v)) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の膨張に有限でない値が入っている: {v}",
                        mask.id
                    )))
                }
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の膨張に数値でない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => 0.0,
            };

            out.push(ResolvedMask {
                mode,
                inverted,
                opacity: opacity.clamp(0.0, 1.0),
                expansion,
                shape,
            });
        }
        Ok(out)
    }

    /// comp 時刻での effect スタック。**disabled な effect はここに現れない**
    /// (`resolve_with_solo` が hidden な layer を `None` で弾くのと同じ形 — 「切る」を
    /// フラグとして運ばず、入口で除く)。空スタックは `self.effects(layer)?` の1回の
    /// 読み出しだけで即 return し、`self.properties(layer)`(component 一覧の走査)
    /// までは踏まない — effect の無い layer の resolve コストを増やさないため。
    ///
    /// **裁定213**: 有効/無効はもう `EffectInstance::enabled` という静止フィールド
    /// ではなく、`PropertyId::effect_enabled` の track(`Value::Bool`、Hold 補間)を
    /// この comp 時刻 `t` で評価して読む——「切る」がキーフレーム可能になった
    /// (`effect.rs` モジュール doc「2026-08-23」節参照)。
    fn resolved_effects(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedEffect>, StoreError> {
        let effects = self.effects(layer)?;
        if effects.is_empty() {
            return Ok(Vec::new());
        }

        // param 名の発見は既存の汎用列挙(裁定57「store に聞く」) — effect 専用の
        // 列挙 API を新設しない(縫い目調査 1a)。
        let properties = self.properties(layer);

        let mut out = Vec::with_capacity(effects.len());
        for effect in effects {
            let enabled_property = crate::PropertyId::effect_enabled(effect.id);
            // キーを打っていない = 既定で有効(`mask_opacity` の「既定 1.0」と同じ
            // 判断 — 裁定20 の応用)。「壊れている」(真偽でない値)は近似せず Err。
            let enabled = match self.value_at(layer, &enabled_property, t)? {
                Some(Value::Bool(v)) => v,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "effect {} の enabled に真偽でない値が入っている: {other:?}",
                        effect.id
                    )))
                }
                None => true,
            };
            if !enabled {
                continue;
            }
            let prefix = format!("{}{}.param.", property::EFFECT_PREFIX, effect.id);
            let mut params = Vec::new();
            for candidate in &properties {
                let Some(param_name) = candidate.name().strip_prefix(prefix.as_str()) else {
                    continue;
                };
                // track の評価は既存の汎用経路(`value_at`)をそのまま再利用する —
                // 新しい評価器は書かない(EXACT TARGET #2)。track が実在すれば
                // `value_at` は必ず `Some` を返す(scalar/vec2 と同じ前提)。
                if let Some(value) = self.value_at(layer, candidate, t)? {
                    params.push((param_name.to_owned(), value));
                }
            }
            out.push(ResolvedEffect {
                plugin_id: effect.plugin_id,
                params,
            });
        }
        Ok(out)
    }

    /// comp 時刻での layer の姿。**合成器へ渡す唯一の形**。
    ///
    /// track が無い property は既定値になる(位置 0、不透明度 1、大きさは素材のまま)。
    /// これは AE で「キーを打っていない property は静止値」と同じ扱いである。
    /// `Ok(None)` = **この時刻にこの layer は居ない**(配置の外)。
    pub fn resolve(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<ResolvedLayer>, StoreError> {
        // `any_solo` は comp 全体を見ないと判定できないので、単発呼び出しではここで
        // 1回だけ走査する。`resolved_layers` は全 layer を回る側で既に同じ走査を
        // 1回すませているので、そちらは `resolve_with_solo` を直接呼んで
        // この再走査を踏まない(2026-08-20 の性能回帰: 層数 N に対し、ここで毎回
        // `any_solo` を呼ぶと `resolved_layers` 経由で N 回 × 全層走査 = O(N²) の
        // attrs 二重読みになっていた)。
        let any_solo = self.any_solo(t)?;
        // 単発呼び出しなので世界合成のメモ/循環ガードもこの1回限りの使い捨て
        // (裁定173 H1)。祖先を跨いで共有したいのは [`Self::resolved_layers`] が
        // 1回の document-wide resolve の中で複数の子から同じ祖先を引く場面 —
        // そちらは呼び出し側で1つの `memo` を作って全 layer 分使い回す。
        let present: HashSet<LayerId> = self.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)
    }

    /// [`Self::resolve`] の本体。`any_solo` を呼び出し側から受け取ることで、
    /// 全層を回る [`Self::resolved_layers`] が層ごとに `any_solo` を再走査しなくて
    /// 済むようにする(1パスで導出した solo 判定を使い回す)。
    ///
    /// `present`/`memo`/`visiting` は世界合成(裁定173 H1、[`Self::world_affine`])の
    /// 入出力 — 呼び出し側([`Self::resolve`]/[`Self::resolved_layers`])が1回の
    /// resolve 呼び出し分だけ作り、複数 layer の resolve を跨いで使い回す
    /// (「同じフレームで親を二度解決しない」がここで成立する)。
    fn resolve_with_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<Option<ResolvedLayer>, StoreError> {
        let Some(meta) = self.meta(layer)? else {
            return Ok(None);
        };

        // hidden は「今フレームは描かない」— present(削除)とは別物(裁定108(c) 系)。
        // 属性が一度も書かれていない layer は既定(非 hidden)として扱う。
        // **裁定214**: hidden は track でも上書きできる(`resolved_hidden` —
        // track があればその値、無ければ静的 `attrs.hidden` が既定、裁定20)。
        let attrs = self.attrs(layer)?.unwrap_or_default();
        let hidden = self.resolved_hidden(layer, t, attrs.hidden)?;
        if hidden {
            return Ok(None);
        }

        // solo: comp のどこかに solo な layer が居るなら、solo でない layer は
        // hidden と同じ経路でここに落ちる。**hidden が勝つ** — 上の hidden 判定を
        // 通り抜けた(= 自分は hidden ではない)layer だけがここへ来るので、
        // 「隠した層を solo で復活させる」ことは構造的に起きない。solo な layer
        // 自身が同時に hidden なら、その layer は上で既に弾かれているが、
        // 「comp のどこかに solo な layer が居る」という判定(`any_solo`)には
        // hidden かどうかを問わず含める — solo フラグは hidden と独立な「意図」の
        // 表明であって、hidden がそれを見えなくするだけで無かったことにはしない
        // (裁定119 のグループ AND 導出と衝突しない、単層の bool のまま)。
        // **裁定214**: solo も track でも上書きできる(`resolved_solo` — hidden と
        // 同じ overlay の形)。
        let solo = self.resolved_solo(layer, t, attrs.solo)?;
        if any_solo && !solo {
            return Ok(None);
        }

        // 時間の判定は Document がする。engine は解決済みの素材フレームを受け取るだけ。
        let Some(composition) = self.composition()? else {
            return Ok(None);
        };
        let comp_frame = t
            .try_to_frame_floor(composition.fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let Some(mut source_frame) = meta.timing.source_frame(comp_frame) else {
            return Ok(None);
        };
        // `sr`(Time Stretch)の track 版(A03「Speed(ATTRS)」、裁定63 の穴)。
        // track があれば `LayerTiming.speed`(静的値)の代わりに使う —
        // `source_frame` は現在フレームだけの純粋関数なので、速度が時間で変わる
        // なら「start からここまでの積算」でなければ正しくない(1点上書きでは
        // 足りない、`LayerTiming::source_frame_with_speed_track` のコメント参照)。
        // `TIME_REMAP` と同時に張られていてもここでは構わない — 下の remap 分岐が
        // 最終的に勝つ(適用順は変えない)。
        if let Some(speed_track) = self.track(layer, &PropertyId::new(property::SPEED)?)? {
            if let Some(v) =
                meta.timing
                    .source_frame_with_speed_track(comp_frame, &speed_track, composition.fps)?
            {
                source_frame = v;
            }
        }
        // `tm`(Time Remap、precomposition-layer)。track があれば**素材のフレーム番号を
        // 直接**上書きする — 通常の speed/trim による写像より優先する(裁定65 が
        // timing から追い出した分、property 側で戻す)。timing が「居る/居ない」を
        // 決める点は変わらないので、上の `covers` 判定はそのまま活きる。
        if let Some(remap) = self.value_at(layer, &PropertyId::new(property::TIME_REMAP)?, t)? {
            match remap {
                Value::F64(v) => source_frame = v.floor() as i64,
                other => {
                    return Err(StoreError::Property(format!(
                        "{} に数値でない値が入っている: {other:?}",
                        property::TIME_REMAP
                    )))
                }
            }
        }
        // 実素材の大きさは probe しないと分からない。ここでは 0 を置き、engine が
        // 「track が無く declared も無い」場合だけ素材の実寸で埋める。
        let size = meta.source.declared_size().unwrap_or([0.0, 0.0]);

        // 標準 property は必ず構築できる(予約語でない・空でない)ので、
        // ここで失敗したら**コードの誤り**であって Document の内容ではない。
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                // track はあるが型が違う。既定値へ落とすと「打ったキーが効かない」に見える。
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };

        // **comp 空間へのアフィン**(裁定173 H1)。局所の行列そのものの意味の正本は
        // 今まで通り `motolii-core`(裁定58)— ここは「親の world アフィンを左から
        // 合成する」再帰の入口を呼ぶだけ([`Self::world_affine`] 参照)。parent が
        // 無い/tombstone/循環なら local のみへ縮退するので、parent を1つも使って
        // いない既存 Document はここまで含めて今まで通りの値を返す。
        let transform = self.world_affine(layer, t, present, memo, visiting)?;

        Ok(Some(ResolvedLayer {
            id: layer,
            placement: LayerPlacement {
                transform,
                opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
                order: meta.order,
                // 裁定113/116: 全員 z=0 既定。`position.x`/`position.y`(split-position)
                // の隣に同じ流儀で置いた `position.z`。
                z: scalar(property::POSITION_Z, 0.0)?,
            },
            declared_size: size,
            source: meta.source,
            source_frame,
            masks: self.resolved_masks(layer, t)?,
            effects: self.resolved_effects(layer, t)?,
            // **裁定214**: blend_mode/matte(mode)も track で上書きできる
            // (`resolved_blend_mode`/`resolved_matte` — hidden/solo と同じ overlay の形)。
            blend_mode: self.resolved_blend_mode(layer, t, attrs.blend_mode)?,
            matte: self.resolved_matte(layer, t, attrs.matte)?,
            pinned: attrs.pinned,
        }))
    }


    /// comp のどこかに `solo=true` な layer が居るか。**hidden は問わない**
    /// (`resolve` のコメント参照 — solo という意図の表明は hidden と独立)。
    ///
    /// **裁定214**: solo は `PropertyId::solo` の track でも表明できるので、この
    /// 判定は comp 全体の `t` 時点の solo 状態を見る必要がある — `t` を受け取り、
    /// 各 layer の solo は [`Self::resolved_solo`](track があればその値、無ければ
    /// 静的値、裁定20)で読む。
    fn any_solo(&self, t: RationalTime) -> Result<bool, StoreError> {
        for layer in self.layers() {
            let static_solo = self.attrs(layer)?.unwrap_or_default().solo;
            if self.resolved_solo(layer, t, static_solo)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// **裁定214**: solo の overlay 本体 — `resolved_masks`/`resolved_text_document`
    /// と同じ形(「track があればその値、無ければ静的値」を `value_at` 1本経由)。
    /// `Value::Bool`(Hold 補間、裁定213)。型が合わなければ黙って近似せず `Err`。
    fn resolved_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: bool,
    ) -> Result<bool, StoreError> {
        match self.value_at(layer, &PropertyId::solo(), t)? {
            Some(Value::Bool(v)) => Ok(v),
            Some(other) => Err(StoreError::Property(format!(
                "solo に真偽でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    /// **裁定214**: hidden の overlay 本体。[`Self::resolved_solo`] と同じ形。
    fn resolved_hidden(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: bool,
    ) -> Result<bool, StoreError> {
        match self.value_at(layer, &PropertyId::hidden(), t)? {
            Some(Value::Bool(v)) => Ok(v),
            Some(other) => Err(StoreError::Property(format!(
                "hidden に真偽でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    /// **裁定214**: blend mode の overlay 本体。`Value::Enum`
    /// (`crate::BlendMode::to_enum_value`/`from_enum_value`)、補間は Hold(裁定213)。
    fn resolved_blend_mode(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: crate::BlendMode,
    ) -> Result<crate::BlendMode, StoreError> {
        match self.value_at(layer, &PropertyId::blend_mode(), t)? {
            Some(Value::Enum(v)) => crate::BlendMode::from_enum_value(v).ok_or_else(|| {
                StoreError::Property(format!(
                    "`blend_mode` track に未知の enum 値が入っている: {v}"
                ))
            }),
            Some(other) => Err(StoreError::Property(format!(
                "`blend_mode` に enum でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    /// **裁定214**: matte の overlay 本体。**`Matte.layer`(参照先)は対象外**
    /// (裁定214 修正版、A03副監督A発注が明示的に後回しにした枝) — track が上書き
    /// できるのは `mode` のみで、`matte` 自身が `None`(このレイヤはマットに
    /// されていない)なら mode track があっても上書き先が無いので効かない
    /// (`PropertyId::matte_mode` doc 参照)。
    fn resolved_matte(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: Option<crate::Matte>,
    ) -> Result<Option<crate::Matte>, StoreError> {
        let Some(mut matte) = static_value else {
            return Ok(None);
        };
        match self.value_at(layer, &PropertyId::matte_mode(), t)? {
            Some(Value::Enum(v)) => {
                matte.mode = crate::MatteMode::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "`matte_mode` track に未知の enum 値が入っている: {v}"
                    ))
                })?;
            }
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "`matte_mode` に enum でない値が入っている(track が壊れている): {other:?}"
                )))
            }
            None => {}
        }
        Ok(Some(matte))
    }

    /// この時刻に描くべき layer を**奥から手前の順**で返す。
    ///
    /// `any_solo` はここで**1回だけ**走査する。`resolve` を層ごとに呼ぶ素朴な実装だと、
    /// `resolve` が毎回 comp 全層を re-scan するので N layer で O(N²) の attrs 二重読みに
    /// なる(2026-08-20 の性能回帰の原因、r2 probe 実測で発覚)。resolve は既に
    /// layer ごとに自分の attrs を読んでいるので、その1パスから solo の有無だけ
    /// 先に導出して使い回す。
    ///
    /// **世界合成のメモ(裁定173 H1)もここで1回だけ作り、全 layer 分使い回す** —
    /// 兄弟が同じ祖先を parent に持つ場合、祖先の world アフィンは document-wide の
    /// この呼び出し1回につき1回しか解決されない(メモ化の呼び出し回数証明 oracle)。
    pub fn resolved_layers(&self, t: RationalTime) -> Result<Vec<ResolvedLayer>, StoreError> {
        let any_solo = self.any_solo(t)?;
        let layers = self.layers();
        let present: HashSet<LayerId> = layers.iter().copied().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        for layer in layers {
            if let Some(resolved) =
                self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)?
            {
                out.push(resolved);
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
    }
}

/// 裁定173 H1 の白箱ユニットテスト。`world_affine`/`resolve_with_solo` は
/// `pub(crate)` ですらない完全 private なので、この crate 自身の `#[cfg(test)]`
/// からしか叩けない(統合テスト `tests/*.rs` は別クレートなので届かない) —
/// 数値証明・serde 往復・tombstone 縮退は公開 API だけで書けるので
/// `tests/transform_hierarchy.rs` に置き、ここには**メモ化の呼び出し回数証明**
/// (private な計測フックが要る)だけを置く。
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Composition, Fps, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};

    fn t(frame: i64) -> RationalTime {
        RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
    }

    fn place(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Null,
                    order: layer.0 as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
        if let Some(parent) = parent {
            doc.apply(Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    parent: Some(Some(parent)),
                    ..Default::default()
                },
            })
            .unwrap();
        }
    }

    /// **メモ化の呼び出し回数証明**(裁定173 H1 oracle)。3階層(root A ← mid B ←
    /// leaf C1/C2 の2兄弟)で、B(と A)を2人の子から引いても `world_affine` の
    /// 本体計算(memo miss)は B/A それぞれちょうど1回しか起きない — 旧世界
    /// `spatial_resolve.rs::ensure_resolve_affine`(メモ化 `HashMap` + 事前解決パス)の
    /// 概念移植が効いていることの直接証拠。
    #[test]
    fn shared_ancestor_is_resolved_exactly_once_across_siblings() {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();

        let (a, b, c1, c2) = (LayerId(1), LayerId(2), LayerId(3), LayerId(4));
        place(&mut doc, a, None);
        place(&mut doc, b, Some(a));
        place(&mut doc, c1, Some(b));
        place(&mut doc, c2, Some(b));

        let view = doc.view();
        let present: HashSet<LayerId> = view.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        reset_world_affine_compute_count();

        // C1 を解決: local(C1) + local(B) + local(A) の3回が「初めて」計算される。
        view.world_affine(c1, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            3,
            "root/mid/leaf1 の3層でちょうど3回のはず(まだ誰も共有していない)"
        );

        // C2 を解決: B と A は memo に既に居るので、C2 自身の local だけが増える。
        view.world_affine(c2, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            4,
            "B(と A)が2人目の子 C2 のために再計算されてしまっている(メモ化が効いていない)"
        );
    }
}
