use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Widget},
};

use std::{
    io,
    time::{Duration, Instant},
};

use crate::widgets::{CheckboxGroup, RadioGroup};

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Main,
    Extra,
    Mode,
    Value,
}
impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Main => Focus::Extra,
            Focus::Extra => Focus::Mode,
            Focus::Mode => Focus::Value,
            Focus::Value => Focus::Main,
        }
    }

    fn prev(self) -> Self {
        match self {
            Focus::Main => Focus::Value,
            Focus::Extra => Focus::Main,
            Focus::Mode => Focus::Extra,
            Focus::Value => Focus::Mode,
        }
    }
}

#[derive(Debug)]
pub struct App {
    is_running: bool,
    focus: Focus,

    extra_box: CheckboxGroup,
    mode_box: RadioGroup,
    value_box: RadioGroup,
}
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_time = Instant::now();

        while self.is_running {
            let now = Instant::now();
            let dt = now.duration_since(last_time);
            last_time = now;

            let dt_secs = dt.as_secs_f32();

            let timeout = FRAME_DURATION
                .checked_sub(now.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            self.handle_events(timeout)?;
            self.update(dt_secs)?;
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn update(&self, dt: f32) -> io::Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    self.handle_key_event(event)
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Tab => {
                self.set_focus(self.focus.next());
                return;
            }
            KeyCode::BackTab => {
                self.set_focus(self.focus.prev());
                return;
            }
            _ => {}
        }

        match self.focus {
            Focus::Extra => {
                self.extra_box.handle_key_event(event);
            }
            Focus::Mode => {
                self.mode_box.handle_key_event(event);
            }
            Focus::Value => {
                self.value_box.handle_key_event(event);
            }
            _ => {}
        };
    }

    fn set_focus(&mut self, focus: Focus) {
        self.extra_box.discard();
        self.mode_box.discard();
        self.value_box.discard();

        self.focus = focus;
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            is_running: true,
            focus: Focus::Main,

            extra_box: CheckboxGroup::new(vec!["punctuation".to_string(), "numbers".to_string()]),
            mode_box: RadioGroup::new(vec!["time".to_string(), "words".to_string()]),
            value_box: RadioGroup::new(vec![
                "15".to_string(),
                "30".to_string(),
                "45".to_string(),
                "60".to_string(),
            ]),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Speed Typing ".red())
            .border_style(if self.focus == Focus::Main {
                Style::default().yellow()
            } else {
                Style::default()
            });
        let inner = block.inner(area);

        block.render(area, buf);

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(inner);

        let [left, center, right] = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(header);

        self.extra_box.render(left, buf, self.focus == Focus::Extra);
        self.mode_box.render(center, buf, self.focus == Focus::Mode);
        self.value_box
            .render(right, buf, self.focus == Focus::Value);
    }
}
