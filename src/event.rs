use std::{sync::mpsc, thread, time::Duration};

use crossterm::event::{Event, KeyEvent};

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub fn new() -> (mpsc::Sender<AppEvent>, mpsc::Receiver<AppEvent>) {
    let (tx, rx) = mpsc::channel();

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match crossterm::event::read().unwrap() {
            Event::Key(ev) => {
                event_tx.send(AppEvent::Key(ev)).unwrap();
            }
            Event::Resize(w, h) => {
                event_tx.send(AppEvent::Resize(w, h)).unwrap();
            }
            _ => {}
        };
    });

    let tick_tx = tx.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(100));
        tick_tx.send(AppEvent::Tick).unwrap();
    });

    (tx, rx)
}
