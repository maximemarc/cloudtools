//! VectorCraft inside Craft Studio: `main` without the window set-up, GPU fallback, control
//! channel and macOS menus. The shared wgpu device's loss callback stays with the shell.

use super::*;

pub(crate) fn create(ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let saved = read_prefs();
    let mut app = VectorcraftApp::new(Session::new(), services(frame.winit_window().cloned(), ctx.clone()));
    load_prefs(&mut app, saved);
    // User Defined swatch and graphic style libraries live next to the preferences.
    let swatches = prefs_path().and_then(|p| Some(p.parent()?.join("Swatches").to_string_lossy().to_string()));
    app.session.swatch_libraries.set_user_dir(swatches);
    let styles = prefs_path().and_then(|p| Some(p.parent()?.join("Graphic Styles").to_string_lossy().to_string()));
    app.session.style_libraries.set_user_dir(styles);
    if prefs_enabled() {
        let libraries = prefs_path().and_then(|p| Some(p.parent()?.join("Libraries").to_string_lossy().to_string()));
        app.session.libraries.set_dir(libraries);
    }
    if std::env::var_os("VECTORCRAFT_NO_PREFS").is_none() {
        let recovery = prefs_path().and_then(|p| Some(p.parent()?.join("Data Recovery").to_string_lossy().to_string()));
        app.session.recovery.set_default_folder(recovery);
    }
    if let Some(rs) = frame.wgpu_render_state() {
        app.graphics_adapter = Some(adapter_summary(&rs.adapter.get_info()));
    }
    Ok(Box::new(Studio(App { app, graphics_loss: GraphicsLoss::default(), graphics_lost: false, frames: 0 })))
}

struct Studio(App);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        file_access::unconfined(|| self.0.logic_frame(ctx, frame));
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        file_access::unconfined(|| self.0.app.raw_input_hook(raw));
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        file_access::unconfined(|| self.0.app.ui(ui));
    }
    fn on_exit(&mut self) {
        file_access::unconfined(|| save_prefs(&self.0.app));
    }
}
