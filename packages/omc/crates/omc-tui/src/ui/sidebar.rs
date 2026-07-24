use crate::app::{App, Focus};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(50),
            ratatui::layout::Constraint::Percentage(50),
        ])
        .split(area);

    let repo_style = if matches!(app.focus, Focus::Repos) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let repo_block = Block::default()
        .borders(Borders::ALL)
        .title(" Repos ")
        .border_style(repo_style);
    let repo_items: Vec<ListItem> = app
        .repos
        .iter()
        .map(|r| ListItem::new(r.path.clone()))
        .collect();
    let mut repo_state = ListState::default();
    repo_state.select(Some(app.selected_repo));
    let repo_list = List::new(repo_items)
        .block(repo_block)
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_stateful_widget(repo_list, chunks[0], &mut repo_state);

    let channel_style = if matches!(app.focus, Focus::Channels) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let channel_block = Block::default()
        .borders(Borders::ALL)
        .title(" Channels ")
        .border_style(channel_style);
    let channel_items: Vec<ListItem> = app
        .channels
        .iter()
        .map(|c| ListItem::new(c.name.clone()))
        .collect();
    let mut channel_state = ListState::default();
    channel_state.select(Some(app.selected_channel));
    let channel_list = List::new(channel_items)
        .block(channel_block)
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_stateful_widget(channel_list, chunks[1], &mut channel_state);
}
