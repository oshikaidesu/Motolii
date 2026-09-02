
use crate::doc::store::{
    property, AssetDraft, Composition, ContentKeyframe, ContentTrack, Document, EffectId,
    EffectInstance, Fps, FontRef, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, Marker, PropertyId, RationalTime,
    SourceFingerprintV1, Speed, TextAlignmentOptions, TextDocument, TextDocumentStyle,
    TextJustify, TextStyleId, Value,
};

pub struct Fixture {
    pub doc: Document,
    pub selected: LayerId,
    pub playhead: i64,
}

const FPS_NUM: i64 = 30;
const DURATION_FRAMES: i64 = 1800;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, FPS_NUM).expect("frame は RationalTime に収まる")
}

struct LayerSpec {
    name: &'static str,
    start: i64,
    duration: i64,
    rgba: [u8; 4],
}

const LAYERS: [LayerSpec; 15] = [
    LayerSpec {
        name: "タイトルロゴ",
        start: 0,
        duration: 90,
        rgba: [235, 235, 225, 255],
    },
    LayerSpec {
        name: "メインボーカル映像",
        start: 0,
        duration: 1800,
        rgba: [180, 140, 120, 255],
    },
    LayerSpec {
        name: "Bロール_街並み",
        start: 60,
        duration: 300,
        rgba: [120, 150, 170, 255],
    },
    LayerSpec {
        name: "1番Aメロ歌詞",
        start: 90,
        duration: 240,
        rgba: [210, 200, 235, 255],
    },
    LayerSpec {
        name: "Bメロ歌詞",
        start: 330,
        duration: 180,
        rgba: [210, 200, 235, 255],
    },
    LayerSpec {
        name: "ダンスカット",
        start: 480,
        duration: 360,
        rgba: [160, 120, 150, 255],
    },
    LayerSpec {
        name: "サビ歌詞",
        start: 510,
        duration: 270,
        rgba: [235, 200, 150, 255],
    },
    LayerSpec {
        name: "グリッチトランジション",
        start: 780,
        duration: 30,
        rgba: [220, 90, 90, 255],
    },
    LayerSpec {
        name: "2番Aメロ歌詞",
        start: 810,
        duration: 240,
        rgba: [210, 200, 235, 255],
    },
    LayerSpec {
        name: "波形ビジュアライザ",
        start: 810,
        duration: 990,
        rgba: [90, 180, 170, 255],
    },
    LayerSpec {
        name: "リリックモーション背景",
        start: 1050,
        duration: 450,
        rgba: [140, 110, 170, 255],
    },
    LayerSpec {
        name: "ラスサビ歌詞",
        start: 1200,
        duration: 300,
        rgba: [235, 200, 150, 255],
    },
    LayerSpec {
        name: "Bロール_夜景",
        start: 1500,
        duration: 200,
        rgba: [70, 90, 130, 255],
    },
    LayerSpec {
        name: "エンドカード",
        start: 1700,
        duration: 100,
        rgba: [235, 235, 225, 255],
    },
    LayerSpec {
        name: "クレジット",
        start: 1740,
        duration: 60,
        rgba: [200, 200, 200, 255],
    },
];

const FIXTURE_MEDIA: [&str; 3] = ["glow_default.png", "glow_strong.png", "blend_screen.png"];

