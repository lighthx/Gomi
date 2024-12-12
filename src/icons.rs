use iced::{widget::text, Element, Font};

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("Microns");

    text(codepoint).font(ICON_FONT).into()
}

pub fn profile_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e738}')
}

pub fn add_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e70d}')
}

pub fn remove_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e70e}')
}
