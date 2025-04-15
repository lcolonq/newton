use teleia::*;

use std::{collections::HashMap, f32::consts::PI};
use lexpr::sexp;
use base64::prelude::*;

use crate::{assets, fig, toggle};

pub struct Chat {
    msg: String,
    time: f64,
    biblicality: f32,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            msg: format!(""),
            time: 0.0,
            biblicality: 0.0,
        }
    }
}

pub struct Overlay {
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    fig: fig::Client,
    tracking_eyes: (f32, f32),
    tracking_mouth: f32,
    tracking_neck: glam::Quat,
    throwshade: newton_throwshade::ThrowShade,
    emacs_cursor: (f32, f32),
    mouse_cursor: (f32, f32),
    emacs_heartrate: i32,
    muzak_author: Option<String>,
    chat: Chat,
    toggles: toggle::Toggles,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let model = scene::Scene::from_gltf(ctx, include_bytes!("../assets/scenes/lcolonq.vrm")); 
        let model_neck_base = model.nodes_by_name.get("J_Bip_C_Neck")
            .and_then(|i| model.nodes.get(*i))
            .expect("failed to find neck joint")
            .transform;
        let throwshade = newton_throwshade::ThrowShade::new();
        Self {
            assets: assets::Assets::new(ctx),
            model,
            model_neck_base,
            fig: fig::Client::new("shiro:32050", &[
                sexp!((avatar toggle)),
                sexp!((avatar toggle set)),
                sexp!((avatar toggle unset)),
                sexp!((avatar text)),
                sexp!((avatar frame)),
                sexp!((avatar reset)),
                sexp!((avatar tracking)),
                sexp!((avatar overlay shader)),
                sexp!((avatar overlay muzak)),
                sexp!((avatar overlay muzak clear)),
                sexp!((avatar overlay chat)),
                sexp!((avatar overlay cursor)),
                sexp!((avatar overlay emacs)),
            ]),
            tracking_eyes: (1.0, 1.0),
            tracking_mouth: 0.0,
            tracking_neck: glam::Quat::IDENTITY,
            emacs_cursor: (0.0, 0.0),
            mouse_cursor: (0.0, 0.0),
            emacs_heartrate: 0,
            throwshade,
            muzak_author: None,
            chat: Chat::new(),
            toggles: toggle::Toggles::new(),
        }
    }
    pub fn handle_reset(&mut self, ctx: &context::Context) {
        // TODO also reset terminal
        if let Some(s) = &mut self.throwshade.shader { s.delete(ctx); }
        self.throwshade.shader = None;
        self.toggles.reset();
    }
    pub fn handle_tracking(&mut self, msg: fig::Message) -> Option<()> {
        let eyes = msg.data.get(0)?;
        let eye_left = eyes.get(0)?.as_str()?.parse::<f32>().ok()?;
        let eye_right = eyes.get(1)?.as_str()?.parse::<f32>().ok()?;
        let mouth = msg.data.get(1)?.as_str()?.parse::<f32>().ok()?;
        let euler = msg.data.get(2)?;
        let euler_x = euler.get(0)?.as_str()?.parse::<f32>().ok()?.to_radians();
        let euler_y = PI - euler.get(1)?.as_str()?.parse::<f32>().ok()?.to_radians();
        let euler_z = euler.get(2)?.as_str()?.parse::<f32>().ok()?.to_radians() + PI/2.0;
        self.tracking_eyes = (eye_left, eye_right);
        self.tracking_mouth = mouth;
        self.tracking_neck = glam::Quat::from_euler(glam::EulerRot::XYZ, euler_x, euler_y, euler_z);
        Some(())
    }
    pub fn handle_overlay_shader(
        &mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::Message
    ) -> Option<()> {
        let ba = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let author = String::from_utf8_lossy(&ba);
        let bs = BASE64_STANDARD.decode(msg.data.get(1)?.as_str()?).ok()?;
        let s = String::from_utf8_lossy(&bs);
        self.throwshade.author = author.to_string();
        if let Err(e) = self.throwshade.set(ctx, st, &s) {
            log::warn!("error compiling shader: {}", e);
            self.throwshade.shader = None;
        }
        Some(())
    }
    pub fn handle_overlay_muzak(&mut self, msg: fig::Message) -> Option<()> {
        let ba = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let author = String::from_utf8_lossy(&ba);
        self.muzak_author = Some(author.to_string());
        Some(())
    }
    pub fn handle_overlay_muzak_clear(&mut self) -> Option<()> {
        self.muzak_author = None;
        Some(())
    }
    pub fn handle_overlay_chat(&mut self, msg: fig::Message) -> Option<()> {
        let bs = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let s = String::from_utf8_lossy(&bs);
        let time = msg.data.get(1)?.as_str()?.parse::<f64>().ok()?;
        let biblicality = msg.data.get(2)?.as_str()?.parse::<f32>().ok()?;
        // log::info!("received chat message: {} {} {}", s, time, biblicality);
        self.chat.msg = s.to_string();
        self.chat.time = time;
        self.chat.biblicality = biblicality;
        Some(())
    }
    pub fn handle_overlay_cursor(&mut self, msg: fig::Message) -> Option<()> {
        let cursor_x = msg.data.get(0)?.as_i64()? as f32;
        let cursor_y = msg.data.get(1)?.as_i64()? as f32;
        self.emacs_cursor = (cursor_x, cursor_y);
        Some(())
    }
    pub fn handle_overlay_emacs(&mut self, msg: fig::Message) -> Option<()> {
        self.emacs_heartrate = msg.data.get(0)?.as_i64()? as i32;
        Some(())
    }
}

