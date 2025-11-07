use teleia::*;
use std::collections::HashMap;

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
    pub fn handle(&mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::BinaryMessage
    ) {
        let nm = if let Ok(s) = str::from_utf8(&msg.data) { s } else {
            log::warn!("failed to decode toggle name");
            return;
        };
        let prev = self.get(ctx, st, nm).map(|t| t.val).unwrap_or(false);
        self.set(ctx, st, nm, !prev);
    }
    pub fn handle_set(&mut self,
        ctx: &context::Context, st: &state::State,
        msg: fig::BinaryMessage, val: bool
    ) {
        let nm = if let Ok(s) = str::from_utf8(&msg.data) { s } else {
            log::warn!("failed to decode toggle name");
            return;
        };
        self.set(ctx, st, nm, val);
    }
}
