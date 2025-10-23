use teleia::*;

use glow::HasContext;
use lexpr::sexp;
use base64::prelude::*;
use rand::Rng;

use crate::overlay;

const SCALE: usize = 15;
const WIDTH: usize = 1920 / SCALE;
const HEIGHT: usize = 1080 / SCALE;

pub struct Pattern {
    w: usize, h: usize,
    cells: Vec<bool>,
}
impl Pattern {
    pub fn from_rle(inp: &str) -> Option<Self> {
        let s = inp.replace(";", "\n");
        let mut data = String::new();
        let mut w = 0;
        let mut h = 0;
        for line in s.split("\n") {
            if let Some('#') = line.trim().chars().nth(0) {
            } else if let Some('x') = line.trim().chars().nth(0) {
                for assign in line.trim().split(",") {
                    if let Some((svar, sval)) = assign.split_once(" = ") {
                        if svar.trim() == "x" {
                            w = sval.trim().parse().ok()?;
                        } else if svar.trim() == "y" {
                            h = sval.trim().parse().ok()?;
                        }
                    }
                }
            } else {
                data.push_str(&line);
            }
        }
        if w == 0 || h == 0 || w > WIDTH || h > HEIGHT { return None }
        let mut ret = Self {
            w, h,
            cells: vec![false; w * h],
        };
        ret.populate(&data);
        Some(ret)
    }
    pub fn idx(&self, x: i32, y: i32) -> usize{
        let ux = x.rem_euclid(self.w as i32) as usize;
        let uy = y.rem_euclid(self.h as i32) as usize;
        uy * self.w + ux
    }
    pub fn get(&self, x: i32, y: i32) -> bool {
        let idx = self.idx(x, y);
        self.cells[idx]
    }
    pub fn set(&mut self, x: i32, y: i32) {
        let idx = self.idx(x, y);
        self.cells[idx] = true;
    }
    pub fn populate(&mut self, s: &str) {
        let mut run = 0;
        let mut x = 0;
        let mut y = 0;
        for c in s.chars() {
            if let Some(d) = c.to_digit(10) {
                run = run * 10 + d;
            } else {
                if run == 0 { run = 1 }
                if c == '$' && x == 0 { run -= 1 }
                while run > 0 {
                    match c {
                        'b' => {
                            x = (x + 1) % (self.w as i32);
                            if x == 0 { y += 1; }
                        },
                        'o' => {
                            self.set(x, y);
                            x = (x + 1) % (self.w as i32);
                            if x == 0 { y += 1; }
                        },
                        '$' => {
                            x = 0;
                            y += 1;
                        },
                        _ => {},
                    }
                    run -= 1;
                }
            }
        }
    }
}

type Cell = u8;

struct CellRule {
    color: [u8; 4],
}

struct CellBuffer {
    buf: [Cell; WIDTH * HEIGHT],
}
impl CellBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; WIDTH * HEIGHT],
        }
    }
    fn idx(x: i32, y: i32) -> usize {
        let ux = x.rem_euclid(WIDTH as i32) as usize;
        let uy = y.rem_euclid(HEIGHT as i32) as usize;
        uy * WIDTH + ux
    }
    pub fn get(&self, x: i32, y: i32) -> Cell {
        self.buf[Self::idx(x, y)]
    }

    pub fn neighbors(&self, x: i32, y: i32) -> [Cell; 8] {
        [ self.get(x-1, y-1),
          self.get(x, y-1),
          self.get(x+1, y-1),
          self.get(x-1, y),
          self.get(x+1, y),
          self.get(x-1, y+1),
          self.get(x, y+1),
          self.get(x+1, y+1),
        ]
    }
    pub fn is_nonzero(&self, x: i32, y: i32) -> bool {
        self.get(x, y) > 0
    }
    pub fn count_cell(&self, x: i32, y: i32) -> i32 {
        self.get(x, y).min(1) as i32
    }
    pub fn count_neighbors(&self, x: i32, y: i32) -> i32 {
        self.neighbors(x, y).into_iter().map(|c| c.min(1) as i32).sum()
    }
    pub fn most_common_neighbor(&self, x: i32, y: i32) -> u8 {
        let mut ns = self.neighbors(x, y);
        ns.sort_unstable();
        let mut winner = 0;
        let mut score = 0;
        let mut cur = 0;
        let mut curscore = 0;
        for c in ns {
            if c == 0 { continue; }
            if c != cur { cur = c; curscore = 1; }
            else { curscore += 1; }
            if curscore >= score { winner = c; score = curscore; }
        }
        winner
    }
    pub fn set(&mut self, x: i32, y: i32, v: Cell) {
        self.buf[Self::idx(x, y)] = v;
    }
}

