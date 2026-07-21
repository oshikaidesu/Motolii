use super::*;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

fn source(id: &str, body: &str) -> ThemeSource {
    ThemeSource {
        id: id.to_owned(),
        bytes: body.as_bytes().to_vec(),
    }
}

fn theme(id: &str, color: &str) -> ThemeSource {
    source(
        id,
        &format!(
            r#"{{
  "$schema": "{SCHEMA_URI}",
  "palette": {{
    "$type": "color",
    "accent": {{ "$value": {{ "colorSpace": "srgb", "components": [{color}, 0.5, 1], "alpha": 0.25 }} }}
  }},
  "layout": {{
    "nested": {{
      "$type": "dimension",
      "gap": {{ "$value": {{ "value": 12.5, "unit": "px" }} }}
    }}
  }},
  "motion": {{
    "delay": {{ "$type": "duration", "$value": {{ "value": 0.2, "unit": "s" }} }},
    "curve": {{ "$type": "cubicBezier", "$value": [0, -2, 1, 3] }}
  }}
}}"#
        ),
    )
}

#[test]
fn generation_is_sorted_hashed_and_deterministic() {
    let light = theme("fixture-light", "0.75");
    let dark = theme("fixture-dark", "0.125");
    let first = generate_bundle(vec![light.clone(), dark.clone()]).unwrap();
    let second = generate_bundle(vec![dark, light]).unwrap();
    assert_eq!(first, second);
    let rust = String::from_utf8(first.tokens_rs).unwrap();
    assert!(rust.contains("GeneratedThemeId::FixtureDark"));
    assert!(rust.contains("pub layout__nested__gap: f32"));
    assert!(rust.contains("f32::from_bits(0x"));
    assert!(rust.contains("from_rgba_unmultiplied(32, 128, 255, 64)"));
    let manifest: serde_json::Value = serde_json::from_slice(&first.manifest_json).unwrap();
    assert_eq!(manifest["generator"]["version"], 1);
    assert_eq!(manifest["themes"][0], "fixture-dark");
    assert_eq!(manifest["outputs"][1], "manifest.json");
    let mut expected_hash = Sha256::new();
    for source in [
        theme("fixture-dark", "0.125"),
        theme("fixture-light", "0.75"),
    ] {
        expected_hash.update((source.id.len() as u64).to_be_bytes());
        expected_hash.update(source.id.as_bytes());
        expected_hash.update((source.bytes.len() as u64).to_be_bytes());
        expected_hash.update(source.bytes);
    }
    assert_eq!(
        manifest["input_sha256"],
        format!("{:x}", expected_hash.finalize())
    );
}

#[test]
fn generate_and_check_obey_directory_and_read_only_contract() {
    let directory = tempdir().unwrap();
    let output = directory.path().join("nested");
    let sources = vec![theme("fixture-dark", "0"), theme("fixture-light", "1")];
    generate_to_dir(sources.clone(), &output).unwrap();
    let before = snapshot(&output);
    check_dir(sources.clone(), &output).unwrap();
    assert_eq!(snapshot(&output), before);

    let tokens = output.join("tokens.rs");
    let mut changed = fs::read(&tokens).unwrap();
    changed[0] ^= 1;
    fs::write(&tokens, changed).unwrap();
    let drifted = snapshot(&output);
    assert!(matches!(
        check_dir(sources.clone(), &output),
        Err(Error::Drift { .. })
    ));
    assert_eq!(snapshot(&output), drifted);

    fs::remove_file(output.join("manifest.json")).unwrap();
    let missing = snapshot(&output);
    assert!(matches!(
        check_dir(sources.clone(), &output),
        Err(Error::MissingOutput { .. })
    ));
    assert_eq!(snapshot(&output), missing);

    fs::write(output.join(".extra"), b"x").unwrap();
    let unexpected = snapshot(&output);
    assert!(matches!(
        check_dir(sources, &output),
        Err(Error::UnexpectedOutputEntry { .. })
    ));
    assert_eq!(snapshot(&output), unexpected);
}

#[test]
fn committed_fixture_bundle_is_byte_exact_generator_output() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../motolii-ui/tests/fixtures/u0e1-token-generator");
    let sources = ["fixture-dark", "fixture-light"]
        .into_iter()
        .map(|id| ThemeSource {
            id: id.to_owned(),
            bytes: fs::read(fixture_root.join("sources").join(format!("{id}.json"))).unwrap(),
        })
        .collect();
    check_dir(sources, &fixture_root.join("generated")).unwrap();
}

