use crate::{
    camera,
    entity_system::{self, EntityId},
    lighting::{LightData, LightSource, LightType},
    linear::{self, Projection},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub transfom: Transform,
    pub children: Vec<Entity>,
    pub mesh_components: Vec<MeshComponent>,
    pub camera_components: Vec<CameraComponent>,
    pub light_components: Vec<LightComponent>,
    pub script_components: Vec<Script>
}

#[derive(Serialize, Deserialize)]
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
        result.set_rotation(&self.orientation.into());

        result
    }
}

#[derive(Serialize, Deserialize, Default)]
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
pub struct MeshComponent {
    pub owner_id: EntityId,
    pub mesh_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub projection: Projection,
    pub owner_id: EntityId,
}

impl Into<entity_system::CameraComponent> for CameraComponent {
    fn into(self) -> entity_system::CameraComponent {
        entity_system::CameraComponent {
            camera: camera::Camera::new(self.projection),
            owner_id: self.owner_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LightComponent {
    pub color: Vec3,
    pub type_: LightType,
    pub position: Vec3,
    pub constant: f32,
    pub direction: Vec3,
    pub linear: f32,
    pub quadratic: f32,
    pub inner_cutoff: f32,
    pub outer_cutoff: f32,
    pub owner_id: EntityId,
}

impl Into<entity_system::LightComponent> for LightComponent {
    fn into(self) -> entity_system::LightComponent {
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
        entity_system::LightComponent {
            light_source: LightSource::new(light_data),
            owner_id: self.owner_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub script_path: String
}
