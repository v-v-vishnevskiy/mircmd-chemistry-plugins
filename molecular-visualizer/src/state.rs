use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct CoordinateAxes {
    pub visible: bool,
    pub labels_visible: bool,
    pub both_directions: bool,
    pub use_origin: bool,
}

#[wasm_bindgen]
pub struct State {
    pub coordinate_axes: CoordinateAxes,
}
