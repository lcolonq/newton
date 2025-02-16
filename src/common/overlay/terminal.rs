use std::{collections::HashMap, io::Write};

use teleia::*;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 64;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Pos {
    pub x: i32, pub y: i32,
}
impl Pos {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }
}
impl std::ops::Add for Pos {
    type Output = Pos;
    fn add(self, rhs: Self) -> Self { Self {x: self.x + rhs.x, y: self.y + rhs.y } }
}

#[derive(Debug, Clone)]
pub struct CharPair {
    pub first: char,
    pub second: Option<char>,
}
impl Default for CharPair {
    fn default() -> Self {
        Self {
            first: 'a',
            second: Some('b'),
        }
    }
}

pub struct Layer<T> {
    pub data: [T; WIDTH * HEIGHT],
}
impl<T> Layer<T> {
    pub fn new() -> Self where T: Default {
        Self {
            data: [(); WIDTH * HEIGHT].map(|_| T::default()),
        }
    }
    pub fn get(&self, p: Pos) -> Option<&T> {
        if p.x < 0 || p.x >= WIDTH as _ || p.y < 0 || p.y >= HEIGHT as _ { return None }
        let idx = p.x as usize + p.y as usize * WIDTH;
        Some(&self.data[idx])
    }
    pub fn set(&mut self, p: Pos, x: T) {
        if p.x < 0 || p.x >= WIDTH as _ || p.y < 0 || p.y >= HEIGHT as _ { return }
        let idx = p.x as usize + p.y as usize * WIDTH;
        self.data[idx] = x;
    }
}
impl Layer<CharPair> {
    pub fn from_str(&mut self, s: &str) {
        let chars: Vec<char> = s.chars().collect();
        if chars.is_empty() { return }
        let mut i: usize = 0;
        for row in 0..64 {
            for col in 0..64 {
                let first = chars[i]; i += 1; i %= chars.len();
                let second = Some(chars[i]); i += 1; i %= chars.len();
                self.set(Pos::new(col, row), CharPair { first, second });
            }
        }
    }
}
impl Layer<glam::Vec3> {
    pub fn from_framebuffer(&mut self, ctx: &context::Context, fb: &framebuffer::Framebuffer) {
        fb.get_pixels(ctx, &mut self.data);
    }
    pub fn get_surrounding(&self, pos: Pos, bgcolor: &glam::Vec3) -> u8 {
        let offs = [
            Pos::new(-1, -1),
            Pos::new(0, -1),
            Pos::new(1, -1),
            Pos::new(-1, 0),
            Pos::new(1, 0),
            Pos::new(-1, 1),
            Pos::new(0, 1),
            Pos::new(1, 1),
        ];
        let mut ret = 0;
        for o in offs {
            ret <<= 1;
            let v = (self.get(pos + o) != Some(bgcolor)) as u8;
            ret |= v;
        }
        ret
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaletteType {
    Hair,
    Eyes,
    Skin,
    Highlight,
    Eyebags,
}
impl PaletteType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "hair" => Some(Self::Hair),
            "eyes" => Some(Self::Eyes),
            "skin" => Some(Self::Skin),
            "highlight" => Some(Self::Highlight),
            "eyebags" => Some(Self::Eyebags),
            _ => None,
        }
    }
    pub fn from_color(c: &glam::Vec3) -> Option<Self> {
        let r = (c.x * 255.0) as u8;
        let g = (c.y * 255.0) as u8;
        let b = (c.z * 255.0) as u8;
        if r >= 186 && r <= 188 && g >= 176 && g <= 178 && b >= 189 && b <= 191 {
            Some(Self::Hair)
        } else if r >= 158 && r <= 162 && g >= 148 && g <= 152 && b >= 159 && b <= 162 {
            Some(Self::Highlight)
        } else if g > r && g > b {
            Some(Self::Eyes)
        } else if r >= 242 && r <= 246 && g >= 238 && g <= 242 && b >= 234 && b <= 238 {
            Some(Self::Skin)
        } else if r == 182 && g == 142 && b == 139 {
            Some(Self::Eyebags)
        } else {
            None
        }
    }
}

pub struct PaletteEntry {
    pub color: Layer<glam::Vec3>,
    pub char: Layer<CharPair>,
}

