#version 450

layout(location = 0) in vec4 vPos;
layout(location = 1) in vec4 vNormal;
layout(location = 2) in vec2 vUV;
layout(location = 3) in vec4 vColor;
layout(location = 4) in vec4 vTangent;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragNormal;
layout(location = 3) out vec3 fragWorldPos;
layout(location = 4) out vec4 fragTangent;

layout(set = 0, binding = 0) uniform sampler2D textures[512];
layout(set = 0, binding = 1, rgba8) uniform image2D storageImages[128];

layout(set = 0, binding = 2) uniform Transform {
    vec4 scale;
    vec4 rot;
    vec4 pos;
} transform[2048];

layout(set = 0, binding = 3) uniform GlobalCamera {
    mat4 model;
    mat4 view;
    mat4 proj;
} camera;

layout(set = 0, binding = 4) readonly buffer SSBO {
    vec4 data[];
} ssbos[256];

layout(push_constant) uniform PushConstants {
    uint transform_id;
    uint time;
    uint texture_ids[2];
} pc;

mat4 quatToMat4(vec4 q) {
    float x = q.x, y = q.y, z = q.z, w = q.w;
    float x2 = x + x, y2 = y + y, z2 = z + z;
    float xx = x * x2, xy = x * y2, xz = x * z2;
    float yy = y * y2, yz = y * z2, zz = z * z2;
    float wx = w * x2, wy = w * y2, wz = w * z2;

    return mat4(
        1.0 - (yy + zz), xy + wz, xz - wy, 0.0,
        xy - wz, 1.0 - (xx + zz), yz + wx, 0.0,
        xz + wy, yz - wx, 1.0 - (xx + yy), 0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

void main() {
    gl_Position = camera.proj * camera.view * camera.model * vec4(vPos.xyz, 1.0);
    vec4 worldPos = camera.model * vec4(vPos.xyz, 1.0);
    fragWorldPos = worldPos.xyz;
    fragColor = vColor.rgb;
    fragUV = vUV;
    fragNormal = mat3(transpose(inverse(camera.model))) * vNormal.xyz;
    fragTangent = vTangent;
}