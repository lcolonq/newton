use std::{collections::HashMap, f32::consts::PI};

use teleia::*;

use lexpr::sexp;
use crate::{assets, fig, terminal};

struct Model {
    model: scene::Scene,
    pos: glam::Vec3,
    neck_base: glam::Mat4,
    tracking_eyes: (f32, f32),
    tracking_mouth: f32,
    tracking_neck: glam::Quat,
}
impl Model {
    pub fn new(ctx: &context::Context, pos: glam::Vec3) -> Self {
        let model = scene::Scene::from_gltf(ctx, include_bytes!("../assets/scenes/lcolonq.vrm"));
        let neck_base = model.nodes_by_name.get("J_Bip_C_Neck")
            .and_then(|i| model.nodes.get(*i))
            .expect("failed to find neck joint")
            .transform;
        Self {
            model,
            pos,
            neck_base,
            tracking_eyes: (1.0, 1.0),
            tracking_mouth: 0.0,
            tracking_neck: glam::Quat::IDENTITY,
        }
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
    fn render(
        &mut self,
        ctx: &context::Context, st: &mut state::State,
        a: &assets::Assets,
    ) -> Option<()> {
        if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
            n.transform = self.neck_base * glam::Mat4::from_quat(self.tracking_neck);
        }
        st.bind_3d(ctx, &a.shader_scene);
        a.shader_scene.set_position_3d(
            ctx,
            &glam::Mat4::from_translation(self.pos),
        );
        self.model.render(ctx, &a.shader_scene);
        Some(())
    }
}

pub struct Overlay {
    assets: assets::Assets,
    models: Vec<Model>,
    fb: framebuffer::Framebuffer,
    term: terminal::Terminal,
    fig: fig::Client,
}
impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let models = vec![
            Model::new(ctx, glam::Vec3::new(0.2, -1.63, 0.42)),
            Model::new(ctx, glam::Vec3::new(-0.2, -1.63, 0.42)),
        ];
        Self {
            assets: assets::Assets::new(ctx),
            fb: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(128.0, 64.0),
                &glam::Vec2::ZERO
            ),
            term: terminal::Terminal::new(ctx, 128, 64),
            models,
            fig: fig::Client::new("shiro:32050", &[
                sexp!((avatar toggle)),
                sexp!((avatar toggle set)),
                sexp!((avatar toggle unset)),
                sexp!((avatar text)),
                sexp!((avatar frame)),
                sexp!((avatar reset)),
                sexp!((avatar tracking)),
            ]),
        }
    }
    fn render_texture(&self, ctx: &context::Context, st: &mut state::State, tex: &texture::Texture, pos: glam::Vec2, dims: glam::Vec2) {
        self.assets.shader_flat.bind(ctx);
        self.assets.shader_flat.set_mat4(&ctx, "projection", &glam::Mat4::IDENTITY);
        let width = self.term.width as f32;
        let halfwidth = dims.x / 2.0;
        let height = self.term.height as f32;
        let halfheight = dims.y / 2.0;
        self.assets.shader_flat.set_mat4(
            ctx, "view",
            &glam::Mat4::from_scale(
                glam::Vec3::new(
                    2.0 / width,
                    2.0 / height,
                    1.0,
                ),
            ),
        );
        self.assets.shader_flat.set_mat4(
            &ctx, "position",
            &glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(halfwidth, halfheight, 1.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(
                    -width / 2.0 + pos.x + halfwidth,
                    height / 2.0 - pos.y - halfheight,
                    0.0,
                ),
            )
        );
        tex.bind(ctx);
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
        st.projection = glam::Mat4::perspective_lh(
            PI / 4.0,
            self.term.width as f32 / self.term.height as f32,
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
                if let Some(eidx) = msg.data.get(3) {
                    if let Some(idx) = eidx.as_u64() {
                        if let Some(m) = self.models.get_mut(idx as usize) {
                            if m.handle_tracking(msg.clone()).is_none() { log::warn!("{}", malformed) }
                        } else { log::warn!("index {} out of bounds", idx) }
                    }
                } else { log::warn!("missing index") }
            }
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        self.fb.bind(ctx);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        self.render_texture(
            ctx, st, &self.assets.texture_operatop,
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(128.0, 76.0),
        );
        ctx.clear_depth();
        for m in self.models.iter_mut() {
            m.render(ctx, st, &self.assets);
        }
        ctx.clear_depth();
        self.render_texture(
            ctx, st, &self.assets.texture_mod,
            glam::Vec2::new(32.0, 0.0),
            glam::Vec2::new(32.0, 16.0),
        );
        self.render_texture(
            ctx, st, &self.assets.texture_operabottom,
            glam::Vec2::new(0.0, 55.0),
            glam::Vec2::new(128.0, 22.0),
        );
        st.render_framebuffer.bind(ctx);
        self.term.update(ctx, &self.fb);
        ctx.clear_color(glam::Vec4::new(0.0, 0.0, 0.0, 0.0));
        ctx.clear();
        self.term.render(ctx, &glam::Vec2::new(32.0, 32.0));
        Ok(())
    }
}
