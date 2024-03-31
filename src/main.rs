mod app;
mod event;
mod interpreter;
mod macros;
mod ui;
mod widget;

use std::{
    io::{stdout, Stdout},
    panic,
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::App;

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

fn run<B: Backend>(terminal: &mut Terminal<B>) -> std::io::Result<()> {
    let (_, rx) = event::new();
    App::new().start(terminal, rx)
}

fn main() -> std::io::Result<()> {
    initialize_panic_handler();
    let mut terminal = setup()?;
    let ret = run(&mut terminal);
    shutdown()?;
    ret
}
