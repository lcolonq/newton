#![allow(dead_code, unused_variables)]
mod assets;
mod terminal;
mod fig;

use teleia::*;
use termion::raw::IntoRawMode;
use clap::{command, Command};

use std::{collections::HashMap, f32::consts::PI};
use lexpr::sexp;
use base64::prelude::*;

pub enum RenderMode {
    Overlay,
    Terminal(termion::raw::RawTerminal<std::io::Stdout>),
}

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

#[derive(Debug, Clone)]
pub struct Toggle {
    val: bool,
    set_time: u64,
}

pub struct Overlay {
    mode: RenderMode,
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    model_fb: framebuffer::Framebuffer,
    terminal: terminal::Terminal,
    fig: fig::Client,
    tracking_eyes: (f32, f32),
    tracking_mouth: f32,
    tracking_neck: glam::Quat,
    throwshade: throwshade::ThrowShade,
    emacs_cursor: (f32, f32),
    mouse_cursor: (f32, f32),
    emacs_heartrate: i32,
    muzak_author: Option<String>,
    chat: Chat,
    toggles: HashMap<String, Toggle>,
}

impl Overlay {
    pub async fn new(ctx: &context::Context, mode: RenderMode) -> Self {
        let model = scene::Scene::from_gltf(ctx, include_bytes!("assets/scenes/lcolonq.vrm")); 
        let model_neck_base = model.nodes_by_name.get("J_Bip_C_Neck")
            .and_then(|i| model.nodes.get(*i))
            .expect("failed to find neck joint")
            .transform;
        let throwshade = throwshade::ThrowShade::new();
        Self {
            mode,
            assets: assets::Assets::new(ctx),
            model,
            model_neck_base,
            model_fb: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(terminal::WIDTH as _, terminal::HEIGHT as _),
                &glam::Vec2::ZERO
            ),
            terminal: terminal::Terminal::new(ctx),
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
            toggles: HashMap::new(),
        }
    }
    pub async fn overlay(ctx: &context::Context) -> Self {
        Self::new(ctx, RenderMode::Overlay).await
    }
    pub async fn terminal(ctx: &context::Context) -> Self {
        let raw_stdout = std::io::stdout().into_raw_mode().expect("failed to set raw mode");
        Self::new(ctx, RenderMode::Terminal(raw_stdout)).await
    }
    pub fn set_toggle(&mut self, ctx: &context::Context, st: &state::State, nm: &str, val: bool) {
        self.toggles.insert(nm.to_string(), Toggle { val, set_time: st.tick });
    }
    pub fn get_toggle(&self, ctx: &context::Context, st: &state::State, nm: &str) -> Option<Toggle> {
        self.toggles.get(nm).cloned()
    }
    pub fn handle_reset(&mut self, ctx: &context::Context) {
        // TODO also reset terminal
        if let Some(s) = &mut self.throwshade.shader { s.delete(ctx); }
        self.throwshade.shader = None;
        self.toggles.clear();
    }
    pub fn handle_toggle(&mut self, ctx: &context::Context, st: &state::State, msg: fig::Message) -> Option<()> {
        let nm = msg.data.get(0)?.as_str()?;
        let prev = self.get_toggle(ctx, st, nm).map(|t| t.val).unwrap_or(false);
        self.set_toggle(ctx, st, nm, !prev);
        Some(())
    }
    pub fn handle_toggle_set(&mut self, ctx: &context::Context, st: &state::State, msg: fig::Message, val: bool) -> Option<()> {
        let nm = msg.data.get(0)?.as_str()?;
        self.set_toggle(ctx, st, nm, val);
        Some(())
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
    pub fn handle_text(&mut self, msg: fig::Message) -> Option<()> {
        let s = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        self.terminal.fill_string(std::str::from_utf8(&s).ok()?);
        Some(())
    }
    pub fn handle_frame(&mut self, msg: fig::Message) -> Option<()> {
        let data = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        for (i, c) in data.chunks_exact(3).enumerate() {
            if let [r, g, b] = c {
                let ii = i as i32;
                let p = terminal::Pos::new(ii % 64, ii / 64);
                self.terminal.base_color.set(
                    p,
                    glam::Vec3::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
                );
            }
        }
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
    fn render_model_terminal(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        self.model_fb.bind(ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        st.bind_3d(ctx, &self.assets.shader_scene);
        self.assets.shader_scene.set_position_3d(
            ctx,
            &glam::Mat4::from_translation(
                glam::Vec3::new(0.0, -1.63, 0.42),
            ),
        );
        self.model.render(ctx, &self.assets.shader_scene);
        st.render_framebuffer.bind(ctx);
        self.terminal.update(ctx, &self.model_fb);
        match &mut self.mode {
            RenderMode::Overlay => {
                ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
                ctx.clear();
                self.terminal.render(ctx, &glam::Vec2::new(12.0, 250.0));
            },
            RenderMode::Terminal(stdout) => if st.tick % 6 == 0 {
                self.terminal.write_tty(stdout);
            },
        }
        // self.model_fb.blit(
        //     ctx, &st.render_framebuffer,
        //     &glam::Vec2::new(ctx.render_width / 2.0 - 512.0, ctx.render_height / 2.0 - 512.0),
        //     &glam::Vec2::new(128.0, 128.0)
        // );
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
        st.projection = glam::Mat4::perspective_lh(
            PI / 4.0,
            terminal::WIDTH as f32 / terminal::HEIGHT as f32,
            0.1, 10.0
        );
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
                if self.handle_toggle(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle set)) {
                if self.handle_toggle_set(ctx, st, msg, true).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle unset)) {
                if self.handle_toggle_set(ctx, st, msg, false).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar text)) {
                if self.handle_text(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar frame)) {
                if self.handle_frame(msg).is_none() { log::warn!("{}", malformed) }
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
                if let Some(t@Toggle { val: true, .. }) = self.get_toggle(ctx, st, "shaderclarity") {
                    ((st.tick - t.set_time) as f32 / 60.0).clamp(0.0, 1.0) * 0.5 + 0.5
                } else if let Some(t@Toggle { val: false, .. }) = self.get_toggle(ctx, st, "shaderclarity") {
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
        if let Some(t@Toggle { val: true, .. }) = self.get_toggle(ctx, st, "adblock") {
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
        // self.render_model_terminal(ctx, st);
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Erm<()> {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("overlay")
                .about("Run the LCOLONQ model renderer in a full-screen transparent overlay")
        )
        .subcommand(
            Command::new("terminal")
                .about("Run the LCOLONQ model renderer in a terminal")
        )
        .subcommand(
            Command::new("server")
                .about("Run the LCOLONQ online websocket server")
        )
        .get_matches();
    match matches.subcommand() {
        Some(("overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::OVERLAY, Overlay::overlay).await?;
        },
        Some(("terminal", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::HIDDEN, Overlay::terminal).await?;
        },
        Some(("server", _cm)) => {
            env_logger::Builder::new().filter(None, log::LevelFilter::Info).init();
            log::info!("starting LCOLONQ server...");
        },
        _ => unreachable!("no subcommand"),
    }
    Ok(())
}
