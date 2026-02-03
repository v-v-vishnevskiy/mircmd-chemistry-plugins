#[cfg(not(feature = "debug-tools"))]
compile_error!(
    "This binary requires the 'debug-tools' feature. Run with: cargo run --bin debug_font_atlas --features debug-tools"
);

#[path = "../core/font_atlas.rs"]
mod font_atlas;

use font_atlas::FontAtlas;

#[cfg(feature = "debug-tools")]
fn main() {
    use image::{ImageBuffer, Rgba};

    println!("Creating font atlas...");

    let atlas = FontAtlas::from_embedded_font(4096, 600.0, 3);

    println!("\nCharacter info:");
    let chars_info = atlas.chars.clone();
    let mut chars: Vec<_> = chars_info.iter().collect();
    chars.sort_by_key(|(ch, _)| *ch);

    for (ch, info) in &chars {
        println!(
            "  '{}': size=({:.1}, {:.1}), uv=({:.4}, {:.4}) - ({:.4}, {:.4})",
            ch, info.width, info.height, info.u_min, info.v_min, info.u_max, info.v_max
        );
    }

    println!("\nTotal characters: {}", chars.len());

    // Convert alpha-only texture to RGBA for visualization
    let pixels = (atlas.size * atlas.size) as usize;
    let mut rgba_data = Vec::with_capacity(pixels * 4);

    for i in 0..pixels {
        let alpha = atlas.texture[i];
        if alpha > 0 {
            rgba_data.extend_from_slice(&[255, 255, 255, alpha]); // White with alpha
        } else {
            rgba_data.extend_from_slice(&[30, 30, 60, 255]); // Dark blue background
        }
    }

    let mut img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(atlas.size, atlas.size, rgba_data).expect("Failed to create image buffer");

    // Draw UV coordinate boundaries for debugging (red rectangles)
    let red = Rgba([255u8, 0, 0, 255]);
    for char_info in chars_info.values() {
        let x1 = (char_info.u_min * atlas.size as f32) as u32;
        let x2 = (char_info.u_max * atlas.size as f32) as u32;
        let y1 = ((1.0 - char_info.v_max) * atlas.size as f32) as u32;
        let y2 = ((1.0 - char_info.v_min) * atlas.size as f32) as u32;

        // Draw horizontal lines (top and bottom)
        for px in x1..=x2 {
            if px < atlas.size {
                if y1 < atlas.size {
                    img.put_pixel(px, y1, red);
                }
                if y2 < atlas.size {
                    img.put_pixel(px, y2, red);
                }
            }
        }

        // Draw vertical lines (left and right)
        for py in y1..=y2 {
            if py < atlas.size {
                if x1 < atlas.size {
                    img.put_pixel(x1, py, red);
                }
                if x2 < atlas.size {
                    img.put_pixel(x2, py, red);
                }
            }
        }
    }

    let output_path = "font_atlas_debug.png";
    img.save(output_path).expect("Failed to save image");
    println!("\nAtlas saved to: {output_path}");
}

#[cfg(not(feature = "debug-tools"))]
fn main() {}
