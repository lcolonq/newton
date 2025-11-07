use teleia::*;
use image::EncodableLayout;
use glow::HasContext;

use crate::overlay;

pub const WIDTH: f32 = 320.0;
pub const HEIGHT: f32 = 450.0;

#[derive(Debug, Clone)]
enum Error {
    NotEnoughFields,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotEnoughFields => write!(f, "not enough fields"),
        }
    }
}
impl std::error::Error for Error {}

struct Card {
    name: String,
    ty: String,
    depicted_subject: String,
    element: String,
    color: glam::Vec4,
    faction: String,
    equity: i64,
    boost_level: i64,
    rarity: String,
    rarity_level: i64,
    body_text: String,
    base_image_name: String,
    flags: String,
}

pub struct Overlay {
    fb: framebuffer::Framebuffer,
    texture: texture::Texture,
    shader_color: shader::Shader,
    shader_screen: shader::Shader,
    shader: shader::Shader,
    font: font::TrueType,
    card: Option<Card>,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            fb: framebuffer::Framebuffer::new(ctx, &glam::Vec2::new(WIDTH, HEIGHT), &glam::Vec2::new(0.0, 0.0)),
            texture: texture::Texture::new_empty(ctx),
            shader_color: shader::Shader::new(ctx, include_str!("../assets/shaders/color/vert.glsl"), include_str!("../assets/shaders/color/frag.glsl")),
            shader_screen: shader::Shader::new(ctx, include_str!("../assets/shaders/tcg_screen/vert.glsl"), include_str!("../assets/shaders/tcg_screen/frag.glsl")),
            shader: shader::Shader::new(ctx, include_str!("../assets/shaders/tcg/vert.glsl"), include_str!("../assets/shaders/tcg/frag.glsl")),
            font: font::TrueType::new(ctx, 20.0, include_bytes!("../assets/fonts/iosevka-comfy-regular.ttf")),
            card: None,
        }
    }
}

impl overlay::Overlay for Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        Ok(())
    }
    fn handle_binary(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, msg: &fig::BinaryMessage) -> Erm<()> {
        match &*msg.event {
            b"overlay tcg generate" => {
                let res: Erm<()> = (|| {
                    let s = std::str::from_utf8(&msg.data)?.to_owned();
                    let mut sp = s.split("\t");
                    let id = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let name = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let ty = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let depicted_subject = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let element = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let color = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let r = i64::from_str_radix(&color[1..=2], 16)?;
                    let g = i64::from_str_radix(&color[3..=4], 16)?;
                    let b = i64::from_str_radix(&color[5..=6], 16)?;
                    let faction = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let equity = sp.next().ok_or(Error::NotEnoughFields)?.parse()?;
                    let boost_level = sp.next().ok_or(Error::NotEnoughFields)?.parse()?;
                    let rarity = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let rarity_level = sp.next().ok_or(Error::NotEnoughFields)?.parse()?;
                    let body_text = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let base_image_name = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let flags = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    unsafe {
                        let img = image::ImageReader::open(format!("crates/renderer/src/assets/textures/tcg/bases/{}.png", base_image_name))?.decode()?.into_rgba8();
                        self.texture.bind(ctx);
                        ctx.gl.tex_image_2d(
                            glow::TEXTURE_2D,
                            0,
                            glow::RGBA as i32,
                            img.width() as i32,
                            img.height() as i32,
                            0,
                            glow::RGBA,
                            glow::UNSIGNED_BYTE,
                            Some(&img.as_bytes()),
                        );
                        ctx.gl.generate_mipmap(glow::TEXTURE_2D);
                    }
                    self.card = Some(Card {
                        name,
                        ty,
                        depicted_subject,
                        element,
                        color: glam::Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
                        faction,
                        equity,
                        boost_level,
                        rarity, rarity_level,
                        body_text,
                        base_image_name,
                        flags,
                    });
                    Ok(())
                })();
                if let Err(e) = res { log::warn!("malformed TCG generate: {}", e); }
            },
            _ => {},
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if let Some(card) = &self.card {
            st.bind_framebuffer(ctx, &self.fb);
            ctx.clear();

            st.bind_2d(ctx, &ost.assets.shader_flat);
            self.texture.bind(ctx);
            ost.assets.shader_flat.set_f32(ctx, "transparency", 0.0);
            ost.assets.shader_flat.set_mat4(ctx, "view", &glam::Mat4::IDENTITY);
            ost.assets.shader_flat.set_mat4(ctx, "position", &glam::Mat4::IDENTITY);
            st.mesh_square.render(ctx);

            st.bind_2d(ctx, &self.shader_color);
            self.shader_color.set_vec4(ctx, "color", &glam::Vec4::new(1.0, 0.0, 0.0, 1.0));
            self.shader_color.set_position_2d(
                ctx, st,
                &glam::Vec2::new(0.0, 10.0),
                &glam::Vec2::new(WIDTH, 32.0)
            );
            st.mesh_square.render(ctx);

            self.font.render_text_helper(ctx, st,
                &glam::Vec2::new(0.0, 0.0),
                &glam::Vec2::new(21.0, 40.0),
                &card.name,
                &[]
            );

            st.bind_render_framebuffer(ctx);
            st.bind_2d(ctx, &self.shader_screen);
            self.fb.bind_texture(ctx);
            self.shader_screen.set_position_2d(
                ctx, st,
                &glam::Vec2::new(1000.0, 200.0),
                &glam::Vec2::new(WIDTH, HEIGHT)
            );
            st.mesh_square.render(ctx);
        }
        Ok(())
    }
}
