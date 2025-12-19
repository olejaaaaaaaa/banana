#version 440

layout(location = 0) out vec2 vUV;

void main() {

    vec2 positions[3] = vec2[](
        vec2(-1.0, -1.0),  // Левый нижний
        vec2( 3.0, -1.0),  // Правый нижний (за пределами экрана)
        vec2(-1.0,  3.0)   // Левый верхний (за пределами экрана)
    );

    vec2 uvs[3] = vec2[](
        vec2(0.0, 1.0),  // Левый нижний → (0, 1)
        vec2(2.0, 1.0),  // Правый нижний
        vec2(0.0, -1.0)  // Левый верхний → (0, 0)
    );
    
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    vUV = uvs[gl_VertexIndex];
}