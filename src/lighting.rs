use crate::{
    linear::{self, Projection, Transform},
    utils::ArrayVec,
};
use nalgebra_glm::{vec3_to_vec4, Mat4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[repr(u32)]
pub enum LightType {
    #[default]
    Directional,
    Point,
    Spot,
}

#[derive(Default, Clone, Copy)]
#[repr(align(16))] // std140 requires align of 16 for user structs
#[repr(C)]
pub struct LightData {
    color: Vec3,
    type_: LightType,
    position: Vec3,
    cos_inner: f32,
    direction: Vec3,
    cos_outer: f32,
}

impl LightData {
    pub fn new_directional(color: Vec3) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.type_ = LightType::Directional;
        source
    }

    pub fn new_point(color: Vec3) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.type_ = LightType::Point;
        source
    }

    pub fn new_spot(color: Vec3, inner: f32, outer: f32) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.type_ = LightType::Spot;
        source.cos_inner = inner.to_radians().cos();
        source.cos_outer = outer.to_radians().cos();
        source
    }
}

pub struct LightSource {
    light_data: LightData,
    pub shadow_distance: f32,
    pub projections: ArrayVec<Mat4, 4>,
}

impl LightSource {
    const DEFAULT_SHADOW_DISTANCE: f32 = 100.0;

    pub fn new(light_data: LightData, shadow_distance: f32) -> Self {
        Self {
            projections: ArrayVec::new(),
            light_data,
            shadow_distance,
        }
    }

    pub fn get_data(&self, transform: &Transform) -> LightData {
        let mut data = self.light_data;
        data.position = transform.global_position();
        data.direction = glm::quat_rotate_vec3(&transform.orientation, &Vec3::z_axis());
        data
    }
}

// pub fn foo(camera: &Camera, light_obj: &LightSource) {
//     let corners = linear::frustum_corners_worldspace(&camera.projection_view());
//     let center = linear::frustum_center(&corners);
//     let mut tr = Transform::new();
//     tr.position = center;
//     tr.orientation = unsafe { (*light_obj.transform).orientation };
//     let light_view = linear::view_matrix(&tr); // might cause a bug
//                                                //or not
//     let mut minx = f32::MAX;
//     let mut maxx = f32::MIN;
//     let mut miny = f32::MAX;
//     let mut maxy = f32::MIN;
//     let mut minz = f32::MAX;
//     let mut maxz = f32::MIN;

//     for corner in &corners {
//         let corner = light_view * corner;
//         minx = minx.min(corner.x);
//         maxx = maxx.max(corner.x);
//         miny = miny.min(corner.y);
//         maxy = maxy.max(corner.y);
//         minz = minz.min(corner.z);
//         maxz = maxz.max(corner.z);
//     }
// }
