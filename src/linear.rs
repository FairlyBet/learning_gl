use glm::Vec3;
use nalgebra_glm::{Mat4, Quat, Vec4};

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
    pub parent: Option<*const Transform>,
    padding: u64,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Vec3::zeros(),
            orientation: glm::quat_identity(),
            scale: Vec3::from_element(1.0),
            parent: None,
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
        let tranlation = glm::translate(&identity, &self.global_position());
        let rotation = glm::quat_to_mat4(&self.orientation);
        let scale = glm::scale(&identity, &self.scale);

        tranlation * rotation * scale
    }

    pub fn set_rotation(&mut self, euler: &Vec3) {
        self.orientation = glm::quat_identity();
        self.rotate(euler);
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += *delta;
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let (local_right, local_upward, local_forward) = self.get_local_axises();
        self.position += local_right * delta.x + local_upward * delta.y + local_forward * delta.z;
    }

    pub fn rotate(&mut self, euler: &Vec3) {
        self.rotate_around_axises(euler, &(*Vec3::x_axis(), *Vec3::y_axis(), *Vec3::z_axis()));
    }

    pub fn rotate_local(&mut self, euler: &Vec3) {
        let local_axises = self.get_local_axises();
        self.rotate_around_axises(euler, &local_axises);
    }

    pub fn get_local_axises(&self) -> (Vec3, Vec3, Vec3) {
        let local_right = glm::quat_rotate_vec3(&self.orientation, &Vec3::x_axis());
        let local_upward = glm::quat_rotate_vec3(&self.orientation, &Vec3::y_axis());
        let local_forward = glm::quat_rotate_vec3(&self.orientation, &Vec3::z_axis());

        (local_right, local_upward, local_forward)
    }

    fn rotate_around_axises(&mut self, euler: &Vec3, axises: &(Vec3, Vec3, Vec3)) {
        let radians = glm::radians(euler);
        let identity = glm::quat_identity();
        let x_rotation = glm::quat_rotate_normalized_axis(&identity, radians.x, &axises.0);
        let y_rotation = glm::quat_rotate_normalized_axis(&identity, radians.y, &axises.1);
        let z_rotation = glm::quat_rotate_normalized_axis(&identity, radians.z, &axises.2);

        self.orientation = z_rotation * y_rotation * x_rotation * self.orientation;
    }
}

#[derive(Clone, Copy)]
pub enum Projection {
    Orthographic(f32, f32, f32, f32, f32, f32),
    Perspective(f32, f32, f32, f32),
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
        Projection::Orthographic(left, right, bottom, top, znear, zfar)
    }

    pub fn new_perspective(aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        Projection::Perspective(aspect, fovy, near, far)
    }

    pub fn matrix(&self) -> Mat4 {
        match *self {
            Projection::Orthographic(left, right, bottom, top, znear, zfar) => {
                glm::ortho(left, right, bottom, top, znear, zfar)
            }
            Projection::Perspective(aspect, fovy, near, far) => {
                glm::perspective(aspect, fovy.to_radians(), near, far)
            }
        }
    }
}

pub fn view_matrix(transform: &Transform) -> Mat4 {
    let translation = glm::translation(&(-transform.position));
    let rotation = glm::inverse(&glm::quat_to_mat4(&transform.orientation));

    rotation * translation // applying quat rotation after translation makes object rotate
                           // around coordinate center and around themselves simultaneoulsy
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
