use nalgebra_glm::{Mat4, Vec3};
use num_derive::FromPrimitive;

use crate::linear::{to_rad, Projection, ViewObject};

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
    pub fn new_directional(color: Vec3, direction: Vec3) -> Self {
        let mut source: LightSource = Default::default();
        source.color = color;
        source.direction = direction;
        source.type_ = LightType::Directional as i32;
        source
    }

    pub fn new_point(
        color: Vec3,
        position: Vec3,
        constant: f32,
        linear: f32,
        quadratic: f32,
    ) -> Self {
        let mut source: LightSource = Default::default();
        source.color = color;
        source.position = position;
        source.constant = constant;
        source.linear = linear;
        source.quadratic = quadratic;
        source.type_ = LightType::Point as i32;
        source
    }

    pub fn new_spot(
        color: Vec3,
        position: Vec3,
        direction: Vec3,
        constant: f32,
        linear: f32,
        quadratic: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
    ) -> Self {
        Self {
            color,
            position,
            direction,
            constant,
            linear,
            quadratic,
            inner_cutoff: to_rad(inner_cutoff).cos(),
            outer_cutoff: to_rad(outer_cutoff).cos(),
            padding: 0,
            type_: LightType::Spot as i32,
        }
    }

    pub fn light_space_matrix(&self, frustum_size: f32) -> Mat4 {
        let type_ = num::FromPrimitive::from_i32(self.type_).unwrap();
        match type_ {
            LightType::Directional => {
                let projection = Projection::new_orthographic(
                    -frustum_size,
                    frustum_size,
                    -frustum_size,
                    frustum_size,
                    0.0,
                    frustum_size,
                );
                let mut view_obj = ViewObject::new(projection);
                view_obj.transform.orientation =
                    glm::quat_look_at(&self.direction, &Vec3::y_axis());
                view_obj.transform.move_(&(-self.direction * frustum_size));
                // view_obj.transform.move_local(&Vec3::from_element(-frustum_size));
                // view_obj.transform.orientation = glm::
            }
            LightType::Point => {}
            _ => {
                todo!("TODO Point light")
            }
        }
        todo!()
    }
}