#[test]
fn source_bytes_not_path_mtime_or_argument_order_define_output() {
    let one = tempdir().unwrap();
    let two = tempdir().unwrap();
    let sources = vec![theme("fixture-dark", "0"), theme("fixture-light", "1")];
    generate_to_dir(sources.clone(), one.path()).unwrap();
    std::thread::sleep(Duration::from_millis(2));
    generate_to_dir(vec![sources[1].clone(), sources[0].clone()], two.path()).unwrap();
    assert_eq!(byte_snapshot(one.path()), byte_snapshot(two.path()));
}

#[test]
fn rejects_duplicate_keys_at_any_object_depth() {
    let json = format!(
        r#"{{"$schema":"{SCHEMA_URI}","group":{{"$type":"color","token":{{"$value":{{"colorSpace":"srgb","components":[0,0,0],"alpha":1,"alpha":0}}}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &json)]),
        Err(Error::DuplicateKey { path }) if path.ends_with("alpha")
    ));
}

#[test]
fn rejects_malformed_json_with_typed_parse_error() {
    assert!(matches!(
        generate_bundle(vec![source("fixture", "{")]),
        Err(Error::InvalidJson { .. })
    ));
}

#[test]
fn validates_names_inheritance_and_structure() {
    let inherited = format!(
        r#"{{"$schema":"{SCHEMA_URI}","outer":{{"$type":"dimension","middle":{{"inner":{{"$value":{{"value":0,"unit":"px"}}}}}}}}}}"#
    );
    generate_bundle(vec![source("fixture", &inherited)]).unwrap();

    let missing_type = format!(r#"{{"$schema":"{SCHEMA_URI}","token":{{"$value":0}}}}"#);
    assert!(matches!(
        generate_bundle(vec![source("fixture", &missing_type)]),
        Err(Error::MissingType { .. })
    ));
    let mixed = format!(
        r#"{{"$schema":"{SCHEMA_URI}","token":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}},"child":{{}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &mixed)]),
        Err(Error::InvalidStructure { .. })
    ));
    let forbidden = format!(
        r#"{{"$schema":"{SCHEMA_URI}","Bad Name":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &forbidden)]),
        Err(Error::InvalidName { .. })
    ));
}

#[test]
fn validates_all_four_value_shapes_without_clamping_or_inference() {
    for bad in [
        r#""components":[-0.01,0,0]"#,
        r#""components":[0,0,1.01]"#,
        r#""components":[0,0,0],"alpha":2"#,
    ] {
        let body = format!(
            r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"color","$value":{{"colorSpace":"srgb",{bad}}}}}}}"#
        );
        assert!(matches!(
            generate_bundle(vec![source("fixture", &body)]),
            Err(Error::InvalidValue { .. })
        ));
    }
    for (kind, value) in [
        ("dimension", r#"{"value":-1,"unit":"px"}"#),
        ("dimension", r#"{"value":1,"unit":"rem"}"#),
        ("duration", r#"{"value":-1,"unit":"ms"}"#),
        ("duration", r#"{"value":1e308,"unit":"s"}"#),
        ("cubicBezier", r#"[-0.1,0,1,0]"#),
        ("cubicBezier", r#"[0,0,1.1,0]"#),
    ] {
        let body =
            format!(r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"{kind}","$value":{value}}}}}"#);
        assert!(matches!(
            generate_bundle(vec![source("fixture", &body)]),
            Err(Error::InvalidValue { .. })
        ));
    }
}

#[test]
fn rejects_unsupported_features_and_unknown_properties() {
    for body in [
        format!(r#"{{"$schema":"{SCHEMA_URI}","$extends":"base"}}"#),
        format!(r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"color","$value":"{{palette.x}}"}}}}"#),
        format!(r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"number","$value":1}}}}"#),
        format!(
            r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"duration","$mystery":true,"$value":{{"value":1,"unit":"ms"}}}}}}"#
        ),
    ] {
        assert!(matches!(
            generate_bundle(vec![source("fixture", &body)]),
            Err(Error::UnsupportedFeature { .. } | Error::UnknownProperty { .. })
        ));
    }
}

#[test]
fn rejects_theme_mismatch_and_identifier_collisions() {
    let full = theme("fixture-a", "0");
    let missing = source(
        "fixture-b",
        &format!(
            r#"{{"$schema":"{SCHEMA_URI}","palette":{{"$type":"color","accent":{{"$value":{{"colorSpace":"srgb","components":[0,0,0]}}}}}}}}"#
        ),
    );
    assert!(matches!(
        generate_bundle(vec![full, missing]),
        Err(Error::ThemeMismatch { differences }) if differences.len() >= 3
    ));

    let collision = format!(
        r#"{{"$schema":"{SCHEMA_URI}","a-b":{{"c":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}},"a_b":{{"c":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &collision)]),
        Err(Error::FieldCollision { .. })
    ));
    let minimal = format!(
        r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![
            source("fixture-dark", &minimal),
            source("fixture_dark", &minimal)
        ]),
        Err(Error::VariantCollision { .. })
    ));
    assert!(matches!(
        generate_bundle(vec![source("self", &minimal)]),
        Err(Error::VariantKeyword { .. })
    ));
}

