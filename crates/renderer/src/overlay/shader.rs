use teleia::*;

use lexpr::sexp;
use base64::prelude::*;

use crate::{fig, overlay, toggle};

pub struct Overlay {
    visualizer: newton_shader::Visualizer,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let visualizer = newton_shader::Visualizer::new();
        Self {
            visualizer,
        }
    }
    pub fn handle_overlay_shader(
        &mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::SexpMessage
    ) -> Option<()> {
        let ba = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let author = String::from_utf8_lossy(&ba);
        let bs = BASE64_STANDARD.decode(msg.data.get(1)?.as_str()?).ok()?;
        let s = String::from_utf8_lossy(&bs);
        self.visualizer.author = author.to_string();
        if let Err(e) = self.visualizer.set(ctx, st, &s) {
            log::warn!("error compiling shader: {}", e);
            self.visualizer.shader = None;
        }
        Some(())
    }
}

impl overlay::Overlay for Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if let Some(s) = &mut self.visualizer.shader { s.delete(ctx); }
        self.visualizer.shader = None;
        Ok(())
    }
    fn handle(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, msg: fig::SexpMessage) -> Erm<()> {
        if msg.event == sexp!((avatar overlay shader)) {
            let malformed = format!("malformed {} data: {}", msg.event, msg.data);
            if self.handle_overlay_shader(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if let Some(s) = &self.visualizer.shader {
            s.bind(ctx);
            s.set_f32(
                ctx, "opacity",
                if let Some(t@toggle::Toggle { val: true, .. }) = ost.toggles.get(ctx, st, "shaderclarity") {
                    ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0) * 0.5 + 0.5
                } else if let Some(t@toggle::Toggle { val: false, .. }) = ost.toggles.get(ctx, st, "shaderclarity") {
                    (1.0 - ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0)) * 0.5 + 0.5
                } else {
                    0.5
                }
            );
            s.set_vec2(ctx, "resolution", &glam::Vec2::new(ctx.render_width, ctx.render_height));
            let elapsed = (st.tick - self.visualizer.tickset) as f32 / 60.0;
            s.set_f32(ctx, "time", elapsed);
            s.set_f32(ctx, "chat_time", (ost.chat.time - self.visualizer.timeset) as f32);
            ctx.render_no_geometry();
            s.set_f32(ctx, "tracking_mouth", ost.tracking_mouth);
            // log::info!("eyes: {:?}", self.tracking_eyes);
            s.set_vec2(ctx, "tracking_eyes", &glam::Vec2::new(ost.tracking_eyes.0, ost.tracking_eyes.1));
            s.set_mat4(ctx, "tracking_neck", &glam::Mat4::from_quat(ost.tracking_neck));
            s.set_vec2(ctx, "emacs_cursor", &glam::Vec2::new(ost.emacs_cursor.0, ost.emacs_cursor.1));
            s.set_vec2(ctx, "mouse_cursor", &glam::Vec2::new(ost.mouse_cursor.0, ost.mouse_cursor.1));
            s.set_i32(ctx, "heartrate", ost.emacs_heartrate);
        }
        if let Some(t@toggle::Toggle { val: true, .. }) = ost.toggles.get(ctx, st, "adblock") {
            st.bind_2d(ctx, &ost.assets.shader_flat);
            ost.assets.texture_adblock.bind(ctx);
            let tr = 1.0 - ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0);
            ost.assets.shader_flat.set_f32(ctx, "transparency", tr);
            ost.assets.shader_flat.set_position_2d(
                ctx,
                &glam::Vec2::new(1100.0, 300.0),
                &glam::Vec2::new(800.0, 600.0)
            );
            st.mesh_square.render(ctx);
        }
        let mut authors = Vec::new();
        if let Some(_) = &self.visualizer.shader {
            authors.push(format!("shader by {}", self.visualizer.author));
        }
        if let Some(a) = &ost.muzak_author {
            authors.push(format!("music by {}", a));
        }
        let astr: String = authors.join(", ");
        ost.assets.font.render_text(ctx, st, &glam::Vec2::new(0.0, 0.0), &astr);
        Ok(())
    }
}
