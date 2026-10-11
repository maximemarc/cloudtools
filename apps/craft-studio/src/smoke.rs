//! `--smoke <dir>`: opens every app in turn, lets it run, saves a screenshot of the window per
//! app (`<dir>/<id>.png`, then `home.png`) and quits. Exits with status 1 if an app could not
//! start or panicked. Used to check a build end to end, e.g. under Xvfb in CI.

use std::path::PathBuf;

/// Frames an app runs before its screenshot (with continuous repaint, a few seconds).
const SETTLE_FRAMES: u32 = 120;

pub struct Smoke {
    dir: PathBuf,
    /// The app being checked; `apps` when every app is done (home, then quit).
    step: usize,
    apps: usize,
    frames: u32,
    awaiting_screenshot: bool,
    quit_sent: bool,
    pub failures: Vec<String>,
}

pub enum Action {
    /// Keep running the current step.
    Wait,
    /// Bring this app to the front (`None`: home).
    Show(Option<usize>),
    /// Every screenshot is saved.
    Quit,
}

impl Smoke {
    pub fn new(dir: PathBuf, apps: usize) -> Self {
        Self { dir, step: 0, apps, frames: 0, awaiting_screenshot: false, quit_sent: false, failures: Vec::new() }
    }

    fn target(&self) -> Option<usize> {
        (self.step < self.apps).then_some(self.step)
    }

    /// Called once per frame, before the apps run. `failed`: the app in front could not start
    /// or stopped (its id and error).
    pub fn tick(&mut self, ctx: &egui::Context, active: Option<usize>, failed: Option<(&str, String)>) -> Action {
        ctx.request_repaint();
        if self.step > self.apps {
            if std::mem::replace(&mut self.quit_sent, true) {
                return Action::Wait;
            }
            return Action::Quit;
        }
        if active != self.target() {
            self.frames = 0;
            return Action::Show(self.target());
        }
        if let Some((id, error)) = failed
            && !self.awaiting_screenshot
        {
            self.failures.push(format!("{id}: {error}"));
            self.step += 1;
            return Action::Wait;
        }
        self.frames += 1;
        if self.frames == SETTLE_FRAMES && !self.awaiting_screenshot {
            self.awaiting_screenshot = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        Action::Wait
    }

    /// Saves a screenshot event, if this frame's input has one, and moves to the next step.
    pub fn on_input(&mut self, input: &egui::RawInput, name: &str) {
        let Some(image) = input.events.iter().find_map(|e| match e {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        }) else {
            return;
        };
        let file = self.dir.join(format!("{name}.png"));
        if let Err(e) = save_png(&image, &file) {
            self.failures.push(format!("{name}: screenshot not saved: {e}"));
        } else {
            log::info!("smoke: {}", file.display());
        }
        self.awaiting_screenshot = false;
        self.step += 1;
    }
}

fn save_png(image: &egui::ColorImage, path: &std::path::Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let [w, h] = image.size;
    let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let buffer = image::RgbaImage::from_raw(w as u32, h as u32, pixels).ok_or("bad image size")?;
    buffer.save(path).map_err(|e| e.to_string())
}
