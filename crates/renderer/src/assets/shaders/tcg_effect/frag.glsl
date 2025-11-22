uniform sampler2D tex;

uniform int mode;
uniform float progress;

void main()
{
    vec2 tc = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);

    switch (mode) {
        // case 0: {
        //     vec4 texel = texture(tex, tc);
        //     texel.a = 1.0;
        //     texel.r = (texel.r + progress) / 2.0;
        //     frag_color = texel;
        // } break;
    default: {
        vec4 texel = texture(tex, tc);
        texel.a = 1.0;
        frag_color = texel;
    } break;
    }
} 
