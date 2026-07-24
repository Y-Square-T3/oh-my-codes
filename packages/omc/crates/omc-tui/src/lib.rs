mod app;
mod event;
mod ui;

use omc_api::client::OmcClient;
use std::io;

pub async fn run(client: OmcClient) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = app::App::new(client);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    event::spawn_event_loop(tx);

    let result = run_loop(&mut terminal, &mut app, &mut rx).await;
    ratatui::restore();
    result
}

async fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut app::App,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<event::AppEvent>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = rx.recv().await {
            match event {
                event::AppEvent::Key(key_event) => {
                    if app.handle_key(key_event) {
                        break;
                    }
                }
                event::AppEvent::Tick => {}
                event::AppEvent::ChannelsLoaded(channels) => {
                    app.channels = channels;
                }
                event::AppEvent::MessagesLoaded(messages) => {
                    app.messages = messages;
                }
                event::AppEvent::Error(e) => {
                    app.error = Some(e);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
