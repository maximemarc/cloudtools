//! EffectCraft inside Craft Studio: `main` without the window, launch guard, control channel,
//! macOS menus and the GPU device handlers (the shared device belongs to the shell).

use super::*;

pub(crate) fn create(ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let gpu_failures = effectcraft_ui_egui::gpu_failure::GpuFailureBridge::new(ctx);
    let mut session = effectcraft_host::session();
    // Lazy open and non-blocking auto-save, as in the standalone app.
    session.check_footage_on_open = true;
    session.autosave.background = true;
    if let Some(dir) = effectcraft_host::config_dir() {
        // WebAssembly effect plug-ins in <config>/Plug-ins load before the menus are built.
        let plugins = dir.join("Plug-ins");
        if plugins.is_dir() {
            let _ = session.execute("effect.plugins.load", json!({"folder": plugins.to_string_lossy()}));
        }
        session.config = Some(std::sync::Arc::new(effectcraft_engine::config::DirConfig::new(dir)));
    }
    session.load_settings();
    let recovery = session.begin_recovery();
    let show_home = session.prefs.startup.show_home_on_launch;
    let _ = session.execute("window.restoreScriptPanels", json!({}));
    let mut app = EffectcraftApp::new(session);
    app.set_gpu_failure_bridge(gpu_failures);
    app.ui.start_screen = show_home;
    if let Some(r) = recovery {
        app.offer_recovery(r);
    }
    app.hooks.pick_files = Some(Box::new(|exts: &[&str]| {
        rfd::FileDialog::new().add_filter("Media", exts).pick_files().unwrap_or_default().into_iter().map(|p| p.to_string_lossy().to_string()).collect()
    }));
    app.hooks.pick_save = Some(Box::new(|name: &str| {
        rfd::FileDialog::new().add_filter("EffectCraft Project", &["ecproj"]).set_file_name(name).save_file().map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.pick_open_project = Some(Box::new(|| {
        rfd::FileDialog::new().add_filter("EffectCraft Project", &["ecproj", "ecprojx"]).pick_file().map(|p| p.to_string_lossy().to_string())
    }));
    app.hooks.system_theme = appearance::service();
    app.hooks.audio_device = Some(Box::new(audio_out::open));
    app.hooks.audio_devices = Some(Box::new(audio_out::devices));
    app.hooks.pick_folder = Some(Box::new(|| rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())));
    app.hooks.pick_save_file = Some(Box::new(|name: &str, ext: &str| {
        let path = std::path::Path::new(name);
        let mut d = if ext.is_empty() { rfd::FileDialog::new() } else { rfd::FileDialog::new().add_filter(ext, &[ext]) };
        if let Some(dir) = path.parent().filter(|d| d.is_dir()) {
            d = d.set_directory(dir);
        }
        let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        d.set_file_name(file).save_file().map(|p| p.to_string_lossy().to_string())
    }));
    Ok(Box::new(app))
}
