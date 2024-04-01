use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, SelectItem, Speed, State},
    widget::memory::Memory,
};

const APP_COLOR: Color = Color::Yellow;
const DEFAULT_COLOR: Color = Color::Reset;
const DISABLED_COLOR: Color = Color::DarkGray;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        vec![
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ],
    )
    .split(f.size());

    render_header(f, chunks[0]);
    render_outputs(f, chunks[1], app);
    render_controls(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = build_header("termbf");
    f.render_widget(header, area);
}

fn render_outputs(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::new(
        Direction::Vertical,
        vec![
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ],
    )
    .split(area);

    let source = source_text(app);
    let source_area = build_textarea(app, "Source", source, SelectItem::Source);
    f.render_widget(source_area, chunks[0]);

    let input = app.interpreter.input();
    let input_area = build_textarea(app, "Input", input, SelectItem::Input);
    f.render_widget(input_area, chunks[1]);

    let mem = app.interpreter.memory();
    let memory = build_memory(app, "Memory", mem, SelectItem::Memory);
    f.render_widget(memory, chunks[2]);

    let output = app.interpreter.output();
    let output_area = build_textarea(app, "Output", output, SelectItem::Output);
    f.render_widget(output_area, chunks[3]);
}

fn render_controls(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::new(
        Direction::Horizontal,
        vec![
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(18),
            Constraint::Min(0),
        ],
    )
    .split(area);

    let reset_button = build_button(app, "Reset", SelectItem::Reset);
    f.render_widget(reset_button, chunks[0]);

    let start_button = build_button(app, "Start", SelectItem::Start);
    f.render_widget(start_button, chunks[1]);

    let pause_button = build_button(app, "Pause", SelectItem::Pause);
    f.render_widget(pause_button, chunks[2]);

    let step_button = build_button(app, "Step", SelectItem::Step);
    f.render_widget(step_button, chunks[3]);

    let speed_select = build_speed_select(app, SelectItem::Speed);
    f.render_widget(speed_select, chunks[4]);
}

fn source_text(app: &App) -> Text {
    let base_style = if app.selected == SelectItem::Source {
        Style::default().fg(DEFAULT_COLOR)
    } else {
        Style::default().fg(DISABLED_COLOR)
    };
    let cur_lp = app.interpreter.current_line_and_pos();

    if app.state == State::Stop || cur_lp.is_none() {
        return Text::from(Line::styled(&app.source, base_style));
    }

    let cur_style = Style::default().fg(APP_COLOR).add_modifier(Modifier::BOLD);

    let (cur_line, cur_pos) = cur_lp.unwrap();
    let lines: Vec<Line> = app
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

fn build_header(label: &str) -> Paragraph {
    Paragraph::new(label).centered().block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(APP_COLOR).add_modifier(Modifier::BOLD)),
    )
}

fn build_textarea<'a, T>(
    app: &'a App,
    label: &'a str,
    content: T,
    item: SelectItem,
) -> Paragraph<'a>
where
    T: Into<Text<'a>>,
{
    Paragraph::new(content)
        .style(get_content_base_style(app, item))
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .title(label)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, item)),
        )
}

fn build_memory<'a>(
    app: &'a App,
    label: &'a str,
    mem: &'a Vec<u8>,
    item: SelectItem,
) -> Memory<'a> {
    let cur_ptr = if app.state == State::Stop {
        None
    } else {
        Some(app.interpreter.current_ptr())
    };
    Memory::new(mem, cur_ptr)
        .style(get_content_base_style(app, item))
        .ptr_style(
            Style::default()
                .fg(APP_COLOR)
                .add_modifier(Modifier::UNDERLINED),
        )
        .block(
            Block::bordered()
                .title(label)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, item)),
        )
}

fn build_button<'a>(app: &'a App, label: &'a str, item: SelectItem) -> Paragraph<'a> {
    Paragraph::new(label)
        .style(get_style_base(app, item, APP_COLOR))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, item)),
        )
}

fn build_speed_select(app: &App, item: SelectItem) -> Paragraph {
    let label = {
        let s = app.speed.read().unwrap();
        match *s {
            Speed::VerySlow => "Speed: Very Slow",
            Speed::Slow => "Speed: Slow",
            Speed::Normal => "Speed: Normal",
            Speed::Fast => "Speed: Fast",
            Speed::VeryFast => "Speed: Very Fast",
        }
    };
    Paragraph::new(label)
        .style(get_style_base(app, item, APP_COLOR))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::symmetric(1, 1))
                .style(get_block_style(app, item)),
        )
}

fn get_content_base_style(app: &App, item: SelectItem) -> Style {
    get_style_base(app, item, DEFAULT_COLOR)
}

fn get_block_style(app: &App, item: SelectItem) -> Style {
    get_style_base(app, item, APP_COLOR)
}

fn get_style_base(app: &App, item: SelectItem, selected_color: Color) -> Style {
    match app.state {
        State::Stop => {
            if app.selected == item {
                Style::default().fg(selected_color)
            } else {
                Style::default().fg(DISABLED_COLOR)
            }
        }
        State::Play | State::AutoPlay | State::Pause => {
            if app.selected == item {
                Style::default().fg(selected_color)
            } else {
                Style::default().fg(DISABLED_COLOR)
            }
        }
    }
}
