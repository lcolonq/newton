use teleia::*;

const VERT: &'static str = include_str!("assets/shaders/throwshade/vert.glsl");
const FRAG: &'static str = include_str!("assets/shaders/throwshade/frag.glsl");

pub struct ThrowShade {
    pub shader: Option<shader::Shader>,
}
impl ThrowShade {
    pub fn new() -> Self {
        Self {
            shader: None,
        }
    }
    pub fn set(&mut self, ctx: &context::Context, src: &str) {
        let fsrc = format!("{}\n{}\n", FRAG, src);
        if let Some(s) = &mut self.shader {
            s.replace(ctx, VERT, &fsrc);
        } else {
            self.shader = Some(shader::Shader::new_nolib(ctx, VERT, &fsrc));
        }
    }
}
