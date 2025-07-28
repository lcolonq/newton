mod client;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_js() {
    teleia::run(240, 160, teleia::Options::empty(), client::Game::new);
}
