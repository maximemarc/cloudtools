//! ArtCraft Native inside Craft Studio. It needs an ArtCraft account: generation runs on
//! ArtCraft's servers (`api.storyteller.ai`); the native app has no analytics.

use super::*;

pub(crate) fn create(ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<Box<dyn eframe::App>, String> {
    Ok(Box::new(app::ArtcraftApp::from_parts(ctx, frame.storage())))
}