impl teleia::state::Game for Overlay {
    fn initialize_audio(&self, ctx: &context::Context, st: &state::State, actx: &audio::Context) -> HashMap<String, audio::Audio> {
        HashMap::new()
    }
    fn finish_title(&mut self, _st: &mut state::State) {}
    fn mouse_press(&mut self, _ctx: &context::Context, _st: &mut state::State) {}
    fn mouse_move(&mut self, _ctx: &context::Context, _st: &mut state::State, _x: i32, _y: i32) {}
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        match mouse_position::mouse_position::Mouse::get_mouse_position() {
            mouse_position::mouse_position::Mouse::Position { x, y } => {
                self.mouse_cursor = (x as f32, y as f32);
            },
            _ => {},
        }
        while let Some(msg) = self.fig.pump() {
            let malformed = format!("malformed {} data: {}", msg.event, msg.data);
            if msg.event == sexp!((avatar tracking)) {
                if self.handle_tracking(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar reset)) {
                self.handle_reset(ctx);
            } else if msg.event == sexp!((avatar toggle)) {
                if self.toggles.handle(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle set)) {
                if self.toggles.handle_set(ctx, st, msg, true).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle unset)) {
                if self.toggles.handle_set(ctx, st, msg, false).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay shader)) {
                if self.handle_overlay_shader(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay muzak)) {
                if self.handle_overlay_muzak(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay muzak clear)) {
                if self.handle_overlay_muzak_clear().is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay chat)) {
                if self.handle_overlay_chat(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay cursor)) {
                if self.handle_overlay_cursor(msg).is_none() { log::warn!("{}", malformed) }
            } else {
                log::info!("received unhandled event {} with data: {}", msg.event, msg.data);
            }
        }
        if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
            n.transform = self.model_neck_base * glam::Mat4::from_quat(self.tracking_neck);
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        if let Some(s) = &self.throwshade.shader {
            s.bind(ctx);
            s.set_f32(
                ctx, "opacity",
                if let Some(t@toggle::Toggle { val: true, .. }) = self.toggles.get(ctx, st, "shaderclarity") {
                    ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0) * 0.5 + 0.5
                } else if let Some(t@toggle::Toggle { val: false, .. }) = self.toggles.get(ctx, st, "shaderclarity") {
                    (1.0 - ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0)) * 0.5 + 0.5
                } else {
                    0.5
                }
            );
            s.set_vec2(ctx, "resolution", &glam::Vec2::new(ctx.render_width, ctx.render_height));
            let elapsed = (st.tick - self.throwshade.tickset) as f32 / 60.0;
            s.set_f32(ctx, "time", elapsed);
            s.set_f32(ctx, "chat_time", (self.chat.time - self.throwshade.timeset) as f32);
            ctx.render_no_geometry();
            s.set_f32(ctx, "tracking_mouth", self.tracking_mouth);
            // log::info!("eyes: {:?}", self.tracking_eyes);
            s.set_vec2(ctx, "tracking_eyes", &glam::Vec2::new(self.tracking_eyes.0, self.tracking_eyes.1));
            s.set_mat4(ctx, "tracking_neck", &glam::Mat4::from_quat(self.tracking_neck));
            s.set_vec2(ctx, "emacs_cursor", &glam::Vec2::new(self.emacs_cursor.0, self.emacs_cursor.1));
            s.set_vec2(ctx, "mouse_cursor", &glam::Vec2::new(self.mouse_cursor.0, self.mouse_cursor.1));
            s.set_i32(ctx, "heartrate", self.emacs_heartrate);
        }
        if let Some(t@toggle::Toggle { val: true, .. }) = self.toggles.get(ctx, st, "adblock") {
            st.bind_2d(ctx, &self.assets.shader_flat);
            self.assets.texture_adblock.bind(ctx);
            let tr = 1.0 - ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0);
            self.assets.shader_flat.set_f32(ctx, "transparency", tr);
            self.assets.shader_flat.set_position_2d(
                ctx,
                &glam::Vec2::new(1100.0, 300.0),
                &glam::Vec2::new(800.0, 600.0)
            );
            self.assets.mesh_square.render(ctx);
        }
        let mut authors = Vec::new();
        if let Some(_) = &self.throwshade.shader {
            authors.push(format!("shader by {}", self.throwshade.author));
        }
        if let Some(a) = &self.muzak_author {
            authors.push(format!("music by {}", a));
        }
        let astr: String = authors.join(", ");
        self.assets.font.render_text(ctx, &glam::Vec2::new(0.0, 0.0), &astr);
        Ok(())
    }
}