pub struct Overlay {
    shader: shader::Shader,
    tex: texture::Texture,
    active: bool,
    buf0: CellBuffer,
    buf1: CellBuffer,
    next_rule: usize,
    rules: [CellRule; 256],
}
impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let rules = std::array::from_fn(|idx| match idx {
            0 => CellRule { color: [0, 0, 0, 0] },
            _ => CellRule { color: [0xff, 0xff, 0xff, 0xff] },
        });
        Self {
            shader: shader::Shader::new(
                ctx,
                include_str!("../assets/shaders/automata/vert.glsl"),
                include_str!("../assets/shaders/automata/frag.glsl"),
            ),
            tex: texture::Texture::new_empty(ctx),
            active: false,
            buf0: CellBuffer::new(),
            buf1: CellBuffer::new(),
            next_rule: 1,
            rules,
        }
    }
    pub fn spawn(&mut self, x: i32, y: i32, c: Cell, pat: &Pattern) {
        let cur = if self.active { &mut self.buf0 } else { &mut self.buf1 };
        for uxoff in 0..pat.w {
            for uyoff in 0..pat.h {
                let xoff = uxoff as i32; let yoff = uyoff as i32;
                cur.set(x + xoff, y + yoff, if pat.get(xoff, yoff) { c } else { 0 });
            }
        }
    }
    pub fn step(&mut self) {
        let (cur, next) = if self.active {
            (&mut self.buf0, &mut self.buf1)
        } else {
            (&mut self.buf1, &mut self.buf0)
        };
        for ux in 0..WIDTH {
            for uy in 0..HEIGHT {
                let x = ux as _; let y = uy as _;
                let n = cur.count_neighbors(x, y);
                if cur.is_nonzero(x, y) && n != 2 && n != 3{
                    next.set(x, y, 0)
                } else if n == 3 {
                    next.set(x, y, cur.most_common_neighbor(x, y))
                } else {
                    next.set(x, y, cur.get(x, y))
                }
            }
        }
        self.active = !self.active;
    }
    pub fn upload(&self, ctx: &context::Context) {
        let cur = if self.active { &self.buf0 } else { &self.buf1 };
        let mut buf = vec![0; WIDTH * HEIGHT * 4];
        for (idx, c) in cur.buf.iter().enumerate() {
            for off in 0..4 { buf[idx * 4 + off] = self.rules[*c as usize].color[off] }
        }
        unsafe {
            self.tex.bind(ctx);
            ctx.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                WIDTH as i32,
                HEIGHT as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&buf),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
    pub fn handle_spawn(&mut self, msg: fig::SexpMessage) -> Option<()> {
        let bs = BASE64_STANDARD.decode(msg.data.get(0)?.as_str()?).ok()?;
        let s = std::str::from_utf8(&bs).ok()?;
        let bcol = BASE64_STANDARD.decode(msg.data.get(2)?.as_str()?).ok()?;
        let scol = std::str::from_utf8(&bcol).ok()?.trim().strip_prefix("#")?;
        let col = u32::from_str_radix(scol, 16).ok()?;
        let r = (col >> 16 & 0xff) as u8;
        let g = (col >> 8 & 0xff) as u8;
        let b = (col & 0xff) as u8;
        if let Some(pat) = Pattern::from_rle(s) {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0..WIDTH);
            let y = rng.gen_range(0..HEIGHT);
            self.rules[self.next_rule] = CellRule { color: [r, g, b, 0xff] };
            self.spawn(x as i32, y as i32, self.next_rule as u8, &pat);
            self.next_rule = (self.next_rule + 1) % 256;
            if self.next_rule == 0 { self.next_rule = 1; }
        }
        Some(())
    }
}
impl overlay::Overlay for Overlay {
    fn reset(&mut self, ctx: &context::Context, st: &mut state::State, _ost: &mut overlay::State) -> Erm<()> {
        let cur = if self.active { &mut self.buf0 } else { &mut self.buf1 };
        for ux in 0..WIDTH {
            for uy in 0..HEIGHT {
                cur.set(ux as i32, uy as i32, 0)
            }
        }
        Ok(())
    }
    fn handle(
        &mut self, ctx: &context::Context, st: &mut state::State, _ost: &mut overlay::State,
        msg: fig::SexpMessage,
    ) -> Erm<()> {
        let malformed = format!("malformed {} data: {}", msg.event, msg.data);
        if msg.event == sexp!((avatar automata spawn)) {
            if self.handle_spawn(msg).is_none() { log::warn!("{}", malformed) }
        }
        Ok(())
    }
    fn update(&mut self, ctx: &context::Context, st: &mut state::State, _ost: &mut overlay::State) -> Erm<()> {
        if st.tick % 10 == 0 {
            self.step();
            self.upload(ctx);
        }
        Ok(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State, _ost: &mut overlay::State) -> Erm<()> {
        st.bind_2d(ctx, &self.shader);
        self.tex.bind(ctx);
        self.shader.set_position_2d(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            &glam::Vec2::new(1920.0, 1080.0)
        );
        st.mesh_square.render(ctx);
        Ok(())
    }
}
