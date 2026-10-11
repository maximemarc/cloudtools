//! SoundCraft inside Craft Studio: `main` without the window, control server and macOS menus.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let engine = Engine::default();
    let no_audio = std::env::var_os("SOUNDCRAFT_NO_AUDIO").is_some();
    let player = if no_audio { None } else { Some(soundcraft_playback::Player::new(Arc::new(engine.session().clone()))) };
    let mut app = SoundApp::new(engine, player, services());
    app.autosave_dir = prefs_path().and_then(|p| p.parent().map(|d| d.join("Autosave")));
    app.preset_dir = prefs_path().and_then(|p| p.parent().map(|d| d.join("Presets")));
    if let Some(ui) = load_prefs() {
        app.ui = ui;
    }
    Ok(Box::new(Studio(app)))
}

struct Studio(SoundApp);

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
        // Files dropped on the window: sessions open, audio/MIDI import.
        let dropped: Vec<String> =
            ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_string_lossy().into_owned()).filter(|s| !s.is_empty()).collect());
        for p in dropped {
            open_path(&mut self.0, &p);
        }
        if std::mem::take(&mut self.0.quit_requested) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        save_prefs(&self.0.ui);
        self.0.player = None;
        soundcraft_engine::shutdown_plugin_hosts();
    }
}
