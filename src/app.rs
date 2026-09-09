use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use indexmap::IndexMap;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Widget},
};

use std::{
    io,
    sync::LazyLock,
    time::{Duration, Instant},
};

use crate::widgets::{CheckboxGroup, Game, GameExtra, GameMode, RadioGroup, SelectionAction};

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS);

static GAME_EXTRAS: LazyLock<IndexMap<String, GameExtra>> = LazyLock::new(|| {
    IndexMap::from([
        ("punctuation".to_string(), GameExtra::Punctuation),
        ("numbers".to_string(), GameExtra::Numbers),
    ])
});

static GAME_MODES: LazyLock<IndexMap<String, GameMode>> = LazyLock::new(|| {
    IndexMap::from([
        ("time".to_string(), GameMode::Time),
        ("words".to_string(), GameMode::Words),
    ])
});

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

#[derive(Debug, PartialEq, Eq)]
enum State {
    Idle,
    Game,
}

#[derive(Debug)]
pub struct App {
    is_running: bool,
    focus: Focus,
    state: State,

    extra_box: CheckboxGroup,
    mode_box: RadioGroup,
    value_box: RadioGroup,
    game: Game,
}
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_frame = Instant::now();

        while self.is_running {
            let frame_start = Instant::now();
            let dt = frame_start.duration_since(last_frame).as_secs_f32();
            last_frame = frame_start;

            self.update(dt)?;
            terminal.draw(|frame| self.draw(frame))?;

            // Wait for input for whatever time remains in this frame's budget.
            // If update+draw already ate the whole budget, this returns immediately.
            let timeout = FRAME_DURATION.saturating_sub(frame_start.elapsed());
            self.handle_events(timeout)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn update(&mut self, dt: f32) -> io::Result<()> {
        self.game.update(dt);
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
        // Handle global key events (like tab, esc, etc.)
        match (event.code, &self.state, self.focus) {
            (KeyCode::Char('c'), _, _) if event.modifiers == KeyModifiers::CONTROL => {
                self.exit();
                return;
            }
            (KeyCode::Tab, State::Idle, _) => {
                self.set_focus(self.focus.next());
                return;
            }
            (KeyCode::BackTab, State::Idle, _) => {
                self.set_focus(self.focus.prev());
                return;
            }
            (KeyCode::Esc, State::Idle, Focus::Main) => {
                self.exit();
                return;
            }
            (KeyCode::Esc, State::Game, Focus::Main) => {
                self.state = State::Idle;
                self.game.reset();
                return;
            }
            (KeyCode::Esc, State::Idle, _) => {
                self.set_focus(Focus::Main);
                return;
            }
            _ => {}
        }

        // If there weren't any global key event - pass to the child components based on the current focus
        let action: Option<SelectionAction> = match self.focus {
            Focus::Extra => Some(self.extra_box.handle_key_event(event)),
            Focus::Mode => Some(self.mode_box.handle_key_event(event)),
            Focus::Value => Some(self.value_box.handle_key_event(event)),
            Focus::Main if self.state == State::Idle => {
                self.state = State::Game;
                self.game.start();
                self.game.handle_key_event(event);
                None
            }
            Focus::Main => {
                self.game.handle_key_event(event);
                None
            }
        };

        if action == Some(SelectionAction::Select) {
            self.update_settings();
        }
    }

    fn update_settings(&mut self) {
        let extras: Vec<GameExtra> = self
            .extra_box
            .checked_keys()
            .filter_map(|key| GAME_EXTRAS.get(key))
            .copied()
            .collect();

        self.game.update_settings(
            extras,
            *GAME_MODES.get(self.mode_box.selected()).unwrap(),
            self.value_box.selected().parse::<usize>().unwrap_or(0),
        );
    }

    fn set_focus(&mut self, focus: Focus) {
        self.extra_box.discard();
        self.mode_box.discard();
        self.value_box.discard();

        self.focus = focus;
    }

    pub fn exit(&mut self) {
        self.is_running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            is_running: true,
            focus: Focus::Main,
            state: State::Idle,

            extra_box: CheckboxGroup::new(GAME_EXTRAS.keys().cloned().collect()),
            mode_box: RadioGroup::new(GAME_MODES.keys().cloned().collect()),
            value_box: RadioGroup::new(vec![
                "15".to_string(),
                "30".to_string(),
                "45".to_string(),
                "60".to_string(),
            ]),
            game: Game::new(),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = match self.focus {
            Focus::Main if self.state == State::Idle => Line::from(vec![
                " Navigate ".white(),
                "<Tab> ".blue(),
                " Exit ".white(),
                "<Esc> ".blue(),
                " Start typing to test your speed ".white(),
            ]),
            Focus::Main if self.state == State::Game => Line::from(vec![
                " Navigate ".white(),
                "<Tab> ".dark_gray(),
                " Stop ".white(),
                "<Esc> ".blue(),
            ]),
            Focus::Extra => Line::from(vec![
                " Navigate ".white(),
                "<Tab> ".blue(),
                " Select ".white(),
                "<⇆> ".blue(),
                " Check/Uncheck ".white(),
                "<Enter> ".blue(),
                " Discard ".white(),
                "<Esc> ".blue(),
            ]),
            Focus::Mode => Line::from(vec![
                " Navigate ".white(),
                "<Tab> ".blue(),
                " Select ".white(),
                "<⇆> ".blue(),
                " Confirm ".white(),
                "<Enter> ".blue(),
                " Discard ".white(),
                "<Esc> ".blue(),
            ]),
            Focus::Value => Line::from(vec![
                " Navigate ".white(),
                "<Tab> ".blue(),
                " Select ".white(),
                "<⇆> ".blue(),
                " Confirm ".white(),
                "<Enter> ".blue(),
                " Discard ".white(),
                "<Esc> ".blue(),
            ]),
            _ => Line::from(""),
        };

        let block = Block::bordered()
            .title(" Speed Typing ".red().bold())
            .title_bottom(instructions.centered())
            .style(Style::default().on_black())
            .border_style(if self.focus == Focus::Main {
                Style::default().red().on_black()
            } else {
                Style::default().on_black()
            });
        let inner = block.inner(area);

        block.render(area, buf);

        let [header, body] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(inner);

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

        self.game.render(body, buf);
    }
}
