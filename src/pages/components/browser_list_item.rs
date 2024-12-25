use iced::{
    widget::{
        button, container,
        image::{self, Viewer},
        row, text, Button, Text,
    },
    Background, Border, Color, Length, Shadow,
};

use super::icon_button::{icon_button, ICON};

pub fn browser_list_item<'a, Message: 'a + Clone>(
    start_icon: Option<Viewer<image::Handle>>,
    name: String,
    on_press: Message,
    end_icon: ICON,
    end_on_press: Message,
    end_tip: String,
) -> Button<'a, Message> {
    let mut content = row![]
        .spacing(12)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(20.0);
    if let Some(icon) = start_icon {
        content = content.push(container(icon).width(Length::Fixed(32.0)));
    }

    content = content.push(
        Text::new(name.clone())
            .size(13)
            .style(|_| text::Style {
                color: Some(Color::from_rgb(0.2, 0.2, 0.2)),
            })
            .width(Length::Fill),
    );

    content = content.push(icon_button(end_icon, end_on_press, end_tip));

    button(content)
        .on_press(on_press)
        .style(|_, _| button::Style {
            background: Some(Background::Color(Color::from_rgb(1.0, 1.0, 1.0))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.9, 0.9, 0.9),
            },
            text_color: Color::from_rgb(0.2, 0.2, 0.2),
            shadow: Shadow {
                color: Color::from_rgb(0.8, 0.8, 0.8),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 5.0,
            },
            ..button::Style::default()
        })
        .padding([10, 16])
}
