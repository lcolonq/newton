use teleia::*;

pub struct Assets {
    pub font: font::Bitmap,
    pub shader_flat: shader::Shader,
    pub shader_scene: shader::Shader,
    pub mesh_square: mesh::Mesh,
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
            mesh_square: mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/square.obj")),
        }
    }
}
