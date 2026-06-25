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
    pub visible: bool,
    pub length: f32,
    pub thickness: f32,
    pub both_directions: bool,
    pub cone_radius_factor: f32,
    pub cone_length_factor: f32,
    pub labels_size: f32,
    pub labels_visible: bool,

    cube_vb: VertexBuffer,
    cone_vb: VertexBuffer,
    axix_x: Axis,
    axix_y: Axis,
    axix_z: Axis,

    axes_instance_buffer: InstanceBuffer,
    cones_instance_buffer: InstanceBuffer,
    labels_instance_buffer: InstanceBuffer,
}

impl CoordinateAxes {
    pub fn new(device: &wgpu::Device, position: Vec3<f32>, visible: bool, font_atlas: &super::core::FontAtlas) -> Self {
        let length = 2.0;
        let thickness = 0.03;
        let both_directions = false;
        let cone_length_factor = 6.0;
        let cone_radius_factor = 2.0;
        let labels_size = 0.16;
        let labels_visible = true;

        let axix_x = Axis {
            direction: Vec3::new(1.0, 0.0, 0.0),
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            label_color: Color::new(1.0, 0.0, 0.0, 1.0),
            label: String::from("X"),
        };

        let axix_y = Axis {
            direction: Vec3::new(0.0, 1.0, 0.0),
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            label_color: Color::new(0.0, 1.0, 0.0, 1.0),
            label: String::from("Y"),
        };

        let axix_z = Axis {
            direction: Vec3::new(0.0, 0.0, 1.0),
            color: Color::new(0.0, 0.0, 1.0, 1.0),
            label_color: Color::new(0.0, 0.0, 1.0, 1.0),
            label: String::from("Z"),
        };

        let mut instance_data_axes = Vec::new();
        let mut instance_data_cones = Vec::new();
        let mut instance_data_labels = Vec::new();

        for axis in [&axix_x, &axix_y, &axix_z] {
            instance_data_axes.push(axis.get_axis_instance_data(position, length, thickness, both_directions));
            instance_data_cones.push(axis.get_cone_instance_data(
                position,
                length,
                cone_length_factor * thickness,
                cone_radius_factor * thickness,
            ));
            let pos = axis.direction * length + axis.direction * thickness * cone_length_factor * 2.0;
            instance_data_labels.append(&mut axis.get_label_instance_data(pos + position, labels_size, font_atlas));
        }

        let axes_instance_buffer = create_instance_buffer(&instance_data_axes, device, "Axes Instance Buffer");
        let cones_instance_buffer = create_instance_buffer(&instance_data_cones, device, "Cones Instance Buffer");
        let labels_instance_buffer =
            create_char_instance_buffer(&instance_data_labels, device, "Labels Instance Buffer");

        Self {
            cube_vb: VertexBuffer::new(device, &mesh_objects::cube::create(2.0)),
            cone_vb: VertexBuffer::new(device, &mesh_objects::cone::create(1.0, 1.0, 30)),
            position,
            visible,
            length,
            thickness,
            both_directions,
            cone_radius_factor,
            cone_length_factor,
            labels_size,
            labels_visible,
            axix_x,
            axix_y,
            axix_z,
            axes_instance_buffer: InstanceBuffer::new(axes_instance_buffer, instance_data_axes.len()),
            cones_instance_buffer: InstanceBuffer::new(cones_instance_buffer, instance_data_cones.len()),
            labels_instance_buffer: InstanceBuffer::new(labels_instance_buffer, instance_data_labels.len()),
        }
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.position = position;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }

    pub fn set_both_directions(&mut self, both_directions: bool) {
        self.both_directions = both_directions;
    }

    pub fn set_cone_radius_factor(&mut self, cone_radius_factor: f32) {
        self.cone_radius_factor = cone_radius_factor;
    }

    pub fn set_cone_length_factor(&mut self, cone_length_factor: f32) {
        self.cone_length_factor = cone_length_factor;
    }

    pub fn set_labels_size(&mut self, labels_size: f32) {
        self.labels_size = labels_size;
    }

    pub fn set_labels_visible(&mut self, labels_visible: bool) {
        self.labels_visible = labels_visible;
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
