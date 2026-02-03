use ab_glyph::{Font, FontRef, Glyph, PxScale, ScaleFont};
use std::collections::HashMap;

const DEFAULT_CHAR: char = '?';
const DEFAULT_ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 _.,:;!?–-+±=/\\|#()[]{}<>*&$%^@~§'\"`";

#[derive(Clone, Copy, Debug)]
pub struct CharInfo {
    pub width: f32,
    pub height: f32,
    pub u_min: f32,
    pub u_max: f32,
    pub v_min: f32,
    pub v_max: f32,
}

pub struct FontAtlas {
    pub size: u32,
    pub font_size: f32,
    pub padding: u32,
    pub chars: HashMap<char, CharInfo>,
    pub default_char_info: CharInfo,
    pub texture: Vec<u8>, // each pixel has only alpha-channel
}

impl FontAtlas {
    pub fn new(font_data: &[u8], size: u32, font_size: f32, alphabet: &str, padding: u32) -> Self {
        let font = FontRef::try_from_slice(font_data).expect("Failed to load font");
        let scaled_font = font.as_scaled(PxScale::from(font_size));

        let chars: Vec<char> = alphabet.chars().collect();

        // Calculate max ascent (highest point above baseline) and max descent (lowest point below)
        // bounds.min.y is negative for glyphs above baseline, bounds.max.y is positive for descenders
        let mut max_ascent: f32 = 0.0;
        let mut max_descent: f32 = 0.0;
        for ch in &chars {
            let glyph: Glyph = scaled_font.scaled_glyph(*ch);
            if let Some(outlined) = scaled_font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();

                // Ascent is the distance above baseline (negative bounds.min.y)
                let ascent = -bounds.min.y;
                if ascent > max_ascent {
                    max_ascent = ascent;
                }
                // Descent is the distance below baseline (positive bounds.max.y)
                if bounds.max.y > max_descent {
                    max_descent = bounds.max.y;
                }
            }
        }
        let max_ascent = max_ascent.ceil();
        let max_descent = max_descent.ceil();
        let max_height = (max_ascent + max_descent) as u32;

        let mut texture = vec![0u8; (size * size) as usize];
        let mut char_infos = HashMap::new();

        let mut x: u32 = 0;
        let mut y: u32 = 0;

        for ch in &chars {
            let glyph: Glyph = scaled_font.scaled_glyph(*ch);
            if let Some(outlined) = scaled_font.outline_glyph(glyph.clone()) {
                let bounds = outlined.px_bounds();
                let char_width = (bounds.max.x - bounds.min.x).ceil() as u32;

                if x + char_width + padding > size {
                    x = 0;
                    y += max_height + padding;
                }

                // Vertical: align baseline across all glyphs (baseline is at row_y + max_ascent)
                // The glyph's top (bounds.min.y) is at baseline + bounds.min.y
                // So draw_y for py=0 should be: row_y + max_ascent + bounds.min.y
                let glyph_ascent = -bounds.min.y;
                let vertical_offset = (max_ascent - glyph_ascent).ceil() as i32;

                outlined.draw(|px, py, coverage| {
                    let draw_x = x as i32 + px as i32;
                    let draw_y = y as i32 + vertical_offset + py as i32;

                    if draw_x >= 0 && draw_y >= 0 && (draw_x as u32) < size && (draw_y as u32) < size {
                        let idx = (draw_y as u32 * size + draw_x as u32) as usize;
                        texture[idx] = (coverage * 255.0) as u8; // alpha
                    }
                });

                let char_info = CharInfo {
                    width: char_width as f32,
                    height: max_height as f32,
                    u_min: x as f32 / size as f32,
                    u_max: (x + char_width) as f32 / size as f32,
                    v_min: 1.0 - ((y + max_height) as f32 / size as f32),
                    v_max: 1.0 - (y as f32 / size as f32),
                };

                char_infos.insert(*ch, char_info);
                x += char_width + padding;
            }
        }

        let default_char_info = *char_infos
            .get(&DEFAULT_CHAR)
            .expect("Default char '?' must be in alphabet");

        Self {
            font_size,
            size,
            padding,
            chars: char_infos,
            default_char_info,
            texture,
        }
    }

    pub fn from_embedded_font(size: u32, font_size: f32, padding: u32) -> Self {
        const FONT_DATA: &[u8] = include_bytes!("../resources/fonts/Inter-Bold.ttf");
        Self::new(FONT_DATA, size, font_size, DEFAULT_ALPHABET, padding)
    }

    pub fn get_char_info(&self, ch: char) -> &CharInfo {
        self.chars.get(&ch).unwrap_or(&self.default_char_info)
    }
}
