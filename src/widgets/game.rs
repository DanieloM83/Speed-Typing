use std::{thread::current, time::Instant};

use color_eyre::owo_colors::colors::css::FloralWhite;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug)]
pub enum GameAction {
    None,
    CharTyped(char),
    Backspace,
    Finished,
    Reset,
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
}

#[derive(Debug)]
pub struct Game {
    target_text: Vec<String>,
    user_input: Vec<String>,
    last_word: String,
    start_time: Option<Instant>,

    rendered_user_words: Vec<Span<'static>>,
    rendered_target_words: Vec<Span<'static>>,

    statistics: GameStatistics,
}
impl Game {
    pub fn new() -> Self {
        let target_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas convallis magna vel turpis lobortis bibendum. Aenean consequat nisl ac augue lobortis ullamcorper. Cras elementum urna ut molestie venenatis. Mauris est eros, ullamcorper malesuada nulla sed, iaculis ultricies ligula. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Mauris egestas urna a mi pulvinar eleifend a ut eros. Vivamus in malesuada massa. Vestibulum pharetra arcu non enim dapibus, consectetur blandit eros posuere. Praesent imperdiet felis quis felis blandit posuere.";
        Self {
            target_text: target_text.split(" ").map(|s| s.to_string()).collect(),
            user_input: Vec::new(),
            last_word: "".to_string(),
            start_time: None,

            rendered_user_words: Vec::new(),
            rendered_target_words: target_text.split(" ").map(|s| s.dark_gray()).collect(),

            statistics: GameStatistics::default(),
        }
    }

    pub fn reset(&mut self) {
        self.target_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas convallis magna vel turpis lobortis bibendum. Aenean consequat nisl ac augue lobortis ullamcorper. Cras elementum urna ut molestie venenatis. Mauris est eros, ullamcorper malesuada nulla sed, iaculis ultricies ligula. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Mauris egestas urna a mi pulvinar eleifend a ut eros. Vivamus in malesuada massa. Vestibulum pharetra arcu non enim dapibus, consectetur blandit eros posuere. Praesent imperdiet felis quis felis blandit posuere.".split(" ").map(|s| s.to_string()).collect();
        self.user_input = Vec::new();
        self.last_word = "".to_string();
        self.start_time = None;
        self.statistics.time = 0.0;
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.statistics.time = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.statistics.time += dt;
    }

    pub fn calculate_stats(&mut self) {
        self.statistics.wpm = (self.statistics.words as f32) / (self.statistics.time / 60.0);
        self.statistics.cpm = (self.statistics.chars as f32) / (self.statistics.time / 60.0);
        self.statistics.acc = (self.statistics.chars as f32
            / (self.statistics.chars + self.statistics.errors) as f32)
            * 100.0;
    }

    fn process_char(&mut self, ch: char) {
        let n = self.last_word.chars().count();
        let current_word = self.target_text.get(self.user_input.len()).unwrap();

        if current_word.chars().nth(n) != Some(ch) {
            self.statistics.errors += 1;
        }

        self.last_word.push(ch);
    }

    fn process_space(&mut self) {
        if self.last_word.is_empty() {
            return;
        }

        if self.last_word == *self.target_text.get(self.user_input.len()).unwrap() {
            self.statistics.words += 1;
            self.statistics.chars += self.last_word.chars().count();
        }

        self.user_input.push(std::mem::take(&mut self.last_word));
        self.calculate_stats()
    }

    fn process_backspace(&mut self) {
        // Current word isn't empty - pop char
        if self.last_word.pop().is_some() {
            return;
        }

        // Previous word is correct - do nothing
        let n = self.user_input.len().saturating_sub(1);
        if self.user_input.last() == self.target_text.get(n) {
            return;
        }

        // Previous word is incorrect - edit
        if let Some(word) = self.user_input.pop() {
            self.last_word = word;
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(ch) if ch != ' ' => self.process_char(ch),
            KeyCode::Char(' ') => self.process_space(),
            KeyCode::Backspace => self.process_backspace(),
            _ => {}
        }
    }

    fn render_word(&self, target: &str, user: Option<&str>, cursor: bool) -> Vec<Span<'static>> {
        if user.is_none() {
            let mut rendered = Vec::new();
            for ch in target.chars() {
                rendered.push(ch.dark_gray());
            }
            return rendered;
        }

        let target: Vec<char> = target.chars().collect();
        let user: Vec<char> = user.unwrap_or("").chars().collect();

        let mut rendered = Vec::new();

        let cursor_pos = user.len();
        let n = target.len().max(user.len());

        let mut correct_style = Style::new().green();
        let mut wrong_style = Style::new().red();
        let mut skipped_style = Style::new().dark_gray();
        let mut over_style = Style::new().light_red();

        if target != user && !cursor {
            correct_style = correct_style.on_red();
            wrong_style = wrong_style.on_red();
            skipped_style = skipped_style.on_red();
            over_style = over_style.on_red();
        }

        for i in 0..=n {
            if cursor && i == cursor_pos {
                rendered.push("|".yellow());
            }

            match (target.get(i), user.get(i)) {
                (Some(t), Some(u)) if t == u => {
                    rendered.push(Span::styled(t.to_string(), correct_style))
                }
                (Some(t), Some(_)) => rendered.push(Span::styled(t.to_string(), wrong_style)),
                (Some(t), None) => rendered.push(Span::styled(t.to_string(), skipped_style)),
                (None, Some(u)) => rendered.push(Span::styled(u.to_string(), over_style)),
                (None, None) => {}
            }
        }

        rendered
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focused: bool) {
        let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Length(3)])
            .areas(area.inner(Margin::new(6, 0)));

        let [counter_container, stats_container] =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(header);

        let body_width = body.width as usize;
        let mut current_width: usize = 0;
        let mut current_line: usize = 0;

        let mut text = Line::from("");

        for (i, target_word) in self.target_text.iter().enumerate() {
            let user_word = if i < self.user_input.len() {
                Some(self.user_input[i].as_str())
            } else if i == self.user_input.len() {
                Some(self.last_word.as_str())
            } else {
                None
            };

            let cursor = i == self.user_input.len();
            let rendered = self.render_word(target_word, user_word, cursor);

            if i <= self.user_input.len() {
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
        .render(stats_container, buf);

        Line::from(vec![self.statistics.time.round().to_span()]).render(counter_container, buf);
    }
}
