use crate::app::{App, Focus};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
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
    f.render_stateful_widget(channel_list, area, &mut channel_state);
}
