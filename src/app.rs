use std::{
    sync::{mpsc, Arc, RwLock},
    time::Duration,
};

use crossterm::event::{Event, KeyCode, KeyEvent};
use itsuki::zero_indexed_enum;
use ratatui::{backend::Backend, Terminal};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{event::AppEvent, interpreter::Interpreter, key_code, key_code_char, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Stop,
    Play,
    AutoPlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditState {
    None,
    EditInput,
}

zero_indexed_enum! {
    SelectItem => [
        Source,
        Input,
        Memory,
        Output,
        Reset,
        Start,
        Pause,
        Step,
        Speed,
    ]
}

zero_indexed_enum! {
    Speed => [
        VerySlow,
        Slow,
        Normal,
        Fast,
        VeryFast,
    ]
}

impl Speed {
    pub fn sleep_duration(&self) -> Duration {
        match self {
            Speed::VerySlow => Duration::from_millis(500),
            Speed::Slow => Duration::from_millis(200),
            Speed::Normal => Duration::from_millis(100),
            Speed::Fast => Duration::from_millis(50),
            Speed::VeryFast => Duration::from_millis(20),
        }
    }
}

pub struct App {
    pub state: State,
    pub edit_state: EditState,
    pub selected: SelectItem,
    pub source: Vec<String>,
    pub source_scroll_offset: usize,
    pub input_input: Input,
    pub interpreter: Interpreter,
    pub speed: Arc<RwLock<Speed>>,
    pub debug: bool,
    quit: bool,
}

impl App {
    pub fn new(source: String, speed: Arc<RwLock<Speed>>, debug: bool) -> App {
        let input_input = Input::default();
        let interpreter = Interpreter::new(&source, input_input.value());
        let source = source.lines().map(|s| s.to_string()).collect();
        App {
            state: State::Stop,
            edit_state: EditState::None,
            selected: SelectItem::Source,
            source,
            source_scroll_offset: 0,
            input_input,
            interpreter,
            speed,
            debug,
            quit: false,
        }
    }

    pub fn start<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        rx: mpsc::Receiver<AppEvent>,
    ) -> std::io::Result<()> {
        while !self.quit {
            terminal.draw(|f| ui::render(f, self))?;

            match rx.recv().unwrap() {
                AppEvent::Key(key) => {
                    self.handle_key_event(key);
                }
                AppEvent::Resize(w, h) => {
                    self.handle_resize(w, h);
                }
                AppEvent::Tick => {
                    self.handle_tick();
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.edit_state == EditState::EditInput {
            match key {
                key_code_char!('c', Ctrl) => {
                    self.quit = true;
                }
                key_code!(KeyCode::Esc) => {
                    self.edit_state = EditState::None;
                    self.interpreter
                        .set_input(self.input_input.value().to_owned());
                }
                _ => {
                    self.input_input.handle_event(&Event::Key(key));
                }
            }
            return;
        }

        match key {
            key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => {
                self.quit = true;
            }
            key_code!(KeyCode::Tab) => {
                self.selected = self.selected.next();
            }
            key_code!(KeyCode::BackTab) => {
                self.selected = self.selected.prev();
            }
            key_code_char!('j') => match self.selected {
                SelectItem::Source => {
                    if self.source_scroll_offset < self.source.len() - 1 {
                        self.source_scroll_offset = self.source_scroll_offset.saturating_add(1);
                    }
                }
                SelectItem::Speed => {
                    let mut s = self.speed.write().unwrap();
                    *s = s.next();
                }
                _ => {}
            },
            key_code_char!('k') => match self.selected {
                SelectItem::Source => {
                    if self.source_scroll_offset > 0 {
                        self.source_scroll_offset = self.source_scroll_offset.saturating_sub(1);
                    }
                }
                SelectItem::Speed => {
                    let mut s = self.speed.write().unwrap();
                    *s = s.prev();
                }
                _ => {}
            },
            key_code_char!('e') => {
                if let (State::Stop, SelectItem::Input) = (self.state, self.selected) {
                    if !self.interpreter.end() {
                        self.edit_state = EditState::EditInput;
                        self.state = State::Stop;
                        self.reset_interpreter();
                    }
                }
            }
            key_code!(KeyCode::Enter) => match (self.state, self.selected) {
                (_, SelectItem::Reset) => {
                    self.state = State::Stop;
                    self.reset_interpreter();
                }
                (_, SelectItem::Start) => {
                    if self.interpreter.end() {
                        self.state = State::Stop;
                    } else {
                        self.state = State::AutoPlay;
                    }
                }
                (_, SelectItem::Pause) => {
                    if self.interpreter.end() {
                        self.state = State::Stop;
                    } else {
                        self.state = State::Play;
                    }
                }
                (_, SelectItem::Step) => {
                    if self.interpreter.end() {
                        self.state = State::Stop;
                    } else {
                        if self.state != State::Stop {
                            self.interpreter.step();
                        }
                        self.state = State::Play;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_resize(&mut self, _w: u16, _h: u16) {}

    fn handle_tick(&mut self) {
        if self.state == State::AutoPlay {
            if self.interpreter.end() {
                self.state = State::Stop;
            } else {
                self.interpreter.step();
            }
        }
    }

    fn reset_interpreter(&mut self) {
        let source = self.source.join("\n");
        self.interpreter = Interpreter::new(&source, self.input_input.value())
    }
}
