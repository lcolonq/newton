use teleia::*;
use image::EncodableLayout;
use glow::HasContext;

use crate::overlay;

pub const IWIDTH: usize = 160;
pub const IHEIGHT: usize = 225;
pub const WIDTH: f32 = IWIDTH as f32;
pub const HEIGHT: f32 = IHEIGHT as f32;

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

#[derive(Debug, Clone)]
struct Card {
    name: String,
    ty: String,
    depicted_subject: String,
    element: String,
    color: glam::Vec4,
    faction: String,
    faction_color: glam::Vec4,
    equity: i64,
    boost_level: String,
    rarity: String,
    rarity_level: i64,
    body_text: String,
    base_image_name: String,
    set: String,
    minted_date: String,
    flags: String,
}

struct RenderedCardSlot {
    card: Option<Card>,
    texture: texture::Texture,
}
impl RenderedCardSlot {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            card: None,
            texture: texture::Texture::new_empty(ctx),
        }
    }
    pub fn set(&mut self, ctx: &context::Context, card: Card, img: &image::RgbaImage) {
        self.card = Some(card);
        unsafe {
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
    }
    pub fn render(&self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        pos: glam::Vec2,
        dim: glam::Vec2,
    ) {
        st.bind_2d(ctx, &ost.assets.shader_tcg_screen);
        self.texture.bind(ctx);
        ost.assets.shader_tcg_screen.set_position_2d(ctx, st, &pos, &dim);
        st.mesh_square.render(ctx);
    }
}

fn load_texture(tex: &texture::Texture, ctx: &context::Context, st: &mut state::State, path: &str) -> Erm<()> {
    unsafe {
        let img = image::ImageReader::open(path)?.decode()?.into_rgba8();
        tex.bind(ctx);
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
        Ok(())
    }
}

