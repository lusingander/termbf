use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{block::BlockExt, Block, Widget},
};

const DIVIDER: &str = "│";

pub struct Memory<'a> {
    block: Option<Block<'a>>,
    style: Style,
    ptr_style: Style,
    memory: &'a Vec<u8>,
    current_ptr: Option<u8>,
}

impl<'a> Memory<'a> {
    pub fn new(memory: &'a Vec<u8>, current_ptr: Option<u8>) -> Memory<'a> {
        Memory {
            block: None,
            style: Style::default(),
            ptr_style: Style::default(),
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

    pub fn ptr_style<S: Into<Style>>(mut self, style: S) -> Memory<'a> {
        self.ptr_style = style.into();
        self
    }
}

impl Widget for Memory<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);
        let inner = self.block.inner_if_some(area);
        self.render_memory(inner, buf);
    }
}

impl Memory<'_> {
    fn render_memory(&self, text_area: Rect, buf: &mut Buffer) {
        let x = text_area.x;
        let y = text_area.y;

        let ms = self.memories_str(text_area.width as usize);
        buf.set_string(x, y, ms, self.style);

        if let Some(cur_ptr) = self.current_ptr {
            let cur_ptr_area = Rect::new(x + (cur_ptr as u16) * 3, y, 2, 1);
            buf.set_style(cur_ptr_area, self.ptr_style)
        }
    }

    fn memories_str(&self, w: usize) -> String {
        self.memory
            .iter()
            .flat_map(|m| format!("{m:>02X}{DIVIDER}").chars().collect::<Vec<_>>())
            .take(w)
            .collect()
    }
}
