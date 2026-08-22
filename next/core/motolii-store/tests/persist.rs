//! 保存と読込 — **形式は上流の `.rrd`**。自前形式を発明していない。

use motolii_store::{
    AutoSaveConfig, Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
    property,
};

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-persist-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn authored() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30000, 1001).unwrap(),
        duration_frames: 900,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    for i in 1..=3u64 {
        let layer = LayerId(i);
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: t(0),
            value: Value::Vec2([0.0, 0.0]),
            interp: Interp::Bezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            spatial: None,
        });
        track.insert(Keyframe {
            t: t(60),
            value: Value::Vec2([500.0, 0.0]),
            interp: Interp::Linear,
            spatial: None,
        });

        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [10 * i as u8, 20, 30, 255],
                        width: 640,
                        height: 360,
                    },
                    order: i as i16,
                    timing: LayerTiming {
                        start: i as i64 * 30,
                        duration: 120,
                        source_in: 5,
                        ..Default::default()
                    },
                },
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                track,
            },
        ])
        .unwrap();
    }
    doc
}

#[test]
fn save_and_load_round_trips_the_whole_document() {
    let doc = authored();
    let path = tmp("roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().composition().unwrap(),
        doc.view().composition().unwrap(),
        "comp 設定が往復しない"
    );
    assert_eq!(loaded.view().layers(), doc.view().layers(), "layer が往復しない");

    for layer in doc.view().layers() {
        assert_eq!(
            loaded.view().meta(layer).unwrap(),
            doc.view().meta(layer).unwrap(),
            "layer {layer:?} の素材・重ね順・配置が往復しない"
        );
        assert_eq!(
            loaded.view().properties(layer),
            doc.view().properties(layer),
            "property の一覧が往復しない"
        );
        for property in doc.view().properties(layer) {
            assert_eq!(
                loaded.view().track(layer, &property).unwrap(),
                doc.view().track(layer, &property).unwrap(),
                "track が往復しない({property:?})"
            );
        }
        // 解決まで一致すること(bezier イージングも含めて)。
        for frame in [30, 45, 60, 90] {
            assert_eq!(
                loaded.view().resolve(layer, t(frame)).unwrap(),
                doc.view().resolve(layer, t(frame)).unwrap(),
                "frame {frame} の解決が往復しない"
            );
        }
    }
}

/// `Composition::background`(黒だと気分が上がらない、利用者要望)専用の保存往復。
/// 既定でない値を使い、シリアライズ漏れを検出する。
#[test]
fn composition_background_round_trips_through_save_and_load() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.2, 0.4, 0.6, 0.8],
    }))
    .unwrap();

    let path = tmp("background_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().composition().unwrap().unwrap().background,
        [0.2, 0.4, 0.6, 0.8],
        "background が保存往復で消えた/変わった"
    );
}

/// 旧保存ファイル(`background` component が無い版)を模した JSON でも読める —
/// `#[serde(default = "Composition::default_background")]` の後方互換確認。
/// 実ファイルではなく JSON 直操作で確かめる: 現行の `Composition` を一度シリアライズし、
/// `background` キーだけを取り除いてから読み戻す(旧 Fps の num/den 等、他フィールドの
/// 形式をハードコードしないための遠回り)。
#[test]
fn composition_without_a_background_field_defaults_to_opaque_black() {
    let current = Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.9, 0.9, 0.9, 0.9],
    };
    let mut value = serde_json::to_value(&current).unwrap();
    value
        .as_object_mut()
        .expect("Composition は JSON object のはず")
        .remove("background")
        .expect("旧形式を模すには background キーが無いことが前提");

    let loaded: Composition = serde_json::from_value(value).expect("旧形式の JSON を読めない");
    assert_eq!(
        loaded.background,
        Composition::default_background(),
        "background 欠落時は既定(不透明黒)へ落ちるはず"
    );
}

/// **保存で履歴を畳む**。畳まないと project file が編集回数に比例して伸びる。
#[test]
fn saving_folds_the_edit_history() {
    let mut doc = authored();
    // たくさん編集する
    for i in 0..200 {
        doc.apply(Intent::SetTiming {
            layer: LayerId(1),
            timing: LayerTiming {
                start: i,
                duration: 120,
                source_in: 5,
                ..Default::default()
            },
        })
        .unwrap();
    }
    assert!(doc.edit_head() > 200);

    let path = tmp("folded.motolii");
    doc.save(&path).unwrap();
    let loaded = Document::load(&path).unwrap();

    assert_eq!(
        loaded.view().meta(LayerId(1)).unwrap().unwrap().timing.start,
        199,
        "最後の状態が保存されていない"
    );
    assert!(
        !loaded.can_undo(),
        "開いた直後に戻せる = 履歴が畳まれていないか、底が立っていない"
    );

    let size = std::fs::metadata(&path).unwrap().len();
    println!("保存サイズ: {size} bytes(編集 {}回のあと)", doc.edit_head());
    assert!(
        size < 256 * 1024,
        "200編集した後の保存が {size} bytes。履歴が畳まれていない"
    );
}

