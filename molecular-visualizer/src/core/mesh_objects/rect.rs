use super::super::mesh::{Mesh, Vertex};

pub fn create(left: f32, right: f32, bottom: f32, top: f32) -> Mesh {
    let normal = [0.0, 0.0, 1.0];

    let tex_coord = [0.0, 0.0];

    let vertices = vec![
        Vertex {
            position: [left, bottom, 0.0],
            normal,
            tex_coord,
        }, // left bottom
        Vertex {
            position: [right, bottom, 0.0],
            normal,
            tex_coord,
        }, // right bottom
        Vertex {
            position: [right, top, 0.0],
            normal,
            tex_coord,
        }, // right top
        Vertex {
            position: [left, top, 0.0],
            normal,
            tex_coord,
        }, // left top
    ];

    Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        num_indices: 6,
    }
}
