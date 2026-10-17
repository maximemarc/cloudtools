//! Craft Studio: every Craft app (PhotoCraft, LightCraft, FilmCraft, PdfCraft, VectorCraft,
//! EffectCraft, WordCraft, DesignCraft, GridCraft, CADCraft, DeckCraft, SoundCraft and ArtCraft)
//! in one native window.
//!
//! Usage: `craft-studio [--app <id>] [--list] [--smoke <dir>] [--version]`
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod global_state;
mod gpu;
mod icon;
mod modules;
mod shell;
mod smoke;

const APP_ID: &str = "craft-studio";

struct Args {
    app: Option<String>,
    smoke: Option<std::path::PathBuf>,
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut app = None;
    let mut smoke = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--app" => app = Some(args.next().ok_or("--app needs an app id (see --list)")?),
            "--smoke" => smoke = Some(args.next().ok_or("--smoke needs a folder for the screenshots")?.into()),
            "--list" => {
                for m in modules::all() {
                    println!("{:<12} {:<12} {}", m.info.id, m.info.name, m.info.summary);
                }
                return Ok(None);
            }
            "--version" | "-V" => {
                println!("craft-studio {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--help" | "-h" => {
                println!("craft-studio [--app <id>] [--list] [--smoke <dir>] [--version]");
                return Ok(None);
            }
            other => return Err(format!("unknown argument {other:?} (try --help)")),
        }
    }
    if let Some(id) = &app
        && !modules::all().iter().any(|m| m.info.id == id)
    {
        return Err(format!("no app {id:?} in this build (see --list)"));
    }
    Ok(Some(Args { app, smoke }))
}

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn")).init();
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => return Ok(()),
        Err(e) => {
            eprintln!("craft-studio: {e}");
            std::process::exit(2);
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Craft Studio")
            .with_app_id(APP_ID)
            .with_inner_size([1600.0, 1000.0])
            .with_min_inner_size([960.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(icon::window_icon()),
        persist_window: true,
        wgpu_options: gpu::wgpu_options(),
        ..Default::default()
    };
    let failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let studio_failed = failed.clone();
    let result = eframe::run_native(
        "Craft Studio",
        options,
        Box::new(move |cc| Ok(Box::new(shell::Studio::new(cc, modules::all(), args.app.as_deref(), args.smoke, studio_failed)))),
    );
    if failed.load(std::sync::atomic::Ordering::Relaxed) {
        std::process::exit(1);
    }
    result
}
