use crate::app::{App, Focus, InputMode};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let input_style = if matches!(app.focus, Focus::Input) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let mode_indicator = match app.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Insert => "INSERT",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Input [{mode_indicator}] "))
        .border_style(input_style);
    let paragraph = Paragraph::new(app.input.as_str()).block(block);
    f.render_widget(paragraph, area);
}
