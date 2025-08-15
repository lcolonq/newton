use teleia::*;
use std::collections::HashMap;

use crate::fig;

#[derive(Debug, Clone)]
pub struct Toggle {
    pub val: bool,
    pub set_time: u64,
}

pub struct Toggles {
    toggles: HashMap<String, Toggle>,
}
impl Toggles {
    pub fn new() -> Self {
        Self {
            toggles: HashMap::new(),
        }
    }
    pub fn set(&mut self, ctx: &context::Context, st: &state::State, nm: &str, val: bool) {
        self.toggles.insert(nm.to_string(), Toggle { val, set_time: st.tick });
    }
    pub fn get(&self, ctx: &context::Context, st: &state::State, nm: &str) -> Option<Toggle> {
        self.toggles.get(nm).cloned()
    }
    pub fn reset(&mut self) {
        self.toggles.clear();
    }
    pub fn handle(&mut self, ctx: &context::Context, st: &state::State, msg: fig::SexpMessage) -> Option<()> {
        let nm = msg.data.get(0)?.as_str()?;
        let prev = self.get(ctx, st, nm).map(|t| t.val).unwrap_or(false);
        self.set(ctx, st, nm, !prev);
        Some(())
    }
    pub fn handle_set(&mut self, ctx: &context::Context, st: &state::State, msg: fig::SexpMessage, val: bool) -> Option<()> {
        let nm = msg.data.get(0)?.as_str()?;
        self.set(ctx, st, nm, val);
        Some(())
    }
}