#[test]
fn enforces_schema_empty_and_resource_limits() {
    assert!(matches!(generate_bundle(Vec::new()), Err(Error::NoThemes)));
    let wrong_schema = r#"{"$schema":"https://example.invalid/schema.json","x":{"$type":"duration","$value":{"value":0,"unit":"ms"}}}"#;
    assert!(matches!(
        generate_bundle(vec![source("fixture", wrong_schema)]),
        Err(Error::InvalidSchema { .. })
    ));
    assert!(matches!(
        generate_bundle(vec![source(
            "fixture",
            &format!(r#"{{"$schema":"{SCHEMA_URI}","empty":{{}}}}"#)
        )]),
        Err(Error::InvalidStructure { .. })
    ));

    let oversized = ThemeSource {
        id: "fixture".to_owned(),
        bytes: vec![b' '; MAX_SOURCE_BYTES + 1],
    };
    assert!(matches!(
        generate_bundle(vec![oversized]),
        Err(Error::SourceTooLarge { .. })
    ));

    let long_segment = "a".repeat(MAX_SEGMENT_BYTES + 1);
    let body = format!(
        r#"{{"$schema":"{SCHEMA_URI}","{long_segment}":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &body)]),
        Err(Error::InvalidName { .. })
    ));

    let mut nested = String::new();
    for _ in 0..=MAX_DEPTH {
        nested.push_str(r#"{"a":"#);
    }
    nested.push_str("null");
    for _ in 0..=MAX_DEPTH {
        nested.push('}');
    }
    assert!(matches!(
        generate_bundle(vec![source("fixture", &nested)]),
        Err(Error::NestingLimit { .. })
    ));

    let segment = "a".repeat(110);
    let mut path_body = format!(r#"{{"$schema":"{SCHEMA_URI}""#);
    for index in 0..5 {
        if index == 0 {
            path_body.push(',');
        }
        path_body.push_str(&format!(r#""{segment}":{{"#));
    }
    path_body.push_str(r#""$type":"duration","$value":{"value":0,"unit":"ms"}"#);
    for _ in 0..6 {
        path_body.push('}');
    }
    let path_error = generate_bundle(vec![source("fixture", &path_body)]).unwrap_err();
    assert!(
        matches!(path_error, Error::PathLimit { .. }),
        "{path_error:?}"
    );

    let mut token_body = format!(r#"{{"$schema":"{SCHEMA_URI}","g":{{"$type":"duration""#);
    for index in 0..=MAX_TOKENS {
        token_body.push_str(&format!(
            r#","t{index}":{{"$value":{{"value":0,"unit":"ms"}}}}"#
        ));
    }
    token_body.push_str("}}");
    assert!(token_body.len() < MAX_SOURCE_BYTES);
    assert!(matches!(
        generate_bundle(vec![source("fixture", &token_body)]),
        Err(Error::TokenLimit)
    ));
}

#[test]
fn rejects_remaining_bundle_and_keyword_errors_with_typed_variants() {
    let minimal = format!(
        r#"{{"$schema":"{SCHEMA_URI}","x":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![
            source("fixture", &minimal),
            source("fixture", &minimal)
        ]),
        Err(Error::DuplicateThemeId { .. })
    ));
    let keyword = format!(
        r#"{{"$schema":"{SCHEMA_URI}","type":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &keyword)]),
        Err(Error::FieldKeyword { .. })
    ));
    let reserved = format!(
        r#"{{"$schema":"{SCHEMA_URI}","box":{{"$type":"duration","$value":{{"value":0,"unit":"ms"}}}}}}"#
    );
    assert!(matches!(
        generate_bundle(vec![source("fixture", &reserved)]),
        Err(Error::FieldKeyword { .. })
    ));
}

fn snapshot(directory: &Path) -> BTreeMap<String, (Vec<u8>, Option<SystemTime>)> {
    let mut result = BTreeMap::new();
    if !directory.exists() {
        return result;
    }
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        result.insert(
            entry.file_name().to_string_lossy().into_owned(),
            (
                if metadata.is_file() {
                    fs::read(entry.path()).unwrap()
                } else {
                    Vec::new()
                },
                metadata.modified().ok(),
            ),
        );
    }
    result
}

fn byte_snapshot(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot(directory)
        .into_iter()
        .map(|(name, (bytes, _))| (name, bytes))
        .collect()
}
