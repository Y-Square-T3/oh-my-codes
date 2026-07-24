use crate::app::{App, Focus};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chat_style = if matches!(app.focus, Focus::Chat) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Chat ")
        .border_style(chat_style);
    let text: String = app
        .messages
        .iter()
        .map(|m| format!("[{}] {}: {}", m.timestamp, m.author_id, m.content))
        .collect::<Vec<_>>()
        .join("\n");
    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}
