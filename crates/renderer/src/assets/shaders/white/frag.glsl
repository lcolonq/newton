uniform sampler2D texture_data;

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    if (texel.r == 0.0) {
        discard;
    }
    frag_color = vec4(1.0, 1.0, 1.0, 1.0);
} 
