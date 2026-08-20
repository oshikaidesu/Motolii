//! `--fixture` 起動が組む「MV 制作の途中に見える」Document(トンマナ検分の器具)。
//!
//! **既存 Intent(`apply_all`)だけで組む** — store のコードは1行も触らない
//! (発注書の柵: 別レーンが `motolii-store` を使用中)。層の表示名は自由文字列
//! (`LayerAttrs.name`)なので地図の制約を受けない — 歌詞風+素材風の混在は
//! MV 制作の実態を模す fixture の意匠であって、store 側の語彙拡張ではない。
//!
//! 15層・timing をずらした配置・マーカー3個・数層に position/opacity キー・
//! 1層選択済み・playhead を中盤へ、は発注書そのものの受入条件(このモジュールの
//! テストが検分する)。

use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, Marker, PropertyId, RationalTime, Speed, Value,
};

/// fixture が組み立てた結果。`Shell::new_fixture` が `Document`/`Session`/status へ写す。
pub struct Fixture {
    pub doc: Document,
    /// 選択済みの1層(「サビ歌詞」)。
    pub selected: LayerId,
    /// 中盤(comp 尺の半分)。
    pub playhead: i64,
    /// status 帯に出す1件の文言。実際の拒否理由ではなく、fixture が
    /// トンマナ検分用に置く固定文言(D2/D7 の status 帯文法を検分する材料)。
    pub status: String,
}

/// 30fps。`duration_frames` は 60秒(1800フレーム)— 典型的な MV の尺。
const FPS_NUM: i64 = 30;
const DURATION_FRAMES: i64 = 1800;

/// フレーム番号を `RationalTime` へ(`RationalTime::try_new(frame, 30)` は
/// `motolii-engine/tests/frame.rs` の `t()` と同じ慣用: fps 分母 1 の下で
/// frame/fps がそのまま秒表現になる)。
fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, FPS_NUM).expect("frame は RationalTime に収まる")
}

struct LayerSpec {
    name: &'static str,
    start: i64,
    duration: i64,
    rgba: [u8; 4],
}

/// 15本。歌詞風(「サビ歌詞」等)+素材風(「Bロール」「ロゴ」等)の混在で、
/// MV 制作の途中に見える名詞集合にしてある(名前は自由文字列 — 地図の制約対象外)。
/// timing は意図的にずらし、重なりも作る(ボーカル映像が終始敷かれた上に
/// 歌詞・カット・トランジションが積む、という実編集の形)。
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

    for (index, spec) in LAYERS.iter().enumerate() {
        let id = LayerId((index + 1) as u64);
        intents.push(Intent::AddLayer(id));
        intents.push(Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: spec.rgba,
                    width: 320,
                    height: 180,
                },
                order: index as i16,
                timing: LayerTiming {
                    start: spec.start,
                    duration: spec.duration,
                    source_in: 0,
                    speed: Speed::NORMAL,
                },
            },
        });
        intents.push(Intent::SetAttrs {
            layer: id,
            patch: LayerAttrsPatch {
                name: Some(spec.name.to_owned()),
                ..Default::default()
            },
        });

        match spec.name {
            "サビ歌詞" => sabi_id = Some(id),
            "タイトルロゴ" => logo_id = Some(id),
            "メインボーカル映像" => vocal_id = Some(id),
            _ => {}
        }
    }

    // マーカー3個 — 曲構成のロケータ(Aメロ/サビ/ラスサビ)。単発ロケータなので
    // duration は ZERO(marker.rs のドキュメント通り)。
    intents.push(Intent::SetMarkers {
        markers: vec![
            Marker {
                name: "Aメロ".to_owned(),
                time: t(150),
                duration: RationalTime::ZERO,
            },
            Marker {
                name: "サビ".to_owned(),
                time: t(510),
                duration: RationalTime::ZERO,
            },
            Marker {
                name: "ラスサビ".to_owned(),
                time: t(1200),
                duration: RationalTime::ZERO,
            },
        ],
    });

    // 数層に position/opacity キー(「数層にキー」の受入条件)。
    let logo_id = logo_id.expect("タイトルロゴ layer がある");
    let sabi_id = sabi_id.expect("サビ歌詞 layer がある");
    let vocal_id = vocal_id.expect("メインボーカル映像 layer がある");

    // タイトルロゴ: 頭でフェードイン→フェードアウト。
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

    // サビ歌詞: サビ入りで下から上へ動く。
    let mut sabi_position = KeyframeTrack::new();
    sabi_position.insert(Keyframe {
        t: t(510),
        value: Value::Vec2([960.0, 760.0]),
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
        value: Value::Vec2([960.0, 540.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    intents.push(Intent::SetTrack {
        layer: sabi_id,
        property: PropertyId::new(property::POSITION).expect("position は予約語ではない"),
        track: sabi_position,
    });

    // メインボーカル映像: 尺いっぱいに敷いて、末尾だけ落とす。
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

    doc.apply_all(intents).expect("fixture を1操作として置ける");

    // 既定 fixture は「編集」ではないので Undo で消えてはいけない
    // (`Shell::new` の既定 comp と同じ扱い)。
    doc.mark_undo_floor();

    Fixture {
        doc,
        selected: sabi_id,
        playhead: DURATION_FRAMES / 2,
        status: "fixture: layer 15 · marker 3 · keyframe済み 3層".to_owned(),
    }
}
