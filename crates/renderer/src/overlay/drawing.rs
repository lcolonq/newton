use teleia::*;

use glow::HasContext;

use crate::{input, overlay};

pub const SCALE: usize = 4;
pub const WIDTH: usize = 1920 / SCALE;
pub const HEIGHT: usize = 1080 / SCALE;
pub struct Overlay {
    pub tex: texture::Texture,
    pub pixels: [u8; WIDTH * HEIGHT],
    pub last_point: Option<(i32, i32)>,
    pub shader_white: shader::Shader,
    pub shader_background: shader::Shader,
}
impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let shader_background = shader::Shader::new(
            ctx,
            include_str!("../assets/shaders/background/vert.glsl"),
            include_str!("../assets/shaders/background/frag.glsl"),
        );
        shader_background.set_i32(ctx, "background", 1);
        Self {
            tex: texture::Texture::new_empty(ctx),
            pixels: [0; WIDTH * HEIGHT],
            last_point: None,
            shader_white: shader::Shader::new(
                ctx,
                include_str!("../assets/shaders/white/vert.glsl"),
                include_str!("../assets/shaders/white/frag.glsl"),
            ),
            shader_background,
        }
    }
    pub fn coord(&self, x: usize, y: usize) -> Option<usize> {
        if x >= WIDTH || y >= HEIGHT {
            None
        } else {
            Some(x + y * WIDTH)
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
                WIDTH as i32,
                HEIGHT as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&self.pixels),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
}
impl overlay::Overlay for Overlay {
    fn update(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        match ost.input.get_command() {
            input::Command::Drawing => {
                let (sx, sy) = ost.input.get_mouse();
                let x = sx / (SCALE as i32);
                let y = sy / (SCALE as i32);
                if let Some(last) = self.last_point {
                    self.line(1, last, (x, y));
                } else {
                    self.point(1, x, y);
                }
                self.last_point = Some((x, y));
            },
            input::Command::EraseAll => {
                self.pixels.fill(0);
                self.last_point = None;
            },
            input::Command::None => {
                self.last_point = None;
            },
        }
        self.upload(ctx);
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        st.bind_2d(ctx, &self.shader_background);
        self.tex.bind(ctx);
        ost.backgrounds.drawing.bind_index(ctx, 1);
        self.shader_background.set_position_2d(
            ctx, st,
            &glam::Vec2::new(0.0, 0.0),
            &glam::Vec2::new(1920.0, 1080.0)
        );
        st.mesh_square.render(ctx);
        Ok(())
    }
}
