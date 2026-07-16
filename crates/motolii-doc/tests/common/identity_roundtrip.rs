//! D1l/journal追補 #197 §2.2: identity lifecycle command の apply/inverse/redo 契約。

use motolii_doc::{Command, Document};

pub fn assert_identity_command_roundtrip(before: &Document, command: Command) {
    let reservation = command
        .stable_id_reservation()
        .expect("identity command must carry reservation");
    let version_before = before.version;
    let min_before = before.min_reader_version;

    let mut working = before.clone();
    command.apply(&mut working).expect("apply");
    assert_eq!(working.version, version_before);
    assert_eq!(working.min_reader_version, min_before);
    let after_apply = working.clone();

    command.inverse().apply(&mut working).expect("undo");
    assert_eq!(
        working.next_stable_id.peek_next(),
        reservation.after(),
        "undo must keep counter at reservation.after"
    );
    let mut normalized = working.clone();
    normalized.next_stable_id = before.next_stable_id;
    assert_eq!(normalized, *before, "undo must restore full document");
    assert_eq!(working.version, version_before);
    assert_eq!(working.min_reader_version, min_before);

    command.apply(&mut working).expect("redo");
    assert_eq!(working, after_apply, "redo must restore first apply");
}
