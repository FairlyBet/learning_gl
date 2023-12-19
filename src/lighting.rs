use crate::linear::{self, Projection, Transform};
use nalgebra_glm::{Mat4, Vec3};
use std::default;

#[derive(Clone, Copy, Default)]
#[repr(i32)]
pub enum LightType {
    #[default]
    Directional = 0,
    Point = 1,
    Spot = 2,
}

#[derive(Default, Clone, Copy)]
#[repr(align(16))] // std140 requires align of 16 for user structs
#[repr(C)]
pub struct LightData {
    color: Vec3,
    type_: LightType,
    position: Vec3,
    constant: f32,
    direction: Vec3,
    linear: f32,
    quadratic: f32,
    inner_cutoff: f32,
    outer_cutoff: f32,
}

impl LightData {
    pub fn new_directional(color: Vec3) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.type_ = LightType::Directional;
        source
    }

    pub fn new_point(color: Vec3, constant: f32, linear: f32, quadratic: f32) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.constant = constant;
        source.linear = linear;
        source.quadratic = quadratic;
        source.type_ = LightType::Point;
        source
    }

    pub fn new_spot(
        color: Vec3,
        constant: f32,
        linear: f32,
        quadratic: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
    ) -> Self {
        let mut source: LightData = Default::default();
        source.color = color;
        source.constant = constant;
        source.linear = linear;
        source.quadratic = quadratic;
        source.inner_cutoff = inner_cutoff.to_radians().cos();
        source.outer_cutoff = outer_cutoff.to_radians().cos();
        source.type_ = LightType::Spot;
        source
    }
}

pub struct LightSource {
    light_data: LightData,
    projection: Mat4,
}

impl LightSource {
    const SHADOW_DISTANCE: f32 = 100.0;

    pub fn new(light_source: LightData) -> Self {
        Self {
            projection: Self::light_projection(&light_source),
            light_data: light_source,
        }
    }

    pub fn light_projection(data: &LightData) -> Mat4 {
        match data.type_ {
            LightType::Directional => {
                //     let projection = Projection::new_orthographic(
                //         -frustum_size,
                //         frustum_size,
                //         -frustum_size,
                //         frustum_size,
                //         0.0,
                //         frustum_size,
                //     );
                //     let mut view_obj = ViewObject::new(projection);
                // view_obj.transform.orientation =
                //     glm::quat_look_at(&direction, &Vec3::y_axis());
                // view_obj.transform.move_(&(-self.direction * frustum_size));
                // view_obj.transform.move_local(&Vec3::from_element(-frustum_size));
                // view_obj.transform.orientation = glm::
                todo!()
            }
            LightType::Point => {
                let projection =
                    Projection::new_perspective(1.0, 90.0, 0.01, Self::SHADOW_DISTANCE);
                projection.matrix();
                todo!()
            }
            LightType::Spot => {
                let projection = Projection::new_perspective(
                    1.0,
                    data.outer_cutoff.acos().to_degrees() * 2.0,
                    0.01,
                    Self::SHADOW_DISTANCE,
                );
                projection.matrix()
            }
        }
    }

    pub fn lightspace(&self, transform: &Transform) -> Mat4 {
        self.projection * linear::view_matrix(transform)
    }

    pub fn get_data(&self, transform: &Transform) -> LightData {
        let mut data = self.light_data;
        data.position = transform.position;
        data.direction = glm::quat_rotate_vec3(&transform.orientation, &(-Vec3::z_axis()));
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
