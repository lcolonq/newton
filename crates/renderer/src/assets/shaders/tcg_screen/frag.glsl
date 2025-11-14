uniform sampler2D texture_data;

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    if (texel.a == 0.0) {
        discard;
    }
    texel.a = 1.0;
    frag_color = texel;
} 
