use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect};

use super::select_group::{move_cursor, render_items};

#[derive(Debug)]
pub struct RadioGroup {
    items: Vec<String>,

    selected: usize,
    cursor: usize,
}

impl RadioGroup {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: 0,
            cursor: 0,
        }
    }

    pub fn discard(&mut self) {
        self.cursor = self.selected;
    }

    pub fn selected(&self) -> &str {
        &self.items[self.selected]
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Left => move_cursor(&mut self.cursor, self.items.len(), -1),
            KeyCode::Right => move_cursor(&mut self.cursor, self.items.len(), 1),
            KeyCode::Enter => self.selected = self.cursor,
            KeyCode::Esc => self.discard(),
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        render_items(
            &self.items,
            |i| i == self.selected,
            self.cursor,
            focused,
            area,
            buf,
        );
    }
}
