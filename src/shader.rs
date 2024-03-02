use crate::{gl_wrappers::Shader, rendering::BindingPoints};
use glfw::Version;
use std::{cell::RefCell, marker::PhantomData};

pub enum ShaderDataSource {
    VertexData,
    MatrixData,
    LightingData,
    FragColorOut,
    Custom(String),
    // VertexAttributes?
}

impl ShaderDataSource {
    const FRAG_COLOR_OUT: &'static str = "\nlayout (location = 0) out vec4 frag_color;";

    const VERTEX_DATA_IN: &'static str = "
in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    vec3 light_space_position;
} vertex;";

    thread_local! {
        static MATRIX_DATA: RefCell<String> =  RefCell::new(format!("
layout (std140, binding = {}) uniform MatrixData {{
    mat4 mvp;
    mat4 model;
    mat4 orientation;
    mat4 light_space;
}};", BindingPoints::MatrixData as u32)) ;

        static LIGHTING_DATA: RefCell<String> =  RefCell::new(format!("
struct LightSource {{
    vec3 color;
    int type; // 0 - Directional, 1 - Point, 2 - Spot
    vec3 position;
    float constant;
    vec3 direction;
    float linear;
    float quadratic;
    float inner_cutoff;
    float outer_cutoff;
}};
layout (std140, binding = {}) uniform LightingData {{
    LightSource light_source;
    vec3 viewer_position;
}};", BindingPoints::LightingData as u32));
    }

    pub fn source(&self) -> String {
        match self {
            ShaderDataSource::VertexData => Self::VERTEX_DATA_IN.to_string(),
            ShaderDataSource::MatrixData => Self::MATRIX_DATA.with(|x| x.borrow().clone()),
            ShaderDataSource::LightingData => Self::LIGHTING_DATA.with(|x| x.borrow().clone()),
            ShaderDataSource::FragColorOut => Self::FRAG_COLOR_OUT.to_string(),
            ShaderDataSource::Custom(src) => src.clone(),
        }
    }
}

#[repr(u32)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
}

pub trait ShaderSource {
    fn type_(&self) -> ShaderType;
    fn source(&self) -> String;
    fn data(&self) -> Vec<ShaderDataSource>;
}

pub trait SubShaderSource: ShaderSource {
    fn signature(&self) -> String;
    fn call_symbol(&self) -> String;
}

pub struct MainVertexShader;

pub struct MainFragmentShader;

pub struct MainShader<T> {
    signatures: Vec<String>,
    calls: Vec<String>,
    pd: PhantomData<T>,
}

impl<T> MainShader<T> {
    const MAIN_VERT: &'static str = "
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

out VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    vec3 light_space_position;
} vertex;

void main() { 
    vertex.position = (model * vec4(position, 1)).xyz;
    vertex.normal = (orientation * vec4(normal, 1)).xyz;
    vertex.tex_coord = tex_coord;
    vec4 light_space_position = (light_space * model * vec4(position, 1));
    vertex.light_space_position = light_space_position.xyz / light_space_position.w;
    gl_Position = mvp * vec4(position, 1);";

    const MAIN_FRAG: &'static str = "void main() {";

    pub fn new() -> Self {
        Self {
            signatures: Default::default(),
            calls: Default::default(),
            pd: PhantomData::<T> {},
        }
    }

    pub fn attach_shader(&mut self, shader_source: &impl SubShaderSource) {
        self.signatures.push(shader_source.signature().clone());
        self.calls.push(shader_source.call_symbol().clone());
    }

    fn build_source(&self, src: &str) -> String {
        let mut source = String::new();
        for signature in &self.signatures {
            source.push_str(&signature);
        }
        let mut calls = String::new();
        for call in &self.calls {
            calls.push_str(&call);
        }
        source.push_str(src);
        source.push_str(&format!("{calls}\n}}\n"));
        source
    }
}

impl ShaderSource for MainShader<MainVertexShader> {
    fn type_(&self) -> ShaderType {
        ShaderType::Vertex
    }

    fn source(&self) -> String {
        self.build_source(Self::MAIN_VERT)
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![ShaderDataSource::MatrixData]
    }
}

impl ShaderSource for MainShader<MainFragmentShader> {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        self.build_source(Self::MAIN_FRAG)
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![]
    }
}

pub struct DefaultLightShader;

impl DefaultLightShader {
    const LIGHT_SHADER_SRC: &'static str = "
#define DIRECTIONAL 0
#define POINT       1
#define SPOT        2

const float AMBIENT_INTENSITY = 0.01;
const float SHININESS = 1;

float blinn_specular(vec3 to_light_source_direction, float shininess) {
    vec3 to_viewer_direction = normalize(viewer_position - vertex.position);
    vec3 halfway_direction = normalize(to_light_source_direction + to_viewer_direction);
    return pow(max(dot(vertex.normal, halfway_direction), 0), shininess);
}

float attenuation() {
    float distance = length(light_source.position - vertex.position);
    return 1 / (light_source.constant + light_source.linear * distance
        + light_source.quadratic * distance * distance);
    // return 1 / distance;
}

float fragment_luminosity() {
    return 1;

    // vec3 fragment_ndc = vertex.light_space_position * 0.5 + 0.5;
    // float illuminated_depth = texture(shadow_map, fragment_ndc.xy).r;
    // return float(fragment_ndc.z <= illuminated_depth);
}

void directional() {
    float diffuse_intensity = clamp(dot(vertex.normal, -light_source.direction), 0, 1);
    float specular_intensity = blinn_specular(-light_source.direction, SHININESS);

    frag_color = vec4((AMBIENT_INTENSITY + fragment_luminosity()
        * (diffuse_intensity + specular_intensity)) * light_source.color, 1);
}

void point() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = clamp(dot(vertex.normal, to_light_source_direction), 0, 1);
    float specular_intensity = blinn_specular(to_light_source_direction, SHININESS);

    frag_color = vec4((AMBIENT_INTENSITY + fragment_luminosity()
        * (diffuse_intensity + specular_intensity)) * light_source.color * attenuation(), 1);
}

void spot() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = clamp(dot(vertex.normal, to_light_source_direction), 0, 1);
    float specular_intensity = blinn_specular(to_light_source_direction, SHININESS);
    float theta = dot(to_light_source_direction, -light_source.direction);
    float epsilon = light_source.inner_cutoff - light_source.outer_cutoff;
    float edge_intensity = clamp((theta - light_source.outer_cutoff) / epsilon, 0, 1);

    frag_color = vec4((AMBIENT_INTENSITY + edge_intensity * fragment_luminosity()
        * (diffuse_intensity + specular_intensity)) * light_source.color * attenuation(), 1);
}

