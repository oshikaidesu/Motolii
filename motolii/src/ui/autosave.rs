//! 自動保存。5 分ごと、手が入っていれば `<名前> auto-save/` へ世代を残す。仕舞い先が無い作品は
//! ~/Library/Application Support/Motolii/autosave/Untitled.rrd の脇へ(一番要る場面 = まだ一度も仕舞っていない時)。
//! lock は写し(flatten)を取る間だけ。disk へ書くのは lock の外。結果は project_notice に載せる。

use crate::ui::session::Session;

pub(crate) fn start(session: &Session) {
    // Session は Rc を含み糸をまたげない。要る物だけ渡す。
    let doc = session.doc.clone();
    let project_path = session.project_path.clone();
    let saved_revision = session.saved_revision.clone();
    let notice = session.project_notice.clone();
    std::thread::spawn(move || {
        let config = crate::doc::store::AutoSaveConfig {
            interval_secs: crate::doc::store::AutoSaveConfig::DEFAULT_INTERVAL_SECS,
            generations: crate::doc::store::AutoSaveConfig::DEFAULT_GENERATIONS,
        };
        let mut since = doc.lock().unwrap().revision();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(config.interval_secs));
            let path = project_path.lock().unwrap().clone().or_else(untitled_path);
            let (flat, revision) = {
                let d = doc.lock().unwrap();
                if d.revision() == *saved_revision.lock().unwrap() || d.revision() == since {
                    continue;
                }
                (d.flattened(), d.revision())
            };
            let result = flat.and_then(|flat| flat.auto_save(path.as_deref(), &since, &config));
            match result {
                Ok(Some(target)) => {
                    since = revision;
                    println!("PROBE room=project verdict=auto-saved {}", target.display());
                }
                Ok(None) => {}
                Err(e) => {
                    *notice.lock().unwrap() = format!("Auto-save failed: {e}");
                    println!("PROBE room=project verdict=auto-save-error {e}");
                }
            }
        }
    });
}

fn untitled_path() -> Option<std::path::PathBuf> {
    crate::ui::host::settings_dir().map(|d| d.join("autosave").join("Untitled.rrd"))
}
