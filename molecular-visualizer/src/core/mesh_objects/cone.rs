use std::f32::consts::PI;

use super::super::mesh::{Mesh, Vertex};

pub fn create(radius: f32, height: f32, slices: u16) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let slices = slices as u32;

    // --- Base (cap) ---
    let base_center_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, -1.0],
        tex_coord: [0.5, 0.5],
    });

    for i in 0..slices {
        let theta = (i as f32) * 2.0 * PI / (slices as f32);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        vertices.push(Vertex {
            position: [radius * cos_t, radius * sin_t, 0.0],
            normal: [0.0, 0.0, -1.0],
            tex_coord: [cos_t * 0.5 + 0.5, sin_t * 0.5 + 0.5],
        });
    }

    for i in 0..slices {
        let next_i = (i + 1) % slices;
        indices.extend_from_slice(&[base_center_idx, base_center_idx + 1 + next_i, base_center_idx + 1 + i]);
    }

    // --- Cone Surface ---
    let normal_z = radius;
    let normal_len = (height * height + radius * radius).sqrt();
    let nz = normal_z / normal_len;
    let n_xy_scale = height / normal_len;

    let surface_base_idx = vertices.len() as u32;
    for i in 0..slices {
        let theta = (i as f32) * 2.0 * PI / (slices as f32);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        vertices.push(Vertex {
            position: [radius * cos_t, radius * sin_t, 0.0],
            normal: [cos_t * n_xy_scale, sin_t * n_xy_scale, nz],
            tex_coord: [i as f32 / slices as f32, 0.0],
        });
    }

    let tip_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [0.0, 0.0, height],
        normal: [0.0, 0.0, 1.0],
        tex_coord: [0.5, 1.0],
    });

    for i in 0..slices {
        let next_i = (i + 1) % slices;
        indices.extend_from_slice(&[surface_base_idx + i, surface_base_idx + next_i, tip_idx]);
    }

    let num_indices = indices.len() as u32;

    Mesh {
        vertices,
        indices,
        num_indices,
    }
}
