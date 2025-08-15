use teleia::*;

const VERT: &'static str = include_str!("assets/shaders/throwshade/vert.glsl");
const FRAG: &'static str = include_str!("assets/shaders/throwshade/frag.glsl");

pub struct Visualizer {
    pub tickset: u64,
    pub timeset: f64,
    pub author: String,
    pub shader: Option<shader::Shader>,
}
impl Visualizer {
    pub fn new() -> Self {
        Self {
            tickset: 0,
            timeset: 0.0,
            author: String::new(),
            shader: None,
        }
    }
    pub fn set(&mut self, ctx: &context::Context, st: &state::State, src: &str) -> Result<(), String> {
        let fsrc = format!("{}\n{}\n", FRAG, src);
        self.tickset = st.tick;
        self.timeset = 0.0;
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(dur) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            self.timeset = dur.as_secs_f64();
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
        struct Game {
            throwshade: ThrowShade,
        }
        impl Game {
            pub fn new(_ctx: &context::Context) -> Self {
                Self {
                    throwshade: ThrowShade::new(),
                }
            }
        }
        impl state::Game for Game {
            fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
                if let Some(s) = &self.throwshade.shader {
                    ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
                    ctx.clear();
                    s.bind(ctx);
                    s.set_f32(ctx, "opacity", 0.5);
                    s.set_vec2(ctx, "resolution", &glam::Vec2::new(ctx.render_width, ctx.render_height));
                    let elapsed = (st.tick - self.throwshade.tickset) as f32 / 60.0;
                    s.set_f32(ctx, "time", elapsed);
                    ctx.render_no_geometry();
                }
                Ok(())
            }
        }

        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        pub fn main_js() {
            teleia::run(1920, 1080, teleia::Options::NORESIZE, Game::new);
        }
        #[wasm_bindgen]
        pub async fn set_shader(s: &str) -> Result<(), String> {
            contextualize(|ctx, st, g: &mut Game| {
                log::info!("set shader: {}", s);
                if let Err(e) = g.throwshade.set(ctx, st, &s) {
                    log::warn!("error compiling shader: {}", e);
                    g.throwshade.shader = None;
                    return Err(format!("{}", e));
                }
                Ok(())
            })
        }
    }
}
