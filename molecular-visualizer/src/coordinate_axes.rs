use super::core::char_instance_data::create_char_instance_buffer;
use super::core::instance_data::create_instance_buffer;
use super::core::{CharInstanceData, InstanceBuffer, InstanceData, Mat4, Quaternion, Vec3, mesh_objects};
use super::types::Color;
use super::utils::get_model_matrix;
use super::vertex_buffer::VertexBuffer;

struct Axis {
    direction: Vec3<f32>,
    color: Color,
    label: String,
    label_color: Color,
}

impl Axis {
    fn new(direction: &Vec3<f32>, color: Color, label: String, label_color: Color) -> Self {
        Self {
            direction: direction.normalized(),
            color,
            label,
            label_color,
        }
    }

    fn get_axis_instance_data(
        &self,
        position: Vec3<f32>,
        length: f32,
        thickness: f32,
        both_directions: bool,
    ) -> InstanceData {
        let mut p = position;
        let mut l = length;
        if !both_directions {
            l = length / 2.0;
            p = position + self.direction * l;
        }

        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(p);
        transform.rotate(Quaternion::rotation_to(Vec3::new(0.0, 0.0, 1.0), self.direction));
        transform.scale(Vec3::new(thickness, thickness, l));

        InstanceData {
            model_matrix: get_model_matrix(&transform),
            color: self.color,
            picking_color: Color::new(0.0, 0.0, 0.0, 1.0),
            lighting_model: 1,
            ray_casting_type: 2,
        }
    }

    fn get_cone_instance_data(
        &self,
        position: Vec3<f32>,
        axis_length: f32,
        cone_length_factor: f32,
        cone_radius_factor: f32,
    ) -> InstanceData {
        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(position + self.direction * axis_length);
        transform.rotate(Quaternion::rotation_to(Vec3::new(0.0, 0.0, 1.0), self.direction));
        transform.scale(Vec3::new(cone_radius_factor, cone_radius_factor, cone_length_factor));

        InstanceData {
            model_matrix: get_model_matrix(&transform),
            color: self.color,
            picking_color: Color::new(0.0, 0.0, 0.0, 1.0),
            lighting_model: 1,
            ray_casting_type: 0,
        }
    }

    fn get_label_instance_data(
        &self,
        position: Vec3<f32>,
        size: f32,
        font_atlas: &super::core::FontAtlas,
    ) -> Vec<CharInstanceData> {
        let mut transform: Mat4<f32> = Mat4::new();

        transform.translate(position);
        transform.scale(Vec3::new(size, size, size));

        let mut data = Vec::new();

        let gap = 0.2;
        let mut total_width = 0.0;
        let mut chars_info = Vec::with_capacity(self.label.len());

        for c in self.label.chars() {
            let info = *font_atlas.get_char_info(c);
            let char_width = (info.width / info.height) * 2.0;
            chars_info.push((c, info, char_width));
            total_width += char_width + gap;
        }

        if total_width > 0.0 {
            total_width -= gap;
        }

        let mut current_left = -total_width / 2.0;

        for (_, info, char_width) in chars_info {
            data.push(CharInstanceData {
                model_matrix: get_model_matrix(&transform),
                uv_rect: [info.u_min, info.v_min, info.u_max, info.v_max],
                width: info.width / info.height,
                char_x_offset: current_left + char_width / 2.0,
                depth_offset: 0.0,
                color: self.label_color,
            });
            current_left += char_width + gap;
        }

        data
    }
}

pub struct CoordinateAxes {
    pub position: Vec3<f32>,
    pub use_origin: bool,
    pub visible: bool,
    pub length: f32,
    pub thickness: f32,
    pub both_directions: bool,
    pub cone_radius_factor: f32,
    pub cone_length_factor: f32,
    pub labels_size: f32,
    pub labels_visible: bool,

    pub label_x: String,
    pub label_y: String,
    pub label_z: String,
    pub color_x: Color,
    pub color_y: Color,
    pub color_z: Color,

    cube_vb: VertexBuffer,
    cone_vb: VertexBuffer,

    axes_instance_buffer: InstanceBuffer,
    cones_instance_buffer: InstanceBuffer,
    labels_instance_buffer: InstanceBuffer,

    dirty: bool,
}

