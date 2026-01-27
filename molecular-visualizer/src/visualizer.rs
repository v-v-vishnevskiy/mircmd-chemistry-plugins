use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct MolecularVisualizer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    data: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl MolecularVisualizer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: &HtmlCanvasElement) -> Result<MolecularVisualizer, JsValue> {
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(MolecularVisualizer {
            canvas: canvas.clone(),
            context,
            data: None,
        })
    }

    #[wasm_bindgen]
    pub fn set_data(&mut self, data: &[u8]) {
        self.data = Some(data.to_vec());
    }

    #[wasm_bindgen]
    pub fn render(&self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear canvas with dark background
        self.context.set_fill_style_str("#2d2d2d");
        self.context.fill_rect(0.0, 0.0, width, height);

        // Draw placeholder content
        self.context.set_fill_style_str("#4a9eff");
        self.context.set_font("24px sans-serif");
        self.context.set_text_align("center");
        self.context.set_text_baseline("middle");
        let _ = self
            .context
            .fill_text("Molecular Visualizer", width / 2.0, height / 2.0 - 20.0);

        self.context.set_fill_style_str("#888888");
        self.context.set_font("14px sans-serif");
        let status = match &self.data {
            Some(data) => format!("Data loaded: {} Mb", data.len() / 1024 / 1024),
            None => "No data loaded".to_string(),
        };
        let _ = self
            .context
            .fill_text(&status, width / 2.0, height / 2.0 + 20.0);

        // Draw a simple border
        self.context.set_stroke_style_str("#4a9eff");
        self.context.set_line_width(2.0);
        self.context.stroke_rect(10.0, 10.0, width - 20.0, height - 20.0);
    }
}
