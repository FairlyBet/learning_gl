use glm::{Mat4, Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub parent: Option<*const Transform>,
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
    #[allow(unused)]
    padding: [u8; 8],
}

impl Transform {
    pub fn new() -> Self {
        Self {
            parent: None,
            position: Vec3::zeros(),
            orientation: glm::quat_identity(),
            scale: Vec3::from_element(1.0),
            padding: Default::default(),
        }
    }

    pub fn global_position(&self) -> Vec3 {
        if let Some(parent_pos) = self.parent {
            unsafe { (*parent_pos).global_position() + self.position }
        } else {
            self.position
        }
    }

    pub fn model(&self) -> Mat4 {
        let identity = glm::identity();
        let translation = glm::translate(&identity, &self.global_position());
        let rotation = glm::quat_to_mat4(&self.orientation);
        let scale = glm::scale(&identity, &self.scale);

        translation * rotation * scale
    }

    pub fn set_orientation(&mut self, euler: &Vec3) {
        self.orientation = glm::quat_identity();
        self.rotate(euler);
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += *delta;
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let (local_right, local_upward, local_forward) = self.get_local_axes();
        self.position += local_right * delta.x + local_upward * delta.y + local_forward * delta.z;
    }

    pub fn rotate(&mut self, euler: &Vec3) {
        self.rotate_around_axes(euler, &(*Vec3::x_axis(), *Vec3::y_axis(), *Vec3::z_axis()));
    }

    pub fn rotate_local(&mut self, euler: &Vec3) {
        let local_axes = self.get_local_axes();
        self.rotate_around_axes(euler, &local_axes);
    }

    pub fn get_local_axes(&self) -> (Vec3, Vec3, Vec3) {
        let local_right = glm::quat_rotate_vec3(&self.orientation, &Vec3::x_axis());
        let local_upward = glm::quat_rotate_vec3(&self.orientation, &Vec3::y_axis());
        let local_forward = glm::quat_rotate_vec3(&self.orientation, &Vec3::z_axis());

        (local_right, local_upward, local_forward)
    }

    fn rotate_around_axes(&mut self, euler: &Vec3, axes: &(Vec3, Vec3, Vec3)) {
        let radians = glm::radians(euler);
        let identity = glm::quat_identity();
        let x_rotation = glm::quat_rotate_normalized_axis(&identity, radians.x, &axes.0);
        let y_rotation = glm::quat_rotate_normalized_axis(&identity, radians.y, &axes.1);
        let z_rotation = glm::quat_rotate_normalized_axis(&identity, radians.z, &axes.2);

        self.orientation = z_rotation * y_rotation * x_rotation * self.orientation;
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    },
    Perspective {
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self::Orthographic {
            left,
            right,
            bottom,
            top,
            znear,
            zfar,
        }
    }

    pub fn new_perspective(aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            aspect,
            fovy,
            near,
            far,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        match *self {
            Self::Orthographic {
                left,
                right,
                bottom,
                top,
                znear,
                zfar,
            } => glm::ortho(left, right, bottom, top, znear, zfar),
            Self::Perspective {
                aspect,
                fovy,
                near,
                far,
            } => glm::perspective(aspect, fovy.to_radians(), near, far),
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective {
            aspect: 1.0,
            fovy: 45.0,
            near: 0.01,
            far: 100.0,
        }
    }
}

pub fn view_matrix(transform: &Transform) -> Mat4 {
    let translation = glm::translation(&(-transform.position));
    let rotation = glm::inverse(&glm::quat_to_mat4(&transform.orientation));

    rotation * translation // applying quat rotation after translation makes object rotate
                           // around coordinate center and around themselves simultaneously
}

pub const FRUSTUM_CORNERS_COUNT: usize = 8;

pub fn frustum_corners_worldspace(projection: &Mat4) -> [Vec4; FRUSTUM_CORNERS_COUNT] {
    let inv = glm::inverse(&projection);
    let mut corners: [Vec4; FRUSTUM_CORNERS_COUNT] = Default::default();

    corners[0] = inv * Vec4::new(1.0, 1.0, 1.0, 1.0);
    corners[1] = inv * Vec4::new(1.0, -1.0, 1.0, 1.0);
    corners[2] = inv * Vec4::new(1.0, -1.0, -1.0, 1.0);
    corners[3] = inv * Vec4::new(1.0, 1.0, -1.0, 1.0);

    corners[4] = inv * Vec4::new(-1.0, 1.0, 1.0, 1.0);
    corners[5] = inv * Vec4::new(-1.0, -1.0, 1.0, 1.0);
    corners[6] = inv * Vec4::new(-1.0, -1.0, -1.0, 1.0);
    corners[7] = inv * Vec4::new(-1.0, 1.0, -1.0, 1.0);

    for i in 0..corners.len() {
        corners[i] = corners[i] / corners[i].w;
    }

    corners
}

pub fn frustum_center(corners: &[Vec4; FRUSTUM_CORNERS_COUNT]) -> Vec3 {
    let mut center: Vec3 = Default::default();
    for corner in corners {
        center += glm::vec4_to_vec3(&corner);
    }
    center /= corners.len() as f32;
    center
}
