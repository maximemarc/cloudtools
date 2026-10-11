//! The egui state that is global to a context but that every app sets up its own way: fonts,
//! styles and visuals, theme, zoom. Each module keeps its own copy, swapped in when it comes to
//! the front, so one app's fonts or theme never leak into another.

#[derive(Clone)]
pub struct GlobalState {
    options: egui::Options,
    fonts: egui::FontDefinitions,
}

impl GlobalState {
    pub fn capture(ctx: &egui::Context) -> Self {
        Self { options: ctx.options(Clone::clone), fonts: ctx.fonts(|f| f.definitions().clone()) }
    }

    pub fn restore(&self, ctx: &egui::Context) {
        ctx.options_mut(|o| *o = self.options.clone());
        ctx.set_fonts(self.fonts.clone());
    }
}
