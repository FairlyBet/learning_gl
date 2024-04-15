use crate::gl_wrappers::Texture;

#[rustfmt::skip]
pub struct Material<'a> {
    pub base_color:     &'a Texture,
    pub metalness:      &'a Texture,
    pub roughness:      &'a Texture,
    pub ao:             &'a Texture,
    pub normal:         &'a Texture,
    pub displacement:   &'a Texture,
}
