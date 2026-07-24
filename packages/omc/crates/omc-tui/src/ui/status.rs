use crate::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let title = if let Some(ref err) = app.error {
        format!(" oh-my-codes - ERROR: {err} ")
    } else {
        " oh-my-codes ".to_string()
    };
    let block = Block::default().borders(Borders::NONE).title(title);
    let paragraph = Paragraph::new("").block(block);
    f.render_widget(paragraph, area);
}
