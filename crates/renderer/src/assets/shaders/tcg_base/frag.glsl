uniform sampler2D texture_data;

uniform vec4 shift_color;

vec3 rgb_to_hsl(vec3 rgb) {
    vec3 ret;
    float min = min(min(rgb.r, rgb.g), rgb.b);
    float max = max(max(rgb.r, rgb.g), rgb.b);
    float lum = (max + min) / 2.0;
    ret.z = lum;
    if (max == min) {
        ret.x = ret.y = 0.0;
    } else {
        float chroma = max - min;
        ret.y = chroma / (1.0 - abs(2.0 * lum - 1.0));
        if (max == rgb.r) {
            ret.x = (rgb.g - rgb.b) / chroma + (rgb.g < rgb.b ? 6.0 : 0.0);
        } else if (max == rgb.g) {
            ret.x = (rgb.b - rgb.r) / chroma + 2.0;
        } else {
            ret.x = (rgb.r - rgb.g) / chroma + 4.0;
        }
        ret.x /= 6.0;
    }
    return ret;
}

float hue_to_rgb(float p, float q, float t) {
   if (t < 0.0) t += 1.0;
   if (t > 1.0) t -= 1.0;
   if (t < 1.0/6.0) return p + (q - p) * 6.0 * t;
   if (t < 1.0/2.0) return q;
   if (t < 2.0/3.0) return p + (q - p) * (2.0/3.0 - t) * 6.0;
   return p;
}

vec3 hsl_to_rgb(vec3 hsl) {
    vec3 ret;
    if (hsl.y == 0.0) {
        ret.r = ret.g = ret.b = hsl.z;
    } else {
        float q = hsl.z < 0.5 ? hsl.z * (1.0 + hsl.y) : hsl.z + hsl.y - hsl.z * hsl.y;
        float p = 2.0 * hsl.z - q;
        ret.r = hue_to_rgb(p, q, hsl.x + 1.0/3.0);
        ret.g = hue_to_rgb(p, q, hsl.x);
        ret.b = hue_to_rgb(p, q, hsl.x - 1.0/3.0);
    }
    return ret;
}

void main()
{
    vec2 tcfull = vec2(vertex_texcoord.x, 1.0 - vertex_texcoord.y);
    vec4 texel = texture(texture_data, tcfull);
    vec3 hsl = rgb_to_hsl(texel.xyz);
    vec3 shift_hsl = rgb_to_hsl(shift_color.xyz);
    hsl.x = shift_hsl.x;
    texel.xyz = hsl_to_rgb(hsl);
    frag_color = texel;
} 
