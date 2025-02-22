use data::config::{self, sidebar, Config};
use data::dashboard::{BufferAction, BufferFocusedAction};
use data::{buffer, file_transfer, history, Version};
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pane_grid, row, scrollable, text,
    vertical_rule, vertical_space, Column, Row, Scrollable, Space,
};
use iced::{padding, Alignment, Length, Task};
use std::time::Duration;

use tokio::time;

use crate::screen::dashboard::sidebar::Sidebar;
use crate::screen::dashboard::Panes;
use crate::widget::{context_menu, Element, Text};
use crate::{icon, theme, window};

const CONFIG_RELOAD_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
}

#[derive(Debug, Clone)]
pub enum Event {}

#[derive(Clone)]
pub struct List {
    sidebar: Sidebar,
}

impl List {
    pub fn new() -> Self {
        Self {
            sidebar: Sidebar::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> (Task<Message>, Option<Event>) {
        (Task::none(), None)
    }

    pub fn view<'a>(
        &'a self,
        clients: &data::client::Map,
        history: &'a history::Manager,
        panes: &'a Panes,
        focus: Option<(window::Id, pane_grid::Pane)>,
        config: data::config::Sidebar,
        keyboard: &'a data::config::Keyboard,
        file_transfers: &'a file_transfer::Manager,
        version: &'a Version,
        main_window: window::Id,
    ) -> Element<'a, Message> {
        text("hi").into()
    }
}
