mod chat;
mod input;
mod sidebar;
mod status;

use crate::app::App;
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(area);

    status::draw(f, chunks[0], app);

    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(20),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(chunks[1]);

    sidebar::draw(f, main_chunks[0], app);
    chat::draw(f, main_chunks[1], app);

    input::draw(f, chunks[2], app);
}
