//! 自動保存。5 分ごと、仕舞い先が決まっていて手が入っていれば `<名前> auto-save/` へ世代を残す。
//! 落ちた時に全部消える、を無くす(persist.rs の auto_save は在ったのに誰も呼んでいなかった)。

use crate::ui::session::Session;

pub(crate) fn start(session: &Session) {
    {
        // Session は Rc を含み糸をまたげない。要る 3 つ(作品・仕舞い先・仕舞った revision)だけ渡す。
        let doc = session.doc.clone();
        let project_path = session.project_path.clone();
        let saved_revision = session.saved_revision.clone();
        std::thread::spawn(move || {
            let config = crate::doc::store::AutoSaveConfig {
                interval_secs: crate::doc::store::AutoSaveConfig::DEFAULT_INTERVAL_SECS,
                generations: crate::doc::store::AutoSaveConfig::DEFAULT_GENERATIONS,
            };
            let mut since = doc.lock().unwrap().revision();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(config.interval_secs));
                let path = project_path.lock().unwrap().clone();
                let d = doc.lock().unwrap();
                if d.revision() == *saved_revision.lock().unwrap() {
                    continue;
                }
                match d.auto_save(path.as_deref(), &since, &config) {
                    Ok(Some(target)) => {
                        since = d.revision();
                        println!("PROBE room=project verdict=auto-saved {}", target.display());
                    }
                    Ok(None) => {}
                    Err(e) => println!("PROBE room=project verdict=auto-save-error {e}"),
                }
            }
        });
    }
}