pub struct Overlay {
    fb: framebuffer::Framebuffer,
    texture_base: texture::Texture,
    texture_art: texture::Texture,
    texture_faction_nate: texture::Texture,
    texture_faction_lever: texture::Texture,
    texture_faction_tony: texture::Texture,
    // font: font::TrueType,
    font: font::Bitmap,
    card: RenderedCardSlot,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            fb: framebuffer::Framebuffer::new(ctx, &glam::Vec2::new(WIDTH, HEIGHT), &glam::Vec2::new(0.0, 0.0)),
            texture_base: texture::Texture::new_empty(ctx),
            texture_art: texture::Texture::new_empty(ctx),
            texture_faction_nate: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/nate.png")),
            texture_faction_lever: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/lever.png")),
            texture_faction_tony: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/tony.png")),
            // font: font::TrueType::new(ctx, 32.0, include_bytes!("../assets/fonts/iosevka-comfy-regular.ttf")),
            font: font::Bitmap::from_image(ctx, 6, 12, 96, 72, include_bytes!("../assets/fonts/terminus.png")),
            card: RenderedCardSlot::new(ctx),
        }
    }
    fn draw_rectangle(&self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        color: glam::Vec4, pos: glam::Vec2, dims: glam::Vec2
    ) {
        st.bind_2d(ctx, &ost.assets.shader_color);
        ost.assets.shader_color.set_vec4(ctx, "color", &color);
        ost.assets.shader_color.set_position_2d(
            ctx, st,
            &pos, &dims,
        );
        st.mesh_square.render(ctx);
    }

    fn generate_card(&self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, card: Card) -> Option<image::RgbaImage> {
        st.bind_framebuffer(ctx, &self.fb);
        ctx.clear();

        st.bind_2d(ctx, &ost.assets.shader_tcg_base);
        self.texture_base.bind(ctx);
        ost.assets.shader_tcg_base.set_vec4(ctx, "shift_color", &card.color);
        ost.assets.shader_tcg_base.set_mat4(ctx, "view", &glam::Mat4::IDENTITY);
        ost.assets.shader_tcg_base.set_mat4(ctx, "position", &glam::Mat4::IDENTITY);
        st.mesh_square.render(ctx);

        // top bar
        self.draw_rectangle(ctx, st, ost,
            card.color.clone(),
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(WIDTH, 16.0),
        );
        self.font.render_text_helper(ctx, st,
            &glam::Vec2::new(8.0, 1.0),
            &card.name,
            &[glam::Vec3::new(0.0, 0.0, 0.0)]
        );
        self.font.render_text_helper(ctx, st,
            &glam::Vec2::new(WIDTH - 8.0 * card.rarity.len() as f32, 1.0),
            &card.rarity,
            &[glam::Vec3::new(0.0, 0.0, 0.0)]
        );

        // art
        self.draw_rectangle(ctx, st, ost,
            glam::Vec4::new(0.1, 0.1, 0.1, 1.0),
            glam::Vec2::new(10.0, 16.0),
            glam::Vec2::new(140.0, 100.0),
        );
        st.bind_2d(ctx, &ost.assets.shader_flat);
        self.texture_art.bind(ctx);
        ost.assets.shader_flat.set_position_2d(
            ctx, st,
            &glam::Vec2::new(10.0, 16.0),
            &glam::Vec2::new(140.0, 100.0)
        );
        st.mesh_square.render(ctx);

        // faction stamp
        let stex = match card.faction.as_ref() {
            "nate" => Some(&self.texture_faction_nate),
            "lever" => Some(&self.texture_faction_lever),
            "tony" => Some(&self.texture_faction_tony),
            _ => None,
        };
        if let Some(tex) = stex {
            st.bind_2d(ctx, &ost.assets.shader_flat);
            tex.bind(ctx);
            ost.assets.shader_flat.set_position_2d(
                ctx, st,
                &glam::Vec2::new(WIDTH - 12.0 - 32.0, 18.0),
                &glam::Vec2::new(32.0, 32.0)
            );
            st.mesh_square.render(ctx);
        }

        // boost text
        let boost_pos = glam::Vec2::new(12.0, 105.0);
        self.font.render_text_helper(ctx, st,
            &boost_pos,
            &card.boost_level,
            &[glam::Vec3::new(0.1, 0.1, 0.1)]
        );
        self.font.render_text_helper(ctx, st,
            &(boost_pos - glam::Vec2::new(1.0, 1.0)),
            &card.boost_level,
            &[glam::Vec3::new(0.9, 0.9, 0.9)]
        );

        // equity marks
        let equity_pos = glam::Vec2::new(12.0, 18.0);
        for i in 0..card.equity {
            self.font.render_text_helper(ctx, st,
                &(equity_pos + glam::Vec2::new(0.0, 10.0) * i as f32),
                "$",
                &[glam::Vec3::new(0.1, 0.1, 0.1)]
            );
        }

        // body text
        self.draw_rectangle(ctx, st, ost,
            glam::Vec4::new(1.0, 1.0, 1.0, 0.5),
            glam::Vec2::new(4.0, 119.0),
            glam::Vec2::new(152.0, 100.0),
        );
        for (i, cs) in card.body_text.chars().collect::<Vec<char>>().chunks(25).enumerate() {
            let line: String = cs.iter().collect();
            self.font.render_text_helper(ctx, st,
                &glam::Vec2::new(5.0, 120.0 + 10.0 * i as f32),
                &format!("{}", line),
                &[glam::Vec3::new(0.2, 0.2, 0.2)]
            );
        }

        // bottom bar
        self.draw_rectangle(ctx, st, ost,
            glam::Vec4::new(0.0, 0.0, 0.0, 0.8),
            glam::Vec2::new(0.0, HEIGHT - 16.0),
            glam::Vec2::new(WIDTH, 16.0),
        );
        self.font.render_text_helper(ctx, st,
            &glam::Vec2::new(1.0, HEIGHT - 15.0),
            &format!("{}", card.set),
            &[glam::Vec3::new(1.0, 1.0, 1.0)]
        );
        self.font.render_text_helper(ctx, st,
            &glam::Vec2::new(WIDTH - 7.0 * (card.minted_date.len() - 1) as f32, HEIGHT - 15.0),
            &format!("{}", card.minted_date),
            &[glam::Vec3::new(1.0, 1.0, 1.0)]
        );

        st.bind_render_framebuffer(ctx);
        let mut pixels = vec![0; IWIDTH * IHEIGHT * 4];
        self.fb.get_pixels_raw(ctx, &mut pixels);
        image::RgbaImage::from_vec(IWIDTH as u32, IHEIGHT as u32, pixels)
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
                    log::info!("msg: {}", s);
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
                    let faction_color = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let f_r = i64::from_str_radix(&faction_color[1..=2], 16)?;
                    let f_g = i64::from_str_radix(&faction_color[3..=4], 16)?;
                    let f_b = i64::from_str_radix(&faction_color[5..=6], 16)?;
                    let equity = sp.next().ok_or(Error::NotEnoughFields)?.parse()?;
                    let boost_level = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let rarity = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let rarity_level = sp.next().ok_or(Error::NotEnoughFields)?.parse()?;
                    let body_text = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let base_image_name = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let set = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let minted_date = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let flags = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    load_texture(&self.texture_base, ctx, st, &format!("crates/renderer/src/assets/textures/tcg/bases/{}.png", base_image_name))?;
                    if load_texture(&self.texture_art, ctx, st, &format!("/home/llll/src/wasp/assets/avatars/{}.png", depicted_subject.to_ascii_lowercase())).is_err() {
                        load_texture(&self.texture_art, ctx, st, "/home/llll/src/wasp/assets/avatars/jontest.png")?;
                    }
                    let card = Card {
                        name,
                        ty,
                        depicted_subject,
                        element,
                        color: glam::Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
                        faction,
                        faction_color: glam::Vec4::new(f_r as f32 / 255.0, f_g as f32 / 255.0, f_b as f32 / 255.0, 1.0),
                        equity,
                        boost_level,
                        rarity, rarity_level,
                        body_text,
                        base_image_name,
                        set,
                        minted_date,
                        flags,
                    };
                    if let Some(img) = self.generate_card(ctx, st, ost, card.clone()) {
                        self.card.set(ctx, card, &img);
                        let err: Erm<()> = (||{
                            let mut buf = Vec::new();
                            let mut cursor = std::io::Cursor::new(&mut buf);
                            img.write_to(&mut cursor, image::ImageFormat::Png)?;
                            let with_meta = web_image_meta::png::add_text_chunk(
                                &buf, "lcolonqtcg", &s
                            )?;
                            // TODO: write to redis here
                            std::fs::write("/tmp/card.png", &with_meta)?;
                            Ok(())
                        })();
                        if let Err(e) = err {
                            log::warn!("failed to encode image: {}", e)
                        }
                    }
                    Ok(())
                })();
                if let Err(e) = res { log::warn!("malformed TCG generate: {}", e); }
            },
            _ => {},
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        if self.card.card.is_some() {
            self.card.render(ctx, st, ost,
                glam::Vec2::new(1000.0, 400.0),
                glam::Vec2::new(WIDTH * 2.0, HEIGHT * 2.0)
            );
        }
        Ok(())
    }
}
