pub mod model;
pub mod shader;
pub mod drawing;
pub mod automata;
pub mod tcg;
pub mod loopback;

use teleia::*;

use std::f32::consts::PI;
use byteorder::{LE, ReadBytesExt};

use crate::{assets, fig, toggle, input, background};

pub struct Chat {
    author: String,
    msg: String,
    time: f32,
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

pub struct Tracking {
    eyes: (f32, f32),
    mouth: f32,
    neck: glam::Quat,
}

pub struct Info {
    mouse_cursor: (f32, f32),
    emacs_cursor: (f32, f32),
    emacs_heartrate: i32,
    muzak_author: Option<String>,
}

pub struct State {
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    fig_binary: fig::BinaryClient,
    tracking: Tracking,
    info: Info,
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
            fig_binary: fig::BinaryClient::new("shiro:32051", &[
                b"overlay reset",
                b"overlay tracking",
                b"overlay background frame",
                b"overlay toggle",
                b"overlay toggle set",
                b"overlay toggle unset",
                b"overlay info emacs",
                b"overlay info emacs cursor",
                b"overlay info credits music",
                b"overlay info credits music clear",
                b"overlay chat",
                b"overlay avatar text",
                b"overlay shader",
                b"overlay shader chat",
                b"overlay automata spawn",
                b"overlay tcg generate",
            ]).expect("failed to connect to bus"),
            tracking: Tracking {
                eyes: (1.0, 1.0),
                mouth: 0.0,
                neck: glam::Quat::IDENTITY,
            },
            info: Info {
                emacs_cursor: (0.0, 0.0),
                mouse_cursor: (0.0, 0.0),
                emacs_heartrate: 0,
                muzak_author: None,
            },
            chat: Chat::new(),
            toggles: toggle::Toggles::new(),
            backgrounds: background::Backgrounds::new(ctx),
            input: input::Input::new(),
        }
    }
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State) {
        self.toggles.reset();
    }
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        let (x, y) = self.input.get_mouse();
        self.info.mouse_cursor = (x as f32, y as f32);
        // update model head transform based on tracking state
        if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
            n.transform = self.model_neck_base * glam::Mat4::from_quat(self.tracking.neck);
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
    fn handle_binary(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut State, msg: &fig::BinaryMessage) -> Erm<()> {
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
        while let Some(msg) = self.state.fig_binary.pump()? {
            for ov in self.overlays.iter_mut() {
                ov.handle_binary(ctx, st, &mut self.state, &msg)?;
            }
            match &*msg.event {
                b"overlay reset" => self.reset(ctx, st)?,
                b"overlay tracking" => {
                    let res: Erm<()> = (|| {
                        let mut reader = std::io::Cursor::new(&msg.data);
                        let eye_left = reader.read_f32::<LE>()?;
                        let eye_right = reader.read_f32::<LE>()?;
                        let mouth = reader.read_f32::<LE>()?;
                        let euler_x = reader.read_f32::<LE>()?.to_radians();
                        let euler_y = PI - reader.read_f32::<LE>()?.to_radians();
                        let euler_z = reader.read_f32::<LE>()?.to_radians() + PI/2.0;
                        self.state.tracking.eyes = (eye_left, eye_right);
                        self.state.tracking.mouth = mouth;
                        self.state.tracking.neck = glam::Quat::from_euler(
                            glam::EulerRot::XYZ, euler_x, euler_y, euler_z
                        );
                        Ok(())
                    })();
                    if let Err(e) = res { log::warn!("malformed tracking update: {}", e); }
                },
                b"overlay background frame" => {
                    if let Some(f) = background::Frame::parse(&mut &*msg.data) {
                        self.state.backgrounds.update(ctx, f);
                    } else {
                        log::warn!("malformed background frame");
                    }
                },
                b"overlay toggle" => self.state.toggles.handle(ctx, st, msg),
                b"overlay toggle set" => self.state.toggles.handle_set(ctx, st, msg, true),
                b"overlay toggle unset" => self.state.toggles.handle_set(ctx, st, msg, false),
                b"overlay info emacs" => {
                    let mut reader = std::io::Cursor::new(&msg.data);
                    match reader.read_i32::<LE>() {
                        Ok(heartrate) => self.state.info.emacs_heartrate = heartrate,
                        Err(e) => log::warn!("malformed Emacs update: {}", e),
                    }
                },
                b"overlay info emacs cursor" => {
                    let res: Erm<()> = (|| {
                        let mut reader = std::io::Cursor::new(&msg.data);
                        let cursor_x = reader.read_f32::<LE>()?;
                        let cursor_y = reader.read_f32::<LE>()?;
                        self.state.info.emacs_cursor = (cursor_x, cursor_y);
                        Ok(())
                    })();
                    if let Err(e) = res { log::warn!("malformed Emacs cursor update: {}", e); }
                },
                b"overlay info credits music" => match str::from_utf8(&msg.data) {
                    Ok(nm) => self.state.info.muzak_author = Some(nm.to_owned()),
                    Err(e) => log::warn!("malformed music credits update: {}", e),
                },
                b"overlay info credits music clear" => { self.state.info.muzak_author = None; },
                b"overlay chat" => {
                    let res: Erm<()> = (|| {
                        let mut reader = std::io::Cursor::new(&msg.data);
                        let author = fig::read_length_prefixed_utf8(&mut reader)?;
                        let msg = fig::read_length_prefixed_utf8(&mut reader)?;
                        let time = reader.read_f32::<LE>()?;
                        let biblicality = reader.read_f32::<LE>()?;
                        self.state.chat.author = author;
                        self.state.chat.msg = msg;
                        self.state.chat.time = time;
                        self.state.chat.biblicality = biblicality;
                        Ok(())
                    })();
                    if let Err(e) = res { log::warn!("malformed chat update: {}", e); }
                },
                _ => {},
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
