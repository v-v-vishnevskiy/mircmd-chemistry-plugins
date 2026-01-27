use super::matrix::Mat4;

pub struct OrthographicProjection {
    width: u32,
    height: u32,
    view_bounds: f32,
    depth_factor: f32,
    matrix: Mat4<f32>,
}

impl OrthographicProjection {
    fn new(width: u32, height: u32, view_bounds: f32, depth_factor: f32) -> Self {
        let mut proj = Self {
            width,
            height,
            view_bounds,
            depth_factor,
            matrix: Mat4::new(),
        };
        proj.set_viewport(width, height);
        proj
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
        let w = width as f32;
        let h = height as f32;
        let left: f32;
        let right: f32;
        let bottom: f32;
        let top: f32;

        if width <= height {
            // Portrait or square viewport
            left = -self.view_bounds;
            right = self.view_bounds;
            bottom = -self.view_bounds * (h / w);
            top = self.view_bounds * (h / w);
        } else {
            // Landscape viewport
            left = -self.view_bounds * (w / h);
            right = self.view_bounds * (w / h);
            bottom = -self.view_bounds;
            top = self.view_bounds;
        }
        let depth_range = self.view_bounds * self.depth_factor;
        let near = -depth_range;
        let far = depth_range;

        self.matrix.set_to_identity();
        self.matrix.ortho(left, right, bottom, top, near, far);
        self.width = width;
        self.height = height;
    }
}

pub struct PerspectiveProjection {
    fov: f32,
    width: u32,
    height: u32,
    near_plane: f32,
    far_plane: f32,
    matrix: Mat4<f32>,
}

impl PerspectiveProjection {
    fn new(fov: f32, width: u32, height: u32, near_plane: f32, far_plane: f32) -> Self {
        let mut proj = Self {
            fov,
            width,
            height,
            near_plane,
            far_plane,
            matrix: Mat4::new(),
        };
        proj.set_viewport(width, height);
        proj
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
        let aspect = width as f32 / height as f32;

        // Calculate effective FOV and frustum planes based on orientation
        let half_fov_rad = (self.fov / 2.0).to_radians();
        let tan_half_fov = half_fov_rad.tan();

        let fov: f32;

        if width <= height {
            // Portrait or square viewport: FOV applies to horizontal axis (narrower)
            // Calculate vertical FOV from horizontal FOV to maintain proper scaling
            fov = (2.0 * (tan_half_fov / aspect).atan()).to_degrees();
        } else {
            // Landscape viewport: FOV applies to vertical axis (standard behavior)
            fov = self.fov;
        }

        self.matrix.set_to_identity();
        self.matrix.perspective(fov, aspect, self.near_plane, self.far_plane);
        self.width = width;
        self.height = height;
    }
}

#[derive(PartialEq)]
pub enum ProjectionMode {
    Orthographic,
    Perspective,
}

pub struct ProjectionManager {
    pub mode: ProjectionMode,
    orthographic_projection: OrthographicProjection,
    perspective_projection: PerspectiveProjection,
}

impl ProjectionManager {
    pub fn new(width: u32, height: u32, mode: ProjectionMode) -> Self {
        Self {
            mode,
            orthographic_projection: OrthographicProjection::new(width, height, 10.0, 10.0),
            perspective_projection: PerspectiveProjection::new(45.0, width, height, 0.1, 1000.0),
        }
    }

    pub fn toggle_projection_mode(&mut self) {
        if self.mode == ProjectionMode::Orthographic {
            self.mode = ProjectionMode::Perspective
        } else {
            self.mode = ProjectionMode::Orthographic
        }
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.orthographic_projection.set_viewport(width, height);
        self.perspective_projection.set_viewport(width, height);
    }

    pub fn matrix(&self) -> &Mat4<f32> {
        if self.mode == ProjectionMode::Orthographic {
            &self.orthographic_projection.matrix
        } else {
            &self.perspective_projection.matrix
        }
    }
}
