//! アプリ内クリップボード(NeoUtl/AviUtl 同型 — **OS clipboard ではない**)。
//!
//! 普通地図 消化第1波 U1(`next/reference/normal-map.tsv`)の発注書:
//! Copy / Paste / Cut / Duplicate / Select All / Deselect All(layer 対象)。
//! 意味は発注書と `next/reference/timeline-grammar.md` §4 が正本:
//!
//! - **layer とその全 track(keyframe 込み)・effect stack の Document 表現**を
//!   [`Clipboard`] が保持する(`meta`/`attrs`/`masks`/`effects`/`shapes`/`text`/
//!   property 単位の track・slot 参照、すべて)
//! - **Paste は元時刻のまま**(playhead ペーストは今回作らない、AE 同型)
//! - **Cut = Copy + 削除(1 undo)**。**Duplicate はクリップボードを経由しないその場複製**
//! - 複製後は増えた方を選ぶ(§4「Duplicate(Cmd+D)…複製後は増えた方を選ぶ」と同型)
//!
//! Document への書き口は[`Intent`]のみ(背骨1)。この module は intent の列を
//! 組み立てるだけで、`apply`/`apply_all` は呼び手(`Shell::update`)の仕事のまま
//! (書き口が1箇所という不変条件を崩さない)。

use motolii_store::{
    EffectInstance, Intent, LayerAttrs, LayerAttrsPatch, LayerId, LayerMeta, Mask, PropertyId,
    PropertySource, ShapeNode, StoreError, StoreView, TextDocument,
};

/// 1 layer 分の Document 表現の写し。**clipboard の中身の正本**。
///
/// `Duplicate` もこれを経由する(クリップボードへは置かないが、capture →
/// instantiate という同じ形を使い回す — 「複製」の意味を2箇所に書かない)。
#[derive(Clone, Debug)]
pub struct LayerSnapshot {
    meta: Option<LayerMeta>,
    attrs: Option<LayerAttrs>,
    masks: Vec<Mask>,
    effects: Vec<EffectInstance>,
    shapes: Vec<ShapeNode>,
    text: Option<TextDocument>,
    /// property ごとの生の出処(`Track` か `Slot` か)。**keyframe 込みで丸ごと**
    /// 保持する — mask の形状track・effect の param track も、この layer が持つ
    /// property である限りここに乗る(`StoreView::properties` が component を
    /// 名前で列挙せず store に聞く口をそのまま使うため、新しい component が
    /// 増えてもここを直さなくてよい)。
    properties: Vec<(PropertyId, PropertySource)>,
}

impl LayerSnapshot {
    /// `layer` の今の Document 表現をまるごと読む。**store に聞く**(裁定57) —
    /// component を名前で列挙しない。
    pub fn capture(view: &StoreView<'_>, layer: LayerId) -> Result<Self, StoreError> {
        let mut properties = Vec::new();
        for property in view.properties(layer) {
            if let Some(source) = view.property_source(layer, &property)? {
                properties.push((property, source));
            }
        }
        Ok(Self {
            meta: view.meta(layer)?,
            attrs: view.attrs(layer)?,
            masks: view.masks(layer)?,
            effects: view.effects(layer)?,
            shapes: view.shapes(layer)?,
            text: view.text_document(layer)?,
            properties,
        })
    }

