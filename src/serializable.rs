use crate::{
    camera,
    lighting::{self, LightData, LightType},
    linear::{self, Projection},
    resources::ResourcePath,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub transform: Transform,
    pub children: Vec<Entity>,
    pub meshes: Vec<MeshData>,
    pub cameras: Vec<Camera>,
    pub light_sources: Vec<LightSource>,
    pub scripts: Vec<ScriptObject>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Vec3,
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

        result.position = self.position.into();
        result.scale = self.scale.into();
        result.set_orientation(&self.orientation.into());

        result
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Into<glm::Vec3> for Vec3 {
    fn into(self) -> glm::Vec3 {
        glm::vec3(self.x, self.y, self.z)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshData {
    pub path: ResourcePath,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct Camera {
    pub projection: Projection,
}

impl Into<camera::Camera> for Camera {
    fn into(self) -> camera::Camera {
        camera::Camera::new(self.projection)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct LightSource {
    pub color: Vec3,
    pub type_: LightType,
    pub position: Vec3,
    pub constant: f32,
    pub direction: Vec3,
    pub linear: f32,
    pub quadratic: f32,
    pub inner_cutoff: f32,
    pub outer_cutoff: f32,
    pub shadow_distance: f32,
}

impl Into<lighting::LightSource> for LightSource {
    fn into(self) -> lighting::LightSource {
        let light_data = match self.type_ {
            LightType::Directional => LightData::new_directional(self.color.into()),
            LightType::Point => LightData::new_point(
                self.color.into(),
                self.constant,
                self.linear,
                self.quadratic,
            ),
            LightType::Spot => LightData::new_spot(
                self.color.into(),
                self.constant,
                self.linear,
                self.quadratic,
                self.inner_cutoff,
                self.outer_cutoff,
            ),
        };
        lighting::LightSource::new(light_data, self.shadow_distance)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScriptObject {
    pub script_path: String,
}