/// レイヤー差し色(`label_color`、index 保存)が保存往復で消えない。
#[test]
fn label_color_round_trips_through_save_and_load() {
    let mut doc = authored();
    doc.apply(Intent::SetAttrs {
        layer: LayerId(1),
        patch: LayerAttrsPatch {
            label_color: Some(Some(9)),
            ..Default::default()
        },
    })
    .unwrap();

    let path = tmp("label_color_roundtrip.motolii");
    doc.save(&path).expect("保存できない");
    let loaded = Document::load(&path).expect("読み込めない");

    assert_eq!(
        loaded.view().attrs(LayerId(1)).unwrap().unwrap().label_color,
        Some(9),
        "label_color が保存往復で消えた/変わった"
    );
}

/// 旧保存ファイル(`label_color` component が無い版)を模した JSON でも読める —
/// `#[serde(default)]` の後方互換確認。`composition_without_a_background_field_
/// defaults_to_opaque_black` と同じ手口(JSON からキーを取り除いてから読み戻す)。
#[test]
fn attrs_without_a_label_color_field_defaults_to_unassigned() {
    let current = motolii_store::LayerAttrs {
        name: "旧ドキュメントの layer".to_owned(),
        ..Default::default()
    };
    let mut value = serde_json::to_value(&current).unwrap();
    value
        .as_object_mut()
        .expect("LayerAttrs は JSON object のはず")
        .remove("label_color")
        .expect("旧形式を模すには label_color キーが無いことが前提");

    let loaded: motolii_store::LayerAttrs =
        serde_json::from_value(value).expect("旧形式の JSON を読めない");
    assert_eq!(loaded.label_color, None, "label_color 欠落時は未割当へ落ちるはず");
}

/// 旧保存ファイル(`Mask.mode` component が無い版、MK2 で `mode` を足す前)を模した
/// JSON でも読める — `#[serde(default)]` の後方互換確認。
/// `attrs_without_a_label_color_field_defaults_to_unassigned` と同じ手口
/// (JSON からキーを取り除いてから読み戻す)。既定は `MaskMode::Add`
/// (R9 発注の指定 — 手前の覆いに単純に足す、AE の新規マスクの既定モードと同型)。
#[test]
fn mask_without_a_mode_field_defaults_to_add() {
    let current = motolii_store::Mask {
        id: motolii_store::MaskId(1),
        // Add 以外を仕込んでおく — テストが「たまたま Add だった」で通らないように。
        mode: motolii_store::MaskMode::Subtract,
        inverted: false,
    };
    let mut value = serde_json::to_value(current).unwrap();
    value
        .as_object_mut()
        .expect("Mask は JSON object のはず")
        .remove("mode")
        .expect("旧形式を模すには mode キーが無いことが前提");

    let loaded: motolii_store::Mask =
        serde_json::from_value(value).expect("旧形式の JSON(mode 無し)を読めない");
    assert_eq!(
        loaded.mode,
        motolii_store::MaskMode::Add,
        "mode 欠落時は Add へ落ちるはず"
    );
}

// ---- 自動保存(発注: 自動保存機構、store 側の意味) ----

fn timing_at(start: i64) -> LayerTiming {
    LayerTiming {
        start,
        duration: 120,
        source_in: 5,
        ..Default::default()
    }
}

/// **保存先が無い(一度も明示 Save していない新規 project)なら何もしない**
/// (AE 先例: Auto-Save は保存済み project の隣にしか書けない)。dirty であっても
/// `project_path` が `None` なら素通りする。
#[test]
fn auto_save_skips_when_there_is_no_project_path() {
    let mut doc = authored();
    let since = Document::new().revision(); // 現在の revision とは必ず異なる(dirty 相当)
    doc.apply(Intent::SetTiming {
        layer: LayerId(1),
        timing: timing_at(42),
    })
    .unwrap();

    let result = doc
        .auto_save(None, &since, &AutoSaveConfig::default())
        .expect("path 無しの自動保存判定自体は失敗しないはず");
    assert!(result.is_none(), "project_path が無いのに自動保存が走った");
}

/// **dirty でなければ何もしない**(`since` == 現在の revision)。ディスクへ一切
/// 触れない(auto-save ディレクトリすら作らない)ことまで確認する。
#[test]
fn auto_save_skips_when_not_dirty() {
    let doc = authored();
    let path = tmp("not-dirty-project.motolii");
    let since = doc.revision();

    let result = doc
        .auto_save(Some(&path), &since, &AutoSaveConfig::default())
        .expect("dirty 判定自体は失敗しないはず");
    assert!(result.is_none(), "dirty ではないのに自動保存が走った");
    assert!(
        !Document::auto_save_dir(&path).exists(),
        "何もしないはずが auto-save ディレクトリを作ってしまった"
    );
}

