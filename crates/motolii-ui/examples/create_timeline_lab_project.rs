//! ユーザーが明示的に作る Timeline Lab 用の最初の project。
//! 自動fallbackではなく、ProjectSession を取得してから1度だけ本体を保存する。

use std::path::{Path, PathBuf};

const TEMPLATE: &str = "docs/mocks-ui/fixtures/reference-document.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = std::env::args().nth(1) else {
        return Err("usage: create_timeline_lab_project <new-project.json>".into());
    };
    let path = PathBuf::from(path);
    if path.exists() {
        return Err(format!("refusing to overwrite existing project: {}", path.display()).into());
    }
    let parent = path
        .parent()
        .ok_or("project path needs a parent directory")?;
    std::fs::create_dir_all(parent)?;
    let template = motolii_doc::load_document(Path::new(TEMPLATE))?;
    let mut session =
        motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())?;
    session.save_document(&template, &motolii_doc::SaveOptions::default())?;
    println!("created {}", session.document_path().display());
    Ok(())
}
