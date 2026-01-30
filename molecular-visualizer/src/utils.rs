use super::types::Color;

pub fn id_to_color(id: usize) -> Color {
    // Supports up to 256³ = 16,777,216 objects

    let r = ((id >> 16) & 0xFF) as f32 / 255.0;
    let g = ((id >> 8) & 0xFF) as f32 / 255.0;
    let b = (id & 0xFF) as f32 / 255.0;
    Color::new(r, g, b, 1.0)
}

// pub fn color_to_id(r: u8, g: u8, b: u8) -> usize {
//     b as usize | (g << 8) as usize | (r << 16) as usize
// }
