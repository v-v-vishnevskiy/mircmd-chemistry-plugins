use super::core::Mat4;
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

pub fn get_model_matrix(mat: &Mat4<f32>) -> [[f32; 4]; 4] {
    let matrix = mat.data;
    [
        [matrix[0], matrix[1], matrix[2], matrix[3]],
        [matrix[4], matrix[5], matrix[6], matrix[7]],
        [matrix[8], matrix[9], matrix[10], matrix[11]],
        [matrix[12], matrix[13], matrix[14], matrix[15]],
    ]
}
