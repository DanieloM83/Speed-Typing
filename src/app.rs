use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Gauge, List, ListItem, ListState, Paragraph, Widget},
};

use std::io;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Copy, Clone)]
enum Focus {
    Main,
    Language,
    Timer,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Main => Focus::Language,
            Focus::Language => Focus::Timer,
            Focus::Timer => Focus::Main,
        }
    }

    fn prev(self) -> Self {
        match self {
            Focus::Main => Focus::Timer,
            Focus::Language => Focus::Main,
            Focus::Timer => Focus::Language,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Window {
    Game,
    Main,
    LanguagePopup,
    TimerPopup,
}

#[derive(Debug)]
pub struct App {
    is_running: bool,
    counter: i32,

    focus: Focus,
    window: Window,

    language_selector: SelectorBox,
    timer_selector: SelectorBox,
    game_widget: Game,

    blocked: bool,
    last_played_time: Instant,
}
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_time = Instant::now();
        let fps = 10;
        let frame_duration = Duration::from_millis(1000 / fps);

        while self.is_running {
            let now = Instant::now();
            let dt = now.duration_since(last_time);
            last_time = now;

            let _dt_seconds = dt.as_secs_f32();

            self.blocked = now.duration_since(self.last_played_time).as_secs() < 1;

            let timeout = frame_duration
                .checked_sub(now.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(timeout)?;

            if self.window == Window::Game {
                if !self.game_widget.tick(_dt_seconds) {
                    self.game_widget.reset();
                    self.last_played_time = Instant::now();
                    self.window = Window::Main;
                };
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.window {
            Window::Main | Window::Game => frame.render_widget(self, frame.area()),
            Window::LanguagePopup => frame.render_widget(&self.language_selector, frame.area()),
            Window::TimerPopup => frame.render_widget(&self.timer_selector, frame.area()),
        }
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
        match self.window {
            Window::LanguagePopup => match self.language_selector.handle_key_event(event) {
                SelectorOutput::Selected(s) => {
                    self.window = Window::Main;
                    self.game_widget.update_language(s);
                }
                SelectorOutput::Discarded => self.window = Window::Main,
                _ => {}
            },
            Window::TimerPopup => match self.timer_selector.handle_key_event(event) {
                SelectorOutput::Selected(s) => {
                    self.window = Window::Main;
                    self.game_widget
                        .update_timer(s[0..s.len().saturating_sub(1)].parse::<f32>().unwrap());
                }
                SelectorOutput::Discarded => self.window = Window::Main,
                _ => {}
            },
            Window::Main => match event.code {
                KeyCode::Esc => self.exit(),
                KeyCode::Up => self.focus = self.focus.prev(),
                KeyCode::Down => self.focus = self.focus.next(),
                KeyCode::Enter => self.apply_focus(),
                KeyCode::Char(c) if !self.blocked => {
                    self.window = Window::Game;
                    self.process_char(c);
                }
                _ => {}
            },
            Window::Game => match event.code {
                KeyCode::Esc => {
                    self.game_widget.reset();
                    self.last_played_time = Instant::now();
                    self.window = Window::Main;
                }
                KeyCode::Char(c) => self.game_widget.process_char(c),
                KeyCode::Backspace => self.game_widget.process_backspace(),
                _ => {}
            },
        }
    }

    fn exit(&mut self) {
        self.is_running = false;
    }

    fn process_char(&mut self, c: char) {
        self.game_widget.process_char(c);
    }

    fn apply_focus(&mut self) {
        match self.focus {
            Focus::Language => self.window = Window::LanguagePopup,
            Focus::Timer => self.window = Window::TimerPopup,
            Focus::Main => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            is_running: true,
            counter: 129,

            focus: Focus::Main,
            window: Window::Main,

            language_selector: SelectorBox::new(vec![
                "english".to_string(),
                "spanish".to_string(),
                "russian".to_string(),
            ]),
            timer_selector: SelectorBox::new(vec![
                "60s".to_string(),
                "30s".to_string(),
                "15s".to_string(),
            ]),
            game_widget: Game::new(),

            blocked: true,
            last_played_time: Instant::now() - Duration::from_secs(1),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
            .margin(5)
            .split(area);

        let title = Line::from(" Typing Speed Test ".bold().blue());
        let instructions = Line::from(vec![
            " Settings".into(),
            if self.window == Window::Main {
                " <↑↓> ".blue().bold()
            } else {
                " <↑↓> ".dark_gray().bold()
            },
            " Select".into(),
            if self.window == Window::Main {
                " <Enter> ".blue().bold()
            } else {
                " <Enter> ".dark_gray().bold()
            },
            if self.window == Window::Main {
                " Exit".into()
            } else {
                " Stop".into()
            },
            " <Esc> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            format!("{:?}", self.focus).yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);

        self.game_widget.render(layout[0], buf);
    }
}

//
//
//
//
//
//
//
//

#[derive(Debug, Default)]
struct Game {
    text: String,
    typed: String,
    seconds: f32,
    max_seconds: f32,

    missed: u16,

    words_per_minute: f32,
    chars_per_minute: f32,
    accuracy: f32,
}
impl Game {
    fn new() -> Self {
        Self {
            text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
            typed: "".to_string(),
            seconds: 60.0,
            max_seconds: 60.0,
            missed: 0,
            words_per_minute: 0.0,
            chars_per_minute: 0.0,
            accuracy: 0.0,
        }
    }

    fn reset(&mut self) {
        self.seconds = self.max_seconds;
        self.typed.clear();
        self.missed = 0;
    }

    fn update_language(&mut self, language: String) {
        if language == "english" {
            self.text = "A diverse and rich experience in implementing planned objectives enables a wide range of (specialists) to participate in shaping the development model. Everyday practice shows that strengthening and developing the structure enables a wide range of (specialists) to participate in shaping the directions of progressive development.".to_string();
        } else if language == "spanish" {
            self.text = "La experiencia variada y rica en la ejecución de las tareas previstas permite que un amplio grupo de especialistas participe en la configuración del modelo de desarrollo. La práctica cotidiana demuestra que el fortalecimiento y el desarrollo de la estructura permiten que un amplio grupo de especialistas participe en la definición de las líneas de desarrollo progresivo.".to_string();
        } else if language == "russian" {
            self.text = "Разнообразный и богатый опыт реализация намеченных плановых заданий обеспечивает широкому кругу (специалистов) участие в формировании модели развития. Повседневная практика показывает, что укрепление и развитие структуры обеспечивает широкому кругу (специалистов) участие в формировании направлений прогрессивного развития.".to_string();
        }

        self.typed = "".to_string();
    }

    fn update_timer(&mut self, max_seconds: f32) {
        self.max_seconds = max_seconds;
        self.seconds = max_seconds;
    }

    fn process_char(&mut self, c: char) {
        let pos = self.typed.chars().count();
        if self.text.chars().nth(pos) != Some(c) {
            self.missed += 1;
        }

        self.typed.push(c);
    }

    fn process_backspace(&mut self) {
        let Some(last) = self.typed.pop() else {
            return;
        };

        let pos = self.typed.chars().count();

        if self.text.chars().nth(pos) != Some(last) {
            self.missed -= 1;
        }
    }

    fn tick(&mut self, dt: f32) -> bool {
        self.seconds -= dt;

        let typed = self.typed.chars().count();
        let elapsed = (self.max_seconds - self.seconds).max(0.001);

        self.chars_per_minute =
            (typed.saturating_sub(self.missed as usize) as f32) * 60.0 / elapsed;

        self.words_per_minute =
            (typed.saturating_sub(self.missed as usize) as f32 / 5.0) * 60.0 / elapsed;
        self.accuracy = (1.0 - (self.missed as f32 / typed as f32)) * 100.0;

        self.seconds > 0.0
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        let statistics_layout = Layout::horizontal(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(layout[4]);

        let line_words_per_minute = Line::from(vec![
            "words/min: ".yellow(),
            format!("{:.2}", self.words_per_minute).blue(),
        ])
        .centered();
        let line_chars_per_minute = Line::from(vec![
            "chars/min: ".yellow(),
            format!("{:.2}", self.chars_per_minute).blue(),
        ])
        .centered();
        let line_accuracy = Line::from(vec![
            "% accuracy: ".yellow(),
            format!("{:.2}", self.accuracy).blue(),
        ])
        .centered();

        line_words_per_minute.render(statistics_layout[0], buf);
        line_chars_per_minute.render(statistics_layout[1], buf);
        line_accuracy.render(statistics_layout[2], buf);

        let mut characters: Vec<Span> = vec![];

        for (i, c) in self.text.chars().enumerate() {
            if i >= self.typed.len() {
                characters.push(c.dark_gray());
            } else if Some(c) == self.typed.chars().nth(i) {
                characters.push(c.green());
            } else {
                let wrong_char = self.typed.chars().nth(i).unwrap();
                characters.push(if wrong_char == ' ' {
                    "_".red()
                } else {
                    wrong_char.red()
                });
            }
        }

        let cursor = self.typed.len();
        let before_width: usize = self
            .text
            .chars()
            .take(cursor)
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
            .sum();
        let current_width = self
            .text
            .chars()
            .nth(cursor)
            .and_then(UnicodeWidthChar::width)
            .unwrap_or(1);
        let cursor_center = before_width + current_width / 2;
        let padding = (area.width as usize / 2).saturating_sub(cursor_center);

        let progress = Gauge::default()
            .gauge_style(Style::new().green())
            .percent((self.seconds * 100.0 / self.max_seconds) as u16)
            .label(format!("{} s", self.seconds));

        let mut spans = Vec::new();
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
            spans.extend(characters);
        } else {
            let visible_left = cursor.saturating_sub(area.width as usize / 2);
            spans.extend_from_slice(&characters[visible_left..]);
        }

        let line = Line::from(spans);
        progress.render(layout[0], buf);
        line.render(layout[2], buf);
    }
}

//
//
//
//
//
//
enum SelectorOutput {
    None,
    Selected(String),
    Discarded,
}
#[derive(Debug)]
struct SelectorBox {
    items: Vec<String>,
    state: ListState,
}
impl SelectorBox {
    fn new(items: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self {
            items: items,
            state: state,
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> SelectorOutput {
        match event.code {
            KeyCode::Esc => SelectorOutput::Discarded,
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    SelectorOutput::Selected(self.items[i].clone())
                } else {
                    SelectorOutput::None
                }
            }
            KeyCode::Up => {
                self.move_selection(-1);
                SelectorOutput::None
            }
            KeyCode::Down => {
                self.move_selection(1);
                SelectorOutput::None
            }
            _ => SelectorOutput::None,
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0);
        let new = (current as i32 + delta).clamp(0, self.items.len() as i32 - 1) as usize;
        self.state.select(Some(new));
    }
}
impl Widget for &SelectorBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let style = if self.state.selected() == Some(i) {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(text.clone()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::bordered().title("Select item"))
            .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::BOLD));

        list.render(area, buf);
    }
}
