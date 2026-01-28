use super::math::{Mat4, Quaternion, Vec3};

fn normalize_angle(mut angle: f32) -> f32 {
    // Normalize angle to range [-180, 180].
    // Examples:
    //     185.0 -> -175.0
    //    -185.0 -> 175.0
    //     370.0 -> 10.0
    //    -370.0 -> -10.0

    if angle < -180.0 {
        angle += (angle / -180.0) * 180.0;
    } else if angle > 180.0 {
        angle -= (angle / 180.0) * 180.0;
    }
    angle
}

pub struct Transform {
    pub position: Vec3<f32>,
    pub scale: Vec3<f32>,
    pub rotation: Quaternion<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    matrix: Mat4<f32>,
    dirty: bool,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            matrix: Mat4::new(),
            dirty: true,
        }
    }

    fn update_matrix(&mut self) {
        self.matrix.set_to_identity();
        self.matrix.translate(self.position);
        self.matrix.rotate(self.rotation);
        self.matrix.scale(self.scale);
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

    pub fn scale(&mut self, factor: Vec3<f32>) {
        self.scale *= factor;
        self.dirty = true;
    }

    pub fn set_scale(&mut self, value: Vec3<f32>) {
        self.scale = value;
        self.dirty = true;
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.pitch = normalize_angle(self.pitch + pitch);
        self.yaw = normalize_angle(self.yaw + yaw);
        self.roll = normalize_angle(self.roll + roll);

        let pitch_quat = Quaternion::from_axis_and_angle(Vec3::new(1.0, 0.0, 0.0), pitch);
        let yaw_quat = Quaternion::from_axis_and_angle(Vec3::new(0.0, 1.0, 0.0), yaw);
        let roll_quat = Quaternion::from_axis_and_angle(Vec3::new(0.0, 0.0, 1.0), roll);

        self.rotation = pitch_quat * yaw_quat * roll_quat * self.rotation;
        self.dirty = true
    }
}
