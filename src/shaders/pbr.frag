#version 430 core

in FragmentData {
    vec3 pos;
    vec3 normal;
    vec2 tex_coord;
    vec3 lightspace_pos;
} fragment;

struct LightSource {
    vec3 color;
    uint type;
    vec3 pos;
    float inner_cutoff;
    vec3 dir;
    float outer_cutoff;
};

layout(std140, binding = 1) uniform LightingData {
    LightSource sources[16];
    vec3 viewer_pos;
    uint source_count;
};

layout(location = 0) out vec4 frag_color;

uniform sampler2D albedo_map;
uniform sampler2D metallic_map;
uniform sampler2D roughness_map;
uniform sampler2D ao_map;
uniform sampler2D normal_map;
uniform sampler2D displacement_map;

#define PI 3.14159265358979323846264338327950288
#define AMBIENT 0.01

struct LightInfo {
    vec3 dir;
    vec3 color;
    float attenuation;
};

LightInfo directional(LightSource light_source) {
    return LightInfo(-light_source.dir, light_source.color, 1.0);
}

LightInfo point(LightSource light_source) {
    vec3 light = normalize(light_source.pos - fragment.pos);
    float distance = length(light_source.pos - fragment.pos);
    float attenuation = 1.0 / (distance * distance);

    return LightInfo(light, light_source.color, attenuation);
}

float scale01(float a, float b, float x) {
    return (x - a) / (b - a);
}

float edge_fade(float x) {
    const float K = 500.0;
    return (1 + sqrt(K)) / (K * x + sqrt(K)) - inversesqrt(K);
}

LightInfo spot(LightSource light_source) {
    vec3 light = normalize(light_source.pos - fragment.pos);
    float distance = length(light_source.pos - fragment.pos);
    float attenuation = 1.0 / (distance * distance);
    float cos_theta = max(dot(light_source.dir, -light), 0.0);
    cos_theta = clamp(cos_theta, light_source.outer_cutoff, light_source.inner_cutoff);
    float fade = edge_fade(scale01(light_source.inner_cutoff, light_source.outer_cutoff, cos_theta));
    vec3 color = light_source.color * fade;

    return LightInfo(light, color, attenuation);
}

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

// vec3 pbr(vec3 light, vec3 light_color, float attenuation) {
//     vec3 viewer = normalize(viewer_pos - fragment.pos);
//     vec3 halfway = normalize(viewer + light);
//     vec3 radiance = light_color * attenuation;
//     vec3 F0 = vec3(0.04);
//     F0 = mix(F0, albedo, metallic);
//     // Cook-Torrance BRDF
//     float NDF = distribution_GGX(fragment.normal, halfway, roughness);
//     float G = geometry_smith(fragment.normal, viewer, light, roughness);
//     vec3 F = fresnel_schlick(max(dot(halfway, viewer), 0.0), F0);
//     vec3 kS = F;
//     vec3 kD = vec3(1.0) - kS;
//     kD *= 1.0 - metallic;
//     vec3 numerator = NDF * G * F;
//     float denominator = 4.0 * max(dot(fragment.normal, viewer), 0.0) * max(dot(fragment.normal, light), 0.0) + 0.0001;
//     vec3 specular = numerator / denominator;
//     float NdotL = max(dot(fragment.normal, light), 0.0);
//     return (kD * albedo / PI + specular) * radiance * NdotL;
// }

void do_light() {
    vec3 albedo = texture(albedo_map, fragment.tex_coord).rgb;
    float metallic = texture(metallic_map, fragment.tex_coord).r;
    float roughness = texture(roughness_map, fragment.tex_coord).r;
    float ao = texture(ao_map, fragment.tex_coord).r;

    vec3 Lo = vec3(0.0);
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    LightInfo info;
    for(int i = 0; i < source_count; i++) {
        if(sources[i].type == 0) {
            info = directional(sources[i]);
        } else if(sources[i].type == 1) {
            info = point(sources[i]);
        } else {
            info = spot(sources[i]);
        }

        vec3 viewer = normalize(viewer_pos - fragment.pos);
        vec3 halfway = normalize(viewer + info.dir);
        vec3 radiance = info.color * info.attenuation;

        // Cook-Torrance BRDF
        float NDF = distribution_GGX(fragment.normal, halfway, roughness);
        float G = geometry_smith(fragment.normal, viewer, info.dir, roughness);
        vec3 F = fresnel_schlick(max(dot(halfway, viewer), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(fragment.normal, viewer), 0.0) * max(dot(fragment.normal, info.dir), 0.0) + 0.0001;
        vec3 specular = numerator / denominator;

        float NdotL = max(dot(fragment.normal, info.dir), 0.0);

        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(AMBIENT) * albedo * ao;
    vec3 color = Lo + ambient;
    frag_color = vec4(color, 1.0);
}