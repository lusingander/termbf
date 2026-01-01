mod app;
mod event;
mod interpreter;
mod ui;
mod widget;

use std::{
    fs::File,
    io::Read,
    sync::{Arc, RwLock},
};

use app::Speed;
use clap::Parser;
use ratatui::{backend::Backend, Terminal};

use crate::app::App;

/// termbf - Terminal Brainf*ck visualizer
#[derive(Parser)]
#[command(version)]
struct Args {
    /// brainf*ck source code file
    #[arg(short = 's', long = "source", value_name = "FILE")]
    source_file: String,

    /// show debug info
    #[arg(long, hide = true)]
    debug: bool,
}

fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    source: String,
    debug: bool,
) -> Result<(), B::Error> {
    let speed = Arc::new(RwLock::new(Speed::Normal));
    let (_, rx) = event::new(speed.clone());
    App::new(source, speed, debug).start(terminal, rx)
}

fn read_source_file(file: &str) -> std::io::Result<String> {
    let mut f = File::open(file)?;
    let mut source = String::new();
    f.read_to_string(&mut source)?;
    Ok(source)
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let source = read_source_file(&args.source_file)?;

    let mut terminal = ratatui::init();
    let ret = run(&mut terminal, source, args.debug);

    ratatui::restore();
    ret
}
