use std::sync::Arc;

use super::config::Config;
use super::core::Vec3;
use super::scene::Scene;
use shared_lib::types::AtomicCoordinates;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct MolecularVisualizer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scene: Scene,
    visualizer_config: Config,
    node_data: AtomicCoordinates,
}

#[wasm_bindgen]
impl MolecularVisualizer {
    /// Creates a new MolecularVisualizer instance.
    /// Use as: `const visualizer = await MolecularVisualizer.create(canvas);`
    pub async fn create(canvas: HtmlCanvasElement, data: Vec<u8>) -> Result<MolecularVisualizer, JsValue> {
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

        let mut scene = Scene::new(&device, &config);
        scene.projection_manager.set_viewport(width, height);

        let node_data: AtomicCoordinates = serde_json::from_slice(&data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize data: {e}")))?;

        scene.load_atomic_coordinates(&device, &visualizer_config, &node_data);

        let device = Arc::into_inner(device).unwrap();

        Ok(MolecularVisualizer {
            surface,
            device,
            queue,
            config,
            scene,
            visualizer_config,
            node_data,
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
        }
    }

    #[wasm_bindgen]
    pub fn rotate_scene(&mut self, pitch: f32, yaw: f32, roll: f32) {
        if pitch == 0.0 && yaw == 0.0 && roll == 0.0 {
            return;
        }

        self.scene.transform.rotate(pitch, yaw, roll);
    }

    #[wasm_bindgen]
    pub fn scale_scene(&mut self, factor: f32) {
        if factor == 1.0 || factor == 0.0 {
            return;
        }

        self.scene.transform.scale(Vec3::new(factor, factor, factor));
    }

    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        self.scene
            .render(&self.surface, &self.device, &self.queue, &self.visualizer_config);

        Ok(())
    }
}
