use crate::{
    camera::Camera,
    linear::{self, Projection, Transform},
};
use nalgebra_glm::{Mat4, Vec3};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Clone, Copy, FromPrimitive)]
pub enum LightType {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

impl Default for LightType {
    fn default() -> Self {
        LightType::Directional
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
// std140 requires structs to be x16 sized
pub struct LightSource {
    color: Vec3,
    type_: i32,
    position: Vec3,
    constant: f32,
    direction: Vec3,
    linear: f32,
    quadratic: f32,
    inner_cutoff: f32,
    outer_cutoff: f32,
    padding: i32,
}

impl LightSource {
    pub fn new_directional(color: Vec3) -> Self {
        let mut source: LightSource = Default::default();
        source.color = color;
        source.type_ = LightType::Directional as i32;
        source
    }

    pub fn new_point(color: Vec3, constant: f32, linear: f32, quadratic: f32) -> Self {
        let mut source: LightSource = Default::default();
        source.color = color;
        source.constant = constant;
        source.linear = linear;
        source.quadratic = quadratic;
        source.type_ = LightType::Point as i32;
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
        let mut source: LightSource = Default::default();
        source.color = color;
        source.constant = constant;
        source.linear = linear;
        source.quadratic = quadratic;
        source.inner_cutoff = inner_cutoff.to_radians().cos();
        source.outer_cutoff = outer_cutoff.to_radians().cos();
        source.type_ = LightType::Spot as i32;
        source
    }
}

pub struct LightObject {
    pub transform: Transform,
    light_source: LightSource,
    projection: Mat4,
}

impl LightObject {
    const SHADOW_DISTANCE: f32 = 100.0;

    pub fn new(light_source: LightSource) -> Self {
        Self {
            transform: Transform::new(),
            projection: Self::light_projection(&light_source),
            light_source,
        }
    }

    pub fn light_projection(source: &LightSource) -> Mat4 {
        let type_ = FromPrimitive::from_i32(source.type_).unwrap();
        match type_ {
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
                projection.matrix()
            }
            LightType::Spot => {
                let projection = Projection::new_perspective(
                    1.0,
                    source.outer_cutoff.acos().to_degrees() * 2.0,
                    0.01,
                    Self::SHADOW_DISTANCE,
                );
                projection.matrix()
            }
        }
    }

    pub fn lightspace(&self) -> Mat4 {
        self.projection * linear::view_matrix(&self.transform)
    }

    pub fn get_source(&mut self) -> LightSource {
        self.light_source.position = self.transform.position;
        self.light_source.direction =
            glm::quat_rotate_vec3(&self.transform.orientation, &(-Vec3::z_axis()));

        self.light_source
    }
}

pub fn foo(camera: &Camera, light_obj: &LightObject) {
    let corners = linear::frustum_corners_worldspace(&camera.projection_view());
    let center = linear::frustum_center(&corners);
    let mut tr = Transform::new();
    tr.position = center;
    tr.orientation = light_obj.transform.orientation;
    let light_view = linear::view_matrix(&tr); // might cause a bug
                                               //or not
    let mut minx = f32::MAX;
    let mut maxx = f32::MIN;
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;
    let mut minz = f32::MAX;
    let mut maxz = f32::MIN;

    for corner in &corners {
        let corner = light_view * corner;
        minx = minx.min(corner.x);
        maxx = maxx.max(corner.x);
        miny = miny.min(corner.y);
        maxy = maxy.max(corner.y);
        minz = minz.min(corner.z);
        maxz = maxz.max(corner.z);
    }
}
