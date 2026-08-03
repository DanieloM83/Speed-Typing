use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

pub enum RadioGroupAction {
    None,
    Selected,
    Discarded,
}

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

    fn move_cursor(&mut self, delta: i32) -> RadioGroupAction {
        if !self.items.is_empty() {
            self.cursor =
                (self.cursor as i32 + delta).clamp(0, self.items.len() as i32 - 1) as usize;
        }

        RadioGroupAction::None
    }

    fn select(&mut self) -> RadioGroupAction {
        self.selected = self.cursor;
        RadioGroupAction::Selected
    }

    pub fn discard(&mut self) -> RadioGroupAction {
        self.cursor = self.selected;
        RadioGroupAction::Discarded
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) -> RadioGroupAction {
        match event.code {
            KeyCode::Left => self.move_cursor(-1),
            KeyCode::Right => self.move_cursor(1),
            KeyCode::Enter => self.select(),
            KeyCode::Esc => self.discard(),
            _ => RadioGroupAction::None,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        let select_style = Style::default().yellow().on_black().bold();
        let cursor_style = if focused {
            Style::default().underlined()
        } else {
            Style::default()
        };
        let border_style = if focused {
            Style::default().red()
        } else {
            Style::default().white()
        };

        let separator = " | ";
        let mut spans: Vec<Span> = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            let style = if i == self.selected && i == self.cursor {
                select_style.patch(cursor_style)
            } else if i == self.selected {
                select_style
            } else if i == self.cursor {
                cursor_style
            } else {
                Style::default().white()
            };

            spans.push(Span::styled(item.clone(), style));
            if i < self.items.len() - 1 {
                spans.push(if focused {
                    separator.red()
                } else {
                    separator.white()
                });
            }
        }

        let paragraph =
            Paragraph::new(Line::from(spans)).block(Block::bordered().border_style(border_style));

        paragraph.render(area, buf)
    }
}
