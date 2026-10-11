//! DesignCraft inside Craft Studio: `main` without the window, control server, GPU start-up
//! sentinel and macOS menus.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let mut session = Session::new();
    // Crash recovery: reopen what a previous run left unsaved, then keep it current.
    session.recovery_dir = profile::recovery_dir();
    let recovered = session.execute("file.recovery.open", &serde_json::json!({})).ok();
    let mut app = DesignApp::new(session, services());
    if let Some(n) = recovered.as_ref().and_then(|r| r["opened"].as_array()).map(Vec::len).filter(|n| *n > 0) {
        app.status(format!("Recovered {n} unsaved document{} from the last session.", if n == 1 { "" } else { "s" }));
    }
    load_prefs(&mut app);
    Ok(Box::new(Studio(app)))
}

struct Studio(DesignApp);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        save_prefs(&self.0);
    }
}
