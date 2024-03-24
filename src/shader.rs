use crate::{
    gl_wrappers::Shader,
    rendering::{self, BindingPoints},
};
use gl::types::GLenum;
use glfw::Version;
use std::{cell::RefCell, marker::PhantomData};

pub enum ShaderDataSource {
    FragmentDataIn,
    FragmentDataOut,
    MatrixData,
    LightingData,
    FragColorOut,
    VertexAttributes,
    Custom(String),
}

impl ShaderDataSource {
    fn frag_color_out() -> String {
        "
layout (location = 0) out vec4 frag_color;
"
        .to_string()
    }

    fn fragment_data_in() -> String {
        "
in FragmentData {
    vec3 pos;
    vec3 normal;
    vec2 tex_coord;
    vec3 lightspace_pos;
} fragment;
"
        .to_string()
    }

    fn fragment_data_out() -> String {
        "
out FragmentData {
    vec3 pos;
    vec3 normal;
    vec2 tex_coord;
    vec3 lightspace_pos;
} fragment;
"
        .to_string()
    }

    fn matrix_data() -> String {
        format!(
            "
layout (std140, binding = {}) uniform MatrixData {{
    mat4 mvp;
    mat4 model;
    mat4 orientation;
    mat4 light_space;
}};
",
            BindingPoints::MatrixData as u32
        )
    }

    fn lighting_data() -> String {
        format!(
            "
struct LightSource {{
    vec3 color;
    uint type;
    vec3 pos;
    float inner_cutoff;
    vec3 dir;
    float outer_cutoff;
}};

layout(std140, binding = {}) uniform LightingData {{
    LightSource sources[{}];
    vec3 viewer_pos;
    uint source_count;
}};
",
            BindingPoints::LightingData as u32,
            rendering::MAX_SOURCES_PER_FRAME
        )
    }

    fn vertex_attributes() -> String {
        "
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;
"
        .to_string()
    }

    pub fn source(&self) -> String {
        match self {
            ShaderDataSource::FragColorOut => Self::frag_color_out(),
            ShaderDataSource::FragmentDataIn => Self::fragment_data_in(),
            ShaderDataSource::FragmentDataOut => Self::fragment_data_out(),
            ShaderDataSource::MatrixData => Self::matrix_data(),
            ShaderDataSource::LightingData => Self::lighting_data(),
            ShaderDataSource::VertexAttributes => Self::vertex_attributes(),
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

pub struct VertShader;

pub struct FragShader;

pub struct MainShader<T> {
    signatures: Vec<String>,
    call_symbols: Vec<String>,
    _p: PhantomData<T>,
}

impl<T> MainShader<T> {
    fn vert_src() -> &'static str {
        "
void main() {
    fragment.pos = (model * vec4(position, 1.0)).xyz;
    fragment.normal = (orientation * vec4(normal, 1.0)).xyz;
    fragment.tex_coord = tex_coord;
    vec4 lightspace_pos = (light_space * model * vec4(position, 1.0));
    fragment.lightspace_pos = lightspace_pos.xyz / lightspace_pos.w;
    gl_Position = mvp * vec4(position, 1.0);
"
    }

    fn frag_src() -> &'static str {
        "
void main() {"
    }

    pub fn new() -> Self {
        Self {
            signatures: Default::default(),
            call_symbols: Default::default(),
            _p: PhantomData::<T> {},
        }
    }

    pub fn attach_shader(&mut self, shader_source: &impl SubShaderSource) {
        self.signatures.push(shader_source.signature().clone());
        self.call_symbols.push(shader_source.call_symbol().clone());
    }

    fn build_source(&self, src: &str) -> String {
        let mut source = String::new();
        source.extend(self.signatures.iter().map(|item| item.as_str()));
        source.push_str(src);
        source.extend(self.call_symbols.iter().map(|item| item.as_str()));
        source.push_str(&"}\n".to_string());

        source
    }
}

impl ShaderSource for MainShader<VertShader> {
    fn type_(&self) -> ShaderType {
        ShaderType::Vertex
    }

    fn source(&self) -> String {
        self.build_source(Self::vert_src())
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![
            ShaderDataSource::VertexAttributes,
            ShaderDataSource::FragmentDataOut,
            ShaderDataSource::MatrixData,
        ]
    }
}

impl ShaderSource for MainShader<FragShader> {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        self.build_source(Self::frag_src())
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![]
    }
}

pub struct BlinnPhongLighting;

impl BlinnPhongLighting {
    fn src() -> String {
        "
#define DIRECTIONAL 0
#define POINT       1
#define SPOT        2

const float AMBIENT_INTENSITY = 0.01;
const float SHININESS = 32;

float blinn_specular(vec3 to_light_source_direction, float shininess) {
    vec3 to_viewer_direction = normalize(viewer_position - vertex.position);
    vec3 halfway_direction = normalize(to_light_source_direction + to_viewer_direction);
    return pow(max(dot(vertex.normal, halfway_direction), 0), shininess);
}

float attenuation() {
    float distance = length(light_source.position - vertex.position);
    return 1 / (light_source.constant + light_source.linear * distance
        + light_source.quadratic * distance * distance);
}

float fragment_luminosity() {
    return 1;

    // vec3 fragment_ndc = vertex.light_space_position * 0.5 + 0.5;
    // float illuminated_depth = texture(shadow_map, fragment_ndc.xy).r;
    // return float(fragment_ndc.z <= illuminated_depth);
}

void directional() {
    float diffuse_intensity = max(dot(vertex.normal, -light_source.direction), 0);
    float specular_intensity = blinn_specular(-light_source.direction, SHININESS);

    frag_color = vec4((AMBIENT_INTENSITY + fragment_luminosity()
        * (diffuse_intensity + specular_intensity)) * light_source.color, 1);
}

void point() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);
    float specular_intensity = blinn_specular(to_light_source_direction, SHININESS);

    frag_color = vec4((AMBIENT_INTENSITY + fragment_luminosity()
        * (diffuse_intensity + specular_intensity)) * light_source.color * attenuation(), 1);
}

void spot() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);
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
}
"
        .to_string()
    }

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for BlinnPhongLighting {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        Self::src()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![
            ShaderDataSource::LightingData,
            ShaderDataSource::FragmentDataIn,
            ShaderDataSource::FragColorOut,
        ]
    }
}

