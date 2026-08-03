use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

pub enum CheckboxGroupAction {
    None,
    Check,
    Uncheck,
    Discarded,
}

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

    fn move_cursor(&mut self, delta: i32) -> CheckboxGroupAction {
        if !self.items.is_empty() {
            self.cursor =
                (self.cursor as i32 + delta).clamp(0, self.items.len() as i32 - 1) as usize;
        }

        CheckboxGroupAction::None
    }

    fn toggle(&mut self) -> CheckboxGroupAction {
        self.checked[self.cursor] = !self.checked[self.cursor];

        if self.checked[self.cursor] {
            CheckboxGroupAction::Check
        } else {
            CheckboxGroupAction::Uncheck
        }
    }

    pub fn discard(&mut self) -> CheckboxGroupAction {
        self.cursor = 0;
        CheckboxGroupAction::Discarded
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) -> CheckboxGroupAction {
        match event.code {
            KeyCode::Left => self.move_cursor(-1),
            KeyCode::Right => self.move_cursor(1),
            KeyCode::Enter => self.toggle(),
            KeyCode::Esc => self.discard(),
            _ => CheckboxGroupAction::None,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        let checked_style = Style::default().yellow().on_black().bold();
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
            let style = if self.checked[i] && i == self.cursor {
                checked_style.patch(cursor_style)
            } else if self.checked[i] {
                checked_style
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
