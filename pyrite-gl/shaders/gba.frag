#version 150 core

in vec2 frag_texcoord;

out vec4 out_color;

uniform sampler2D tex;

void main() {
    vec4 col = texture(tex, frag_texcoord);
    out_color = vec4(col.rgb, 1.0);
}