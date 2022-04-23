#![feature(try_blocks, iter_intersperse)]

use log::LevelFilter;
use simplelog::{Config, SimpleLogger};

mod config;
mod modes;
mod spotify_api;
mod mpd_api;
mod util;

fn main() {
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

    let config = config::read_config()
        .expect("unable to read config");

    if config.auth_configured() {
        modes::track_cover_url::run(config)
    } else {
        modes::interactive_auth::run(config)
    }
}
