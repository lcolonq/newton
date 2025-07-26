use teleia::*;

use byteorder::ReadBytesExt;

use glow::HasContext;

pub struct Frame<'a> {
    tag: &'a [u8],
    width: u32,
    height: u32,
    pixels: &'a [u8],
}
impl<'a> Frame<'a> {
    fn read_length_prefixed(reader: &mut &'a [u8]) -> Option<&'a [u8]> {
        let len = reader.read_u32::<byteorder::LE>().ok()? as usize;
        log::info!("len: {}", len);
        let (x, xs) = reader.split_at(len);
        *reader = xs;
        Some(x)
    }
    pub fn parse(reader: &mut &'a [u8]) -> Option<Self> {
        log::info!("message: {:?}", reader);
        let tag = Self::read_length_prefixed(reader)?;
        log::info!("tag: {:?}", tag);
        let width = reader.read_u32::<byteorder::LE>().ok()?;
        log::info!("width: {:?}", width);
        let height = reader.read_u32::<byteorder::LE>().ok()?;
        log::info!("height: {:?}", height);
        let pixels = *reader;
        Some(Self { tag, width, height, pixels })
    }
}

pub struct Backgrounds {
    pub drawing: texture::Texture,
}
impl Backgrounds {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            // drawing: texture::Texture::new_empty(ctx), 
            drawing: texture::Texture::new(ctx, include_bytes!("assets/textures/everest.jpg")), 
        }
    }
    pub fn update<'a>(&self, ctx: &context::Context, f: Frame<'a>) {
        unsafe {
            let err = ctx.gl.get_error();
            self.drawing.bind(ctx);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                f.width as i32,
                f.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&f.pixels),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
}
