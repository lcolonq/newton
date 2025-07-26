use teleia::*;

use std::{collections::HashMap, f32::consts::PI};
use lexpr::sexp;
use base64::prelude::*;
use device_query::DeviceQuery;

use glow::HasContext;

use crate::{assets, fig, toggle};

pub struct Chat {
    author: String,
    msg: String,
    time: f64,
    biblicality: f32,
}
impl Chat {
    pub fn new() -> Self {
        Self {
            author: format!(""),
            msg: format!(""),
            time: 0.0,
            biblicality: 0.0,
        }
    }
}

const DRAWING_WIDTH: usize = 1920 / 4;
const DRAWING_HEIGHT: usize = 1080 / 4;
pub enum DrawingCommand {
    None,
    Drawing,
    EraseAll,
}
pub struct Drawing {
    tex: texture::Texture,
    pixels: [u8; DRAWING_WIDTH * DRAWING_HEIGHT],
    last_point: Option<(i32, i32)>,
    shader_white: shader::Shader,
}
impl Drawing {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            tex: texture::Texture::new_empty(ctx),
            pixels: [0; DRAWING_WIDTH * DRAWING_HEIGHT],
            last_point: None,
            shader_white: shader::Shader::new(
                ctx,
                include_str!("../assets/shaders/white/vert.glsl"),
                include_str!("../assets/shaders/white/frag.glsl"),
            ),
        }
    }
    pub fn coord(&self, x: usize, y: usize) -> Option<usize> {
        if x > DRAWING_WIDTH || y > DRAWING_HEIGHT {
            None
        } else {
            Some(x + y * DRAWING_WIDTH)
        }
    }
    pub fn set(&mut self, val: u8, x: i32, y: i32) {
        self.coord(x as usize, y as usize).map(|idx| self.pixels[idx] = val);
    }
    pub fn point(&mut self, val: u8, x: i32, y: i32) {
        self.set(val, x, y - 1);
        self.set(val, x - 1, y);
        self.set(val, x, y);
        self.set(val, x + 1, y);
        self.set(val, x, y + 1);
    }
    pub fn line(&mut self, val: u8, (mut x0, mut y0): (i32, i32), (x1, y1): (i32, i32)) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -((y1 - y0).abs());
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.point(val, x0, y0);
            let e2 = 2 * error;
            if e2 >= dy {
                if x0 == x1 { break; }
                error += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 { break; }
                error += dx;
                y0 += sy;
            }
        }
    }
    pub fn upload(&self, ctx: &context::Context) {
        unsafe {
            let err = ctx.gl.get_error();
            self.tex.bind(ctx);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                DRAWING_WIDTH as i32,
                DRAWING_HEIGHT as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&self.pixels),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
}

pub struct Overlay {
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    fig: fig::Client,
    fig_binary: fig::BinaryClient,
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
    drawing: Drawing,
    device: device_query::DeviceState,
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
            fig_binary: fig::BinaryClient::new("shiro:32051", &[
                b"test event"
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
            drawing: Drawing::new(ctx),
            device: device_query::DeviceState::new(),
        }
    }
    fn get_mouse(&self) -> (i32, i32) {
        self.device.get_mouse().coords
    }
    fn get_drawing_command(&mut self) -> DrawingCommand {
        let keys = self.device.get_keys();
        if keys.contains(&device_query::Keycode::LMeta) {
            DrawingCommand::Drawing
        } else if keys.contains(&device_query::Keycode::RMeta) {
            DrawingCommand::EraseAll
        } else {
            DrawingCommand::None
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
        let ba = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let a = String::from_utf8_lossy(&ba);
        let bs = BASE64_STANDARD.decode(msg.data.get(1)?.as_str()?).ok()?;
        let s = String::from_utf8_lossy(&bs);
        let time = msg.data.get(2)?.as_str()?.parse::<f64>().ok()?;
        let biblicality = msg.data.get(3)?.as_str()?.parse::<f32>().ok()?;
        self.chat.author = a.to_string();
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
    pub fn render_drawing(&self, ctx: &context::Context, st: &mut state::State) {
        st.bind_2d(ctx, &self.drawing.shader_white);
        self.drawing.tex.bind(ctx);
        self.drawing.shader_white.set_position_2d(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            &glam::Vec2::new(1920.0, 1080.0)
        );
        self.assets.mesh_square.render(ctx);
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
        let (x, y) = self.get_mouse();
        self.mouse_cursor = (x as f32, y as f32);
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
        while let Some(msg) = self.fig_binary.pump() {
            log::info!("binary message: {:?}", msg);
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
        match self.get_drawing_command() {
            DrawingCommand::Drawing => {
                let (sx, sy) = self.get_mouse();
                let x = sx / 4;
                let y = sy / 4;
                if let Some(last) = self.drawing.last_point {
                    self.drawing.line(1, last, (x, y));
                } else {
                    self.drawing.point(1, x, y);
                }
                self.drawing.last_point = Some((x, y));
            },
            DrawingCommand::EraseAll => {
                self.drawing.pixels.fill(0);
                self.drawing.last_point = None;
            },
            DrawingCommand::None => {
                self.drawing.last_point = None;
            },
        }
        self.drawing.upload(ctx);
        self.render_drawing(ctx, st);
        Ok(())
    }
}
