//! Turns `src-tauri/icons/source/app-icon-mockup.png` (dark page, rounded tile, document glyph)
//! into the four layers `tauri icon` consumes via `app-icon.json`:
//!   app-icon.png            full tile with transparent rounded corners (desktop / legacy launchers)
//!   app-icon-bg.png         edge-to-edge tile gradient (adaptive background)
//!   app-icon-fg.png         document glyph on transparency, centred (adaptive foreground)
//!   app-icon-monochrome.png white silhouette of the glyph with accent details cut out (themed icon)
//! The tiny title text of the mockup is removed: it is illegible at launcher sizes.
//!
//! Usage: cargo run --release -- <mockup.png> <out-dir>

use image::{ImageBuffer, Rgba, RgbaImage, imageops};

const OUT: u32 = 1024;

fn lum(p: &Rgba<u8>) -> f32 {
    0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2])
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Bounding box (inclusive) of pixels brighter than `threshold` along the centre row/column.
fn extent(img: &RgbaImage, threshold: f32) -> (u32, u32, u32, u32) {
    let (w, h) = img.dimensions();
    let (cx, cy) = (w / 2, h / 2);
    let xs: Vec<u32> = (0..w)
        .filter(|&x| lum(img.get_pixel(x, cy)) > threshold)
        .collect();
    let ys: Vec<u32> = (0..h)
        .filter(|&y| lum(img.get_pixel(cx, y)) > threshold)
        .collect();
    (xs[0], *xs.last().expect("row"), ys[0], *ys.last().expect("col"))
}

