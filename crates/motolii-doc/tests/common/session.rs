use std::path::Path;
use std::thread;
use std::time::Duration;

use motolii_doc::{
    Document, OpenProjectOutcome, ProjectSession, ResourceLimits, SaveOptions, SaveProjectOptions,
    SessionError,
};

pub fn production_limits() -> ResourceLimits {
    ResourceLimits::production()
}

pub fn acquire_session(path: &Path) -> ProjectSession {
    ProjectSession::acquire(path, &production_limits()).expect("acquire session")
}

pub fn save_document_via_session(path: &Path, doc: &Document) {
    save_document_via_session_with_retry(path, doc, &SaveOptions::default());
}

fn save_document_via_session_with_retry(path: &Path, doc: &Document, options: &SaveOptions) {
    loop {
        match ProjectSession::acquire(path, &production_limits()) {
            Ok(mut session) => {
                session.save_document(doc, options).expect("save document");
                return;
            }
            Err(SessionError::ProjectAlreadyOpen) => thread::sleep(Duration::from_millis(1)),
            Err(e) => panic!("acquire session: {e:?}"),
        }
    }
}

pub fn save_document_via_session_with_options(
    path: &Path,
    doc: &Document,
    options: &SaveOptions,
) -> Result<(), motolii_doc::PersistError> {
    let mut session = acquire_session(path);
    session.save_document(doc, options)
}

pub fn migrate_document_file_via_session(
    path: &Path,
    options: &motolii_doc::MigrateFileOptions,
) -> Result<motolii_doc::MigrateFileResult, motolii_doc::MigrateError> {
    let mut session = acquire_session(path);
    session.migrate_document_file(options)
}

pub fn save_journal(path: &Path, doc: &Document, options: &SaveProjectOptions) {
    save_journal_result(path, doc, options).expect("save with journal");
}

pub fn save_journal_result(
    path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), motolii_doc::ProjectError> {
    let mut session = acquire_session(path);
    session.save_with_journal(doc, options).map_err(|e| *e)
}

pub fn open_recovered(path: &Path) -> (ProjectSession, OpenProjectOutcome) {
    ProjectSession::open(path, &production_limits()).expect("open project")
}
