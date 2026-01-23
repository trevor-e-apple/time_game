use cgmath::{Matrix3, Matrix4, Point3, SquareMatrix, Vector3, perspective};

// TODO: Is this obviously a POD type? If not, we might want to add a "new" method.
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

const OPEN_GL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPEN_GL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Matrix4::identity().into(),
        }
    }

    pub fn with_camera(camera: &Camera) -> Self {
        let mut uniform = Self::new();
        uniform.update_view_projection(camera);
        uniform
    }

    // TODO: can we just fold this into with_camera?
    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera2DUniform {
    projection: [[f32; 4]; 4],
}

impl Camera2DUniform {
    pub fn new(width: f32, height: f32) -> Self {
        let mat4 = Matrix4::new(
            2.0 / width,
            0.0,
            0.0,
            0.0,
            0.0,
            2.0 / height,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0,
            -1.0,
            0.0,
            1.0,
        );

        Self {
            projection: mat4.into(),
        }
    }
}
