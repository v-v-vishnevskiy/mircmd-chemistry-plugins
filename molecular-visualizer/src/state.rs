use serde::Serialize;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Serialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub(crate) fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct Transform {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub scale: f32,
    pub perspective: bool,
}

#[derive(Clone, Copy, Serialize)]
pub struct AtomLabels {
    pub symbol_visible: bool,
    pub number_visible: bool,
    pub size: f32,
    pub offset: f32,
}

#[derive(Clone, Serialize)]
pub struct Surface {
    pub id: u32,
    pub inverted: bool,
    pub visible: bool,
    pub color: Rgba,
}

impl Surface {
    pub(crate) fn new(id: u32, inverted: bool, visible: bool, color: Rgba) -> Self {
        Self {
            id,
            inverted,
            visible,
            color,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct SurfaceGroup {
    pub id: u32,
    pub value: f64,
    pub visible: bool,
    pub surfaces: Vec<Surface>,
}

impl SurfaceGroup {
    pub(crate) fn new(id: u32, value: f64, visible: bool, surfaces: Vec<Surface>) -> Self {
        Self {
            id,
            value,
            visible,
            surfaces,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct CubesAndSurfaces {
    pub available: bool,
    pub groups: Vec<SurfaceGroup>,
}

impl CubesAndSurfaces {
    pub(crate) fn new(available: bool, groups: Vec<SurfaceGroup>) -> Self {
        Self { available, groups }
    }
}

#[derive(Clone, Serialize)]
pub struct CoordinateAxis {
    pub color: Rgba,
    pub label_color: Rgba,
    pub label: String,
}

impl CoordinateAxis {
    pub(crate) fn new(color: Rgba, label_color: Rgba, label: String) -> Self {
        Self {
            color,
            label_color,
            label,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct CoordinateAxes {
    pub visible: bool,
    pub labels_visible: bool,
    pub both_directions: bool,
    pub use_origin: bool,
    pub length: f32,
    pub thickness: f32,
    pub font_size: u32,
    pub auto_adjust_available: bool,
    pub x: CoordinateAxis,
    pub y: CoordinateAxis,
    pub z: CoordinateAxis,
}

impl CoordinateAxes {
    pub(crate) fn new(
        visible: bool,
        labels_visible: bool,
        both_directions: bool,
        use_origin: bool,
        length: f32,
        thickness: f32,
        font_size: u32,
        auto_adjust_available: bool,
        x: CoordinateAxis,
        y: CoordinateAxis,
        z: CoordinateAxis,
    ) -> Self {
        Self {
            visible,
            labels_visible,
            both_directions,
            use_origin,
            length,
            thickness,
            font_size,
            auto_adjust_available,
            x,
            y,
            z,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Appearance {
    pub background: Rgba,
    pub style: String,
    pub style_names: Vec<String>,
}

impl Appearance {
    pub(crate) fn new(background: Rgba, style: String, style_names: Vec<String>) -> Self {
        Self {
            background,
            style,
            style_names,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct State {
    pub transform: Transform,
    pub atom_labels: AtomLabels,
    pub cubes_and_surfaces: CubesAndSurfaces,
    pub coordinate_axes: CoordinateAxes,
    pub appearance: Appearance,
}

impl State {
    pub(crate) fn new(
        transform: Transform,
        atom_labels: AtomLabels,
        cubes_and_surfaces: CubesAndSurfaces,
        coordinate_axes: CoordinateAxes,
        appearance: Appearance,
    ) -> Self {
        Self {
            transform,
            atom_labels,
            cubes_and_surfaces,
            coordinate_axes,
            appearance,
        }
    }
}

pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|err| JsValue::from_str(&err.to_string()))
}

pub(crate) fn rendered_image_to_js(width: u32, height: u32, data: &[u8]) -> Result<JsValue, JsValue> {
    let object = js_sys::Object::new();
    js_sys::Reflect::set(&object, &JsValue::from_str("width"), &JsValue::from(width))?;
    js_sys::Reflect::set(&object, &JsValue::from_str("height"), &JsValue::from(height))?;
    js_sys::Reflect::set(&object, &JsValue::from_str("data"), &js_sys::Uint8Array::from(data))?;
    Ok(object.into())
}
