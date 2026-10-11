//! PdfCraft inside Craft Studio: `app_creator` without the window, single-instance server,
//! control channel and macOS Apple Events.

use super::*;

pub(crate) fn create(_ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    let mut app = PdfCraftApp::new();
    if let Some(json) = frame.storage().and_then(|s| s.get_string("pdfcraft").or_else(|| s.get_string(LEGACY_STORAGE_KEY))) {
        app.restore(&json);
    }
    app.system_text_scale = text_scale::system();
    // Help › Check for Updates asks GitHub only when the user picks it.
    app.update_source = Some(std::sync::Arc::new(updates::latest_release));
    app.printer_source = Some(pdfcraft_ui_egui::system_printers());
    app.os_key_store_ids = cfg!(any(target_os = "macos", target_os = "windows"));
    // Autosave unsaved changes; offer to recover documents a crashed session left behind.
    if let Some(dir) = pdfcraft_ui_egui::RecoveryStore::default_dir() {
        app.enable_recovery(pdfcraft_ui_egui::RecoveryStore::new(dir));
    }
    if let Some(state) = frame.wgpu_render_state() {
        notify_software_renderer(&mut app, state.adapter.get_info().device_type);
    }
    Ok(Box::new(app))
}
