#version 430 core

in FragmentData {
    vec3 pos;
    vec3 normal;
    vec2 tex_coord;
    vec3 lightspace_pos;
} fragment;

struct LightSource {
    vec3 color;
    int type;
    vec3 pos;
    vec3 dir;
};

layout(std140, binding = 1) uniform LightingData {
    LightSource light_source;
    vec3 viewer_pos;
};
out layout(location = 0) vec4 frag_color;

#define PI 3.14159265358979323846264338327950288
vec3 albedo = vec3(10.0, 10.0, 10.0);
float metallic = 0.0;
float roughness = 0.5;
float ao = 1.0;


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

vec3 fresnel_schlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 pbr(vec3 light, vec3 light_color, float attenuation) {
    vec3 viewer = normalize(viewer_pos - fragment.pos);
    vec3 halfway = normalize(viewer + light);

    vec3 radiance = light_color * attenuation;

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    vec3 F = fresnel_schlick(max(dot(halfway, viewer), 0.0), F0);

    float NDF = distribution_GGX(fragment.normal, halfway, roughness);
    float G = geometry_smith(fragment.normal, viewer, light, roughness);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(fragment.normal, viewer), 0.0) * max(dot(fragment.normal, light), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;

    kD *= 1.0 - metallic;

    float NdotL = max(dot(fragment.normal, light), 0.0);
    vec3 Lo = (kD * albedo / PI + specular) * radiance * NdotL;

    vec3 ambient = vec3(0.03) * albedo * ao;

    vec3 color = ambient + Lo;
    return color;
}

void directional() {
}

void point() {
    vec3 light = normalize(light_source.pos - fragment.pos);
    float distance = length(light_source.pos - fragment.pos);
    float attenuation = 1.0 / (distance * distance);
    frag_color = vec4(pbr(light, light_source.color, attenuation), 1.0);
}

void spot() {
}

void do_light() {
    if(light_source.type == 0) {
        directional();
    }
    if(light_source.type == 1) {
        point();
    }
    if(light_source.type == 2) {
        spot();
    }
}