use teleia::*;

pub struct Assets {
    pub font: font::Bitmap,
    pub shader_flat: shader::Shader,
    pub shader_scene: shader::Shader,
    pub texture_adblock: texture::Texture,
    pub texture_mod: texture::Texture,
    pub texture_operatop: texture::Texture,
    pub texture_operabottom: texture::Texture,
}

impl Assets {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Bitmap::new(ctx),
            shader_flat: shader::Shader::new(
                ctx,
                include_str!("assets/shaders/flat/vert.glsl"),
                include_str!("assets/shaders/flat/frag.glsl"),
            ),
            shader_scene: shader::Shader::new(
                ctx,
                include_str!("assets/shaders/scene/vert.glsl"),
                include_str!("assets/shaders/scene/frag.glsl")
            ),
            texture_adblock: texture::Texture::new(ctx, include_bytes!("assets/textures/adblock.png")),
            texture_mod: texture::Texture::new(ctx, include_bytes!("assets/textures/mod.png")),
            texture_operatop: texture::Texture::new(ctx, include_bytes!("assets/textures/operatop.png")),
            texture_operabottom: texture::Texture::new(ctx, include_bytes!("assets/textures/operabottom.png")),
        }
    }
}
