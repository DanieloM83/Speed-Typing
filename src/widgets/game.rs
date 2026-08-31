use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::utils::generate_words;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Time,
    Words,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameExtra {
    Punctuation,
    Numbers,
}

#[derive(Debug, Default)]
pub struct GameStatistics {
    wpm: f32,
    cpm: f32,
    acc: f32,
    time: f32,

    errors: usize,
    words: usize,
    chars: usize,

    mode: GameMode,
    mode_value: usize,
    extras: Vec<GameExtra>,
}

#[derive(Debug)]
pub struct Game {
    target_words: Vec<String>,
    completed_words: Vec<String>,
    current_word: String,

    is_running: bool,

    statistics: GameStatistics,
}

impl Game {
    pub fn new() -> Self {
        let mut statistics = GameStatistics::default();
        statistics.mode_value = 15;
        Self {
            target_words: generate_words(300, false, false),
            completed_words: Vec::new(),
            current_word: String::new(),

            is_running: false,

            statistics: statistics,
        }
    }

    pub fn reset(&mut self) {
        let amount_to_generate = match self.statistics.mode {
            GameMode::Time => 300,
            GameMode::Words => self.statistics.mode_value,
        };

        let use_punctuation = self.statistics.extras.contains(&GameExtra::Punctuation);
        let use_numbers = self.statistics.extras.contains(&GameExtra::Numbers);

        self.target_words = generate_words(amount_to_generate, use_punctuation, use_numbers);

        self.completed_words = Vec::new();
        self.current_word = String::new();
        self.statistics.time = 0.0;
        self.is_running = false;
    }

    pub fn start(&mut self) {
        self.statistics.time = 0.0;
        self.is_running = true;
    }

    pub fn update_settings(&mut self, extras: Vec<GameExtra>, mode: GameMode, mode_value: usize) {
        self.statistics.extras = extras;
        self.statistics.mode = mode;
        self.statistics.mode_value = mode_value;

        self.reset()
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_running {
            return;
        }

        self.statistics.time += dt;

        if self.completed_words.len() >= self.target_words.len() {
            self.is_running = false;
        }

        self.is_running = !match self.statistics.mode {
            GameMode::Time => self.statistics.time >= self.statistics.mode_value as f32,
            GameMode::Words => self.completed_words.len() >= self.statistics.mode_value,
        }
    }

    /// The word the player is expected to be typing right now, if any.
    /// `None` once every target word has been completed.
    fn expected_word(&self) -> Option<&str> {
        self.target_words
            .get(self.completed_words.len())
            .map(String::as_str)
    }

    fn calculate_stats(&mut self) {
        if self.statistics.time <= 0.0 {
            return;
        }

        let minutes = self.statistics.time / 60.0;
        self.statistics.wpm = self.statistics.words as f32 / minutes;
        self.statistics.cpm = self.statistics.chars as f32 / minutes;

        let attempted_chars = self.statistics.chars + self.statistics.errors;
        self.statistics.acc = if attempted_chars == 0 {
            0.0
        } else {
            (self.statistics.chars as f32 / attempted_chars as f32) * 100.0
        };
    }

    fn process_char(&mut self, ch: char) {
        let Some(expected_word) = self.expected_word() else {
            return;
        };

        let typed_index = self.current_word.chars().count();
        if expected_word.chars().nth(typed_index) != Some(ch) {
            self.statistics.errors += 1;
        }

        self.current_word.push(ch);
    }

    fn process_space(&mut self) {
        if self.current_word.is_empty() {
            return;
        }

        if Some(self.current_word.as_str()) == self.expected_word() {
            self.statistics.words += 1;
            self.statistics.chars += self.current_word.chars().count();
        }

        self.completed_words
            .push(std::mem::take(&mut self.current_word));
        self.calculate_stats();
    }

