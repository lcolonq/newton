use teleia::*;

use std::{cell::RefCell, io::Write, rc::Rc};

use redis::Commands;
use image::EncodableLayout;
use glow::HasContext;
use glam::Vec4Swizzles;

use crate::overlay;

pub const CARD_SLOTS: usize = 11;
pub const CARD_SPACING: u64 = 300;
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
    frames: u32, encoded: String,
    owner: String, owner_id: String,
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
    card_fb: framebuffer::Framebuffer,
    effect_fb: framebuffer::Framebuffer, 
}
impl RenderedCardSlot {
    pub fn new(ctx: &context::Context) -> Self {
        let card_fb = framebuffer::Framebuffer::new(ctx,
            &glam::Vec2::new(WIDTH, HEIGHT),
            &glam::Vec2::ZERO
        );
        unsafe {
            card_fb.bind_texture(ctx);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as _);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as _);
        }
        Self {
            card: None,
            card_fb,
            effect_fb: framebuffer::Framebuffer::new(ctx, &glam::Vec2::new(WIDTH, HEIGHT), &glam::Vec2::ZERO),
        }
    }
    pub fn set(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        renderer: &CardRenderer, card: Card
    ) {
        renderer.render_card_framebuffer(ctx, st, ost, &card, &self.card_fb);
        self.card = Some(card);
    }
    pub fn apply_effect(&self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        progress: f32,
    ) {
        st.bind_framebuffer(ctx, &self.effect_fb);
        ctx.clear();
        st.bind_2d(ctx, &ost.assets.shader_tcg_effect);
        ost.assets.shader_tcg_effect.set_i32(ctx, "mode", 0);
        ost.assets.shader_tcg_effect.set_f32(ctx, "progress", progress);
        self.card_fb.bind_texture(ctx);
        ost.assets.shader_tcg_effect.set_position_2d(ctx, st, &glam::Vec2::new(0.0, 0.0), &glam::Vec2::new(WIDTH, HEIGHT));
        st.mesh_square.render(ctx);
        st.bind_render_framebuffer(ctx);
    }
    pub fn bind(&self, ctx: &context::Context) {
        self.card_fb.bind(ctx)
    }
    pub fn render(&self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        progress: f32,
        pos: glam::Vec2,
        dim: glam::Vec2,
    ) {
        self.apply_effect(ctx, st, ost, progress);
        st.bind_2d(ctx, &ost.assets.shader_tcg_screen);
        self.effect_fb.bind_texture(ctx);
        ost.assets.shader_tcg_screen.set_position_2d(ctx, st, &pos, &dim);
        st.mesh_square.render(ctx);
    }
    pub fn render_3d(&self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        back: &texture::Texture,
        progress: f32,
        pos: glam::Mat4,
    ) {
        self.apply_effect(ctx, st, ost, progress);
        st.bind_3d(ctx, &ost.assets.shader_tcg_screen);
        ost.assets.shader_tcg_screen.set_i32(ctx, "texture_front", 0);
        ost.assets.shader_tcg_screen.set_i32(ctx, "texture_back", 1);
        self.effect_fb.bind_texture(ctx);
        back.bind_index(ctx, 1);
        ost.assets.shader_tcg_screen.set_position_3d(ctx, st, &pos);
        st.mesh_square.render(ctx);
    }
}

struct ImageWrite {
    buf: Rc<RefCell<Vec<u8>>>,
}
impl Write for ImageWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
struct ImageEncoder {
    frames: u32,
    frames_left: u32,
    buf: Rc<RefCell<Vec<u8>>>,
    writer: png::Writer<ImageWrite>,
}
impl ImageEncoder {
    fn build_writer(frames: u32, w: ImageWrite) -> Option<png::Writer<ImageWrite>> {
        let mut encoder = png::Encoder::new(w, IWIDTH as _, IHEIGHT as _);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        encoder.set_source_chromaticities(png::SourceChromaticities::new(
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        ));
        encoder.set_animated(frames, 0).ok()?;
        encoder.set_frame_delay(1, 20).ok()?;
        encoder.write_header().ok()
    }
    fn start(frames: u32) -> Option<Self> {
        let buf = Rc::new(RefCell::new(Vec::new()));
        let w = ImageWrite { buf: buf.clone() };
        let writer = Self::build_writer(frames, w)?;
        Some(Self {
            frames,
            frames_left: frames,
            buf,
            writer,
        })
    }
    fn write_frame(&mut self, pixels: &[u8]) {
        if self.frames_left > 0 {
            let _ = self.writer.write_image_data(&pixels);
            self.frames_left -= 1;
        }
    }
    fn is_finished(&self) -> bool {
        self.frames_left == 0
    }
    fn finish(self) -> Option<Vec<u8>> {
        if self.is_finished() {
            self.writer.finish().expect("failed to finish");
            Some(self.buf.replace(Vec::new()))
        } else { None }
    }
}

