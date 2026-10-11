//! DeckCraft inside Craft Studio: `main` without the window and control server.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let mut app = SlideApp::new(Session::new(), services());
    load_prefs(&mut app);
    // AutoRecover: reopen what a previous run left unsaved, then keep it current.
    if std::env::var_os("DECKCRAFT_NO_RECOVERY").is_none() {
        app.session.recovery_dir = deckcraft_engine::recovery::default_dir();
        if let Ok(v) = app.session.execute("file.recovery.open", &serde_json::json!({}))
            && let Some(n) = v.get("opened").and_then(|o| o.as_array()).map(Vec::len).filter(|n| *n > 0)
        {
            app.set_status(format!("Recovered {n} unsaved presentation{} from the last session", if n == 1 { "" } else { "s" }));
        }
    }
    Ok(Box::new(App(app)))
}
