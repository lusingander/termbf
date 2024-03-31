use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, SelectItem, State},
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
    let header = Paragraph::new("termbf").centered().block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(APP_COLOR).add_modifier(Modifier::BOLD)),
    );
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
    let source_area = Paragraph::new(source)
        .style(get_content_base_style(app, SelectItem::Source))
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .title("Source")
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Source)),
        );
    f.render_widget(source_area, chunks[0]);

    let input = app.interpreter.input();
    let input_area = Paragraph::new(input)
        .style(get_content_base_style(app, SelectItem::Input))
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .title("Input")
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Input)),
        );
    f.render_widget(input_area, chunks[1]);

    let mem = app.interpreter.memory();
    let cur_ptr = if app.state == State::Stop {
        None
    } else {
        Some(app.interpreter.current_ptr())
    };
    let memory = Memory::new(mem, cur_ptr)
        .style(get_content_base_style(app, SelectItem::Memory))
        .ptr_style(
            Style::default()
                .fg(APP_COLOR)
                .add_modifier(Modifier::UNDERLINED),
        )
        .block(
            Block::bordered()
                .title("Memory")
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Memory)),
        );
    f.render_widget(memory, chunks[2]);

    let output = app.interpreter.output();
    let output_area = Paragraph::new(output)
        .style(get_content_base_style(app, SelectItem::Output))
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .title("Output")
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Output)),
        );
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
            Constraint::Min(0),
        ],
    )
    .split(area);

    let reset_button = Paragraph::new("Reset")
        .style(get_content_base_style(app, SelectItem::Reset))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Reset)),
        );
    f.render_widget(reset_button, chunks[0]);

    let start_button = Paragraph::new("Start")
        .style(get_content_base_style(app, SelectItem::Start))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Start)),
        );
    f.render_widget(start_button, chunks[1]);

    let pause_button = Paragraph::new("Pause")
        .style(get_content_base_style(app, SelectItem::Pause))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Pause)),
        );
    f.render_widget(pause_button, chunks[2]);

    let step_button = Paragraph::new("Step")
        .style(get_content_base_style(app, SelectItem::Step))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(get_block_style(app, SelectItem::Step)),
        );
    f.render_widget(step_button, chunks[3]);
}

fn source_text(app: &App) -> Text {
    let base_style = Style::default().fg(DEFAULT_COLOR);
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
                Style::default().fg(DEFAULT_COLOR)
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
