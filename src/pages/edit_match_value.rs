use iced::widget::text_editor;
use iced::Renderer;
use iced::{
    widget::{
        button, container,
        row, text,
        text_editor::Action, Column, Container, Text,
    },
    Alignment, Background, Color, Length,
};

use super::components::icon_button;

pub fn edit_match_value<'a, Message: 'a + Clone>(
    back: Message,
    save_and_open: Message,
    type_match_contain_text: impl Fn(Action) -> Message + 'a,
    match_container_text: &'a text_editor::Content<Renderer>,
) -> Container<'a, Message> {
    let mut content = Column::new()
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);
    let footer = row![
        icon_button::icon_button(
            icon_button::ICON::Back,
            back,
            "Back to previous page".to_string()
        ),
        button(Text::new("Save And Open").size(14).style(|_| text::Style {
            color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
        }))
        .style(|_, _| button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 1.0))),
            ..Default::default()
        })
        .padding([6, 16])
        .on_press(save_and_open)
    ]
    .align_y(Alignment::Center)
    .spacing(12);
    content = content
        .push(text_editor(&match_container_text).on_action(type_match_contain_text))
        .push(footer);
    container(content)
}
