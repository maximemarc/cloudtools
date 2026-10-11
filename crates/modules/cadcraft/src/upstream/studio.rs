//! CADCraft inside Craft Studio: `main` without the window, control server and macOS menus.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    static IO: std::sync::Once = std::sync::Once::new();
    IO.call_once(install_io);
    let mut app = CadApp::new(Session::empty(), services());
    let (workspace, restored, workspace_error) = workspace::Workspace::load();
    if let Some(ui) = restored {
        app.ui = ui;
    }
    if let Some(error) = &workspace_error {
        app.set_status(error);
    }
    app.system_languages = std::env::var("CADCRAFT_LOCALE").map(|s| vec![s]).unwrap_or_else(|_| sys_locale::get_locales().collect());
    if let Some(prefs) = frame.storage().and_then(|s| s.get_string(cadcraft_ui_egui::PREFS_KEY)) {
        app.load_prefs(&prefs);
    }
    if let Some(rs) = frame.wgpu_render_state() {
        app.set_wgpu(rs);
    }
    if app.session.docs.is_empty() {
        app.session.new_drawing(false);
    }
    Ok(Box::new(Studio(App {
        cad: app,
        workspace,
        workspace_error,
        #[cfg(target_os = "macos")]
        native_menu: None,
    })))
}

struct Studio(App);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.cad.logic(ctx);
        self.0.persist_workspace(false);
        if std::mem::take(&mut self.0.cad.quit_requested) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.cad.raw_input_hook(raw);
        cadcraft_ui_egui::cmdline::capture_tab(ctx, raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.cad.ui(ui);
        self.0.persist_workspace(false);
    }
    fn on_exit(&mut self) {
        self.0.persist_workspace(true);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(cadcraft_ui_egui::PREFS_KEY, self.0.cad.prefs_json());
    }
}
