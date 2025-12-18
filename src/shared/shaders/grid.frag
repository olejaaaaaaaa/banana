#version 450

layout(location = 0) in vec3 fragUV;
layout(location = 0) out vec4 outColor;

void main() {

    float scale = 20.0;
    vec2 grid = fract(fragUV.xy * scale);
    
    float lineWidth = 0.02;
    
    float lines = step(1.0 - lineWidth, grid.x) + 
                  step(1.0 - lineWidth, grid.y);
    
    vec2 majorGrid = fract(fragUV.xy * scale / 1.0);
    float majorLines = step(0.97, majorGrid.x) + step(0.97, majorGrid.y);
    
    vec3 color = vec3(0.5) * lines + vec3(0.8) * majorLines;
    
    outColor = vec4(color, 0.3);
}