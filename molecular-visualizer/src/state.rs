use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
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

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Transform {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub scale: f32,
    pub perspective: bool,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct AtomLabels {
    pub symbol_visible: bool,
    pub number_visible: bool,
    pub size: f32,
    pub offset: f32,
}

#[wasm_bindgen]
#[derive(Clone)]
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct SurfaceGroup {
    pub id: u32,
    pub value: f32,
    pub visible: bool,
    #[wasm_bindgen(skip)]
    surfaces: Vec<Surface>,
}

#[wasm_bindgen]
impl SurfaceGroup {
    #[wasm_bindgen(getter)]
    pub fn surfaces(&self) -> Array {
        self.surfaces.iter().cloned().map(JsValue::from).collect()
    }
}

impl SurfaceGroup {
    pub(crate) fn new(id: u32, value: f32, visible: bool, surfaces: Vec<Surface>) -> Self {
        Self {
            id,
            value,
            visible,
            surfaces,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct CubesAndSurfaces {
    pub available: bool,
    #[wasm_bindgen(skip)]
    groups: Vec<SurfaceGroup>,
}

#[wasm_bindgen]
impl CubesAndSurfaces {
    #[wasm_bindgen(getter)]
    pub fn groups(&self) -> Array {
        self.groups.iter().cloned().map(JsValue::from).collect()
    }
}

impl CubesAndSurfaces {
    pub(crate) fn new(available: bool, groups: Vec<SurfaceGroup>) -> Self {
        Self { available, groups }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct CoordinateAxis {
    pub color: Rgba,
    pub label_color: Rgba,
    #[wasm_bindgen(skip)]
    label: String,
}

#[wasm_bindgen]
impl CoordinateAxis {
    #[wasm_bindgen(getter)]
    pub fn label(&self) -> String {
        self.label.clone()
    }
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct CoordinateAxes {
    pub visible: bool,
    pub labels_visible: bool,
    pub both_directions: bool,
    pub use_origin: bool,
    pub length: f32,
    pub thickness: f32,
    pub font_size: u32,
    pub auto_adjust_available: bool,
    #[wasm_bindgen(skip)]
    x: CoordinateAxis,
    #[wasm_bindgen(skip)]
    y: CoordinateAxis,
    #[wasm_bindgen(skip)]
    z: CoordinateAxis,
}

#[wasm_bindgen]
impl CoordinateAxes {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> CoordinateAxis {
        self.x.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> CoordinateAxis {
        self.y.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> CoordinateAxis {
        self.z.clone()
    }
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct Appearance {
    pub background: Rgba,
    #[wasm_bindgen(skip)]
    style: String,
}

#[wasm_bindgen]
impl Appearance {
    #[wasm_bindgen(getter)]
    pub fn style(&self) -> String {
        self.style.clone()
    }
}

impl Appearance {
    pub(crate) fn new(background: Rgba, style: String) -> Self {
        Self { background, style }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct State {
    pub transform: Transform,
    pub atom_labels: AtomLabels,
    #[wasm_bindgen(skip)]
    cubes_and_surfaces: CubesAndSurfaces,
    #[wasm_bindgen(skip)]
    coordinate_axes: CoordinateAxes,
    #[wasm_bindgen(skip)]
    appearance: Appearance,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(getter)]
    pub fn cubes_and_surfaces(&self) -> CubesAndSurfaces {
        self.cubes_and_surfaces.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn coordinate_axes(&self) -> CoordinateAxes {
        self.coordinate_axes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn appearance(&self) -> Appearance {
        self.appearance.clone()
    }
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
