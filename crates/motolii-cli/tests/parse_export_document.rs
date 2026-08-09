use std::path::PathBuf;

use motolii_cli::{parse_args, Command};

#[test]
fn parses_export_document_arguments() {
    let command = parse_args([
        "export-document",
        "--document",
        "document.json",
        "--output",
        "movie.mp4",
        "--frame-count",
        "120",
        "--qp0",
    ])
    .unwrap();

    let Command::ExportDocument(args) = command else {
        panic!("expected export-document command");
    };
    assert_eq!(args.document, PathBuf::from("document.json"));
    assert_eq!(args.output, PathBuf::from("movie.mp4"));
    assert_eq!(args.frame_count, Some(120));
    assert!(args.qp0);
}

#[test]
fn export_document_defaults_match_existing_flags() {
    let command = parse_args([
        "export-document",
        "--document",
        "document.json",
        "--output",
        "movie.mp4",
    ])
    .unwrap();

    let Command::ExportDocument(args) = command else {
        panic!("expected export-document command");
    };
    assert_eq!(args.frame_count, None);
    assert!(!args.qp0);
}

#[test]
fn rejects_invalid_export_document_arguments() {
    assert!(parse_args(["export-document", "--document", "doc.json"]).is_err());
    assert!(parse_args([
        "export-document",
        "--document",
        "doc.json",
        "--output",
        "out.mp4",
        "--frame-count",
        "many",
    ])
    .is_err());
    assert!(parse_args(["unknown-command"]).is_err());
}