    /// `new_id` として新規配置する intent 列を組む。**呼び手は必ず `apply_all` で
    /// 1回にまとめること**(1操作 = 1 undo、Copy→Paste も Duplicate もこの形)。
    ///
    /// **元の値をそのまま複製する**(order・timing・parent を含む) — Paste は
    /// 「元時刻のまま」(意味カプセルの拘束)、Duplicate は「その場複製」で、
    /// どちらも同じ複製規則になる。
    pub fn instantiate(&self, new_id: LayerId) -> Vec<Intent> {
        let mut intents = vec![Intent::AddLayer(new_id)];
        if let Some(meta) = &self.meta {
            intents.push(Intent::SetMeta {
                layer: new_id,
                meta: meta.clone(),
            });
        }
        if let Some(attrs) = &self.attrs {
            intents.push(Intent::SetAttrs {
                layer: new_id,
                patch: full_patch(attrs),
            });
        }
        if !self.masks.is_empty() {
            intents.push(Intent::SetMasks {
                layer: new_id,
                masks: self.masks.clone(),
            });
        }
        if !self.effects.is_empty() {
            intents.push(Intent::SetEffects {
                layer: new_id,
                effects: self.effects.clone(),
            });
        }
        if !self.shapes.is_empty() {
            intents.push(Intent::SetShapes {
                layer: new_id,
                shapes: self.shapes.clone(),
            });
        }
        if let Some(text) = &self.text {
            intents.push(Intent::SetTextDocument {
                layer: new_id,
                document: text.clone(),
            });
        }
        for (property, source) in &self.properties {
            intents.push(match source {
                PropertySource::Track(track) => Intent::SetTrack {
                    layer: new_id,
                    property: property.clone(),
                    track: track.clone(),
                },
                PropertySource::Slot(slot) => Intent::SetPropertySlot {
                    layer: new_id,
                    property: property.clone(),
                    slot: slot.clone(),
                },
            });
        }
        intents
    }
}

/// `attrs` の全フィールドを埋めた `LayerAttrsPatch`。**新規 layer への初回書き込み専用**
/// — 既存 layer の read-modify-write に使うと `Intent::SetAttrs` の read-modify-write と
/// 二重に読むだけで害は無いが、意味は「新規配置」に限定する(呼び手も新規 `AddLayer` の
/// 直後にしか呼ばない)。
fn full_patch(attrs: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
        hidden: Some(attrs.hidden),
        parent: Some(attrs.parent),
        blend_mode: Some(attrs.blend_mode),
        matte: Some(attrs.matte),
        name: Some(attrs.name.clone()),
        auto_orient: Some(attrs.auto_orient),
        pinned: Some(attrs.pinned),
        solo: Some(attrs.solo),
        locked: Some(attrs.locked),
        // Copy/Paste/Duplicate は元 layer の複製(AE 同型) — 差し色も他の属性と
        // 同じく「そのまま複製」で、新規生成点の決定論割当(`Shell::
        // label_color_for_new_layer`)は使わない。複製した瞬間に色が変わったら
        // 「複製した」という実感を裏切る。
        label_color: Some(attrs.label_color),
    }
}

/// アプリ内クリップボード本体。**OS clipboard ではない**(NeoUtl/AviUtl 同型) —
/// `Shell` が持つだけの front 状態で、Document には一切乗らない(`Session` と同じ
/// 「undo の対象ではない」身分)。今回の束は単一 layer のみを運ぶ(複数 layer 同時
/// Copy は multi-select UI が要る — write-set 外、RETURN の finding 参照)。
#[derive(Clone, Debug, Default)]
pub struct Clipboard {
    layer: Option<LayerSnapshot>,
}

impl Clipboard {
    pub fn set(&mut self, snapshot: LayerSnapshot) {
        self.layer = Some(snapshot);
    }

    pub fn get(&self) -> Option<&LayerSnapshot> {
        self.layer.as_ref()
    }
}

