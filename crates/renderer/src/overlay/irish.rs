use teleia::*;

use lexpr::sexp;
use base64::prelude::*;

use crate::{fig, overlay};

pub const WIDTH: f32 = 500.0;
pub const HEIGHT: f32 = 700.0;

pub struct Quote {
    text: String,
    start_time: u64,
}

pub struct Overlay {
    fb: framebuffer::Framebuffer,
    shader: shader::Shader,
    font: font::TrueType,
    quote: Option<Quote>,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            fb: framebuffer::Framebuffer::new(ctx, &glam::Vec2::new(WIDTH, HEIGHT), &glam::Vec2::new(0.0, 0.0)),
            shader: shader::Shader::new(ctx, include_str!("../assets/shaders/irish/vert.glsl"), include_str!("../assets/shaders/irish/frag.glsl")),
            font: font::TrueType::new(ctx, 40.0, include_bytes!("../assets/fonts/HennyPenny-Regular.ttf")),
            quote: None,
        }
    }
    pub fn handle_start(
        &mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::SexpMessage
    ) -> Option<()> {
        let bq = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let quote = String::from_utf8_lossy(&bq);
        self.quote = Some(Quote { text: quote.to_string(), start_time: st.tick });
        Some(())
    }
    pub fn handle_update(
        &mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::SexpMessage
    ) -> Option<()> {
        let bq = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let quote = String::from_utf8_lossy(&bq);
        if let Some(q) = &mut self.quote {
            q.text = quote.to_string();
        }
        Some(())
    }
    pub fn handle_save(
        &mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::SexpMessage
    ) -> Option<()> {
        let bq = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let path = String::from_utf8_lossy(&bq);
        let mut buf = vec![0; (WIDTH * HEIGHT * 4.0) as usize];
        self.fb.get_pixels_raw(ctx, &mut buf);
        let img = image::RgbaImage::from_raw(WIDTH as u32, HEIGHT as u32, buf)?;
        img.save_with_format(path.to_string(), image::ImageFormat::Png).ok()?;
        Some(())
    }
}

impl overlay::Overlay for Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        self.quote = None;
        Ok(())
    }
    fn handle(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, msg: fig::SexpMessage) -> Erm<()> {
        let malformed = format!("malformed {} data: {}", msg.event, msg.data);
        if msg.event == sexp!((overlay irish start)) {
            if self.handle_start(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
        } else if msg.event == sexp!((overlay irish update)) {
            if self.handle_update(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
        } else if msg.event == sexp!((overlay irish save)) {
            if self.handle_save(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if let Some(q) = &self.quote {
            st.bind_framebuffer(ctx, &self.fb);
            ctx.clear();
            st.bind_2d(ctx, &ost.assets.shader_flat);
            ost.backgrounds.drawing.bind(ctx);
            ost.assets.shader_flat.set_f32(ctx, "transparency", 0.0);
            ost.assets.shader_flat.set_mat4(ctx, "view", &glam::Mat4::IDENTITY);
            ost.assets.shader_flat.set_mat4(ctx, "position", &glam::Mat4::IDENTITY);
            st.mesh_square.render(ctx);
            self.font.render_text_helper(ctx, st,
                &glam::Vec2::new(0.0, 0.0),
                &glam::Vec2::new(21.0, 40.0),
                &q.text,
                &[]
            );
            st.bind_render_framebuffer(ctx);
            st.bind_2d(ctx, &self.shader);
            self.fb.bind_texture(ctx);
            self.shader.set_position_2d(
                ctx, st,
                &glam::Vec2::new(1000.0, 200.0),
                &glam::Vec2::new(WIDTH, HEIGHT)
            );
            st.mesh_square.render(ctx);
        }
        Ok(())
    }
}