impl SubShaderSource for BlinnPhongLighting {
    fn signature(&self) -> String {
        "\nvoid compute_lighting();\n".to_string()
    }

    fn call_symbol(&self) -> String {
        "\ncompute_lighting();\n".to_string()
    }
}

pub struct DirectPBR;

impl DirectPBR {
    fn src() -> String {
        "
#define PI 3.14159265358979323846264338327950288
#define AMBIENT 0.1

vec3 albedo = vec3(0.4, 0.37, 0.25);
float metallic = 0.1;
float roughness = 0.6;
float ao = 0.08;

float distribution_GGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float geometry_schlick_GGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_GGX(NdotV, roughness);
    float ggx1 = geometry_schlick_GGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

vec3 pbr(vec3 light, vec3 light_color, float attenuation) {
    vec3 viewer = normalize(viewer_pos - fragment.pos);
    vec3 halfway = normalize(viewer + light);
    vec3 radiance = light_color * attenuation;

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    // Cook-Torrance BRDF
    float NDF = distribution_GGX(fragment.normal, halfway, roughness);
    float G = geometry_smith(fragment.normal, viewer, light, roughness);
    vec3 F = fresnel_schlick(max(dot(halfway, viewer), 0.0), F0);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(fragment.normal, viewer), 0.0) * max(dot(fragment.normal, light), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;

    float NdotL = max(dot(fragment.normal, light), 0.0);

    return (kD * albedo / PI + specular) * radiance * NdotL;
}

vec3 directional(LightSource light_source) {
    return pbr(-light_source.dir, light_source.color, 1.0);
}

vec3 point(LightSource light_source) {
    vec3 light = normalize(light_source.pos - fragment.pos);
    float distance = length(light_source.pos - fragment.pos);
    float attenuation = 1.0 / (distance * distance);

    return pbr(light, light_source.color, attenuation);
}

float scale01(float a, float b, float x) {
    return (x - a) / (b - a);
}

float edge_fade(float x) {
    const float K = 500.0;
    return (1 + sqrt(K)) / (K * x + sqrt(K)) - inversesqrt(K);
}

vec3 spot(LightSource light_source) {
    vec3 light = normalize(light_source.pos - fragment.pos);
    float distance = length(light_source.pos - fragment.pos);
    float attenuation = 1.0 / (distance * distance);
    float cos_theta = max(dot(light_source.dir, -light), 0.0);
    cos_theta = clamp(cos_theta, light_source.outer_cutoff, light_source.inner_cutoff);
    float fade = edge_fade(scale01(light_source.inner_cutoff, light_source.outer_cutoff, cos_theta));
    vec3 color = light_source.color * fade;

    return pbr(light, color, attenuation);
}

void do_light() {
    vec3 Lo = vec3(0.0);

    for(int i = 0; i < source_count; i++) {
        if(sources[i].type == 0) {
            Lo += directional(sources[i]);
        }
        else if(sources[i].type == 1) {
            Lo += point(sources[i]);
        }
        else if(sources[i].type == 2) {
            Lo += spot(sources[i]);
        }
    }

    vec3 ambient = vec3(AMBIENT) * albedo * ao;
    vec3 color = Lo + ambient;
    frag_color = vec4(color, 1.0);
}
".to_string()
    }

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for DirectPBR {
    fn type_(&self) -> ShaderType {
        ShaderType::Fragment
    }

    fn source(&self) -> String {
        Self::src()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![
            ShaderDataSource::FragmentDataIn,
            ShaderDataSource::LightingData,
            ShaderDataSource::FragColorOut,
        ]
    }
}

impl SubShaderSource for DirectPBR {
    fn signature(&self) -> String {
        "
void do_light();
"
        .to_string()
    }

