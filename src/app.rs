use std::sync::mpsc;

use crossterm::event::KeyCode;
use itsuki::zero_indexed_enum;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{event::AppEvent, interpreter::Interpreter, widget::Memory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Stop,
    Play,
    AutoPlay,
    Pause,
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
    ]
}

pub struct App {
    state: State,
    selected: SelectItem,
    source: String,
    input: String,
    interpreter: Interpreter,
}

impl App {
    pub fn new() -> App {
        // fixme
        let source = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
        let input = String::new();
        let interpreter = Interpreter::new(&source, &input);
        App {
            state: State::Stop,
            selected: SelectItem::Source,
            source,
            input,
            interpreter,
        }
    }

    fn reset_interpreter(&mut self) {
        self.interpreter = Interpreter::new(&self.source, &self.input)
    }

    pub fn start<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        rx: mpsc::Receiver<AppEvent>,
    ) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            match rx.recv().unwrap() {
                AppEvent::Key(key) => match key.code {
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char(c) => match c {
                        'j' => {
                            self.selected = self.selected.next();
                        }
                        'k' => {
                            self.selected = self.selected.prev();
                        }
                        _ => {}
                    },
                    KeyCode::Enter => match (self.state, self.selected) {
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
                                self.state = State::Pause;
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
                },
                AppEvent::Resize(_, _) => {}
                AppEvent::Tick => {
                    if self.state == State::AutoPlay {
                        if self.interpreter.end() {
                            self.state = State::Stop;
                        } else {
                            self.interpreter.step();
                        }
                    }
                }
            }
        }
    }

    fn render(&self, f: &mut Frame) {
        let chunks = Layout::new(
            Direction::Vertical,
            vec![
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ],
        )
        .split(f.size());

        let header = Paragraph::new("termbf").centered().block(
            Block::default().borders(Borders::NONE).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        f.render_widget(header, chunks[0]);

        let source = self.source_text();
        let source_area = Paragraph::new(source)
            .style(self.get_content_base_style(SelectItem::Source))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title("Source")
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Source)),
            );
        f.render_widget(source_area, chunks[1]);

        let input = self.interpreter.input();
        let input_area = Paragraph::new(input)
            .style(self.get_content_base_style(SelectItem::Input))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title("Input")
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Input)),
            );
        f.render_widget(input_area, chunks[2]);

        let mem = self.interpreter.memory();
        let cur_ptr = if self.state == State::Stop {
            None
        } else {
            Some(self.interpreter.current_ptr())
        };
        let memory = Memory::new(mem, cur_ptr)
            .style(self.get_content_base_style(SelectItem::Memory))
            .block(
                Block::bordered()
                    .title("Memory")
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Memory)),
            );
        f.render_widget(memory, chunks[3]);

        let output = self.interpreter.output();
        let output_area = Paragraph::new(output)
            .style(self.get_content_base_style(SelectItem::Output))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title("Output")
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Output)),
            );
        f.render_widget(output_area, chunks[4]);

        let chunks = Layout::new(
            Direction::Horizontal,
            vec![
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Min(0),
            ],
        )
        .split(chunks[5]);

        let reset_button = Paragraph::new("Reset")
            .style(self.get_content_base_style(SelectItem::Reset))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Reset)),
            );
        f.render_widget(reset_button, chunks[0]);

        let start_button = Paragraph::new("Start")
            .style(self.get_content_base_style(SelectItem::Start))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Start)),
            );
        f.render_widget(start_button, chunks[1]);

        let pause_button = Paragraph::new("Pause")
            .style(self.get_content_base_style(SelectItem::Pause))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Pause)),
            );
        f.render_widget(pause_button, chunks[2]);

        let step_button = Paragraph::new("Step")
            .style(self.get_content_base_style(SelectItem::Step))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1))
                    .style(self.get_block_style(SelectItem::Step)),
            );
        f.render_widget(step_button, chunks[3]);
    }

    fn source_text(&self) -> Text {
        let base_style = Style::default().fg(Color::Reset);
        let cur_lp = self.interpreter.current_line_and_pos();

        if self.state == State::Stop || cur_lp.is_none() {
            return Text::from(Line::styled(&self.source, base_style));
        }

        let cur_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let (cur_line, cur_pos) = cur_lp.unwrap();
        let lines: Vec<Line> = self
            .source
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == cur_line {
                    let mut cs = line.chars();
                    let init = cs.by_ref().take(cur_pos).collect::<String>();
                    let cur = cs.next().unwrap();
                    let tail = cs.collect::<String>();
                    Line::from(vec![
                        Span::styled(init, base_style),
                        Span::styled(cur.to_string(), cur_style),
                        Span::styled(tail, base_style),
                    ])
                } else {
                    Line::styled(line, base_style)
                }
            })
            .collect();
        Text::from(lines)
    }

    fn get_content_base_style(&self, item: SelectItem) -> Style {
        self.get_style_base(item, Color::Reset)
    }

    fn get_block_style(&self, item: SelectItem) -> Style {
        self.get_style_base(item, Color::Yellow)
    }

    fn get_style_base(&self, item: SelectItem, selected_color: Color) -> Style {
        match self.state {
            State::Stop => {
                if self.selected == item {
                    Style::default().fg(selected_color)
                } else {
                    Style::default().fg(Color::Reset)
                }
            }
            State::Play | State::AutoPlay | State::Pause => {
                if self.selected == item {
                    Style::default().fg(selected_color)
                } else {
                    Style::default().fg(Color::DarkGray)
                }
            }
        }
    }
}