struct MarqueeSlot {
    card: RenderedCardSlot, 
    encoder: Option<ImageEncoder>,
    active: Option<u64>, // ticks active
    height_offset: bool,
}
impl MarqueeSlot {
    fn write_frame(&mut self, ctx: &context::Context) {
        let mut pixels = [0; IWIDTH * IHEIGHT * 4];
        if let Some(enc) = &mut self.encoder {
            if enc.frames_left > 0 {
                self.card.effect_fb.get_pixels_raw(ctx, &mut pixels);
                enc.write_frame(&pixels);
            }
        }
    }
    fn render(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        progress: f32,
        pos: glam::Vec2,
        dim: glam::Vec2,
    ) {
        self.card.render(ctx, st, ost, progress, pos, dim);
        self.write_frame(ctx);
    }
    fn render_3d(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        back: &texture::Texture,
        progress: f32,
        pos: glam::Mat4,
    ) {
        self.card.render_3d(ctx, st, ost, back, progress, pos);
        self.write_frame(ctx);
    }
}
struct Marquee {
    texture_back: texture::Texture,
    font: font::Bitmap,
    slots: [MarqueeSlot; CARD_SLOTS],
    next_slot: usize,
    queue: std::collections::VecDeque<Card>,
    most_recent: u64,
    height_offset: bool,
}
impl Marquee {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            texture_back: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/cardback.png")),
            font: font::Bitmap::from_image(ctx, 6, 12, 96, 72, include_bytes!("../assets/fonts/terminus.png")),
            slots: std::array::from_fn(|_| MarqueeSlot {
                card: RenderedCardSlot::new(ctx),
                active: None,
                encoder: None,
                height_offset: false,
            }),
            next_slot: 0,
            queue: std::collections::VecDeque::new(),
            most_recent: 0,
            height_offset: false,
        }
    }
    fn set_slot(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        renderer: &CardRenderer,
        sidx: usize, card: Card
    ) {
        self.slots[sidx].card.set(ctx, st, ost, renderer, card.clone());
        self.slots[sidx].active = Some(st.tick);
        self.slots[sidx].height_offset = self.height_offset;
        self.slots[sidx].encoder = ImageEncoder::start(card.frames);
        self.height_offset = !self.height_offset;
        self.most_recent = st.tick;
    }
    fn fill_slot(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        renderer: &CardRenderer,
        card: &Card,
    ) -> bool {
        if st.tick - self.most_recent > CARD_SPACING {
            for (idx, s) in self.slots.iter_mut().enumerate() {
                if s.active.is_none() {
                    self.set_slot(ctx, st, ost, renderer, idx, card.clone());
                    return true;
                }
            }
        }
        false
    }
    pub fn add(&mut self,
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        renderer: &CardRenderer,
        card: Card,
    ) {
        if !self.fill_slot(ctx, st, ost, renderer, &card) {
            self.queue.push_back(card)
        }
    }
    fn upload_card(ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        c: &Card, buf: &[u8]
    ) -> Erm<()> {
        let with_meta = web_image_meta::png::add_text_chunk(
            buf, "lcolonqtcg", &c.encoded,
        )?;
        let uuid = uuid::Uuid::new_v4();
        let _: () = ost.redis_conn.hset("tcg:cards", uuid.to_string(), &with_meta)?;
        let inventory_key = format!("tcg-inventory:{}", c.owner_id);
        let _: () = ost.redis_conn.lpush(inventory_key, uuid.to_string())?;
        Ok(())
    }
    pub fn render(&mut self, 
        ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State,
        renderer: &CardRenderer,
    ) {
        for s in self.slots.iter_mut() {
            if let Some(b) = s.encoder.take_if(|e| e.is_finished()).and_then(|enc| enc.finish()) {
                if let Some(c) = &s.card.card {
                    let _ = Self::upload_card(ctx, st, ost, &c, &b).expect("failed to upload");
                }
            }
        }
        for idx in 0..CARD_SLOTS {
            if self.slots[idx].active.is_none() && st.tick - self.most_recent > CARD_SPACING {
                if let Some(c) = self.queue.pop_front() {
                    self.set_slot(ctx, st, ost, renderer, idx, c);
                } else {
                    break;
                }
            }
        }
        for s in self.slots.iter_mut() {
            if let Some(spawn) = s.active {
                let p = st.tick - spawn;
                let pos = (p as f32) / 200.0 - 7.0;
                if pos > 8.0 {
                    s.active = None;
                } else {
                    let trans = glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(0.7111, 1.0, 1.0),
                        glam::Quat::from_rotation_y(p as f32 / 75.0),
                        glam::Vec3::new(pos, -2.0, -8.0)
                    );
                    let proj = glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
                    let p_norm = st.projection.mul_vec4(st.view().mul_vec4(trans.mul_vec4(proj)));
                    let p_xy = (p_norm.xy() / p_norm.w) * glam::Vec2::new(1.0, -1.0);
                    let p_screen = (p_xy + glam::Vec2::new(1.0, 1.0)) / 2.0 * st.render_dims;
                    let progress = if let Some(c) = &s.card.card {
                        if c.frames == 0 { 1.0 } else {
                            (p as u32 % c.frames) as f32 / c.frames as f32
                        }
                    } else { 1.0 };
                    s.render_3d(ctx, st, ost, &self.texture_back, progress, trans);
                    if let Some(c) = &s.card.card {
                        let scale = 4.0;
                        let owner_width = c.owner.len() as f32 * self.font.char_width as f32 * scale;
                        self.font.render_text_parameterized(ctx, st,
                            &(p_screen + glam::Vec2::new(-owner_width / 2.0,
                                -200.0 + if s.height_offset { -50.0 } else { 0.0 })),
                            &c.owner,
                            font::BitmapParams {
                                color: &[glam::Vec3::new(1.0, 1.0, 1.0)],
                                scale: glam::Vec2::new(scale, scale),
                            },
                        );
                    }
                }
            }
        }
    }
}

