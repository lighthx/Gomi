use iced::{
    widget::{
        button, center, container, text, Column, Container, Text,
    },
    Background, Border, Color, Length, Shadow,
};

pub fn set_default_browser<'a, Message: 'a + Clone>(
    set_default: Message,
) -> Container<'a, Message> {
    container(center(
        Column::new()
            .push(
                Text::new("Set as your default browser")
                    .size(14)
                    .style(|_| text::Style {
                        color: Some(Color::from_rgb(0.4, 0.4, 0.4)),
                    }),
            )
            .push(
                button(Text::new("Set Default").size(14).style(|_| text::Style {
                    color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
                }))
                .style(|_, _| button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 1.0))),
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                    ..button::Style::default()
                })
                .padding([6, 16])
                .on_press(set_default),
            )
            .spacing(12)
            .align_x(iced::Alignment::Center),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(|_| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
        ..Default::default()
    })
}
