//! 作品の仕舞いと開き直し。窓の外(rfd の dialog)と Document の間。

use crate::ui::session::Session;

/// 作品を仕舞う。行き先が決まっていなければ聞く。
pub(super) async fn put_away(
    session: Session,
    poke: crate::ui::host::Poke,
    ask: bool,
) -> bool {
    let known = session.project_path.lock().unwrap().clone();
    let out = match known.filter(|_| !ask) {
        Some(path) => path,
        None => {
            let picked = rfd::AsyncFileDialog::new()
                .set_file_name("song.rrd")
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
    let save_result = session.doc.lock().unwrap().save(&out);
    let (word, saved) = match save_result {
        Ok(()) => {
            session.mark_saved(out.clone());
            (format!("Saved {}", out.display()), true)
        }
        Err(e) => (format!("Save failed: {e}"), false),
    };
    println!("PROBE room=project verdict=save {word}");
    *session.project_notice.lock().unwrap() = word;
    poke.poke();
    saved
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
        .set_title("Save changes?")
        .set_description("This project has changes that are not saved.")
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
