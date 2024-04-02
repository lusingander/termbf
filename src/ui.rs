use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, EditState, SelectItem, Speed, State},
    widget::memory::Memory,
};

const APP_COLOR: Color = Color::Yellow;
const DEFAULT_COLOR: Color = Color::Reset;
const DISABLED_COLOR: Color = Color::DarkGray;

pub fn render(f: &mut Frame, app: &App) {
    let debug_area_length = if app.debug { 1 } else { 0 };
    let chunks = Layout::new(
        Direction::Vertical,
        vec![
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(debug_area_length),
        ],
    )
    .split(f.size());

    render_header(f, chunks[0]);
    render_outputs(f, chunks[1], app);
    render_controls(f, chunks[2], app);
    render_help(f, chunks[3], app);
    render_debug_info(f, chunks[4], app);
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

    let input = input_content(app, chunks[1]);
    let input_area = build_textarea(app, "Input", input, SelectItem::Input);
    f.render_widget(input_area, chunks[1]);
    set_input_cursor(f, app, chunks[1]);

    let mem = app.interpreter.memory();
    let memory = build_memory(app, "Memory", mem, SelectItem::Memory);
    f.render_widget(memory, chunks[2]);

    let output = output_content(app, chunks[3]);
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

fn render_help(f: &mut Frame, area: Rect, app: &App) {
    let help = build_help(app);
    f.render_widget(help, area);
}

fn render_debug_info(f: &mut Frame, area: Rect, app: &App) {
    if app.debug {
        let debug_info = build_debug_info(app);
        f.render_widget(debug_info, area);
    }
}

fn source_text(app: &App) -> Text {
    let base_style = if app.selected == SelectItem::Source {
        Style::default().fg(DEFAULT_COLOR)
    } else {
        Style::default().fg(DISABLED_COLOR)
    };
    let cur_lp = app.interpreter.current_line_and_pos();

    if app.state == State::Stop || cur_lp.is_none() {
        let lines: Vec<Line> = app
            .source
            .iter()
            .skip(app.source_scroll_offset)
            .map(|line| Line::styled(line, base_style))
            .collect();
        return Text::from(lines);
    }

    let cur_style = Style::default().fg(APP_COLOR).add_modifier(Modifier::BOLD);

    let (cur_line, cur_pos) = cur_lp.unwrap();
    let lines: Vec<Line> = app
        .source
        .iter()
        .enumerate()
        .skip(app.source_scroll_offset)
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

fn input_content(app: &App, area: Rect) -> &str {
    let input = if app.interpreter.running() {
        app.interpreter.input()
    } else {
        app.input_input.value()
    };

    if app.edit_state == EditState::EditInput {
        let max_width = area.width - 4 /* border + padding */;
        let input_start = input.len().saturating_sub(max_width as usize);
        &input[input_start..]
    } else {
        input
    }
}

fn set_input_cursor(f: &mut Frame, app: &App, area: Rect) {
    if app.edit_state == EditState::EditInput {
        let visual_cursor = app.input_input.visual_cursor() as u16;
        let max_width = area.width - 4 /* border + padding */;
        let cursor_x = area.x + 2 /* border + padding */ + visual_cursor.min(max_width);
        let cursor_y = area.y + 1 /* border */;
        f.set_cursor(cursor_x, cursor_y);
    }
}

fn output_content(app: &App, area: Rect) -> &str {
    let output = app.interpreter.output();

    let max_width = area.width - 4 /* border + padding */;
    let output_start = output.len().saturating_sub(max_width as usize);
    &output[output_start..]
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
        State::Play | State::AutoPlay => {
            if app.selected == item {
                Style::default().fg(selected_color)
            } else {
                Style::default().fg(DISABLED_COLOR)
            }
        }
    }
}

fn build_help(app: &App) -> Paragraph {
    let help = match app.selected {
        SelectItem::Source => "<Esc> quit app, <Tab/BackTab> next/prev, <j/k> scroll",
        SelectItem::Input => {
            if app.edit_state == EditState::EditInput {
                "<Esc> exit editing"
            } else if app.state != State::Stop || app.interpreter.end() {
                "<Esc> quit app, <Tab/BackTab> next/prev"
            } else {
                "<Esc> quit app, <Tab/BackTab> next/prev, <e> enter editing"
            }
        }
        SelectItem::Memory => "<Esc> quit app, <Tab/BackTab> next/prev",
        SelectItem::Output => "<Esc> quit app, <Tab/BackTab> next/prev",
        SelectItem::Reset => {
            if app.state != State::Stop || app.interpreter.end() {
                "<Esc> quit app, <Tab/BackTab> next/prev, <Enter> reset"
            } else {
                "<Esc> quit app, <Tab/BackTab> next/prev"
            }
        }
        SelectItem::Start => {
            if app.state == State::AutoPlay || app.interpreter.end() {
                "<Esc> quit app, <Tab/BackTab> next/prev"
            } else {
                "<Esc> quit app, <Tab/BackTab> next/prev, <Enter> start"
            }
        }
        SelectItem::Pause => {
            if app.state == State::AutoPlay {
                "<Esc> quit app, <Tab/BackTab> next/prev, <Enter> pause"
            } else {
                "<Esc> quit app, <Tab/BackTab> next/prev"
            }
        }
        SelectItem::Step => {
            if app.interpreter.end() {
                "<Esc> quit app, <Tab/BackTab> next/prev"
            } else {
                "<Esc> quit app, <Tab/BackTab> next/prev, <Enter> step"
            }
        }
        SelectItem::Speed => "<Esc> quit app, <Tab/BackTab> next/prev, <j/k> select",
    };
    Paragraph::new(help)
        .style(Style::default().fg(DISABLED_COLOR))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .padding(Padding::horizontal(1)),
        )
}

fn build_debug_info(app: &App) -> Paragraph {
    let i = &app.interpreter;
    let debug = format!(
        "pos = {:?}, ptr = {:?}, total_step = {:?}, state = {:?}",
        i.current_line_and_pos(),
        i.current_ptr(),
        i.total_step_count(),
        app.state,
    );
    Paragraph::new(debug)
        .style(Style::default().fg(DISABLED_COLOR))
        .block(Block::default().padding(Padding::horizontal(1)))
}
