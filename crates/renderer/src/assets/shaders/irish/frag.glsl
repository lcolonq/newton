uniform sampler2D texture_data;

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    frag_color = texel;
} 
