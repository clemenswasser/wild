#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} uo;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = uo.proj * uo.view * uo.model * vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}
