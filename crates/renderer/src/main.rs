#![allow(dead_code, unused_variables)]
mod assets;
mod terminal;
mod background;
mod toggle;
mod overlay;
mod input;

use teleia::*;
use clap::{command, Command};

pub fn main() -> Erm<()> {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("overlay")
                .about("Run the full-screen transparent overlay")
        )
        .subcommand(
            Command::new("model-terminal")
                .about("Run the LCOLONQ model renderer in a terminal")
        )
        .get_matches();
    match matches.subcommand() {
        Some(("overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::OVERLAY, |ctx| {
                overlay::Overlays::new(ctx, vec![
                    Box::new(overlay::automata::Overlay::new(ctx)),
                    Box::new(overlay::shader::Overlay::new(ctx)),
                    Box::new(overlay::drawing::Overlay::new(ctx)),
                    // Box::new(overlay::model::Overlay::new(ctx)),
                ])
            })?;
        },
        Some(("model-terminal", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::HIDDEN, overlay::model::Terminal::new)?;
        },
        _ => unreachable!("no subcommand"),
    }
    Ok(())
}
