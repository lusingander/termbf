use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};

use ratatui::crossterm::event::{Event, KeyEvent};

use crate::app::Speed;

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub fn new(speed: Arc<RwLock<Speed>>) -> (mpsc::Sender<AppEvent>, mpsc::Receiver<AppEvent>) {
    let (tx, rx) = mpsc::channel();

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match ratatui::crossterm::event::read().unwrap() {
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
        let d = {
            let s = speed.read().unwrap();
            s.sleep_duration()
        };
        thread::sleep(d);
        tick_tx.send(AppEvent::Tick).unwrap();
    });

    (tx, rx)
}