void compute_lighting() {
    if (light_source.type == DIRECTIONAL) {
        directional();
    } else if (light_source.type == POINT) {
        point();
    } else if (light_source.type == SPOT) {
        spot();
    }
}\n";

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for DefaultLightShader {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        Self::LIGHT_SHADER_SRC.to_string()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![
            ShaderDataSource::LightingData,
            ShaderDataSource::VertexData,
            ShaderDataSource::FragColorOut,
        ]
    }
}

impl SubShaderSource for DefaultLightShader {
    fn signature(&self) -> String {
        "\nvoid compute_lighting();\n".to_string()
    }

    fn call_symbol(&self) -> String {
        "\ncompute_lighting();\n".to_string()
    }
}

pub struct ScreenShaderVert;

impl ScreenShaderVert {
    const SRC: &'static str = "
layout (location = 0) in vec3 position;
layout (location = 2) in vec2 tex_coord;

out VertexData {
    vec2 tex_coord;
} vertex_data;

void main() {
    vertex_data.tex_coord = tex_coord;
    gl_Position = vec4(position, 1.0);
}\n";

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for ScreenShaderVert {
    fn type_(&self) -> ShaderType {
        ShaderType::Vertex
    }

    fn source(&self) -> String {
        Self::SRC.to_string()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![]
    }
}

pub struct ScreenShaderFrag;

impl ScreenShaderFrag {
    pub const GAMMA_CORRECTION_LOCATION: i32 = 3;

    thread_local! {
        static SRC: RefCell<String> = RefCell::new(format!("
in VertexData {{
    vec2 tex_coord;
}} vertex_data;
uniform sampler2D screen_texture;
layout(location = {}) uniform float gamma_correction;
void main() {{
    vec4 color = texture(screen_texture, vertex_data.tex_coord);
    vec3 correction = pow(color.rgb, vec3(gamma_correction));
    frag_color = vec4(correction, color.a);
    // frag_color = vec4(0, 1, 0, 1);
}}\n", ScreenShaderFrag::GAMMA_CORRECTION_LOCATION));
    }

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for ScreenShaderFrag {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        Self::SRC.with(|x| x.borrow().clone())
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![ShaderDataSource::FragColorOut]
    }
}

pub fn build_shader(shader_source: &impl ShaderSource, context_version: Version) -> Shader {
    let shader = Shader::new(shader_source.type_() as u32).unwrap();
    let mut source = format!("#version {}{}0 core\n", context_version.major, context_version.minor);
    for data in shader_source.data() {
        source.push_str(&data.source());
    }
    source.push_str(&shader_source.source());
    println!("{source}");
    shader.set_source(&source);
    shader.compile();
    if !shader.compile_success() {
        println!("{}", shader.info_log());
        panic!("Shader compilation error")
    }

    shader
}