struct CardRenderer {
    font: font::Bitmap,
    texture_base: texture::Texture,
    texture_art: texture::Texture,
    texture_faction_nate: texture::Texture,
    texture_faction_lever: texture::Texture,
    texture_faction_tony: texture::Texture,
}
impl CardRenderer {
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
    fn new(ctx: &context::Context) -> Self {
        Self {
            texture_base: texture::Texture::new_empty(ctx),
            texture_art: texture::Texture::new_empty(ctx),
            texture_faction_nate: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/nate.png")),
            texture_faction_lever: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/lever.png")),
            texture_faction_tony: texture::Texture::new(ctx, include_bytes!("../assets/textures/tcg/factions/tony.png")),
            font: font::Bitmap::from_image(ctx, 6, 12, 96, 72, include_bytes!("../assets/fonts/terminus.png")),
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

    fn render_card_framebuffer(&self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State, card: &Card, fb: &framebuffer::Framebuffer) {
        let _ = Self::load_texture(&self.texture_base, ctx, st, &format!("crates/renderer/src/assets/textures/tcg/bases/{}.png", card.base_image_name));
        if Self::load_texture(&self.texture_art, ctx, st, &format!("/home/llll/src/wasp/assets/avatars/{}.png", card.depicted_subject.to_ascii_lowercase())).is_err() {
            let _ = Self::load_texture(&self.texture_art, ctx, st, "/home/llll/src/wasp/assets/avatars/jontest.png");
        }
        st.bind_framebuffer(ctx, &fb);
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
    }

}

pub struct Overlay {
    renderer: CardRenderer,
    marquee: Marquee,
}
impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let fb = framebuffer::Framebuffer::new(ctx, &glam::Vec2::new(WIDTH, HEIGHT), &glam::Vec2::new(0.0, 0.0));
        unsafe {
            fb.bind_texture(ctx);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as _);
            ctx.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as _);
        }
        Self {
            renderer: CardRenderer::new(ctx),
            marquee: Marquee::new(ctx),
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
                    log::info!("msg: {}", s);
                    let mut sp = s.split("\t");
                    let owner = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
                    let owner_id = sp.next().ok_or(Error::NotEnoughFields)?.to_owned();
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
                    let card = Card {
                        frames: 20, encoded: s.clone(),
                        owner,
                        owner_id,
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
                    self.marquee.add(ctx, st, ost, &self.renderer, card);
                    Ok(())
                })();
                if let Err(e) = res { log::warn!("malformed TCG generate: {}", e); }
            },
            _ => {},
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, ost: &mut overlay::State) -> Erm<()> {
        st.render_framebuffer.bind(ctx);
        ctx.clear_depth();
        self.marquee.render(ctx, st, ost, &self.renderer);
        Ok(())
    }
}
