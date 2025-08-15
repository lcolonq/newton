use teleia::*;

use glow::HasContext;

const SCALE: usize = 15;
const WIDTH: usize = 1920 / SCALE;
const HEIGHT: usize = 1080 / SCALE;

struct Pattern {
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
        let mut ret = Self {
            w, h,
            cells: vec![false; w * h],
        };
        log::info!("dims: {w} {h} data: {data}");
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
        log::info!("pattern: {x} {y}");
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
    color: glam::Vec4,
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
    pub fn is_nonzero(&self, x: i32, y: i32) -> bool {
        self.get(x, y) > 0
    }
    pub fn count_cell(&self, x: i32, y: i32) -> i32 {
        self.get(x, y).min(1) as i32
    }
    pub fn count_neighbors(&self, x: i32, y: i32) -> i32 {
        self.count_cell(x-1, y-1) + self.count_cell(x, y-1) + self.count_cell(x+1, y-1)
            + self.count_cell(x-1, y) + self.count_cell(x+1, y)
            + self.count_cell(x-1, y+1) + self.count_cell(x, y+1) + self.count_cell(x+1, y+1)
    }
    pub fn set(&mut self, x: i32, y: i32, v: Cell) {
        self.buf[Self::idx(x, y)] = v;
    }
}

pub struct Board {
    shader: shader::Shader,
    tex: texture::Texture,
    active: bool,
    buf0: CellBuffer,
    buf1: CellBuffer,
    rules: [CellRule; 256],
}
impl Board {
    pub fn new(ctx: &context::Context) -> Self {
        let rules = std::array::from_fn(|idx| match idx {
            0 => CellRule { color: glam::Vec4::new(0.0, 0.0, 0.0, 0.0) },
            _ => CellRule { color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0) },
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
    pub fn test_glider(&mut self) {
        // let cur = if self.active { &mut self.buf0 } else { &mut self.buf1 };
        // cur.set(1, 0, 1);
        // cur.set(2, 1, 1);
        // cur.set(0, 2, 1);
        // cur.set(1, 2, 1);
        // cur.set(2, 2, 1);
        if let Some(pat) = Pattern::from_rle("
#N Tanner's p46
#O Tanner Jacobi
#C A period 46 oscillator discovered by Tanner Jacobi in October 2017.
#C https://conwaylife.com/wiki/Tanner%27s_p46
x = 13, y = 26, rule = B3/S23
2b2o9b$2bo10b$3bo9b$2b2o9b$13b$9b2o2b$9bo3b$10bo2b$9b2o2b$b2o10b$b2o6b
2o2b$o7bobo2b$b2o6bo3b$b2o7b3o$12bo$13b$13b$13b$13b$13b$13b$b2o10b$b2o
2b2o6b$5bobo5b$7bo5b$7b2o4b!
") {
            self.spawn(30, 10, 1, &pat);
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
                    next.set(x, y, 1)
                } else {
                    next.set(x, y, cur.get(x, y))
                }
            }
        }
        self.active = !self.active;
    }
    pub fn upload(&self, ctx: &context::Context) {
        let cur = if self.active { &self.buf0 } else { &self.buf1 };
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
                Some(&cur.buf),
            );
            ctx.gl.generate_mipmap(glow::TEXTURE_2D);
        }
    }
    pub fn update(&mut self, ctx: &context::Context, st: &mut state::State) {
        if st.tick % 10 == 0 {
            self.step();
            self.upload(ctx);
        }
    }
    pub fn render(&self, ctx: &context::Context, st: &mut state::State) {
        st.bind_2d(ctx, &self.shader);
        self.tex.bind(ctx);
        self.shader.set_position_2d(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            &glam::Vec2::new(1920.0, 1080.0)
        );
        st.mesh_square.render(ctx);
    }
}
