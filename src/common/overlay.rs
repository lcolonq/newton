#![allow(dead_code, unused_variables)]
mod assets;
mod terminal;
mod fig;

use teleia::*;

use std::{collections::HashMap, f32::consts::PI};
use lexpr::sexp;

pub struct Overlay {
    assets: assets::Assets,
    model: scene::Scene,
    model_neck_base: glam::Mat4,
    model_fb: framebuffer::Framebuffer,
    terminal: terminal::Terminal,
    fig: fig::Client,
    tracking_eyes: (f32, f32),
    tracking_neck: glam::Quat,
}

impl Overlay {
    pub async fn new(ctx: &context::Context) -> Self {
        let model = scene::Scene::from_gltf(ctx, include_bytes!("overlay/assets/scenes/lcolonq.vrm")); 
        let model_neck_base = model.nodes_by_name.get("J_Bip_C_Neck")
            .and_then(|i| model.nodes.get(*i))
            .expect("failed to find neck joint")
            .transform;
        Self {
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
                sexp!((avatar text)),
                sexp!((avatar frame)),
                sexp!((avatar reset)),
                sexp!((avatar tracking)),
            ]),
            tracking_eyes: (1.0, 1.0),
            tracking_neck: glam::Quat::IDENTITY,
        }
    }
    pub fn handle_tracking(&mut self, msg: fig::Message) -> Option<()> {
        let eyes = msg.data.get(0)?;
        let eye_left = eyes.get(0)?.as_str()?.parse::<f32>().ok()?;
        let eye_right = eyes.get(1)?.as_str()?.parse::<f32>().ok()?;
        let euler = msg.data.get(1)?;
        let euler_x = euler.get(0)?.as_str()?.parse::<f32>().ok()?.to_radians();
        let euler_y = PI - euler.get(1)?.as_str()?.parse::<f32>().ok()?.to_radians();
        let euler_z = euler.get(2)?.as_str()?.parse::<f32>().ok()?.to_radians() + PI/2.0;
        self.tracking_eyes = (eye_left, eye_right);
        self.tracking_neck = glam::Quat::from_euler(glam::EulerRot::XYZ, euler_x, euler_y, euler_z);
        Some(())
    }
    pub fn handle_text(&mut self, msg: fig::Message) -> Option<()> {
        let s = msg.data.get(0)?.as_str()?;
        self.terminal.fill_string(s);
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
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
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
        while let Some(msg) = self.fig.pump() {
            let malformed = format!("malformed {} data: {}", msg.event, msg.data);
            if msg.event == sexp!((avatar tracking)) {
                if self.handle_tracking(msg).is_none() { log::warn!("{}", malformed) }
            } else if msg.event == sexp!((avatar text)) {
                if self.handle_text(msg).is_none() { log::warn!("{}", malformed) }
            } else {
                log::info!("received unhandled event {} with data: {}", msg.event, msg.data);
            }
        }
        if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
            n.transform = self.model_neck_base * glam::Mat4::from_quat(self.tracking_neck);
        }
        Some(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        self.model_fb.bind(ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 1.0));
        ctx.clear();
        st.bind_3d(ctx, &self.assets.shader_scene);
        self.assets.shader_scene.set_position_3d(
            ctx,
            &glam::Mat4::from_translation(
                glam::Vec3::new(0.0, -1.6, 0.5),
            ),
        );
        self.model.render(ctx, &self.assets.shader_scene);
        st.render_framebuffer.bind(ctx);
        self.terminal.update(ctx, &self.model_fb);
        self.terminal.render(ctx, &glam::Vec2::new(400.0, 200.0));
        // self.model_fb.blit(
        //     ctx, &st.render_framebuffer,
        //     &glam::Vec2::new(ctx.render_width / 2.0 - 512.0, ctx.render_height / 2.0 - 512.0),
        //     &glam::Vec2::new(128.0, 128.0)
        // );
        Some(())
    }
}