/// **Select All(正典 §4「Cmd+A 正: 見えている行だけ」)の意味関数**。
///
/// `visible` に渡された集合を**そのまま**採用し、自分で store へ聞き直さない —
/// 畳まれた(fold で隠れた)行を除く判断は呼び手の仕事のままにする契約。
///
/// **fold 状態は 2026-08-21 時点でまだ shell に無い**
/// (`docs/reviews/2026-08-19-egui-timeline-capability-ledger.md` の指摘どおり)ので、
/// 今の唯一の呼び手(`Shell::select_all_layers`)は `StoreView::layers()`(=
/// present な全 layer)をそのまま渡している。fold が実装された日、呼び手側が
/// 畳まれた行を弾いた `visible` を渡すだけでこの関数自体は変更不要になる —
/// 「渡された集合をそのまま返す(=再取得しない)」という契約がそのまま fold 対応の
/// 受け皿になる。
pub fn select_all(visible: &[LayerId]) -> Vec<LayerId> {
    let mut ids = visible.to_vec();
    ids.sort();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{
        BlendMode, Composition, ContentKeyframe, ContentTrack, Document, EffectId, FontRef, Fps,
        Interp, Keyframe, KeyframeTrack, LayerSource, LayerTiming, MaskId, MaskMode, PathSource,
        RationalTime, Shape, Slot, SlotId, TextAlignmentOptions, TextDocumentStyle, TextJustify,
        TextStyleId, Value,
    };

    fn t(frame: i64) -> RationalTime {
        RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
    }

    fn keys(values: &[(i64, f64)]) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        for (frame, value) in values {
            track.insert(Keyframe {
                t: t(*frame),
                value: Value::F64(*value),
                interp: Interp::Hold,
                spatial: None,
            });
        }
        track
    }

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        doc
    }

    fn minimal_text() -> TextDocument {
        let mut content = ContentTrack::new();
        content.insert(ContentKeyframe {
            t: t(0),
            content: "hello".to_owned(),
        });
        TextDocument {
            content,
            justify: TextJustify::Center,
            wrap_size: None,
            styles: vec![TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef {
                    path: "/fonts/test.otf".to_owned(),
                    fingerprint: None,
                    family: "Test".to_owned(),
                    style: "Regular".to_owned(),
                },
                size: 32.0,
                fill: [1.0, 1.0, 1.0, 1.0],
                line_height: None,
                tracking: 0.0,
                stroke_color: None,
                stroke_width: 0.0,
                stroke_over_fill: false,
                axes: Vec::new(),
                features: Vec::new(),
            }],
            slot_id: None,
            ranges: Vec::new(),
            alignment: TextAlignmentOptions::default(),
            runs: Vec::new(),
        }
    }

    /// **中核オラクル**: track(2キー = animated)・effect stack・mask・shape・
    /// text・slot 参照を全部持つ layer を capture → instantiate すると、コピー先が
    /// 全部同じ値になる(「track・effect ごと複製」)。
    #[test]
    fn capture_then_instantiate_reproduces_tracks_masks_effects_shapes_text_and_slot_refs() {
        let mut doc = doc_with_comp();
        let source = LayerId(1);
        let mask_id = MaskId(7);
        let effect_id = EffectId(3);
        let position = PropertyId::new("position").unwrap();
        let mask_shape = PropertyId::mask_shape(mask_id);
        let effect_param = PropertyId::effect_param(effect_id, "amount").unwrap();
        let slotted = PropertyId::new("opacity").unwrap();
        let slot_id = SlotId("primary".to_owned());

        doc.apply(Intent::SetSlots {
            slots: vec![Slot {
                id: slot_id.clone(),
                track: keys(&[(0, 1.0)]),
            }],
        })
        .unwrap();

        doc.apply_all([
            Intent::AddLayer(source),
            Intent::SetMeta {
                layer: source,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [10, 20, 30, 255],
                        width: 64,
                        height: 64,
                    },
                    order: 5,
                    timing: LayerTiming {
                        start: 12,
                        duration: 48,
                        source_in: 0,
                        ..Default::default()
                    },
                },
            },
            Intent::SetAttrs {
                layer: source,
                patch: LayerAttrsPatch {
                    hidden: Some(true),
                    name: Some("元 layer".to_owned()),
                    blend_mode: Some(BlendMode::Multiply),
                    ..Default::default()
                },
            },
            Intent::SetMasks {
                layer: source,
                masks: vec![Mask {
                    id: mask_id,
                    mode: MaskMode::Add,
                    inverted: true,
                }],
            },
            Intent::SetEffects {
                layer: source,
                effects: vec![EffectInstance {
                    id: effect_id,
                    plugin_id: "vism.glow".to_owned(),
                    enabled: true,
                }],
            },
            Intent::SetShapes {
                layer: source,
                // 裁定173 H4: `Layer:shapes` は `Vec<ShapeNode>`。既存の平坦 `Shape` は
                // `ShapeNode::Leaf` として渡す(clipboard 自体は group を作らない —
                // 中身をそのまま複製するだけの倉庫役なので、group の有無を問わず
                // clone するだけで足りる)。
                shapes: vec![ShapeNode::Leaf(Shape::new(PathSource::Rectangle {
                    size: motolii_store::VectorPoint { x: 10.0, y: 20.0 },
                }))],
            },
            Intent::SetTextDocument {
                layer: source,
                document: minimal_text(),
            },
            // 2キー track(animated) — 「値ごとコピー」だけでなく keyframe 込みで
            // 複製されることを確かめる。
            Intent::SetTrack {
                layer: source,
                property: position.clone(),
                track: keys(&[(0, 0.0), (30, 100.0)]),
            },
            Intent::SetTrack {
                layer: source,
                property: mask_shape.clone(),
                track: keys(&[(0, 1.0)]),
            },
            Intent::SetTrack {
                layer: source,
                property: effect_param.clone(),
                track: keys(&[(0, 0.5), (10, 0.9)]),
            },
            Intent::SetPropertySlot {
                layer: source,
                property: slotted.clone(),
                slot: slot_id.clone(),
            },
        ])
        .unwrap();

        let snapshot = LayerSnapshot::capture(&doc.view(), source).unwrap();
        let copy = LayerId(2);
        doc.apply_all(snapshot.instantiate(copy)).unwrap();

        let view = doc.view();
        assert_eq!(view.meta(copy).unwrap(), view.meta(source).unwrap());
        assert_eq!(view.attrs(copy).unwrap(), view.attrs(source).unwrap());
        assert_eq!(view.masks(copy).unwrap(), view.masks(source).unwrap());
        assert_eq!(view.effects(copy).unwrap(), view.effects(source).unwrap());
        assert_eq!(view.shapes(copy).unwrap(), view.shapes(source).unwrap());
        assert_eq!(
            view.text_document(copy).unwrap(),
            view.text_document(source).unwrap()
        );
        assert_eq!(
            view.track(copy, &position).unwrap(),
            view.track(source, &position).unwrap()
        );
        assert_eq!(
            view.track(copy, &position).unwrap().unwrap().keys().len(),
            2,
            "animated track の keyframe が畳まれている"
        );
        assert_eq!(
            view.track(copy, &mask_shape).unwrap(),
            view.track(source, &mask_shape).unwrap()
        );
        assert_eq!(
            view.track(copy, &effect_param).unwrap(),
            view.track(source, &effect_param).unwrap()
        );
        // slot 参照はスロット表を複製せず、同じ id を指すだけ。
        assert_eq!(
            view.property_source(copy, &slotted).unwrap(),
            Some(PropertySource::Slot(slot_id))
        );
    }

    /// [`select_all`] は渡された集合をそのまま返すだけで、自分で store へ
    /// 聞き直さない — 畳まれた(fold で隠れた)行を模して1枚だけ渡す。
    #[test]
    fn select_all_only_selects_what_the_caller_marks_visible() {
        let visible = vec![LayerId(1), LayerId(3)]; // LayerId(2) は畳まれて非表示という想定
        let selected = select_all(&visible);
        assert_eq!(selected, vec![LayerId(1), LayerId(3)]);
        assert!(
            !selected.contains(&LayerId(2)),
            "渡されていない(=畳まれた想定の)行まで選んでしまっている"
        );
    }
}
