use teleia::*;

use crate::{overlay, toggle};

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
}

impl overlay::Overlay for Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if let Some(s) = &mut self.visualizer.shader { s.delete(ctx); }
        self.visualizer.shader = None;
        Ok(())
    }
    fn handle_binary(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, msg: &fig::BinaryMessage) -> Erm<()> {
        if msg.event == b"overlay shader" {
            let res: Erm<()> = (|| {
                let mut reader = std::io::Cursor::new(&msg.data);
                let author = fig::read_length_prefixed_utf8(&mut reader)?;
                let shader = fig::read_length_prefixed_utf8(&mut reader)?;
                self.visualizer.author = author.to_string();
                if let Err(e) = self.visualizer.set(ctx, st, &shader) {
                    log::warn!("error compiling shader: {}", e);
                    self.visualizer.shader = None;
                }
                Ok(())
            })();
            if let Err(e) = res { log::warn!("malformed shader update: {}", e); }
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
            s.set_f32(ctx, "chat_time", ost.chat.time - self.visualizer.timeset as f32);
            ctx.render_no_geometry();
            s.set_f32(ctx, "tracking_mouth", ost.tracking.mouth);
            s.set_vec2(ctx, "tracking_eyes", &glam::Vec2::new(ost.tracking.eyes.0, ost.tracking.eyes.1));
            s.set_mat4(ctx, "tracking_neck", &glam::Mat4::from_quat(ost.tracking.neck));
            s.set_vec2(ctx, "emacs_cursor", &glam::Vec2::new(ost.info.emacs_cursor.0, ost.info.emacs_cursor.1));
            s.set_vec2(ctx, "mouse_cursor", &glam::Vec2::new(ost.info.mouse_cursor.0, ost.info.mouse_cursor.1));
            s.set_i32(ctx, "heartrate", ost.info.emacs_heartrate);
        }
        if let Some(t@toggle::Toggle { val: true, .. }) = ost.toggles.get(ctx, st, "adblock") {
            st.bind_2d(ctx, &ost.assets.shader_flat);
            ost.assets.texture_adblock.bind(ctx);
            let tr = 1.0 - ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0);
            ost.assets.shader_flat.set_f32(ctx, "transparency", tr);
            ost.assets.shader_flat.set_position_2d(
                ctx, st,
                &glam::Vec2::new(1100.0, 300.0),
                &glam::Vec2::new(800.0, 600.0)
            );
            st.mesh_square.render(ctx);
        }
        let mut authors = Vec::new();
        if let Some(_) = &self.visualizer.shader {
            authors.push(format!("shader by {}", self.visualizer.author));
        }
        if let Some(a) = &ost.info.muzak_author {
            authors.push(format!("music by {}", a));
        }
        let astr: String = authors.join(", ");
        ost.assets.font.render_text(ctx, st, &glam::Vec2::new(0.0, 0.0), &astr);
        Ok(())
    }
}
