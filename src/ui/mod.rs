use iced::{
    exit,
    widget::{button, column},
    Element, Task,
};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct GbaUi {}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Close,
}

impl GbaUi {
    pub fn view(&self) -> Element<'_, Message> {
        column![button("Close").on_press(Message::Close),].into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => exit(),
        }
    }
}