impl CoordinateAxes {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            cube_vb: VertexBuffer::new(device, &mesh_objects::cube::create(2.0)),
            cone_vb: VertexBuffer::new(device, &mesh_objects::cone::create(1.0, 1.0, 30)),
            position: Vec3::new(0.0, 0.0, 0.0),
            use_origin: false,
            visible: false,
            length: 2.0,
            thickness: 0.03,
            both_directions: false,
            cone_radius_factor: 2.0,
            cone_length_factor: 6.0,
            labels_size: 0.16,
            labels_visible: true,
            label_x: String::from("X"),
            label_y: String::from("Y"),
            label_z: String::from("Z"),
            color_x: Color::new(1.0, 0.0, 0.0, 1.0),
            color_y: Color::new(0.0, 1.0, 0.0, 1.0),
            color_z: Color::new(0.0, 0.0, 1.0, 1.0),
            axes_instance_buffer: InstanceBuffer::new(
                create_instance_buffer(&Vec::new(), device, "Axes Instance Buffer"),
                0,
            ),
            cones_instance_buffer: InstanceBuffer::new(
                create_instance_buffer(&Vec::new(), device, "Cones Instance Buffer"),
                0,
            ),
            labels_instance_buffer: InstanceBuffer::new(
                create_char_instance_buffer(&Vec::new(), device, "Labels Instance Buffer"),
                0,
            ),
            dirty: true,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, font_atlas: &super::core::FontAtlas) {
        if !self.dirty {
            return;
        }

        let position = if self.use_origin {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            self.position
        };

        let axix_x = Axis {
            direction: Vec3::new(1.0, 0.0, 0.0),
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            label_color: Color::new(1.0, 0.0, 0.0, 1.0),
            label: self.label_x.clone(),
        };

        let axix_y = Axis {
            direction: Vec3::new(0.0, 1.0, 0.0),
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            label_color: Color::new(0.0, 1.0, 0.0, 1.0),
            label: self.label_y.clone(),
        };

        let axix_z = Axis {
            direction: Vec3::new(0.0, 0.0, 1.0),
            color: Color::new(0.0, 0.0, 1.0, 1.0),
            label_color: Color::new(0.0, 0.0, 1.0, 1.0),
            label: self.label_z.clone(),
        };

        let mut instance_data_axes = Vec::new();
        let mut instance_data_cones = Vec::new();
        let mut instance_data_labels = Vec::new();

        for axis in [&axix_x, &axix_y, &axix_z] {
            instance_data_axes.push(axis.get_axis_instance_data(
                position,
                self.length,
                self.thickness,
                self.both_directions,
            ));
            instance_data_cones.push(axis.get_cone_instance_data(
                position,
                self.length,
                self.cone_length_factor * self.thickness,
                self.cone_radius_factor * self.thickness,
            ));
            let pos = axis.direction * self.length + axis.direction * self.thickness * self.cone_length_factor * 2.0;
            instance_data_labels.append(&mut axis.get_label_instance_data(
                pos + position,
                self.labels_size,
                font_atlas,
            ));
        }

        let axes_wgpu_buffer = create_instance_buffer(&instance_data_axes, device, "Axes Instance Buffer");
        let cones_wgpu_buffer = create_instance_buffer(&instance_data_cones, device, "Cones Instance Buffer");
        let labels_wgpu_buffer = create_char_instance_buffer(&instance_data_labels, device, "Labels Instance Buffer");

        self.axes_instance_buffer = InstanceBuffer::new(axes_wgpu_buffer, instance_data_axes.len());
        self.cones_instance_buffer = InstanceBuffer::new(cones_wgpu_buffer, instance_data_cones.len());
        self.labels_instance_buffer = InstanceBuffer::new(labels_wgpu_buffer, instance_data_labels.len());

        self.dirty = false;
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    pub fn set_use_origin(&mut self, value: bool) {
        self.use_origin = value;
        self.dirty = true;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
        self.dirty = true;
    }

    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
        self.dirty = true;
    }

    pub fn set_both_directions(&mut self, both_directions: bool) {
        self.both_directions = both_directions;
        self.dirty = true;
    }

    pub fn set_cone_radius_factor(&mut self, cone_radius_factor: f32) {
        self.cone_radius_factor = cone_radius_factor;
        self.dirty = true;
    }

    pub fn set_cone_length_factor(&mut self, cone_length_factor: f32) {
        self.cone_length_factor = cone_length_factor;
        self.dirty = true;
    }

    pub fn set_labels_size(&mut self, labels_size: f32) {
        self.labels_size = labels_size;
        self.dirty = true;
    }

    pub fn set_labels_visible(&mut self, labels_visible: bool) {
        self.labels_visible = labels_visible;
        self.dirty = true;
    }

    pub fn render_axes(&self, render_pass: &mut wgpu::RenderPass) {
        if !self.visible {
            return;
        }

        // render axes
        render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, self.axes_instance_buffer.buffer.slice(..));
        render_pass.draw_indexed(
            0..self.cube_vb.num_indices,
            0,
            0..self.axes_instance_buffer.count as u32,
        );

        // render cones
        render_pass.set_vertex_buffer(0, self.cone_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cone_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_vertex_buffer(1, self.cones_instance_buffer.buffer.slice(..));
        render_pass.draw_indexed(
            0..self.cone_vb.num_indices,
            0,
            0..self.cones_instance_buffer.count as u32,
        );
    }

    pub fn render_labels(&self, render_pass: &mut wgpu::RenderPass, rect_vb: &VertexBuffer) {
        if !self.visible || !self.labels_visible || self.labels_instance_buffer.count == 0 {
            return;
        }

        render_pass.set_vertex_buffer(1, self.labels_instance_buffer.buffer.slice(..));
        render_pass.draw_indexed(0..rect_vb.num_indices, 0, 0..self.labels_instance_buffer.count as u32);
    }
}
