use crate::linear;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn into_actual(&self) -> linear::Transform {
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

#[derive(Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
