#![allow(dead_code, unused_variables)]
mod assets;

use std::collections::HashMap;
use teleia::*;

pub struct Game {
    assets: assets::Assets,
}

impl Game {
    pub async fn new(ctx: &context::Context) -> Self {
        Self {
            assets: assets::Assets::new(ctx),
        }
    }
}

impl teleia::state::Game for Game {
    fn initialize_audio(&self, ctx: &context::Context, st: &state::State, actx: &audio::Context) -> HashMap<String, audio::Audio> {
        HashMap::from_iter(vec![
            ("test".to_owned(), audio::Audio::new(&actx, include_bytes!("client/assets/audio/test.wav"))),
        ])
    }
    fn finish_title(&mut self, _st: &mut state::State) {}
    fn mouse_press(&mut self, _ctx: &context::Context, _st: &mut state::State) {}
    fn mouse_move(&mut self, _ctx: &context::Context, _st: &mut state::State, _x: i32, _y: i32) {}
    fn update(&mut self, ctx: &context::Context, _st: &mut state::State) -> Option<()> {
        Some(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        ctx.clear();
        self.assets.font.render_text(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            "hello computer",
        );
        st.bind_2d(ctx, &self.assets.shader_flat);
        self.assets.texture_test.bind(ctx);
        self.assets.shader_flat.set_position_2d(
            ctx,
            &glam::Vec2::new(40.0, 40.0),
            &glam::Vec2::new(16.0, 16.0),
        );
        self.assets.mesh_square.render(ctx);
        Some(())
    }
}
