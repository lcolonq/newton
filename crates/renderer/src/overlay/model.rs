use teleia::*;
use termion::raw::IntoRawMode;

use std::f32::consts::PI;
use lexpr::sexp;
use base64::prelude::*;

use crate::{overlay, terminal};

pub struct Terminal {
    ost: overlay::State,
    output: termion::raw::RawTerminal<std::io::Stdout>,
    terminal: terminal::Terminal,
    model_fb: framebuffer::Framebuffer,
}
impl Terminal {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            ost: overlay::State::new(ctx),
            output: std::io::stdout().into_raw_mode().expect("failed to set raw mode"),
            terminal: terminal::Terminal::new(ctx, 64, 64),
            model_fb: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(64.0, 64.0),
                &glam::Vec2::ZERO
            ),
        }
    }
}
impl teleia::state::Game for Terminal {
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        st.projection = glam::Mat4::perspective_lh(
            PI / 4.0,
            self.terminal.width as f32 / self.terminal.height as f32,
            0.1, 10.0
        );
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        self.model_fb.bind(ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        st.bind_3d(ctx, &self.ost.assets.shader_scene);
        self.ost.assets.shader_scene.set_position_3d(
            ctx,
            &glam::Mat4::from_translation(
                glam::Vec3::new(0.0, -1.63, 0.42),
            ),
        );
        self.ost.model.render(ctx, &self.ost.assets.shader_scene);
        st.render_framebuffer.bind(ctx);
        self.terminal.update(ctx, &self.model_fb);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        self.terminal.render(ctx, st, &glam::Vec2::new(12.0, 250.0));
        Ok(())
    }
}

pub struct Overlay {
    terminal: terminal::Terminal,
    model_fb: framebuffer::Framebuffer,
}
impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            terminal: terminal::Terminal::new(ctx, 64, 64),
            model_fb: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(64.0, 64.0),
                &glam::Vec2::ZERO
            ),
        }
    }
    pub fn handle_text(&mut self, msg: fig::SexpMessage) -> Option<()> {
        let bs = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let s = std::str::from_utf8(&bs).ok()?;
        log::info!("handle_text: {}", s);
        self.terminal.fill_string(s);
        Some(())
    }
    fn render_model(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        // self.model_fb.blit(
        //     ctx, &st.render_framebuffer,
        //     &glam::Vec2::new(ctx.render_width / 2.0 - 512.0, ctx.render_height / 2.0 - 512.0),
        //     &glam::Vec2::new(128.0, 128.0)
        // );
        Some(())
    }
}

impl overlay::Overlay for Overlay {
    fn handle(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, msg: fig::SexpMessage) -> Erm<()> {
        let malformed = format!("malformed {} data: {}", msg.event, msg.data);
        if msg.event == sexp!((avatar text)) {
            if self.handle_text(msg).is_none() { log::warn!("{}", malformed) }
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        let old_projection = st.projection;
        st.projection = glam::Mat4::perspective_lh(
            PI / 4.0,
            self.terminal.width as f32 / self.terminal.height as f32,
            0.1, 10.0
        );
        self.model_fb.bind(ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        st.bind_3d(ctx, &ost.assets.shader_scene);
        ost.assets.shader_scene.set_position_3d(
            ctx,
            &glam::Mat4::from_translation(
                glam::Vec3::new(0.0, -1.63, 0.42),
            ),
        );
        ost.model.render(ctx, &ost.assets.shader_scene);
        st.render_framebuffer.bind(ctx);
        self.terminal.update(ctx, &self.model_fb);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        self.terminal.render(ctx, st, &glam::Vec2::new(12.0, 250.0));
        st.projection = old_projection;
        Ok(())
    }
}
