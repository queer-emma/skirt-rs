#![allow(dead_code)]
#![feature(result_option_inspect)]

mod aabb;
mod args;
mod error;
mod parameters;
mod pattern;
mod reader;
mod render;

use color_eyre::eyre::Error;
use structopt::StructOpt;

use crate::args::Args;

fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    color_eyre::install()?;
    pretty_env_logger::init();

    let args = Args::from_args();
    args.run()?;

    Ok(())
}
