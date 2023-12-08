use crate::{
    camera,
    entity_sys::{CameraComponent, EntityId},
    linear::{self, Projection},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            orientation: Default::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

impl Into<linear::Transform> for Transform {
    fn into(self) -> linear::Transform {
        let mut result = linear::Transform::new();

        result.position.x = self.position.x;
        result.position.y = self.position.y;
        result.position.z = self.position.z;

        result.orientation.coords.x = self.orientation.x;
        result.orientation.coords.y = self.orientation.y;
        result.orientation.coords.z = self.orientation.z;
        result.orientation.coords.w = self.orientation.w;

        result.scale.x = self.scale.x;
        result.scale.y = self.scale.y;
        result.scale.z = self.scale.z;

        result
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub owner_id: EntityId,
    pub mesh_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub projection: Projection,
    pub owner_id: EntityId,
}

impl Into<CameraComponent> for Camera {
    fn into(self) -> CameraComponent {
        CameraComponent {
            camera: camera::Camera::new(self.projection),
            owner_id: self.owner_id,
        }
    }
}
