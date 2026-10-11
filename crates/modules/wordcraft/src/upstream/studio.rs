//! WordCraft inside Craft Studio: what `main` does, minus the window, the control server and the
//! macOS Apple Events (the Studio shell owns the window).

use super::*;

pub(crate) fn create(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let (file_dialog, dialogs) = file_dialogs::hook(ctx);
    let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), services(file_dialog));
    load_prefs(&mut app);
    Ok(Box::new(Studio { app, dialogs }))
}

struct Studio {
    app: WordApp,
    dialogs: file_dialogs::Launcher,
}

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx);
        self.dialogs.show(frame);
        if std::mem::take(&mut self.app.quit_requested) {
            save_prefs(&self.app);
            studio_core::request_close(ctx);
        }
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui);
        self.dialogs.show(frame);
    }
    fn on_exit(&mut self) {
        save_prefs(&self.app);
    }
}
