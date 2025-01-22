#![allow(dead_code, unused_variables)]
mod assets;
mod terminal;

use std::collections::HashMap;
use teleia::*;

pub struct Overlay {
    assets: assets::Assets,
    model: scene::Scene,
    model_fb: framebuffer::Framebuffer,
    terminal: terminal::Terminal,
}

impl Overlay {
    pub async fn new(ctx: &context::Context) -> Self {
        Self {
            assets: assets::Assets::new(ctx),
            model: scene::Scene::from_gltf(ctx, include_bytes!("overlay/assets/scenes/lcolonq.vrm")),
            model_fb: framebuffer::Framebuffer::new(
                ctx,
                &glam::Vec2::new(terminal::WIDTH as _, terminal::HEIGHT as _),
                &glam::Vec2::ZERO
            ),
            terminal: terminal::Terminal::new(ctx),
        }
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
            std::f32::consts::PI / 4.0,
            terminal::WIDTH as f32 / terminal::HEIGHT as f32,
            0.1, 10.0
        );
        st.move_camera(
            ctx,
            &glam::Vec3::new(0.0, 0.0, 1.0),
            &glam::Vec3::new(0.0, 0.0, -1.0),
            &glam::Vec3::new(0.0, 1.0, 0.0),
        );
        // if let Some(n) = self.model.nodes_by_name.get("J_Bip_C_Neck").and_then(|i| self.model.nodes.get_mut(*i)) {
        //     n.transform *= glam::Mat4::from_rotation_y(0.05);
        // }
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
        // self.model_fb.blit(
        //     ctx, &st.render_framebuffer,
        //     &glam::Vec2::new(ctx.render_width / 2.0 - 512.0, ctx.render_height / 2.0 - 512.0),
        //     &glam::Vec2::new(1024.0, 1024.0)
        // );
        self.terminal.update(ctx, &self.model_fb);
        self.terminal.render(ctx, &glam::Vec2::new(400.0, 200.0));
        Some(())
    }
}