    fn process_backspace(&mut self) {
        // Still editing the current word - pop a char.
        if self.current_word.pop().is_some() {
            return;
        }

        // Nothing to un-submit.
        let Some(last_completed) = self.completed_words.last() else {
            return;
        };

        // Last submitted word was correct - leave it alone, don't reopen it.
        let last_index = self.completed_words.len() - 1;
        if Some(last_completed.as_str()) == self.target_words.get(last_index).map(String::as_str) {
            return;
        }

        // Last submitted word was wrong - reopen it for editing.
        self.current_word = self.completed_words.pop().unwrap();
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if !self.is_running {
            return;
        }

        match event.code {
            KeyCode::Char(' ') => self.process_space(),
            KeyCode::Char(ch) => self.process_char(ch),
            KeyCode::Backspace => self.process_backspace(),
            _ => {}
        }
    }

    /// Renders a single target word alongside what the player typed for it.
    /// `is_current` marks the word the cursor is currently sitting in.
    fn render_word(
        &self,
        target: &str,
        typed: Option<&str>,
        is_current: bool,
    ) -> Vec<Span<'static>> {
        let Some(typed) = typed else {
            return target.chars().map(|c| c.dark_gray()).collect();
        };

        let target_chars: Vec<char> = target.chars().collect();
        let typed_chars: Vec<char> = typed.chars().collect();
        let is_wrong = target_chars != typed_chars;

        let mut correct_style = Style::new().green();
        let mut wrong_style = Style::new().red();
        let mut skipped_style = Style::new().dark_gray();
        let mut extra_style = Style::new().light_red();

        // A completed, incorrect word gets a red background to stand out
        // once the cursor has moved past it.
        if is_wrong && !is_current {
            correct_style = correct_style.on_red();
            wrong_style = wrong_style.on_red();
            skipped_style = skipped_style.on_red();
            extra_style = extra_style.on_red();
        }

        let max_len = target_chars.len().max(typed_chars.len());
        let cursor_position = typed_chars.len();

        let mut rendered = Vec::new();
        for i in 0..=max_len {
            if is_current && i == cursor_position {
                rendered.push("|".yellow());
            }

            match (target_chars.get(i), typed_chars.get(i)) {
                (Some(t), Some(u)) if t == u => {
                    rendered.push(Span::styled(t.to_string(), correct_style))
                }
                (Some(t), Some(_)) => rendered.push(Span::styled(t.to_string(), wrong_style)),
                (Some(t), None) => rendered.push(Span::styled(t.to_string(), skipped_style)),
                (None, Some(u)) => rendered.push(Span::styled(u.to_string(), extra_style)),
                (None, None) => {}
            }
        }

        rendered
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let [left, right] =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        let left_value = match self.statistics.mode {
            GameMode::Time => Span::from(
                self.statistics
                    .mode_value
                    .saturating_sub(self.statistics.time as usize)
                    .to_string(),
            ),
            GameMode::Words => Span::from(self.completed_words.len().to_string()),
        };

        Line::from(vec![left_value]).render(left, buf);
        Line::from(vec![
            "WPM: ".yellow().bold(),
            self.statistics.wpm.round().to_span(),
            "   ".to_span(),
            "CPM: ".yellow().bold(),
            self.statistics.cpm.round().to_span(),
            "   ".to_span(),
            "ACC: ".yellow().bold(),
            self.statistics.acc.round().to_span(),
        ])
        .render(right, buf);
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Length(3)])
            .areas(area.inner(Margin::new(6, 0)));

        let body_width = body.width as usize;
        let mut current_width: usize = 0;
        let mut current_line: usize = 0;

        let mut text = Line::from("");

        for (i, target_word) in self.target_words.iter().enumerate() {
            let user_word = if i < self.completed_words.len() {
                Some(self.completed_words[i].as_str())
            } else if i == self.completed_words.len() {
                Some(self.current_word.as_str())
            } else {
                None
            };

            let cursor = i == self.completed_words.len();
            let rendered = self.render_word(target_word, user_word, cursor);

            if i <= self.completed_words.len() {
                let word_width = rendered.len();

                if current_width + word_width > body_width {
                    current_line += 1;
                    current_width = 0;
                }

                current_width += word_width + 1;
            }

            text.extend(rendered);
            text.push_span(" ");
        }

        let offset = current_line.saturating_sub(1);

        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll((offset as u16, 0))
            .render(body, buf);

        self.render_header(header, buf);
    }
}