/// Signed distance from a point to a rounded square centred at (c, c) (negative = inside).
fn rounded_square_sdf(x: f32, y: f32, c: f32, half: f32, r: f32) -> f32 {
    let qx = (x - c).abs() - (half - r);
    let qy = (y - c).abs() - (half - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - r
}

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("mockup path");
    let out = std::path::PathBuf::from(args.next().expect("output dir"));
    std::fs::create_dir_all(&out).expect("create out dir");
    let mut src = image::open(&input).expect("open mockup").to_rgba8();

    // --- tile: square region of the rounded tile (the glow below is ignored by using the width) ---
    let (tx0, tx1, ty0, _) = extent(&src, 30.0);
    let tile_w = tx1 - tx0 + 1;
    let tile_color = *src.get_pixel(tx0 + 25, ty0 + tile_w / 2);
    println!(
        "tile: x={tx0}..{tx1} y={ty0}.. size={tile_w} color=#{:02X}{:02X}{:02X}",
        tile_color[0], tile_color[1], tile_color[2]
    );

    // --- document glyph bbox ---
    let (dx0, dx1, dy0, dy1) = extent(&src, 200.0);
    println!("doc: x={dx0}..{dx1} y={dy0}..{dy1}");

    // Remove the title text: the text band (below the folded corner, above the rule line) is
    // repainted row by row with the paper colour sampled at the left margin, which keeps the
    // vertical paper gradient and leaves no anti-aliasing speckle behind.
    let band_top = dy0 + (dy1 - dy0) * 22 / 100;
    let band_bottom = dy0 + (dy1 - dy0) * 36 / 100;
    let band_left = dx0 + (dx1 - dx0) * 8 / 100;
    let band_right = dx0 + (dx1 - dx0) * 84 / 100;
    for y in band_top..band_bottom {
        let paper = *src.get_pixel(dx0 + 12, y);
        for x in band_left..band_right {
            src.put_pixel(x, y, paper);
        }
    }

    // --- app-icon.png: tile with transparent rounded corners, scaled to OUT ---
    let tile = imageops::crop_imm(&src, tx0, ty0, tile_w, tile_w).to_image();
    let mut full = imageops::resize(&tile, OUT, OUT, imageops::FilterType::Lanczos3);
    let radius = OUT as f32 * 0.22;
    let (c, half) = (OUT as f32 / 2.0, OUT as f32 / 2.0);
    for (x, y, p) in full.enumerate_pixels_mut() {
        let d = rounded_square_sdf(x as f32 + 0.5, y as f32 + 0.5, c, half, radius);
        let a = clamp01(0.5 - d); // one-pixel anti-aliased edge
        p[3] = (a * 255.0).round() as u8;
    }
    full.save(out.join("app-icon.png")).expect("save app-icon.png");

    // --- app-icon-bg.png: the vertical gradient of the tile itself, sampled at its left margin
    // (well outside the document) and stretched across the whole canvas edge to edge ---
    let mut bg: RgbaImage = ImageBuffer::new(OUT, OUT);
    let sample_x = tx0 + tile_w / 12;
    for y in 0..OUT {
        let src_y = ty0 + (u64::from(y) * u64::from(tile_w - 1) / u64::from(OUT - 1)) as u32;
        let mut col = *src.get_pixel(sample_x, src_y);
        col[3] = 255;
        for x in 0..OUT {
            bg.put_pixel(x, y, col);
        }
    }
    bg.save(out.join("app-icon-bg.png")).expect("save app-icon-bg.png");

    // --- app-icon-fg.png: document glyph keyed off the tile, centred at 44% of the canvas ---
    let margin = 24;
    let (gx0, gy0) = (dx0 - margin, dy0 - margin);
    let (gw, gh) = (dx1 - dx0 + 1 + 2 * margin, dy1 - dy0 + 1 + 2 * margin);
    let mut glyph: RgbaImage = ImageBuffer::new(gw, gh);
    let tile_l = lum(&tile_color);
    for (x, y, p) in glyph.enumerate_pixels_mut() {
        let s = *src.get_pixel(gx0 + x, gy0 + y);
        let l = lum(&s);
        // Paper and accent pixels are far brighter than the tile; the soft shadow is darker.
        let a = clamp01((l - tile_l) / (120.0 - tile_l));
        if a <= 0.0 {
            *p = Rgba([0, 0, 0, 0]);
            continue;
        }
        // Un-mix the tile colour out of anti-aliased edge pixels.
        let mut col = [0u8; 3];
        for i in 0..3 {
            let v = (f32::from(s[i]) - f32::from(tile_color[i]) * (1.0 - a)) / a;
            col[i] = v.round().clamp(0.0, 255.0) as u8;
        }
        *p = Rgba([col[0], col[1], col[2], (a * 255.0).round() as u8]);
    }
    let target_h = (OUT as f32 * 0.44) as u32; // keeps the glyph inside the 66dp safe zone
    let scale = target_h as f32 / gh as f32;
    let glyph_scaled = imageops::resize(
        &glyph,
        (gw as f32 * scale) as u32,
        target_h,
        imageops::FilterType::Lanczos3,
    );
    let mut fg: RgbaImage = ImageBuffer::from_pixel(OUT, OUT, Rgba([0, 0, 0, 0]));
    let ox = (OUT - glyph_scaled.width()) / 2;
    let oy = (OUT - glyph_scaled.height()) / 2;
    imageops::overlay(&mut fg, &glyph_scaled, i64::from(ox), i64::from(oy));
    fg.save(out.join("app-icon-fg.png")).expect("save app-icon-fg.png");

    // --- app-icon-monochrome.png: paper silhouette in white, accent/ink details as holes ---
    let mut mono: RgbaImage = ImageBuffer::from_pixel(OUT, OUT, Rgba([0, 0, 0, 0]));
    for (x, y, p) in fg.enumerate_pixels() {
        if p[3] == 0 {
            continue;
        }
        let is_paper = p[0].min(p[1]).min(p[2]) > 205;
        let a = if is_paper { p[3] } else { 0 };
        mono.put_pixel(x, y, Rgba([255, 255, 255, a]));
    }
    mono.save(out.join("app-icon-monochrome.png"))
        .expect("save app-icon-monochrome.png");

    std::fs::write(
        out.join("app-icon.json"),
        "{\n  \"default\": \"app-icon.png\",\n  \"android_bg\": \"app-icon-bg.png\",\n  \"android_fg\": \"app-icon-fg.png\",\n  \"android_fg_scale\": 100,\n  \"android_monochrome\": \"app-icon-monochrome.png\"\n}\n",
    )
    .expect("write manifest");
    println!("wrote layers to {}", out.display());
}
