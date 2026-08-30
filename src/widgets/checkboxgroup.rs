use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect};

use super::select_group::{SelectionAction, move_cursor, render_items};

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

    pub fn discard(&mut self) -> SelectionAction {
        self.cursor = 0;
        SelectionAction::Discard
    }

    pub fn checked_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.checked
            .iter()
            .enumerate()
            .filter_map(|(i, &checked)| checked.then_some(i))
    }

    pub fn checked_keys(&self) -> impl Iterator<Item = &String> + '_ {
        self.checked_indices().map(|i| &self.items[i])
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) -> SelectionAction {
        match event.code {
            KeyCode::Left => move_cursor(&mut self.cursor, self.items.len(), -1),
            KeyCode::Right => move_cursor(&mut self.cursor, self.items.len(), 1),
            KeyCode::Enter => {
                self.checked[self.cursor] = !self.checked[self.cursor];
                SelectionAction::Select
            }
            KeyCode::Esc => self.discard(),
            _ => SelectionAction::None,
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
