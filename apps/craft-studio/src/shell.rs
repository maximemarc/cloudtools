//! The Studio window: a sidebar with every app, a home page, and the app in front.
//!
//! Apps are created the first time they are opened and keep running in the background when
//! another one comes to the front (their documents, undo history and background jobs stay). Only
//! the app in front receives frames and input. Each app's egui-wide state (fonts, theme, zoom) is
//! swapped in and out with it ([`GlobalState`]).

use crate::global_state::GlobalState;
use crate::smoke::{Action, Smoke};
use egui::{Color32, RichText, Vec2};
use std::panic::{AssertUnwindSafe, catch_unwind};
use studio_core::Module;

const SIDEBAR_WIDTH: f32 = 60.0;
const BUTTON: f32 = 40.0;

struct Slot {
    module: Module,
    app: Option<Box<dyn eframe::App>>,
    error: Option<String>,
    /// The module's global egui state while it is not in front.
    state: Option<GlobalState>,
}

pub struct Studio {
    slots: Vec<Slot>,
    /// `None` is the home page.
    active: Option<usize>,
    /// The Studio's own look; apps that were never opened start from it too. Captured on the
    /// first frame: egui has no fonts before it.
    baseline: Option<GlobalState>,
    home_state: Option<GlobalState>,
    /// An app asked to quit (File › Quit): its window close request is cancelled next frame and
    /// the app is closed instead of the Studio.
    closing: Option<usize>,
    /// Asked for on the command line, switched to on the first frame.
    pending: Option<usize>,
    smoke: Option<Smoke>,
    /// The first frame the app in front may run in: fonts set with `Context::set_fonts` (by an
    /// app's constructor, or when its state is swapped in) only apply on the next frame, and an
    /// app that draws with a font family that is not bound yet panics.
    ready_at: u64,
    /// Set when `--smoke` found a failure; `main` turns it into the exit status.
    pub failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Studio {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        modules: Vec<Module>,
        open: Option<&str>,
        smoke: Option<std::path::PathBuf>,
        failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let pending = open.and_then(|id| modules.iter().position(|m| m.info.id == id));
        let smoke = smoke.map(|dir| Smoke::new(dir, modules.len()));
        let slots = modules.into_iter().map(|module| Slot { module, app: None, error: None, state: None }).collect();
        Self { slots, active: None, baseline: None, home_state: None, closing: None, pending, smoke, ready_at: 0, failed }
    }

    /// Runs `f` on the app in front. A panic stops that app (its state can't be trusted any
    /// more) and shows the error in its place; the other apps and the Studio keep running.
    fn guarded(&mut self, i: usize, f: impl FnOnce(&mut dyn eframe::App)) {
        let Some(app) = self.slots[i].app.as_mut() else { return };
        if let Err(panic) = catch_unwind(AssertUnwindSafe(|| f(app.as_mut()))) {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".into());
            let slot = &mut self.slots[i];
            log::error!("{} stopped after an internal error: {message}", slot.module.info.name);
            slot.app = None;
            slot.error = Some(format!("Stopped after an internal error: {message}"));
        }
    }

