use crossterm::event::KeyEvent;
use omc_api::client::OmcClient;
use omc_core::types::{Channel, Message};

pub enum Focus {
    Channels,
    Chat,
    Input,
}

pub enum InputMode {
    Normal,
    Insert,
}

#[allow(dead_code)]
pub struct App {
    pub client: OmcClient,
    pub channels: Vec<Channel>,
    pub messages: Vec<Message>,
    pub selected_channel: usize,
    pub focus: Focus,
    pub input_mode: InputMode,
    pub input: String,
    pub chat_scroll: u16,
    pub error: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(client: OmcClient) -> Self {
        Self {
            client,
            channels: Vec::new(),
            messages: Vec::new(),
            selected_channel: 0,
            focus: Focus::Channels,
            input_mode: InputMode::Normal,
            input: String::new(),
            chat_scroll: 0,
            error: None,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        match self.input_mode {
            InputMode::Insert => match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    self.input.clear();
                    self.input_mode = InputMode::Normal;
                }
                _ => {}
            },
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => {
                    return true;
                }
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Insert;
                }
                KeyCode::Char('j') | KeyCode::Down => match self.focus {
                    Focus::Channels => {
                        if self.selected_channel < self.channels.len().saturating_sub(1) {
                            self.selected_channel += 1;
                        }
                    }
                    Focus::Chat => {
                        self.chat_scroll = self.chat_scroll.saturating_sub(1);
                    }
                    _ => {}
                },
                KeyCode::Char('k') | KeyCode::Up => match self.focus {
                    Focus::Channels => {
                        self.selected_channel = self.selected_channel.saturating_sub(1);
                    }
                    Focus::Chat => {
                        self.chat_scroll += 1;
                    }
                    _ => {}
                },
                KeyCode::Char('h') | KeyCode::Left => {
                    self.focus = match self.focus {
                        Focus::Chat => Focus::Channels,
                        Focus::Input => Focus::Chat,
                        _ => Focus::Channels,
                    };
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.focus = match self.focus {
                        Focus::Channels => Focus::Chat,
                        Focus::Chat => Focus::Input,
                        _ => Focus::Input,
                    };
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return true;
                }
                _ => {}
            },
        }
        false
    }
}