    fn call_symbol(&self) -> String {
        "
    do_light();
"
        .to_string()
    }
}
pub struct ScreenShaderVert;

impl ScreenShaderVert {
    fn src() -> String {
        "
layout (location = 0) in vec3 position;
layout (location = 2) in vec2 tex_coord;

out VertexData {
    vec2 tex_coord;
} vertex_data;

void main() {
    vertex_data.tex_coord = tex_coord;
    gl_Position = vec4(position, 1.0);
}
"
        .to_string()
    }

    pub fn new() -> Self {
        Self {}
    }
}

impl ShaderSource for ScreenShaderVert {
    fn type_(&self) -> ShaderType {
        ShaderType::Vertex
    }

    fn source(&self) -> String {
        Self::src()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![]
    }
}

pub struct ScreenShaderFrag;

impl ScreenShaderFrag {
    pub const GAMMA_LOCATION: i32 = 30;
    pub const EXPOSURE_LOCATION: i32 = 40;

    fn src() -> String {
        format!(
            "
in VertexData {{
    vec2 tex_coord;
}} vertex_data;
        
uniform sampler2D color_buffer;

layout(location = {}) uniform float gamma;
layout(location = {}) uniform float exposure;
const float GRADE = 256.0;

vec3 color_grade(vec3 color) {{
    ivec3 icolor = ivec3(color * GRADE * 10.0);
    icolor /= int(GRADE);
    return vec3(icolor) / 10.0;
}}

void main() {{
    vec4 color = texture(color_buffer, vertex_data.tex_coord);
    vec3 mapped = vec3(1.0) - exp(-color.rgb * exposure); // tone mapping
    vec3 corrected = pow(mapped, vec3(gamma)); // gamma correction
    // vec3 grade = color_grade(corrected);
    frag_color = vec4(corrected, color.a);
}}
        ",
            ScreenShaderFrag::GAMMA_LOCATION,
            ScreenShaderFrag::EXPOSURE_LOCATION
        )
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
        Self::src()
    }

    fn data(&self) -> Vec<ShaderDataSource> {
        vec![ShaderDataSource::FragColorOut]
    }
}

pub fn build_shader(shader_source: &impl ShaderSource, context_version: Version) -> Shader {
    let shader = Shader::new(shader_source.type_() as GLenum).unwrap();
    let mut source = format!(
        "#version {}{}0 core\n",
        context_version.major, context_version.minor
    );
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
