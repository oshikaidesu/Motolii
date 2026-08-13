use std::path::PathBuf;

use motolii_cli::{dump_document, new_document, parse_args, Command};
use motolii_testkit::tmp_dir;

#[test]
fn parses_new_arguments() {
    let command = parse_args(["new", "--document", "document.json"]).unwrap();
    let Command::New(args) = command else {
        panic!("expected new command");
    };
    assert_eq!(args.document, PathBuf::from("document.json"));
}

#[test]
fn rejects_invalid_new_arguments() {
    assert!(parse_args(["new"]).is_err());
    assert!(parse_args(["new", "--output", "out.json"]).is_err());
}

#[test]
fn new_document_seeds_empty_track_and_dump_has_version() {
    let path = tmp_dir("cli-new").join("document.json");
    new_document(&path).unwrap();
    let json = dump_document(&path).unwrap();
    assert!(json.contains("\"version\""), "{json}");
    assert!(json.contains("V1"), "{json}");
}

#[test]
fn new_document_refuses_existing_path() {
    let path = tmp_dir("cli-new-exists").join("document.json");
    new_document(&path).unwrap();
    assert!(new_document(&path).is_err());
}
