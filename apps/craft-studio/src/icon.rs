//! The window icon, drawn in code: four app-coloured tiles on a dark rounded square.

const SIZE: u32 = 64;

pub fn window_icon() -> egui::IconData {
    const TILES: [[u8; 3]; 4] = [[0x31, 0xa8, 0xff], [0xff, 0x9a, 0x00], [0x21, 0x8a, 0x4c], [0xe0, 0x3c, 0x31]];
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let inside = rounded(x, y, 14);
            let (tx, ty) = (x.saturating_sub(10) / 23, y.saturating_sub(10) / 23);
            let in_tile = (10..54).contains(&x) && (10..54).contains(&y) && (x - 10) % 23 < 21 && (y - 10) % 23 < 21;
            let [r, g, b] = if in_tile { TILES[(ty * 2 + tx).min(3) as usize] } else { [0x17, 0x19, 0x1d] };
            rgba.extend_from_slice(&[r, g, b, if inside { 255 } else { 0 }]);
        }
    }
    egui::IconData { rgba, width: SIZE, height: SIZE }
}

/// Inside a square with corners of radius `r`.
fn rounded(x: u32, y: u32, r: u32) -> bool {
    let (x, y, r, max) = (x as i64, y as i64, r as i64, SIZE as i64 - 1);
    let cx = x.clamp(r, max - r);
    let cy = y.clamp(r, max - r);
    (x - cx).pow(2) + (y - cy).pow(2) <= r * r
}
