use crate::ui::GbaUi;

mod ui;

fn main() -> iced::Result {
    iced::run("GiBi Advance", GbaUi::update, GbaUi::view)
}
