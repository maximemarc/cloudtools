//! PhotoCraft inside Craft Studio: `main` without the window set-up, GPU start-up planning,
//! control channel, tablet readers and macOS menus.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let mut services = services::native(None);
    // Brush presets load in the background; the app attaches them when they arrive.
    services.preset_store = services::presets_dir().map(photocraft_engine::preset_store::open_dir_async);
    let mut app = PhotocraftApp::new(Session::new(), services);
    // Long commands and file opens run as background jobs with progress and Cancel.
    app.background_jobs = std::env::var_os("PHOTOCRAFT_INLINE_JOBS").is_none();
    app.perf.gpu_info.canvas = "cpu".into();
    if let Some(rs) = frame.wgpu_render_state().cloned() {
        app.perf.gpu_info.set_adapter(&rs.adapter.get_info());
        let software_window = rs.adapter.get_info().device_type == eframe::wgpu::DeviceType::Cpu;
        if !software_window
            && std::env::var_os("PHOTOCRAFT_CPU_CANVAS").is_none()
            && app.session.prefs().performance.effective_rendering_mode() != photocraft_engine::prefs::RenderingMode::Cpu
        {
            app.set_wgpu(rs);
        }
    }
    if let Some(w) = &app_dirs::current().warning {
        photocraft_ui_egui::notices::post(&mut app, "Portable mode is off", vec![w.clone()], false, None);
    }
    Ok(Box::new(app))
}
