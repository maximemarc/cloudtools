//! Contract between the Craft Studio shell and the app modules it hosts.
//!
//! A module is one Craft app (PhotoCraft, WordCraft…) running inside the Studio window. The shell
//! creates it lazily, the first time the user opens it, from the live `egui::Context` and
//! `eframe::Frame` (the `CreationContext` an app normally gets cannot be built outside eframe),
//! then forwards `eframe::App` calls to the module that is in front.

/// What the shell shows about a module before it is created.
#[derive(Clone, Copy, Debug)]
pub struct ModuleInfo {
    /// Stable identifier, also the `--app` value on the command line (`photocraft`).
    pub id: &'static str,
    /// Product name (`PhotoCraft`).
    pub name: &'static str,
    /// Two letters for the app's button in the sidebar (`Ph`).
    pub monogram: &'static str,
    /// One line on what it is for.
    pub summary: &'static str,
    /// Accent colour of the app's tile, sRGB.
    pub accent: [u8; 3],
}

/// Builds the app on the first frame it is shown. Errors are shown in place of the app.
pub type Create = fn(&egui::Context, &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String>;

/// A module as registered with the shell.
#[derive(Clone, Copy)]
pub struct Module {
    pub info: ModuleInfo,
    pub create: Create,
}

fn close_id() -> egui::Id {
    egui::Id::new("craft-studio.close-module")
}

/// Called by a module whose app asked to quit (File › Quit, ⌘Q): the shell closes the module
/// and goes back home instead of closing the Studio window.
pub fn request_close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(close_id(), true));
}

/// Whether a module asked to close since the last call (used by the shell).
pub fn take_close_request(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(close_id()).unwrap_or(false))
}