    /// Brings a module (or home, `None`) to the front, swapping the global egui state.
    fn activate(&mut self, ctx: &egui::Context, target: Option<usize>) {
        if self.active == target {
            return;
        }
        let outgoing = GlobalState::capture(ctx);
        match self.active {
            Some(i) => self.slots[i].state = Some(outgoing),
            None => self.home_state = Some(outgoing),
        }
        self.active = target;
        self.ready_at = ctx.cumulative_frame_nr() + 1;
        let incoming = match target {
            Some(i) => self.slots[i].state.as_ref(),
            None => self.home_state.as_ref(),
        };
        if let Some(state) = incoming.or(self.baseline.as_ref()) {
            state.restore(ctx);
        }
        let title = target.map_or_else(|| "Craft Studio".to_string(), |i| format!("{} — Craft Studio", self.slots[i].module.info.name));
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    /// Closes a module: it saves what it keeps on quitting, then is dropped.
    fn close(&mut self, ctx: &egui::Context, i: usize) {
        if self.active == Some(i) {
            self.activate(ctx, None);
        }
        let slot = &mut self.slots[i];
        if let Some(mut app) = slot.app.take() {
            app.on_exit();
            log::info!("{} closed", slot.module.info.name);
        }
        slot.state = None;
        slot.error = None;
    }

    fn ensure_created(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, i: usize) {
        let slot = &mut self.slots[i];
        if slot.app.is_some() || slot.error.is_some() {
            return;
        }
        let started = std::time::Instant::now();
        let created = catch_unwind(AssertUnwindSafe(|| (slot.module.create)(ctx, frame))).unwrap_or_else(|_| Err("internal error while starting".into()));
        match created {
            Ok(app) => {
                log::info!("{} started in {:?}", slot.module.info.name, started.elapsed());
                slot.app = Some(app);
                self.ready_at = ctx.cumulative_frame_nr() + 1;
            }
            Err(e) => {
                log::error!("{} could not start: {e}", slot.module.info.name);
                slot.error = Some(e);
            }
        }
    }

    /// An app that sent `ViewportCommand::Close` (or asked through `studio_core`) wants to quit.
    fn note_quit(&mut self, ctx: &egui::Context, i: usize) {
        let asked = studio_core::take_close_request(ctx) || ctx.viewport(|v| v.commands.contains(&egui::ViewportCommand::Close));
        if asked {
            self.closing = Some(i);
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Option<Option<usize>> {
        let mut clicked = None;
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            if home_button(ui, self.active.is_none()).on_hover_text("Home").clicked() {
                clicked = Some(None);
            }
            ui.separator();
            for (i, slot) in self.slots.iter().enumerate() {
                let [r, g, b] = slot.module.info.accent;
                let accent = Color32::from_rgb(r, g, b);
                let selected = self.active == Some(i);
                let fill = if selected { accent } else { accent.gamma_multiply(0.35) };
                let text = RichText::new(slot.module.info.monogram).strong().size(15.0).color(Color32::WHITE);
                let button = egui::Button::new(text).fill(fill).corner_radius(8.0).min_size(Vec2::splat(BUTTON));
                let response = ui.add(button).on_hover_text(format!("{}\n{}", slot.module.info.name, slot.module.info.summary));
                if slot.app.is_some() {
                    // A dot under the buttons of running apps.
                    let dot = response.rect.center_bottom() + egui::vec2(0.0, 3.0);
                    ui.painter().circle_filled(dot, 2.0, ui.visuals().strong_text_color());
                }
                if response.clicked() {
                    clicked = Some(Some(i));
                }
            }
        });
        clicked
    }

    fn home(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        const CARD: Vec2 = Vec2::new(300.0, 104.0);
        const GAP: f32 = 12.0;
        let mut open = None;
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            egui::Frame::NONE.inner_margin(egui::Margin::symmetric(28, 24)).show(ui, |ui| {
                ui.heading(RichText::new("Craft Studio").size(30.0).strong());
                ui.label("Every Craft app in one window. Native Rust, offline-first, no telemetry.");
                ui.add_space(24.0);
                let columns = (((ui.available_width() + GAP) / (CARD.x + GAP)).floor() as usize).max(1);
                for row in self.slots.chunks(columns).enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = GAP;
                        for (j, slot) in row.1.iter().enumerate() {
                            if Self::card(ui, slot, CARD).clicked() {
                                open = Some(row.0 * columns + j);
                            }
                        }
                    });
                    ui.add_space(GAP);
                }
                ui.add_space(12.0);
                ui.weak("ArtCraft generates on ArtCraft's servers and needs an ArtCraft account. Every other app works offline.");
            });
        });
        open
    }

    fn card(ui: &mut egui::Ui, slot: &Slot, size: Vec2) -> egui::Response {
        let [r, g, b] = slot.module.info.accent;
        let accent = Color32::from_rgb(r, g, b);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let visuals = ui.style().interact(&response);
        let painter = ui.painter_at(rect);
        painter.rect(rect, 10.0, ui.visuals().faint_bg_color, visuals.bg_stroke, egui::StrokeKind::Inside);
        if response.hovered() {
            painter.rect_stroke(rect, 10.0, egui::Stroke::new(1.5, accent), egui::StrokeKind::Inside);
        }
        let tile = egui::Rect::from_min_size(rect.min + egui::vec2(16.0, 18.0), Vec2::splat(40.0));
        painter.rect_filled(tile, 9.0, accent);
        painter.text(tile.center(), egui::Align2::CENTER_CENTER, slot.module.info.monogram, egui::FontId::proportional(16.0), Color32::WHITE);
        let text = ui.visuals().strong_text_color();
        let x = tile.right() + 14.0;
        painter.text(egui::pos2(x, rect.top() + 20.0), egui::Align2::LEFT_TOP, slot.module.info.name, egui::FontId::proportional(17.0), text);
        painter.text(
            egui::pos2(x, rect.top() + 44.0),
            egui::Align2::LEFT_TOP,
            slot.module.info.summary,
            egui::FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
        let state = if slot.app.is_some() { "Running · click to switch" } else { "Click to open" };
        painter.text(egui::pos2(x, rect.bottom() - 18.0), egui::Align2::LEFT_CENTER, state, egui::FontId::proportional(12.0), accent);
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }

    fn error(&mut self, ui: &mut egui::Ui, i: usize) {
        let name = self.slots[i].module.info.name;
        let message = self.slots[i].error.clone().unwrap_or_default();
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.heading(format!("{name} could not start"));
            ui.add_space(8.0);
            ui.label(RichText::new(message).monospace());
            ui.add_space(16.0);
            if ui.button("Try again").clicked() {
                self.slots[i].error = None;
            }
        });
    }
}

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.baseline.is_none() {
            self.baseline = Some(GlobalState::capture(ctx));
        }
        if let Some(smoke) = &mut self.smoke {
            let failed = self.active.and_then(|i| self.slots[i].error.as_ref().map(|e| (self.slots[i].module.info.id, e.clone())));
            match smoke.tick(ctx, self.active, failed) {
                Action::Wait => {}
                Action::Show(target) => self.activate(ctx, target),
                Action::Quit => {
                    if !smoke.failures.is_empty() {
                        log::error!("smoke: {}", smoke.failures.join("; "));
                        self.failed.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if let Some(i) = self.closing.take() {
            // The close request is the app's own Quit, not the user closing the Studio.
            if ctx.input(|input| input.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
            self.close(ctx, i);
        }
        if let Some(i) = self.pending.take() {
            self.activate(ctx, Some(i));
        }
        let Some(i) = self.active else { return };
        self.ensure_created(ctx, frame, i);
        if ctx.cumulative_frame_nr() < self.ready_at {
            ctx.request_repaint();
            return;
        }
        if self.slots[i].app.is_some() {
            self.guarded(i, |app| app.logic(ctx, frame));
            self.note_quit(ctx, i);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let sidebar = egui::Panel::left("craft-studio-sidebar")
            .resizable(false)
            .exact_size(SIDEBAR_WIDTH)
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(0.0))
            .show(ui, |ui| self.sidebar(ui))
            .inner;
        let mut target = sidebar;
        match self.active {
            None => {
                egui::CentralPanel::default().show(ui, |ui| {
                    if let Some(i) = self.home(ui) {
                        target = Some(Some(i));
                    }
                });
            }
            Some(i) if self.slots[i].error.is_some() => {
                egui::CentralPanel::default().show(ui, |ui| self.error(ui, i));
            }
            Some(i) if ui.ctx().cumulative_frame_nr() < self.ready_at || self.slots[i].app.is_none() => {
                let name = self.slots[i].module.info.name;
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.centered_and_justified(|ui| ui.weak(format!("Starting {name}…")));
                });
            }
            Some(i) => {
                egui::CentralPanel::no_frame().show(ui, |ui| {
                    self.guarded(i, |app| app.ui(ui, frame));
                    self.note_quit(ui.ctx(), i);
                });
            }
        }
        if let Some(target) = target {
            self.activate(ui.ctx(), target);
        }
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(smoke) = &mut self.smoke {
            smoke.on_input(raw_input, self.active.map_or("home", |i| self.slots[i].module.info.id));
        }
        if let Some(i) = self.active {
            self.guarded(i, |app| app.raw_input_hook(ctx, raw_input));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        for app in self.slots.iter_mut().filter_map(|s| s.app.as_mut()) {
            app.save(storage);
        }
    }

    fn on_exit(&mut self) {
        for slot in &mut self.slots {
            if let Some(app) = slot.app.as_mut() {
                app.on_exit();
            }
        }
    }

    fn persist_egui_memory(&self) -> bool {
        // The apps keep their own preferences; mixing their egui memory would leak state.
        false
    }
}

/// The Home button: a house drawn with lines, so it shows whatever fonts the app in front set.
fn home_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON), egui::Sense::click());
    let visuals = if selected { ui.visuals().widgets.active } else { *ui.style().interact(&response) };
    let fill = if selected { ui.visuals().selection.bg_fill } else { visuals.weak_bg_fill };
    ui.painter().rect_filled(rect, 8.0, fill);
    let stroke = egui::Stroke::new(1.8, if selected { ui.visuals().selection.stroke.color } else { visuals.fg_stroke.color });
    let c = rect.center();
    let roof = [c + egui::vec2(-10.0, -1.0), c + egui::vec2(0.0, -10.0), c + egui::vec2(10.0, -1.0)];
    ui.painter().line(roof.to_vec(), stroke);
    let body = egui::Rect::from_min_max(c + egui::vec2(-7.0, -2.0), c + egui::vec2(7.0, 10.0));
    ui.painter().rect_stroke(body, 1.0, stroke, egui::StrokeKind::Middle);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
