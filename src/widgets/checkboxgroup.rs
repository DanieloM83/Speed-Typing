use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect};

use super::select_group::{move_cursor, render_items};

#[derive(Debug)]
pub struct CheckboxGroup {
    items: Vec<String>,

    checked: Vec<bool>,
    cursor: usize,
}

impl CheckboxGroup {
    pub fn new(items: Vec<String>) -> Self {
        let n = items.len();
        Self {
            items,
            checked: vec![false; n],
            cursor: 0,
        }
    }

    pub fn discard(&mut self) {
        self.cursor = 0;
    }

    pub fn checked_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.checked
            .iter()
            .enumerate()
            .filter_map(|(i, &checked)| checked.then_some(i))
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Left => move_cursor(&mut self.cursor, self.items.len(), -1),
            KeyCode::Right => move_cursor(&mut self.cursor, self.items.len(), 1),
            KeyCode::Enter => self.checked[self.cursor] = !self.checked[self.cursor],
            KeyCode::Esc => self.discard(),
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        render_items(
            &self.items,
            |i| self.checked[i],
            self.cursor,
            focused,
            area,
            buf,
        );
    }
}
