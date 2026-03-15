#version 450
layout(location=0) in vec3 pos;
layout(location=1) in vec3 color;
layout(set=0, binding=0) uniform UBO {
    mat4 MVP;
};
layout(location=0) out vec3 fragColor;
void main() {
    gl_Position = MVP * vec4(pos, 1.0);
    fragColor = color;
}
