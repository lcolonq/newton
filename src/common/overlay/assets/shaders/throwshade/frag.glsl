#version 300 es
precision highp float;

in vec2 vertex_texcoord;
out vec4 frag_color;

vec4 shade(vec2);

void main() {
    frag_color = shade(vertex_texcoord);
} 

// "The Cutoff"
