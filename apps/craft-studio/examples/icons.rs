//! Writes the Studio icon for the installers: `craft-studio-<size>.png` and `craft-studio.ico`.
//!
//! `cargo run --release -p craft-studio --example icons -- <dir>`

#[allow(dead_code)]
#[path = "../src/icon.rs"]
mod icon;

/// What the Linux icon theme, the macOS `.icns` and the Windows `.ico` take.
const PNG_SIZES: [u32; 8] = [16, 32, 48, 64, 128, 256, 512, 1024];
const ICO_SIZES: [u32; 5] = [16, 32, 48, 64, 256];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(std::env::args_os().nth(1).ok_or("usage: icons <dir>")?);
    std::fs::create_dir_all(&dir)?;
    for size in PNG_SIZES {
        let image = image::RgbaImage::from_raw(size, size, icon::rgba(size)).ok_or("bad icon buffer")?;
        image.save(dir.join(format!("craft-studio-{size}.png")))?;
    }
    let frames = ICO_SIZES
        .iter()
        .map(|&size| image::codecs::ico::IcoFrame::as_png(&icon::rgba(size), size, size, image::ExtendedColorType::Rgba8))
        .collect::<Result<Vec<_>, _>>()?;
    image::codecs::ico::IcoEncoder::new(std::fs::File::create(dir.join("craft-studio.ico"))?).encode_images(&frames)?;
    println!("icons written to {}", dir.display());
    Ok(())
}
