#![allow(dead_code, unused_variables)]
mod assets;
mod terminal;
mod fig;
mod toggle;
mod overlay;

use teleia::*;
use clap::{command, Command};

pub fn main() -> Erm<()> {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("shader-overlay")
                .about("Run the shader display in a full-screen transparent overlay")
        )
        .subcommand(
            Command::new("model-overlay")
                .about("Run the LCOLONQ model renderer in a full-screen transparent overlay")
        )
        .subcommand(
            Command::new("model-terminal")
                .about("Run the LCOLONQ model renderer in a terminal")
        )
        .subcommand(
            Command::new("model-multi-overlay")
                .about("Run the LCOLONQ + Maude multi model renderer in a full-screen transparent overlay")
        )
        .subcommand(
            Command::new("server")
                .about("Run the LCOLONQ online websocket server")
        )
        .get_matches();
    match matches.subcommand() {
        Some(("shader-overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::OVERLAY, overlay::shader::Overlay::new)?;
        },
        Some(("model-overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::OVERLAY, overlay::model::Overlay::overlay)?;
        },
        Some(("model-terminal", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::HIDDEN, overlay::model::Overlay::terminal)?;
        },
        Some(("model-multi-overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, teleia::Options::OVERLAY, overlay::multi::Overlay::new)?;
        },
        Some(("server", _cm)) => {
            env_logger::Builder::new().filter(None, log::LevelFilter::Info).init();
            log::info!("starting LCOLONQ server...");
        },
        _ => unreachable!("no subcommand"),
    }
    Ok(())
}
