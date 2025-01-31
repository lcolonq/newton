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

pub struct Terminal {
    pub font: font::Bitmap,
    pub base_color: Layer<glam::Vec3>,
    pub set_color: Layer<glam::Vec3>,
    pub set_char: Layer<CharPair>,
}
impl Terminal {
    pub fn new(ctx: &context::Context) -> Self {
        let mut set_char = Layer::new();
        set_char.from_str("lcolonq");
        Self {
            font: font::Bitmap::from_image(ctx, 6, 12, 96, 72, include_bytes!("assets/fonts/terminus.png")),
            base_color: Layer::new(),
            set_color: Layer::new(),
            set_char,
        }
    }
    pub fn get_color(&self, pos: Pos) -> glam::Vec3 {
        if let Some(c) = self.set_color.get(pos) {
            if *c != glam::Vec3::new(0.0, 0.0, 0.0) {
                return *c;
            }
        }
        if let Some(c) = self.base_color.get(pos) {
            if *c != glam::Vec3::new(0.0, 0.0, 0.0) {
                return *c;
            }
        }
        glam::Vec3::new(0.0, 0.0, 0.0)
    }
    pub fn update(&mut self, ctx: &context::Context, fb: &framebuffer::Framebuffer) {
        self.base_color.from_framebuffer(ctx, fb);
    }
    pub fn fill_string(&mut self, s: &str) {
        self.set_char.from_str(s);
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
}
