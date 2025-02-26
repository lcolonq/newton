#version 300 es
precision highp float;

in vec2 vertex_texcoord;
out vec4 frag_color;

uniform float opacity;

uniform vec2 resolution;

uniform float time;

uniform float bpm;

uniform vec2 cursor;

uniform float chat_time;
uniform float chat_biblicality;

uniform float tracking_mouth;
uniform vec2 tracking_eyes;
uniform mat4 tracking_neck;

vec4 shade(vec2);

void main() {
    vec2 inverted = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    frag_color = shade(inverted);
    frag_color.a = clamp(frag_color.a * opacity, 0.0, opacity);
} 

// "The Cutoff"
