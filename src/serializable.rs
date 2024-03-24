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
    pub meshes: Vec<Mesh>,
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

#[derive(Serialize, Deserialize)]
pub struct Mesh {
    pub path: ResourcePath,
    pub material: Material,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Material {
    pub albedo: MateialInfo,
    pub metalness: MateialInfo,
    pub roughness: MateialInfo,
    pub normal: MateialInfo,
    pub ao: MateialInfo,
}

impl Material {
    pub fn iter(&self) -> impl IntoIterator<Item = &MateialInfo> {
        [
            &self.albedo,
            &self.metalness,
            &self.roughness,
            &self.normal,
            &self.ao,
        ]
        .into_iter()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub enum MateialInfo {
    #[default]
    None,
    Default,
    Custom(String),
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
    pub inner: f32,
    pub outer: f32,
    pub shadow_distance: f32,
}

impl Into<lighting::LightSource> for LightSource {
    fn into(self) -> lighting::LightSource {
        let light_data = match self.type_ {
            LightType::Directional => LightData::new_directional(self.color.into()),
            LightType::Point => LightData::new_point(self.color.into()),
            LightType::Spot => LightData::new_spot(self.color.into(), self.inner, self.outer),
        };
        lighting::LightSource::new(light_data, self.shadow_distance)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScriptObject {
    pub script_path: String,
}
