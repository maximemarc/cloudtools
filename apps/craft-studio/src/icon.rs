//! The Studio icon, drawn in code: four app-coloured tiles on a dark rounded square. The window
//! uses it at 64 px; `examples/icons.rs` exports it at the sizes the installers need.

/// The drawing grid: shapes are laid out in 64 units whatever the output size.
const GRID: f32 = 64.0;
const CORNER: f32 = 14.0;
/// The tile area (10 units from each edge) and the tiles: 21 units with 2-unit gaps.
const TILES_START: f32 = 10.0;
const TILES_END: f32 = 54.0;
const TILE_STRIDE: f32 = 23.0;
const TILE_SIDE: f32 = 21.0;
const TILES: [[u8; 3]; 4] = [[0x31, 0xa8, 0xff], [0xff, 0x9a, 0x00], [0x21, 0x8a, 0x4c], [0xe0, 0x3c, 0x31]];
const BACKGROUND: [u8; 3] = [0x17, 0x19, 0x1d];
/// Samples per pixel side (antialiasing).
const SAMPLES: u32 = 4;

pub fn window_icon() -> egui::IconData {
    egui::IconData { rgba: rgba(64), width: 64, height: 64 }
}

/// The icon as `size` × `size` RGBA pixels.
pub fn rgba(size: u32) -> Vec<u8> {
    let scale = GRID / size as f32;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (mut sum, mut covered) = ([0u32; 3], 0u32);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let u = (x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) * scale;
                    let v = (y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) * scale;
                    if inside(u, v) {
                        let colour = tile(u, v).unwrap_or(BACKGROUND);
                        sum.iter_mut().zip(colour).for_each(|(s, c)| *s += u32::from(c));
                        covered += 1;
                    }
                }
            }
            let [r, g, b] = sum.map(|s| if covered == 0 { 0 } else { (s / covered) as u8 });
            out.extend_from_slice(&[r, g, b, (covered * 255 / (SAMPLES * SAMPLES)) as u8]);
        }
    }
    out
}

/// Inside the rounded square.
fn inside(u: f32, v: f32) -> bool {
    let (cu, cv) = (u.clamp(CORNER, GRID - CORNER), v.clamp(CORNER, GRID - CORNER));
    (u - cu).powi(2) + (v - cv).powi(2) <= CORNER * CORNER
}

/// The colour of the tile at (`u`, `v`), if there is one.
fn tile(u: f32, v: f32) -> Option<[u8; 3]> {
    let span = TILES_START..TILES_END;
    if !span.contains(&u) || !span.contains(&v) {
        return None;
    }
    let (du, dv) = (u - TILES_START, v - TILES_START);
    if du % TILE_STRIDE >= TILE_SIDE || dv % TILE_STRIDE >= TILE_SIDE {
        return None;
    }
    let (column, row) = ((du / TILE_STRIDE) as usize, (dv / TILE_STRIDE) as usize);
    TILES.get(row * 2 + column).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    }

    #[test]
    fn corners_are_transparent_and_tiles_coloured_at_every_size() {
        for size in [16, 64, 256, 1024] {
            let icon = rgba(size);
            assert_eq!(icon.len(), (size * size * 4) as usize);
            assert_eq!(pixel(&icon, size, 0, 0)[3], 0, "{size} px corner");
            assert_eq!(pixel(&icon, size, size - 1, size - 1)[3], 0, "{size} px corner");
            // The centre of the top-left tile (20.5 units) and of the gap between tiles (32 units).
            let at = |units: f32| (units / GRID * size as f32) as u32;
            assert_eq!(pixel(&icon, size, at(20.5), at(20.5)), [0x31, 0xa8, 0xff, 255], "{size} px tile");
            if size >= 64 {
                assert_eq!(pixel(&icon, size, at(32.0), at(20.5)), [0x17, 0x19, 0x1d, 255], "{size} px gap");
            }
        }
    }

    #[test]
    fn is_symmetric() {
        let size = 256;
        let icon = rgba(size);
        for (x, y) in [(0, 128), (3, 3), (10, 40), (60, 200)] {
            let (a, b) = (pixel(&icon, size, x, y)[3], pixel(&icon, size, size - 1 - x, size - 1 - y)[3]);
            // One sample of rounding at most.
            assert!(a.abs_diff(b) <= 16, "({x}, {y}): {a} vs {b}");
        }
    }
}
