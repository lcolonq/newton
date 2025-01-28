mod common;

#[cfg(not(target_arch = "wasm32"))]
use clap::{command, Command};

#[cfg(target_arch = "wasm32")]
pub fn main() {} // dummy main, real wasm32 main is lib::main_js

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("overlay")
                .about("Run the LCOLONQ model renderer / overlay")
        )
        .subcommand(
            Command::new("server")
                .about("Run the LCOLONQ online websocket server")
        )
        .get_matches();
    match matches.subcommand() {
        Some(("overlay", _cm)) => {
            teleia::run("LCOLONQ", 1920, 1080, true, common::overlay::Overlay::new).await;
        },
        Some(("server", _cm)) => {
            env_logger::Builder::new().filter(None, log::LevelFilter::Info).init();
            log::info!("starting LCOLONQ server...");
        },
        _ => unreachable!("no subcommand"),
    }
}
