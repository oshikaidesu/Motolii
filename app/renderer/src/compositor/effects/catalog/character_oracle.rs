use super::*;
/// 棚の全 Vism について、欄の性格が使われ方から読めること(名前は見ていない)。
/// 期待値は 2026-09-08 の目視で確定した写像。変えるなら理由をここに書く。
#[test]
fn every_shader_param_has_its_character_read_from_use() {
    let runtime = super::CatalogRuntime::default();
    super::refresh_effect_catalog_for(&runtime);
    let snapshot = runtime.0.owner.lock().unwrap().snapshot.clone().unwrap();
    let mut seen = std::collections::BTreeMap::new();
    for d in snapshot.descriptors.iter() {
        for p in &d.params {
            seen.insert(format!("{}.{}", d.plugin_id, p.name), (p.subtype.clone(), p.unit.clone(), p.group.clone()));
        }
    }
    let s = |v: &str| Some(v.to_owned());
    let expected: &[(&str, Option<String>, Option<String>, Option<String>)] = &[
        ("motolii.isf_bloom.threshold", s("LEVEL"), s(""), None),
        ("motolii.isf_bloom.intensity", s("FACTOR"), s(""), None),
        ("motolii.isf_bloom.radius", s("DISTANCE"), s("px"), None),
        ("motolii.blur.radius", s("DISTANCE"), s("px"), None),
        ("motolii.gain.gain", s("FACTOR"), s(""), None),
        ("motolii.glass.roughness", s("FACTOR"), s("%"), None),
        ("motolii.glow.threshold", s("LEVEL"), s(""), None),
        ("motolii.glow.intensity", s("FACTOR"), s(""), None),
        ("motolii.glow.radius", s("DISTANCE"), s("px"), None),
        // shader が読まない欄(定数式にだけ使う)は不明のまま — 嘘を付けない。
        ("motolii.tri_led.glow", None, None, None),
        ("motolii.turbulent_displace.size", s("DISTANCE"), s("px"), None),
        ("motolii.turbulent_displace.complexity", s("COUNT"), None, None),
        // 使われ方では平行移動に見える。manifest の SUBTYPE が勝つ(宣言 > 解析)。
        ("motolii.turbulent_displace.evolution", s("TIME"), None, None),
        ("motolii.turbulent_displace.offset_x", s("TRANSLATION"), s("px"), s("offset_x")),
        ("motolii.turbulent_displace.offset_y", s("TRANSLATION"), s("px"), s("offset_x")),
        ("motolii.turbulent_displace.offset_z", s("TRANSLATION"), s("px"), s("offset_x")),
        // shader を持たない棚(配置)は解析の外。
        ("motolii.repeat.seed", None, None, None),
    ];
    for (key, subtype, unit, group) in expected {
        assert_eq!(seen.get(*key), Some(&(subtype.clone(), unit.clone(), group.clone())), "{key}");
    }
}