pub struct Terminal {
    pub font: font::Bitmap,
    pub base_color: Layer<glam::Vec3>,
    pub palette: HashMap<PaletteType, PaletteEntry>,
}
impl Terminal {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Bitmap::from_image(ctx, 6, 12, 96, 72, include_bytes!("assets/fonts/terminus.png")),
            base_color: Layer::new(),
            palette: HashMap::new(),
        }
    }
    pub fn update(&mut self, ctx: &context::Context, fb: &framebuffer::Framebuffer) {
        self.base_color.from_framebuffer(ctx, fb);
    }
    pub fn fill_string(&mut self, s: &str) {
        // self.set_char.from_str(s);
    }
    pub fn outline_pattern(&self, pos: Pos) -> Option<String> {
        let sur = self.base_color.get_surrounding(pos, &glam::Vec3::new(0.0, 0.0, 0.0));
        let res = match sur {
            0b01101011 | 0b01101111 => " |",
            0b11010110 | 0b11010111 => "| ",
            0b00101011 | 0b00101111 | 0b00101110 | 0b00101100 | 0b00001011 => " /",
            0b10010110 | 0b10010111 | 0b10010011 | 0b10010001 | 0b00010110 => "\\ ",
            0b00111111 => "-/", 0b10011111 => "\\-",
            0b00011111 => "--", 0b00000111 => "__",
            0b00001111 => " _", 0b00010111 => "_ ",
            _ => return None,
        };
        Some(res.to_owned())
    }
    pub fn get(&self, pos: Pos) -> (CharPair, glam::Vec3) {
        if let Some(c) = self.base_color.get(pos) {
            if *c != glam::Vec3::new(0.0, 0.0, 0.0) {
                return *c;
            }
        }
        let c = self.get_color(pos);
        colors.push(c); colors.push(c);
        let new = if let Some(p) = self.set_char.get(pos) {
            if c == glam::Vec3::new(0.0, 0.0, 0.0) {
                String::from("  ")
            } else if let Some(pat) = self.outline_pattern(pos) {
                pat
            } else {
                format!("{}{}", p.first, if let Some(snd) = p.second { snd } else { ' ' })
            }
        } else {
            String::from("  ")
        };
        glam::Vec3::new(0.0, 0.0, 0.0)
    }
    pub fn render(&self, ctx: &context::Context, pos: &glam::Vec2) {
        let mut s = String::new();
        let mut colors = Vec::new();
        for row in 0..64 {
            for col in 0..64 {
                let pos = Pos::new(col, row);
                let c = self.get_color(pos);
                colors.push(c); colors.push(c);
                let new = if let Some(p) = self.set_char.get(pos) {
                    if c == glam::Vec3::new(0.0, 0.0, 0.0) {
                        String::from("  ")
                    } else if let Some(pat) = self.outline_pattern(pos) {
                        pat
                    } else {
                        format!("{}{}", p.first, if let Some(snd) = p.second { snd } else { ' ' })
                    }
                } else {
                    String::from("  ")
                };
                s += &new;
            }
            s += "\n";
            colors.push(glam::Vec3::new(1.0, 1.0, 1.0));
        }
        self.font.render_text_helper(ctx, pos, &s, &colors);
    }
    pub fn write_tty<W>(&self, out: &mut W)
    where W: Write {
        let mut output: Vec<u8> = Vec::new();
        write!(output, "\x1b[2J\x1b[1;1H").expect("failed to write output");
        for row in 0..64 {
            for col in 0..64 {
                let pos = Pos::new(col, row);
                let c = self.get_color(pos);
                let new = if let Some(p) = self.set_char.get(pos) {
                    if c == glam::Vec3::new(0.0, 0.0, 0.0) {
                        String::from("  ")
                    } else if let Some(pat) = self.outline_pattern(pos) {
                        pat
                    } else {
                        format!("{}{}", p.first, if let Some(snd) = p.second { snd } else { ' ' })
                    }
                } else {
                    String::from("  ")
                };
                write!(
                    output, "{}{}",
                    termion::color::Fg(
                        termion::color::Rgb(
                            (c.x * 255.0) as u8,
                            (c.y * 255.0) as u8,
                            (c.z * 255.0) as u8),
                    ),
                    new
                ).unwrap();
            }
            write!(output, "\r\n").unwrap();
        }
        out.write(&output).expect("failed to write to terminal");
    }
}