/// **世代ローテーション上限**。`generations` を超えた古い世代は削除され、
/// ディレクトリの中身は常に上限以下に保たれる。各世代の中身も正しく読める
/// (最新の状態が正しく往復している)ことも確認する。
#[test]
fn auto_save_rotates_and_caps_at_generations() {
    let mut doc = authored();
    let path = tmp("rotation-project.motolii");
    let config = AutoSaveConfig {
        interval_secs: 1,
        generations: 3,
    };
    let mut since = doc.revision();

    for start in 0..5i64 {
        doc.apply(Intent::SetTiming {
            layer: LayerId(1),
            timing: timing_at(start),
        })
        .unwrap();

        let written = doc
            .auto_save(Some(&path), &since, &config)
            .expect("自動保存が失敗した")
            .expect("dirty なのに自動保存されなかった");
        since = doc.revision();

        let loaded = Document::load(&written).expect("自動保存した世代が読めない");
        assert_eq!(
            loaded.view().meta(LayerId(1)).unwrap().unwrap().timing.start,
            start,
            "世代 {written:?} の中身が直前の編集と一致しない"
        );
    }

    let dir = Document::auto_save_dir(&path);
    let remaining: Vec<_> = std::fs::read_dir(&dir)
        .expect("auto-save ディレクトリが無い")
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        remaining.len(),
        3,
        "世代数上限(3)を超えて残っている: {remaining:?}"
    );

    // 生き残っているのは新しい3世代(start=2,3,4)のはず — 最新の状態が消えていない。
    let mut starts: Vec<i64> = remaining
        .iter()
        .map(|p| Document::load(p).unwrap().view().meta(LayerId(1)).unwrap().unwrap().timing.start)
        .collect();
    starts.sort_unstable();
    assert_eq!(starts, vec![2, 3, 4], "古い世代を消し損ねている/新しい世代を消してしまっている");
}

/// **atomic 書き込み**: 次に書くはずの世代の tmp スクラッチ位置に、クラッシュ痕を
/// 模した残骸(ゴミバイト列)を仕込んでおいても、自動保存は正常に完了し、
/// **既存の(直前に書いた)正本ファイルは無傷のまま**である。
///
/// tmp のファイル名は `Document::auto_save` の doc に明記した規約
/// (`.{stem}.autosave-{seq}{ext}.tmp`)に基づく — 実装の内部詳細だが、この規約自体が
/// atomic 書き込みの契約(rename 前は tmp、rename 後だけ正本を差し替える)なので
/// ここで直接検証する。
#[test]
fn auto_save_survives_stray_tmp_debris_without_corrupting_the_prior_generation() {
    let mut doc = authored();
    let path = tmp("debris-project.motolii");
    let config = AutoSaveConfig::default();
    let since0 = doc.revision();

    doc.apply(Intent::SetTiming {
        layer: LayerId(1),
        timing: timing_at(7),
    })
    .unwrap();
    let first = doc
        .auto_save(Some(&path), &since0, &config)
        .unwrap()
        .expect("1回目の自動保存が走らなかった");
    let since1 = doc.revision();
    let first_bytes_before = std::fs::read(&first).unwrap();

    // 2回目が使うはずの tmp 位置へ、クラッシュ痕(ゴミ)を仕込む。
    let dir = Document::auto_save_dir(&path);
    let stray_tmp = dir.join(".debris-project.autosave-2.motolii.tmp");
    std::fs::write(&stray_tmp, b"garbage left behind by a crashed auto-save").unwrap();

    doc.apply(Intent::SetTiming {
        layer: LayerId(1),
        timing: timing_at(9),
    })
    .unwrap();
    let second = doc
        .auto_save(Some(&path), &since1, &config)
        .unwrap()
        .expect("2回目の自動保存が走らなかった(tmp 残骸に足を取られた)");

    // 1世代目(正本)は巻き添えを食っていない。
    assert_eq!(
        std::fs::read(&first).unwrap(),
        first_bytes_before,
        "tmp 残骸のせいで既存の世代ファイルが変わった"
    );
    // 2世代目はゴミではなく、ちゃんと今の状態が読める(tmp を rename で正しく差し替えた)。
    let loaded = Document::load(&second).expect("2世代目が読めない(tmp 残骸で壊れた)");
    assert_eq!(
        loaded.view().meta(LayerId(1)).unwrap().unwrap().timing.start,
        9
    );
    // rename が起きたので、tmp の残骸はもう残っていない。
    assert!(
        !stray_tmp.exists(),
        "tmp が rename されずに残骸のまま残っている"
    );
}
