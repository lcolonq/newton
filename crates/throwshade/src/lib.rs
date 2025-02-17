use teleia::*;

const VERT: &'static str = include_str!("assets/shaders/throwshade/vert.glsl");
const FRAG: &'static str = include_str!("assets/shaders/throwshade/frag.glsl");

pub struct ThrowShade {
    pub tickset: u64,
    pub timeset: f64,
    pub shader: Option<shader::Shader>,
}
impl ThrowShade {
    pub fn new() -> Self {
        Self {
            tickset: 0,
            timeset: 0.0,
            shader: None,
        }
    }
    pub fn set(&mut self, ctx: &context::Context, st: &state::State, src: &str) -> Result<(), String> {
        let fsrc = format!("{}\n{}\n", FRAG, src);
        self.tickset = st.tick;
        if let Ok(dur) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            self.timeset = dur.as_secs_f64();
            log::info!("the time: {}", self.timeset);
        }
        if let Some(s) = &mut self.shader {
            s.replace(ctx, VERT, &fsrc)?;
        } else {
            self.shader = Some(shader::Shader::new_helper(ctx, VERT, &fsrc)?);
        }
        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        struct Game {}
        impl Game {
            pub async fn new(_ctx: &context::Context) -> Self {
                Self {}
            }
        }
        impl state::Game for Game {
            fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
                ctx.clear_color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0));
                ctx.clear();
                Some(())
            }
        }

        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        pub async fn main_js() {
            teleia::run(480, 270, teleia::Options::NORESIZE, Game::new).await;
        }
    }
}
