use super::super::mesh::{Mesh, Vertex};

pub fn create(edge_length: f32) -> Mesh {
    // Generate cube vertices centered at origin.
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half_edge = edge_length / 2.0;

    let tex_coord = [0.0, 0.0];

    // FRONT FACE (Z+)
    let normal_front = [0.0, 0.0, 1.0];
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, half_edge],
        normal: normal_front,
        tex_coord,
    }); // 0: left bottom
    vertices.push(Vertex {
        position: [half_edge, -half_edge, half_edge],
        normal: normal_front,
        tex_coord,
    }); // 1: right bottom
    vertices.push(Vertex {
        position: [half_edge, half_edge, half_edge],
        normal: normal_front,
        tex_coord,
    }); // 2: right top
    vertices.push(Vertex {
        position: [-half_edge, half_edge, half_edge],
        normal: normal_front,
        tex_coord,
    }); // 3: left top
    indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

    // RIGHT FACE (X+)
    let normal_right = [1.0, 0.0, 0.0];
    vertices.push(Vertex {
        position: [half_edge, -half_edge, half_edge],
        normal: normal_right,
        tex_coord,
    }); // 4
    vertices.push(Vertex {
        position: [half_edge, -half_edge, -half_edge],
        normal: normal_right,
        tex_coord,
    }); // 5
    vertices.push(Vertex {
        position: [half_edge, half_edge, -half_edge],
        normal: normal_right,
        tex_coord,
    }); // 6
    vertices.push(Vertex {
        position: [half_edge, half_edge, half_edge],
        normal: normal_right,
        tex_coord,
    }); // 7
    indices.extend_from_slice(&[4, 5, 6, 4, 6, 7]);

    // BACK FACE (Z-)
    let normal_back = [0.0, 0.0, -1.0];
    vertices.push(Vertex {
        position: [half_edge, -half_edge, -half_edge],
        normal: normal_back,
        tex_coord,
    }); // 8: right bottom
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, -half_edge],
        normal: normal_back,
        tex_coord,
    }); // 9: left bottom
    vertices.push(Vertex {
        position: [-half_edge, half_edge, -half_edge],
        normal: normal_back,
        tex_coord,
    }); // 10: left top
    vertices.push(Vertex {
        position: [half_edge, half_edge, -half_edge],
        normal: normal_back,
        tex_coord,
    }); // 11: right top
    indices.extend_from_slice(&[8, 9, 10, 8, 10, 11]);

    // LEFT FACE (X-)
    let normal_left = [-1.0, 0.0, 0.0];
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, -half_edge],
        normal: normal_left,
        tex_coord,
    }); // 12
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, half_edge],
        normal: normal_left,
        tex_coord,
    }); // 13
    vertices.push(Vertex {
        position: [-half_edge, half_edge, half_edge],
        normal: normal_left,
        tex_coord,
    }); // 14
    vertices.push(Vertex {
        position: [-half_edge, half_edge, -half_edge],
        normal: normal_left,
        tex_coord,
    }); // 15
    indices.extend_from_slice(&[12, 13, 14, 12, 14, 15]);

    // TOP FACE (Y+)
    let normal_top = [0.0, 1.0, 0.0];
    vertices.push(Vertex {
        position: [-half_edge, half_edge, half_edge],
        normal: normal_top,
        tex_coord,
    }); // 16: left front
    vertices.push(Vertex {
        position: [half_edge, half_edge, half_edge],
        normal: normal_top,
        tex_coord,
    }); // 17: right front
    vertices.push(Vertex {
        position: [half_edge, half_edge, -half_edge],
        normal: normal_top,
        tex_coord,
    }); // 18: right back
    vertices.push(Vertex {
        position: [-half_edge, half_edge, -half_edge],
        normal: normal_top,
        tex_coord,
    }); // 19: left back
    indices.extend_from_slice(&[16, 17, 18, 16, 18, 19]);

    // BOTTOM FACE (Y-)
    let normal_bottom = [0.0, -1.0, 0.0];
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, -half_edge],
        normal: normal_bottom,
        tex_coord,
    }); // 20: left back
    vertices.push(Vertex {
        position: [half_edge, -half_edge, -half_edge],
        normal: normal_bottom,
        tex_coord,
    }); // 21: right back
    vertices.push(Vertex {
        position: [half_edge, -half_edge, half_edge],
        normal: normal_bottom,
        tex_coord,
    }); // 22: right front
    vertices.push(Vertex {
        position: [-half_edge, -half_edge, half_edge],
        normal: normal_bottom,
        tex_coord,
    }); // 23: left front
    indices.extend_from_slice(&[20, 21, 22, 20, 22, 23]);

    let num_indices = indices.len() as u32;

    Mesh {
        vertices,
        indices,
        num_indices,
    }
}
