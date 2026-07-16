//! D1l Stage B-0: `StableIdReservation` の serde 契約。

use motolii_doc::StableIdReservation;

#[test]
fn roundtrip_preserves_before_and_after() {
    let reservation = StableIdReservation::new(10, 15);
    let json = serde_json::to_string(&reservation).unwrap();
    assert_eq!(json, r#"{"before":10,"after":15}"#);
    let back: StableIdReservation = serde_json::from_str(&json).unwrap();
    assert_eq!(back, reservation);
    assert_eq!(back.before(), 10);
    assert_eq!(back.after(), 15);
}

#[test]
fn rejects_missing_before() {
    assert!(serde_json::from_str::<StableIdReservation>(r#"{"after":5}"#).is_err());
}

#[test]
fn rejects_missing_after() {
    assert!(serde_json::from_str::<StableIdReservation>(r#"{"before":5}"#).is_err());
}

#[test]
fn rejects_unknown_field() {
    assert!(
        serde_json::from_str::<StableIdReservation>(r#"{"before":1,"after":2,"extra":3}"#).is_err()
    );
}

#[test]
fn decodes_empty_interval_without_correction() {
    let reservation: StableIdReservation =
        serde_json::from_str(r#"{"before":5,"after":5}"#).unwrap();
    assert_eq!(reservation.before(), 5);
    assert_eq!(reservation.after(), 5);
}

#[test]
fn decodes_reverse_interval_without_correction() {
    let reservation: StableIdReservation =
        serde_json::from_str(r#"{"before":10,"after":3}"#).unwrap();
    assert_eq!(reservation.before(), 10);
    assert_eq!(reservation.after(), 3);
}

#[test]
fn decodes_equal_values_without_correction() {
    let reservation: StableIdReservation =
        serde_json::from_str(r#"{"before":0,"after":0}"#).unwrap();
    assert_eq!(reservation.before(), 0);
    assert_eq!(reservation.after(), 0);
}
