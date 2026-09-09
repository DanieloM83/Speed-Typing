use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAction {
    Select,
    Discard,
    Navigate,
    None,
}

/// Moves `cursor` by `delta`, clamped to the valid index range for `len` items.
pub(super) fn move_cursor(cursor: &mut usize, len: usize, delta: i32) -> SelectionAction {
    if len != 0 {
        *cursor = (*cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
    }
    SelectionAction::Navigate
}

/// Renders a horizontal, `" | "`-separated list of items, styling each one
/// based on whether it's "marked" (checked, or the selected radio option)
/// and/or currently under the cursor.
pub(super) fn render_items(
    items: &[String],
    is_marked: impl Fn(usize) -> bool,
    cursor: usize,
    focused: bool,
    area: Rect,
    buf: &mut Buffer,
) {
    let marked_style = Style::default().yellow().on_black().bold();
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
    for (i, item) in items.iter().enumerate() {
        let style = match (is_marked(i), i == cursor) {
            (true, true) => marked_style.patch(cursor_style),
            (true, false) => marked_style,
            (false, true) => cursor_style,
            (false, false) => Style::default().white(),
        };

        spans.push(Span::styled(item.clone(), style));
        if i + 1 < items.len() {
            spans.push(if focused {
                separator.red()
            } else {
                separator.white()
            });
        }
    }

    Paragraph::new(Line::from(spans))
        .block(Block::bordered().border_style(border_style))
        .render(area, buf);
}
