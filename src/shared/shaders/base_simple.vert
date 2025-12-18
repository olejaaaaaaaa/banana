#version 440

//#include "common.glsl"

// layout(location = 0) in vec3 vPos;
// layout(location = 1) in vec3 vColor;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragPos;

void main() {

    vec3 positions[3] = vec3[3](
        vec3( 0.0, -0.5, 0.0),
        vec3(-0.5,  0.5, 0.0),  
        vec3( 0.5,  0.5, 0.0)   
    );
    
    vec3 colors[3] = vec3[3](
        vec3(1.0, 0.0, 0.0),    
        vec3(0.0, 1.0, 0.0),    
        vec3(0.0, 0.0, 1.0)     
    );
    
    gl_Position = vec4(positions[gl_VertexIndex], 1.0);
    fragPos = positions[gl_VertexIndex];
    fragColor = colors[gl_VertexIndex];
}