pub fn build() -> Fixture {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(FPS_NUM, 1).expect("30fps"),
        duration_frames: DURATION_FRAMES,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置ける");

    let mut intents = Vec::new();
    let mut sabi_id = None;
    let mut logo_id = None;
    let mut vocal_id = None;
    let mut waveform_id = None;
    let mut dance_id = None;
    let mut glitch_id = None;
    let mut second_verse_id = None;

    for (index, spec) in LAYERS.iter().enumerate() {
        let id = LayerId((index + 1) as u64);
        intents.push(Intent::AddLayer(id));
        intents.push(Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: index as i16,
                timing: LayerTiming {
                    start: spec.start,
                    duration: spec.duration,
                    source_in: 0,
                    speed: Speed::NORMAL,
                },
            },
        });
        intents.push(Intent::SetShapes {
            layer: id,
            shapes: vec![crate::doc::store::rect_shape(spec.rgba, [320.0, 180.0])],
        });
        // 動かない初期値は**キーではなく値**で置く。キーで置くと、
        // 利用者が何も触っていないのに菱形が並ぶ。
        intents.push(Intent::SetConstant {
            layer: id,
            property: PropertyId::new(property::ANCHOR).expect("anchor は予約語ではない"),
            value: Value::Vec2([160.0, 90.0]),
        });
        intents.push(Intent::SetConstant {
            layer: id,
            property: PropertyId::new(property::POSITION).expect("position は予約語ではない"),
            value: Value::Vec2([160.0, 90.0]),
        });
        intents.push(Intent::SetAttrs {
            layer: id,
            patch: LayerAttrsPatch {
                name: Some(spec.name.to_owned()),
                label_color: Some(Some((id.0 % crate::ui::tokens::LABEL_PALETTE_LEN as u64) as u8)),
                ..Default::default()
            },
        });

        match spec.name {
            "サビ歌詞" => sabi_id = Some(id),
            "タイトルロゴ" => logo_id = Some(id),
            "メインボーカル映像" => vocal_id = Some(id),
            "波形ビジュアライザ" => waveform_id = Some(id),
            "ダンスカット" => dance_id = Some(id),
            "グリッチトランジション" => glitch_id = Some(id),
            "2番Aメロ歌詞" => second_verse_id = Some(id),
            _ => {}
        }
    }

    intents.push(Intent::SetMarkers {
        markers: vec![
            Marker {
                name: "Aメロ".to_owned(),
                time: t(150),
                duration: RationalTime::ZERO,
                body: String::new(),
            },
            Marker {
                name: "サビ".to_owned(),
                time: t(510),
                duration: RationalTime::ZERO,
                body: String::new(),
            },
            Marker {
                name: "ラスサビ".to_owned(),
                time: t(1200),
                duration: RationalTime::ZERO,
                body: String::new(),
            },
        ],
    });

    let logo_id = logo_id.expect("タイトルロゴ layer がある");
    let sabi_id = sabi_id.expect("サビ歌詞 layer がある");
    let vocal_id = vocal_id.expect("メインボーカル映像 layer がある");

    let mut logo_opacity = KeyframeTrack::new();
    logo_opacity.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    logo_opacity.insert(Keyframe {
        t: t(20),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    logo_opacity.insert(Keyframe {
        t: t(70),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    logo_opacity.insert(Keyframe {
        t: t(90),
        value: Value::F64(0.0),
        interp: Interp::Hold,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: logo_id,
        property: PropertyId::new(property::OPACITY).expect("opacity は予約語ではない"),
        track: logo_opacity,
    });

    let mut sabi_position = KeyframeTrack::new();
    sabi_position.insert(Keyframe {
        t: t(510),
        value: Value::Vec2([1120.0, 850.0]),
        interp: Interp::Bezier {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        },
        spatial: None,
    });
    sabi_position.insert(Keyframe {
        t: t(570),
        value: Value::Vec2([1120.0, 630.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: sabi_id,
        property: PropertyId::new(property::POSITION).expect("position は予約語ではない"),
        track: sabi_position,
    });

    let mut vocal_opacity = KeyframeTrack::new();
    vocal_opacity.insert(Keyframe {
        t: t(0),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    vocal_opacity.insert(Keyframe {
        t: t(1770),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    vocal_opacity.insert(Keyframe {
        t: t(1799),
        value: Value::F64(0.0),
        interp: Interp::Hold,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: vocal_id,
        property: PropertyId::new(property::OPACITY).expect("opacity は予約語ではない"),
        track: vocal_opacity,
    });

    let waveform_id = waveform_id.expect("波形ビジュアライザ layer がある");
    let glow = EffectId(0);
    intents.push(Intent::SetEffects {
        layer: waveform_id,
        effects: vec![EffectInstance {
            id: glow,
            plugin_id: "motolii.glow".to_owned(),
        }],
    });
    let mut glow_threshold = KeyframeTrack::new();
    glow_threshold.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.35),
        interp: Interp::Hold,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: waveform_id,
        property: PropertyId::effect_param(glow, "threshold").expect("threshold は予約語ではない"),
        track: glow_threshold,
    });
    let mut glow_intensity = KeyframeTrack::new();
    glow_intensity.insert(Keyframe {
        t: t(0),
        value: Value::F64(1.5),
        interp: Interp::Hold,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: waveform_id,
        property: PropertyId::effect_param(glow, "intensity").expect("intensity は予約語ではない"),
        track: glow_intensity,
    });

    intents.push(Intent::SetEffects {
        layer: logo_id,
        effects: vec![EffectInstance {
            id: EffectId(1),
            plugin_id: "motolii.gradient".to_owned(),
        }],
    });

    let second_verse_id = second_verse_id.expect("2番Aメロ歌詞 layer がある");
    intents.push(Intent::SetEffects {
        layer: second_verse_id,
        effects: vec![EffectInstance {
            id: EffectId(2),
            plugin_id: "motolii.tri_led".to_owned(),
        }],
    });

    for file in FIXTURE_MEDIA {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../engine/motolii-engine/tests/golden")
            .join(file);
        let Ok(reader) = std::fs::File::open(&path) else {
            continue;
        };
        let Ok(fingerprint) = SourceFingerprintV1::from_reader(reader) else {
            continue;
        };
        let draft =
            AssetDraft::from_probed_source("image/png".to_owned(), &fingerprint, &path, None);
        intents.push(Intent::AdmitAsset { draft });
    }

    let text_id = LayerId(16);
    intents.push(Intent::AddLayer(text_id));
    intents.push(Intent::SetMeta {
        layer: text_id,
        meta: LayerMeta {
            source: LayerSource::Text,
            order: 15,
            timing: LayerTiming {
                start: 810,
                duration: 990,
                source_in: 0,
                speed: Speed::NORMAL,
            },
        },
    });
    intents.push(Intent::SetAttrs {
        layer: text_id,
        patch: LayerAttrsPatch {
            name: Some("歌詞テキスト".to_owned()),
            label_color: Some(Some(
                (text_id.0 % crate::ui::tokens::LABEL_PALETTE_LEN as u64) as u8,
            )),
            ..Default::default()
        },
    });
    intents.push(Intent::SetTextDocument {
        layer: text_id,
        document: TextDocument {
            content: {
                let mut track = ContentTrack::new();
                track.insert(ContentKeyframe {
                    t: t(0),
                    content: "文字が画素になる".to_owned(),
                });
                track
            },
            justify: TextJustify::Center,
            wrap_size: None,
            styles: vec![TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef {
                    path: "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".to_owned(),
                    fingerprint: None,
                    family: "Hiragino Sans".to_owned(),
                    style: "W3".to_owned(),
                },
                size: 96.0,
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
        },
    });

    doc.apply_all(intents).expect("fixture を1操作として置ける");

    let dance_id = dance_id.expect("ダンスカット layer がある");
    doc.apply(Intent::SetAttrs {
        layer: glitch_id.expect("グリッチトランジション layer がある"),
        patch: LayerAttrsPatch {
            parent: Some(Some(dance_id)),
            ..Default::default()
        },
    })
    .expect("親子例を置ける");

    doc.mark_undo_floor();

    Fixture {
        doc,
        selected: sabi_id,
        playhead: DURATION_FRAMES / 2,
    }
}
