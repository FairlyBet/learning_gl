use crate::gl_wrappers::Texture;

#[rustfmt::skip]
pub struct Material {
    pub base_color:     usize,
    pub metalness:      usize,
    pub roughness:      usize,
    pub ao:             usize,
    pub normals:        usize,
    pub displacement:   usize,
}
