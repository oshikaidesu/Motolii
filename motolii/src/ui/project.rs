//! 作品の仕舞いと開き直し。窓の外(rfd の dialog)と Document の間。

use dioxus_native::prelude::*;

use crate::ui::session::Session;

/// 作品を仕舞う。行き先が決まっていなければ聞く。
/// 窓に紐付く sheet として開く file dialog。親が無ければ素の窓(HIG: 書類の dialog はその窓の sheet)。
pub(super) fn sheet(window: Option<&dyn dioxus_native::winit::window::Window>) -> rfd::AsyncFileDialog {
    let dialog = rfd::AsyncFileDialog::new();
    match window {
        Some(window) => dialog.set_parent(window),
        None => dialog,
    }
}

pub(super) async fn put_away(
    session: Session,
    poke: crate::ui::host::Poke,
    ask: bool,
) -> bool {
    // dialog を待つ間に Cmd+S をもう一度押しても 2 枚目は開かない。
    if session.saving.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    let done = put_away_inner(&session, ask).await;
    session.saving.store(false, std::sync::atomic::Ordering::SeqCst);
    println!("PROBE room=project verdict=save {}", session.project_notice.lock().unwrap());
    poke.poke();
    done
}

async fn put_away_inner(session: &Session, ask: bool) -> bool {
    let known = session.project_path.lock().unwrap().clone();
    let out = match known.filter(|_| !ask) {
        Some(path) => path,
        None => {
            let picked = rfd::AsyncFileDialog::new()
                .add_filter("Motolii Project", &["rrd"])
                .set_file_name(default_file_name(session))
                .save_file()
                .await;
            let Some(file) = picked else { return false };
            let mut out = file.path().to_path_buf();
            if out.extension().is_none_or(|e| !e.eq_ignore_ascii_case("rrd")) {
                out.set_extension("rrd");
            }
            out
        }
    };
    save_to(session, out)
}

/// 仕舞う本体。**窓を閉じる時の Save もここを通す**(経路が 2 本だと文言と後始末が食い違う)。
/// 仕舞う物と「仕舞った revision」は同じ lock の中で決める。取り直すと間の編集が消える。
pub(super) fn save_to(session: &Session, out: std::path::PathBuf) -> bool {
    let save_result = {
        let d = session.doc.lock().unwrap();
        d.save(&out).map(|()| d.revision())
    };
    let (word, saved) = match save_result {
        Ok(revision) => {
            session.mark_saved(out.clone(), revision);
            (format!("Saved {}", out.display()), true)
        }
        Err(e) => (format!("Save failed: {e}"), false),
    };
    *session.project_notice.lock().unwrap() = word;
    saved
}

/// 仕舞う先の既定名(作品名.rrd、無ければ Untitled.rrd)。
pub(super) fn default_file_name(session: &Session) -> String {
    format!("{}.rrd", session.document_title())
}

pub(super) async fn allow_project_replacement(
    session: Session,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
) -> bool {
    if !session.is_dirty() {
        return true;
    }
    let mut dialog = rfd::AsyncMessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title(&format!("Do you want to save the changes you made to {}?", session.document_title()))
        .set_description("Your changes will be lost if you don't save them.")
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            "Save".to_owned(),
            "Don't Save".to_owned(),
            "Cancel".to_owned(),
        ));
    if let Some(window) = window.as_deref() {
        dialog = dialog.set_parent(window);
    }
    let answer = dialog.show().await;
    match answer {
        rfd::MessageDialogResult::Custom(label) if label == "Save" => {
            put_away(session, poke, false).await
        }
        rfd::MessageDialogResult::Custom(label) if label == "Don't Save" => true,
        _ => false,
    }
}

/// 白紙にする(File▸New、⌘N)。未保存なら先に聞く。
pub(super) async fn new_project(
    session: Session,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
    mut revision: dioxus_native::prelude::Signal<u32>,
    mut selected: dioxus_native::prelude::Signal<Option<crate::doc::store::LayerId>>,
) {
    if !allow_project_replacement(session.clone(), poke, window).await {
        return;
    }
    session.replace_project(crate::ui::blank_project(), None);
    session.selection.set(None);
    selected.set(None);
    *revision.write() += 1;
}

/// 開く(File▸Open…、⌘O)。未保存なら先に聞く。
pub(super) async fn open_project(
    session: Session,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
    mut revision: dioxus_native::prelude::Signal<u32>,
    mut selected: dioxus_native::prelude::Signal<Option<crate::doc::store::LayerId>>,
) {
    if !allow_project_replacement(session.clone(), poke, window.clone()).await {
        return;
    }
    let mut dialog = rfd::AsyncFileDialog::new().add_filter("Motolii Project", &["rrd"]);
    if let Some(window) = window.as_deref() {
        dialog = dialog.set_parent(window);
    }
    let Some(file) = dialog.pick_file().await else { return };
    match crate::doc::store::Document::load(file.path()) {
        Ok(loaded) => {
            session.replace_project(loaded, Some(file.path().to_path_buf()));
            session.selection.set(None);
            selected.set(None);
            *session.project_notice.lock().unwrap() = String::new();
        }
        Err(e) => {
            *session.project_notice.lock().unwrap() = format!("Open failed: {e}");
        }
    }
    *revision.write() += 1;
}
