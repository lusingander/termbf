use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{block::BlockExt, Block, Widget, WidgetRef},
};

pub struct Memory<'a> {
    block: Option<Block<'a>>,
    style: Style,
    memory: &'a Vec<u8>,
    current_ptr: Option<u8>,
}

impl<'a> Memory<'a> {
    pub fn new(memory: &'a Vec<u8>, current_ptr: Option<u8>) -> Memory<'a> {
        Memory {
            block: None,
            style: Style::default(),
            memory,
            current_ptr,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Memory<'a> {
        self.block = Some(block);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Memory<'a> {
        self.style = style.into();
        self
    }
}

impl<'a> Widget for Memory<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.render_ref(area, buf);
        let inner = self.block.inner_if_some(area);
        self.render_memory(inner, buf);
    }
}

impl Memory<'_> {
    fn render_memory(&self, text_area: Rect, buf: &mut Buffer) {
        let x = text_area.x;
        let y = text_area.y;

        let ms = self
            .memory
            .iter()
            .flat_map(|m| format!("{:>02X}│", m).chars().collect::<Vec<_>>())
            .take(text_area.width as usize)
            .collect::<String>();
        buf.set_string(x, y, ms, self.style);

        if let Some(cur_ptr) = self.current_ptr {
            let cur_ptr_area = Rect::new(x + (cur_ptr as u16) * 3, y, 2, 1);
            let cur_ptr_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED);
            buf.set_style(cur_ptr_area, cur_ptr_style)
        }
    }
}
