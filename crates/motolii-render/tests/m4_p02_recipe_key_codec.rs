//! M4-P02-CODEC mutation corpus for RecipeKeyV1 / ArtifactDigest.

use motolii_render::{ArtifactDigest, RecipeKeyError, RecipeKeyV1, RecipeKeyV1Input};

fn base_input() -> RecipeKeyV1Input {
    RecipeKeyV1Input {
        recipe_format_version: 1,
        node_id: "solid.source".into(),
        node_version: 1,
        params: vec![
            ("alpha".into(), vec![0x01, 0x02]),
            ("color".into(), vec![0xaa, 0xbb, 0xcc]),
        ],
        input_digests: vec![
            ArtifactDigest::from_bytes(b"input-a"),
            ArtifactDigest::from_bytes(b"input-b"),
        ],
        time: (1001, 30000),
        quality: 2,
        platform_salt: [0x11; 32],
    }
}

#[test]
fn identical_input_yields_identical_key() {
    let input = base_input();
    let a = RecipeKeyV1::encode(&input).unwrap();
    let b = RecipeKeyV1::encode(&input).unwrap();
    assert_eq!(a, b);
}

#[test]
fn each_field_mutation_changes_key() {
    let base = RecipeKeyV1::encode(&base_input()).unwrap();

    let mut version = base_input();
    version.recipe_format_version += 1;
    assert_ne!(RecipeKeyV1::encode(&version).unwrap(), base);

    let mut node_id = base_input();
    node_id.node_id.push('x');
    assert_ne!(RecipeKeyV1::encode(&node_id).unwrap(), base);

    let mut node_version = base_input();
    node_version.node_version += 1;
    assert_ne!(RecipeKeyV1::encode(&node_version).unwrap(), base);

    let mut params = base_input();
    params.params[0].1[0] ^= 0xff;
    assert_ne!(RecipeKeyV1::encode(&params).unwrap(), base);

    let mut inputs = base_input();
    inputs
        .input_digests
        .push(ArtifactDigest::from_bytes(b"input-c"));
    assert_ne!(RecipeKeyV1::encode(&inputs).unwrap(), base);

    let mut time = base_input();
    time.time.0 += 1;
    assert_ne!(RecipeKeyV1::encode(&time).unwrap(), base);

    let mut quality = base_input();
    quality.quality += 1;
    assert_ne!(RecipeKeyV1::encode(&quality).unwrap(), base);

    let mut salt = base_input();
    salt.platform_salt[0] ^= 0xff;
    assert_ne!(RecipeKeyV1::encode(&salt).unwrap(), base);
}

#[test]
fn params_input_order_is_canonicalized_by_sort() {
    let mut a = base_input();
    a.params = vec![
        ("alpha".into(), vec![0x01, 0x02]),
        ("color".into(), vec![0xaa, 0xbb, 0xcc]),
    ];
    let mut b = base_input();
    b.params = vec![
        ("color".into(), vec![0xaa, 0xbb, 0xcc]),
        ("alpha".into(), vec![0x01, 0x02]),
    ];
    assert_eq!(
        RecipeKeyV1::encode(&a).unwrap(),
        RecipeKeyV1::encode(&b).unwrap()
    );
}

#[test]
fn input_digests_order_is_significant() {
    let mut a = base_input();
    a.input_digests = vec![
        ArtifactDigest::from_bytes(b"input-a"),
        ArtifactDigest::from_bytes(b"input-b"),
    ];
    let mut b = base_input();
    b.input_digests = vec![
        ArtifactDigest::from_bytes(b"input-b"),
        ArtifactDigest::from_bytes(b"input-a"),
    ];
    assert_ne!(
        RecipeKeyV1::encode(&a).unwrap(),
        RecipeKeyV1::encode(&b).unwrap()
    );
}

#[test]
fn duplicate_param_id_is_typed_error() {
    let mut input = base_input();
    input.params = vec![("color".into(), vec![0x01]), ("color".into(), vec![0x02])];
    assert_eq!(
        RecipeKeyV1::encode(&input).unwrap_err(),
        RecipeKeyError::DuplicateParamId
    );
}

#[test]
fn non_positive_time_denominator_is_typed_error() {
    let mut zero = base_input();
    zero.time.1 = 0;
    assert_eq!(
        RecipeKeyV1::encode(&zero).unwrap_err(),
        RecipeKeyError::InvalidTime
    );

    let mut negative = base_input();
    negative.time.1 = -1;
    assert_eq!(
        RecipeKeyV1::encode(&negative).unwrap_err(),
        RecipeKeyError::InvalidTime
    );
}

#[test]
fn artifact_digest_from_bytes_identity_and_size() {
    let a = ArtifactDigest::from_bytes(b"abc");
    let b = ArtifactDigest::from_bytes(b"abc");
    assert_eq!(a, b);
    assert_eq!(a.size, 3);

    let mutated = ArtifactDigest::from_bytes(b"abd");
    assert_ne!(a, mutated);
    assert_eq!(mutated.size, 3);
}

#[test]
fn display_forms_use_fixed_prefix_and_hex_length() {
    let key = RecipeKeyV1::encode(&base_input()).unwrap();
    let key_s = key.canonical_string();
    assert!(key_s.starts_with("motolii-recipe-v1:sha256:"));
    let key_hex = key_s.strip_prefix("motolii-recipe-v1:sha256:").unwrap();
    assert_eq!(key_hex.len(), 64);
    assert!(key_hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));

    let digest = ArtifactDigest::from_bytes(b"abc");
    let digest_s = digest.canonical_string();
    assert!(digest_s.starts_with("motolii-artifact-v1:sha256:"));
    let rest = digest_s
        .strip_prefix("motolii-artifact-v1:sha256:")
        .unwrap();
    let (hex, size) = rest.rsplit_once(':').unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
    assert_eq!(size, "3");
}
