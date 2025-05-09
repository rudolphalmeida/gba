use crate::ui::AppState;

mod ui;

fn main() -> iced::Result {
    iced::application(AppState::default, AppState::update, AppState::view)
        .theme(AppState::theme)
        .centered()
        .run()
}
