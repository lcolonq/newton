uniform sampler2D texture_front;
uniform sampler2D texture_back;

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = gl_FrontFacing ? texture(texture_back, tcfull) : texture(texture_front, tcfull);
    if (texel.a == 0.0) {
        discard;
    }
    texel.a = 1.0;
    frag_color = texel;
} 
