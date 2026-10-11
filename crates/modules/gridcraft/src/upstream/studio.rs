//! GridCraft inside Craft Studio: `main` without the window, control server and macOS menus.

use super::*;

pub(crate) fn create(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, services());
    app.after_engine();
    load_prefs(&mut app);
    SheetApp::setup_context_for_language(ctx, app.ui.dark, app.ui.language);
    Ok(Box::new(Studio(app)))
}

struct Studio(SheetApp);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
        // Files dropped on the window open as workbooks.
        for f in std::mem::take(&mut raw.dropped_files) {
            if let Some(p) = f.path().to_str().map(str::to_string).filter(|p| !p.is_empty()) {
                self.0.open_path(&p);
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        save_prefs(&self.0);
    }
}
