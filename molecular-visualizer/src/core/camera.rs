use super::{Mat4, Vec3};

pub struct Camera {
    // Camera for managing 3D camera position, orientation and view matrix.
    // The camera uses a look-at approach where it maintains:
    // - position: camera location in world space
    // - target: point the camera is looking at
    // - up_vector: up direction for camera orientation
    position: Vec3<f32>,
    target: Vec3<f32>,
    up_vector: Vec3<f32>,
    matrix: Mat4<f32>,
    dirty: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up_vector: Vec3::new(0.0, 1.0, 0.0),
            matrix: Mat4::new(),
            dirty: true,
        }
    }

    fn update_matrix(&mut self) {
        // Update the view matrix based on current camera parameters.

        self.matrix.set_to_identity();
        self.matrix.look_at(self.position, self.target, self.up_vector);
        self.dirty = false;
    }

    pub fn get_matrix(&mut self) -> &Mat4<f32> {
        if self.dirty == true {
            self.update_matrix();
        }
        &self.matrix
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    pub fn reset_to_default(&mut self) {
        // Reset camera to default position and orientation.

        self.position = Vec3::new(0.0, 0.0, 1.0);
        self.target = Vec3::new(0.0, 0.0, 0.0);
        self.up_vector = Vec3::new(0.0, 1.0, 0.0);
        self.dirty = true;
    }
}
