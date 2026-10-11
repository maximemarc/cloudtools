//! LightCraft inside Craft Studio: `main` without the window, GPU crash marker, control channel
//! and macOS menus.

use super::*;

pub(crate) fn create(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let (prefs, prefs_warning, keep_prefs_file) = load_prefs(false);
    if let Some(ui) = &prefs {
        lightcraft_ui_egui::i18n::set_language(ui.language);
    }
    // The library last opened from Settings, else the default location.
    let library_dir = prefs
        .as_ref()
        .map(|u| u.settings.library_path.clone())
        .filter(|p| !p.is_empty() && std::env::var_os("LIGHTCRAFT_LIBRARY").is_none())
        .map(Into::into)
        .or_else(lightcraft_engine::library::default_dir);
    let (mut session, problem) = open_session(false, library_dir, false);
    session.segmenter.dir =
        std::env::var_os("LIGHTCRAFT_SAM3_DIR").map(std::path::PathBuf::from).or_else(|| config_dir().map(|d| d.join("models").join("sam3")));
    session.segmenter.mirrors_file = config_dir().map(|d| d.join("models").join("sam3-mirrors.txt"));
    let mut app = LightcraftApp::new(session, services(ctx.clone(), None));
    if let Some(ui) = prefs {
        app.ui = ui;
    }
    lightcraft_ui_egui::i18n::set_language(app.ui.language);
    app.notices.extend(prefs_warning);
    let writer = PrefsWriter {
        path: prefs_path(),
        written: serde_json::to_vec_pretty(&app.ui).unwrap_or_default(),
        library: app.ui.settings.library_path.clone(),
        keep_file: keep_prefs_file,
        ..Default::default()
    };
    if let Some(p) = problem {
        app.library_problem = Some(p);
    }
    Ok(Box::new(Studio(app, writer)))
}

struct Studio(LightcraftApp, PrefsWriter);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
        self.1.tick(&mut self.0, ctx);
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        if let Err(e) = self.1.save(&self.0) {
            log::error!("{e}");
        }
        if let Err(e) = self.0.session.close_library() {
            log::error!("saving the library failed: {e}");
        }
        if !self.0.shutdown(EXIT_WAIT) {
            log::warn!("quit: background work was still running after {EXIT_WAIT:?}");
        }
    }
}
