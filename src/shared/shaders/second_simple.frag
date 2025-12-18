#version 440

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragPos;
layout(set = 1, binding = 0) uniform sampler2D lastFrame;
layout(location = 0) out vec4 outColor;

void main() {
    // Получаем текстурные координаты из позиции фрагмента
    // Предполагаем, что fragPos находится в диапазоне [-1, 1]
    vec2 texCoord = fragPos.xy * 0.5 + 0.5;
    
    // Получаем цвет из предыдущего кадра
    vec4 previousColor = texture(lastFrame, texCoord);
    
    // Смешиваем с текущим цветом (например, 50/50)
    vec4 currentColor = vec4(fragColor, 1.0);
    
    // Простое смешивание - можно настроить по вкусу
    outColor = mix(currentColor, previousColor, 0.5);
    
    // Или альтернативно: накопление цвета
    // outColor = (currentColor + previousColor) * 0.5;
    
    // Или только предыдущий кадр с оттенком текущего цвета:
    // outColor = previousColor * vec4(fragColor, 1.0);
}