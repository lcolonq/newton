uniform sampler2D texture_data;
uniform sampler2D background;

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    frag_color = texel;
}
