use std::sync::Arc;

use shared_lib::types::{AtomicCoordinates, VolumeCube};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use super::atom::AtomInfo;
use super::config::Config;
use super::core::ProjectionMode;
use super::core::Vec3;
use super::scene::Scene;
use super::state::{Appearance, AtomLabels, CoordinateAxes, CoordinateAxis, CubesAndSurfaces, Rgba, State, Transform};
use super::types::Color;

fn color_to_rgba(color: Color) -> Rgba {
    Rgba::new(color.r, color.g, color.b, color.a)
}

#[wasm_bindgen]
pub struct MolecularVisualizer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scene: Scene,
    visualizer_config: Config,
}

#[wasm_bindgen]
impl MolecularVisualizer {
    /// Creates a new MolecularVisualizer instance.
    /// Use as: `const visualizer = await MolecularVisualizer.create(canvas);`
    pub async fn create(
        canvas: HtmlCanvasElement,
        node_type: String,
        data: Vec<u8>,
    ) -> Result<MolecularVisualizer, JsValue> {
        let width = canvas.width();
        let height = canvas.height();

        // Create wgpu instance with WebGPU backend
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // Create surface from canvas
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {e}")))?;

        // Request adapter (GPU handle)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to find an appropriate adapter: {e}")))?;

        // Request device and queue
        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("WebGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {e}")))?;

        let device = Arc::new(device);

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let visualizer_config = Config::new();

        let mut scene = Scene::new(&device, &queue, &config);
        scene.projection_manager.set_viewport(width, height);

        if node_type == "mircmd:chemistry:atomic_coordinates" {
            match serde_json::from_slice::<AtomicCoordinates>(&data) {
                Ok(data) => {
                    scene.load_atomic_coordinates(&device, &visualizer_config, data);
                }
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Failed to deserialize atomic coordinates data: {e}"
                    )));
                }
            }
        } else if node_type == "mircmd:chemistry:volume_cube" {
            match serde_json::from_slice::<VolumeCube>(&data) {
                Ok(data) => {
                    scene.load_volume_cube(data);
                    scene.volume_cube.as_mut().unwrap().add_isosurface(
                        &device,
                        Color::new(1.0, 0.0, 0.0, 0.5),
                        0.05,
                        1.0,
                    );
                    scene.volume_cube.as_mut().unwrap().add_isosurface(
                        &device,
                        Color::new(0.0, 1.0, 0.0, 1.0),
                        0.15,
                        1.0,
                    );
                }
                Err(e) => {
                    return Err(JsValue::from_str(&format!(
                        "Failed to deserialize volume cube data: {e}"
                    )));
                }
            }
        }

        let device = Arc::into_inner(device).unwrap();

        Ok(MolecularVisualizer {
            surface,
            device,
            queue,
            config,
            scene,
            visualizer_config,
        })
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.scene.resize(&self.device, &self.config);
            self.scene.projection_manager.set_viewport(width, height);
            self.scene
                .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
        }
    }

    #[wasm_bindgen]
    pub fn rotate_scene(&mut self, pitch: f32, yaw: f32, roll: f32) {
        if pitch == 0.0 && yaw == 0.0 && roll == 0.0 {
            return;
        }

        self.scene.transform.rotate(pitch, yaw, roll);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn scale_scene(&mut self, factor: f32) {
        if factor == 1.0 || factor == 0.0 {
            return;
        }

        self.scene.transform.scale(Vec3::new(factor, factor, factor));
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_scene_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.scene.transform.set_rotation(pitch, yaw, roll);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_scene_scale(&mut self, factor: f32) {
        if factor == 0.0 {
            return;
        }

        self.scene.transform.set_scale(Vec3::new(factor, factor, factor));
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub async fn new_cursor_position(&mut self, x: u32, y: u32) -> Option<AtomInfo> {
        let (atom, needs_render) = self
            .scene
            .new_cursor_position(x, y, &self.device, &self.queue, &self.visualizer_config)
            .await;

        if needs_render {
            self.scene
                .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
        }

        atom
    }

    #[wasm_bindgen]
    pub async fn toggle_atom_selection(&mut self, x: u32, y: u32) {
        if self
            .scene
            .toggle_atom_selection(x, y, &self.device, &self.queue, &self.visualizer_config)
            .await
        {
            self.scene
                .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
        }
    }

    #[wasm_bindgen]
    pub async fn toggle_projection(&mut self) {
        self.scene.toggle_projection().await;
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_state(&self) -> State {
        let axes = &self.scene.coordinate_axes;
        let color_x = color_to_rgba(axes.color_x);
        let color_y = color_to_rgba(axes.color_y);
        let color_z = color_to_rgba(axes.color_z);
        let style = &self.visualizer_config.style;
        let label = &style.atom_label;
        let bg = style.background_color;

        State::new(
            Transform {
                pitch: self.scene.transform.pitch,
                yaw: self.scene.transform.yaw,
                roll: self.scene.transform.roll,
                scale: self.scene.transform.scale.x,
                perspective: self.scene.projection_manager.mode == ProjectionMode::Perspective,
            },
            AtomLabels {
                symbol_visible: label.symbol_visible,
                number_visible: label.number_visible,
                size: label.size,
                offset: label.offset,
            },
            CubesAndSurfaces::new(self.scene.volume_cube.is_some(), Vec::new()),
            CoordinateAxes::new(
                axes.visible,
                axes.labels_visible,
                axes.both_directions,
                axes.use_origin,
                axes.length,
                axes.thickness,
                // Temporary mapping: runtime uses world-space labels_size; panel expects int font size.
                (axes.labels_size * 100.0).round().max(1.0) as u32,
                self.scene.has_molecule(),
                CoordinateAxis::new(color_x, color_x, axes.label_x.clone()),
                CoordinateAxis::new(color_y, color_y, axes.label_y.clone()),
                CoordinateAxis::new(color_z, color_z, axes.label_z.clone()),
            ),
            Appearance::new(color_to_rgba(bg), String::from("Default")),
        )
    }

    #[wasm_bindgen]
    pub fn set_atom_labels_symbol_visible(&mut self, value: bool) {
        self.visualizer_config.style.atom_label.symbol_visible = value;
        self.scene.set_atom_labels_symbol_visible(value);
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_atom_labels_number_visible(&mut self, value: bool) {
        self.visualizer_config.style.atom_label.number_visible = value;
        self.scene.set_atom_labels_number_visible(value);
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_atom_labels_size(&mut self, value: f32) {
        self.visualizer_config.style.atom_label.size = value;
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_atom_labels_offset(&mut self, value: f32) {
        self.visualizer_config.style.atom_label.offset = value;
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_all_atom_labels_visible(&mut self, value: bool) {
        self.visualizer_config.style.atom_label.label_visible = value;
        self.scene.set_all_atom_labels_visible(value);
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn set_selected_atom_labels_visible(&mut self, value: bool) {
        self.scene.set_selected_atom_labels_visible(value);
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn toggle_all_atom_labels_visible(&mut self) {
        let visible = self.scene.toggle_all_atom_labels_visible();
        self.visualizer_config.style.atom_label.label_visible = visible;
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub fn toggle_selected_atom_labels_visible(&mut self) {
        self.scene.toggle_selected_atom_labels_visible();
        self.scene.update_atom_labels(&self.device, &self.visualizer_config);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub async fn set_coordinate_axes_visible(&mut self, value: bool) {
        self.scene.coordinate_axes.set_visible(value);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub async fn set_coordinate_axes_labels_visible(&mut self, value: bool) {
        self.scene.coordinate_axes.set_labels_visible(value);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub async fn set_coordinate_axes_both_directions(&mut self, value: bool) {
        self.scene.coordinate_axes.set_both_directions(value);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }

    #[wasm_bindgen]
    pub async fn set_coordinate_axes_use_origin(&mut self, value: bool) {
        self.scene.coordinate_axes.set_use_origin(value);
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config, 0);
    }
}
