use super::types::Color;

pub fn padded_bytes_per_row(width: u32) -> u32 {
    let unpadded = width.saturating_mul(4);
    unpadded.div_ceil(256) * 256
}

pub fn unpad_rows(src: &[u8], width: u32, height: u32, bytes_per_row: u32) -> Vec<u8> {
    let row_bytes = (width as usize) * 4;
    let mut pixels = Vec::with_capacity(row_bytes * height as usize);
    for y in 0..height as usize {
        let start = y * bytes_per_row as usize;
        pixels.extend_from_slice(&src[start..start + row_bytes]);
    }
    pixels
}

pub fn swizzle_bgra_to_rgba(pixels: &mut [u8]) {
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
}

pub fn color_to_u8(color: Color) -> [u8; 4] {
    [
        (color.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.a.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

pub fn is_bgra(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    )
}

pub fn crop_to_content(pixels: &[u8], width: u32, height: u32, background: [u8; 4]) -> (u32, u32, Vec<u8>) {
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            if pixels[i..i + 4] != background {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x > max_x {
        return (width, height, pixels.to_vec());
    }
    copy_rect(pixels, width, min_x, min_y, max_x + 1, max_y + 1)
}

fn copy_rect(pixels: &[u8], src_width: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> (u32, u32, Vec<u8>) {
    let width = x1 - x0;
    let height = y1 - y0;
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in y0..y1 {
        let start = ((y * src_width + x0) * 4) as usize;
        let end = start + (width as usize) * 4;
        out.extend_from_slice(&pixels[start..end]);
    }
    (width, height, out)
}
