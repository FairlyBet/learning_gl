use crate::gl_wrappers::Texture;

pub struct Meteial {
    albedo: Texture,
    metalness: Texture,
    roughness: Texture,
    normal: Texture,
    ao: Texture,
}
