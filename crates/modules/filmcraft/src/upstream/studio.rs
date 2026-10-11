//! FilmCraft inside Craft Studio: `main` without the window, control channel and macOS menus.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    // Process-wide registrations `main` does before the window opens.
    static REGISTER: std::sync::Once = std::sync::Once::new();
    REGISTER.call_once(|| {
        register_hardware_decoders();
        register_gpu_frame_renderer();
    });
    let mut session = Session::default();
    if let Some(dir) = default_data_dir() {
        let mut cfg = AutosaveConfig::new(dir);
        cfg.local_offset = local_offset;
        if let Err(e) = session.start_autosave(cfg) {
            log::warn!("auto-save and crash recovery unavailable: {e}");
        }
    }
    // voice-over recording reads the microphone through cpal
    session.voiceover.input = Some(Box::new(audio_in::CpalIn::new(&session.prefs.audio_hardware.device_class)));
    if session.prefs.general.at_startup == "openMostRecent" {
        let recent = session.prefs.general.recent_projects.iter().find(|p| std::path::Path::new(p).exists()).cloned();
        if let Some(p) = recent
            && let Err(e) = session.execute("file.open", json!({"path": p}))
        {
            log::error!("{e}");
        }
    }
    let mut app = FilmcraftApp::new(session);
    if let Some(rs) = frame.wgpu_render_state().cloned()
        && std::env::var_os("FILMCRAFT_CPU_COMPOSITE").is_none()
    {
        app.set_wgpu(rs);
    }
    app.audio = Some(Box::new(audio::CpalOut::new()));
    app.hooks.pick_files = Some(Box::new(|exts: &[&str]| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t("Media"), &file_filters::extensions(exts))
            .pick_files()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }));
    app.hooks.pick_file_for_relink = Some(Box::new(|exts: &[&str], _hint| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t("Media"), &file_filters::extensions(exts))
            .pick_file()
            .map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.pick_save = Some(Box::new(|name: &str| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t("FilmCraft Project"), &file_filters::extensions(&["fcproj"]))
            .set_file_name(name)
            .save_file()
            .map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.pick_save_as = Some(Box::new(|filter: &str, exts: &[&str], name: &str| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t(filter), &file_filters::extensions(exts))
            .set_file_name(name)
            .save_file()
            .map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.pick_folder = Some(Box::new(|| rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())));
    app.hooks.pick_folder_at =
        Some(Box::new(|dir: &str| rfd::FileDialog::new().set_directory(dir).pick_folder().map(|p| p.to_string_lossy().into_owned())));
    app.hooks.open_path = Some(Box::new(open_path));
    app.hooks.open_url = Some(Box::new(|url: &str| open::that_detached(url).map_err(|e| e.to_string())));
    app.hooks.clipboard_media = Some(Box::new(clipboard_media));
    app.hooks.cursor_screen_position = Some(Box::new(filmcraft_platform::cursor::cursor_screen_position));
    app.hooks.system_languages = Some(Box::new(|| sys_locale::get_locales().collect()));
    app.hooks.system_theme = appearance::service();
    app.hooks.pick_open_file = Some(Box::new(|filter: &str, exts: &[&str]| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t(filter), &file_filters::extensions(exts))
            .pick_file()
            .map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.pick_open_project = Some(Box::new(|| {
        rfd::FileDialog::new()
            .add_filter(filmcraft_ui_egui::i18n::t("FilmCraft Project"), &file_filters::extensions(&["fcproj"]))
            .pick_file()
            .map(|p| p.to_string_lossy().to_string())
    }));
    Ok(Box::new(app))
}
