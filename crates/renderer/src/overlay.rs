pub mod model;
pub mod shader;
pub mod drawing;
pub mod automata;
pub mod irish;

use teleia::*;

use std::f32::consts::PI;
use lexpr::sexp;
use base64::prelude::*;

use crate::{assets, fig, toggle, input, background};

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
pub struct State {
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    fig: fig::SexpClient,
    fig_binary: fig::BinaryClient,
    tracking_eyes: (f32, f32),
    tracking_mouth: f32,
    tracking_neck: glam::Quat,
    emacs_cursor: (f32, f32),
    mouse_cursor: (f32, f32),
    emacs_heartrate: i32,
    muzak_author: Option<String>,
    chat: Chat,
    toggles: toggle::Toggles,
    input: input::Input,
    backgrounds: background::Backgrounds,
}
impl State {
    pub fn new(ctx: &context::Context) -> Self {
        let model = scene::Scene::from_gltf(ctx, include_bytes!("assets/scenes/lcolonq.vrm")); 
        let model_neck_base = model.nodes_by_name.get("J_Bip_C_Neck")
            .and_then(|i| model.nodes.get(*i))
            .expect("failed to find neck joint")
            .transform;
        Self {
            assets: assets::Assets::new(ctx),
            model,
            model_neck_base,
            fig: fig::SexpClient::new("shiro:32050", &[
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
                sexp!((avatar automata spawn)),
                sexp!((overlay irish start)),
                sexp!((overlay irish update)),
                sexp!((overlay irish save)),
            ]),
            fig_binary: fig::BinaryClient::new("shiro:32051", false, &[
                b"background frame"
            ]),
            tracking_eyes: (1.0, 1.0),
            tracking_mouth: 0.0,
            tracking_neck: glam::Quat::IDENTITY,
            emacs_cursor: (0.0, 0.0),
            mouse_cursor: (0.0, 0.0),
            emacs_heartrate: 0,
            muzak_author: None,
            chat: Chat::new(),
            toggles: toggle::Toggles::new(),
            backgrounds: background::Backgrounds::new(ctx),
            input: input::Input::new(),
        }
    }
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State) {
        self.toggles.reset();
    }
    pub fn handle_tracking(&mut self, msg: fig::SexpMessage) -> Option<()> {
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
    fn handle_overlay_muzak(&mut self, msg: fig::SexpMessage) -> Option<()> {
        let ba = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let author = String::from_utf8_lossy(&ba);
        self.muzak_author = Some(author.to_string());
        Some(())
    }
    fn handle_overlay_muzak_clear(&mut self) -> Option<()> {
        self.muzak_author = None;
        Some(())
    }
    fn handle_overlay_chat(&mut self, msg: fig::SexpMessage) -> Option<()> {
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
    fn handle_overlay_cursor(&mut self, msg: fig::SexpMessage) -> Option<()> {
        let cursor_x = msg.data.get(0)?.as_i64()? as f32;
        let cursor_y = msg.data.get(1)?.as_i64()? as f32;
        self.emacs_cursor = (cursor_x, cursor_y);
        Some(())
    }
    fn handle_overlay_emacs(&mut self, msg: fig::SexpMessage) -> Option<()> {
        self.emacs_heartrate = msg.data.get(0)?.as_i64()? as i32;
        Some(())
    }
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        let (x, y) = self.input.get_mouse();
        self.mouse_cursor = (x as f32, y as f32);
        if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
            n.transform = self.model_neck_base * glam::Mat4::from_quat(self.tracking_neck);
        }
        Ok(())
    }
}

pub trait Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut State) -> Erm<()> {
        Ok(())
    }
    fn update(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut State) -> Erm<()> {
        Ok(())
    }
    fn handle(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut State, msg: fig::SexpMessage) -> Erm<()> {
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut State) -> Erm<()> {
        Ok(())
    }
}

pub struct Overlays {
    state: State,
    overlays: Vec<Box<dyn Overlay>>
}
impl Overlays {
    pub fn new(ctx: &context::Context, overlays: Vec<Box<dyn Overlay>>) -> Self {
        Self {
            state: State::new(ctx),
            overlays,
        }
    }
    pub fn reset(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        self.state.reset(ctx, st);
        for ov in self.overlays.iter_mut() {
            ov.reset(ctx, st, &mut self.state)?;
        }
        Ok(())
    }
}
impl teleia::state::Game for Overlays {
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        while let Some(msg) = self.state.fig.pump() {
            let malformed = format!("malformed {} data: {}", msg.event, msg.data);
            for ov in self.overlays.iter_mut() {
                ov.handle(ctx, st, &mut self.state, msg.clone())?;
            }
            if msg.event == sexp!((avatar tracking)) {
                if self.state.handle_tracking(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar reset)) {
                self.reset(ctx, st)?;
            } else if msg.event == sexp!((avatar toggle)) {
                if self.state.toggles.handle(ctx, st, msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle set)) {
                if self.state.toggles.handle_set(ctx, st, msg, true).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar toggle unset)) {
                if self.state.toggles.handle_set(ctx, st, msg, false).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay muzak)) {
                if self.state.handle_overlay_muzak(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay muzak clear)) {
                if self.state.handle_overlay_muzak_clear().is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay chat)) {
                if self.state.handle_overlay_chat(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar overlay cursor)) {
                if self.state.handle_overlay_cursor(msg).is_none() { log::warn!("{}", malformed) }
            }
        }
        while let Some(msg) = self.state.fig_binary.pump() {
            match &*msg.event {
                b"background frame" => {
                    if let Some(f) = background::Frame::parse(&mut &*msg.data) {
                        self.state.backgrounds.update(ctx, f);
                    } else {
                        log::warn!("failed to parse frame");
                    }
                },
                ev => {
                    log::info!("unhandled event: {:?}", ev);
                },
            }
        }
        self.state.update(ctx, st)?;
        for ov in self.overlays.iter_mut() {
            ov.update(ctx, st, &mut self.state)?;
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        for ov in self.overlays.iter_mut() {
            ov.render(ctx, st, &mut self.state)?;
        }
        Ok(())
    }
}
