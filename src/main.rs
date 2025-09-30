use crate::ui::{boot_page, theme, title, update, view};

mod ui;

fn main() -> iced::Result {
    iced::application(boot_page, update, view)
        .title(title)
        .theme(theme)
        .run()
}
