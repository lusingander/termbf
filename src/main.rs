mod app;
mod event;
mod interpreter;
mod macros;
mod ui;
mod widget;

use std::{
    fs::File,
    io::{stdout, Read, Stdout},
    panic,
    sync::{Arc, RwLock},
};

use app::Speed;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::App;

/// termbf - Terminal Brainf*ck visualizer
#[derive(Parser)]
#[command(version)]
struct Args {
    /// brainf*ck source code file
    #[arg(short = 's', long = "source", value_name = "FILE")]
    source_file: String,
}

fn setup() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

fn shutdown() -> std::io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn initialize_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        shutdown().unwrap();
        original_hook(panic_info);
    }));
}

fn run<B: Backend>(terminal: &mut Terminal<B>, source: String) -> std::io::Result<()> {
    let speed = Arc::new(RwLock::new(Speed::Normal));
    let (_, rx) = event::new(speed.clone());
    App::new(source, speed).start(terminal, rx)
}

fn read_source_file(file: &str) -> std::io::Result<String> {
    let mut f = File::open(file)?;
    let mut source = String::new();
    f.read_to_string(&mut source)?;
    Ok(source)
}

fn main() -> std::io::Result<()> {
    initialize_panic_handler();

    let args = Args::parse();
    let source = read_source_file(&args.source_file)?;

    let mut terminal = setup()?;
    let ret = run(&mut terminal, source);

    shutdown()?;
    ret
}
