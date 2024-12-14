use iced::{
    widget::{button, center, Button},
    Background, Border, Color, Shadow,
};

use super::icons::{add_icon, back_icon, profile_icon, remove_icon};

pub enum ICON {
    Profile,
    Add,
    Remove,
    Back,
}

pub fn icon_button<'a, Message: 'a>(icon: ICON, on_press: Message) -> Button<'a, Message> {
    button(center(match icon {
        ICON::Profile => profile_icon(),
        ICON::Add => add_icon(),
        ICON::Remove => remove_icon(),
        ICON::Back => back_icon(),
    }))
    .padding(0.0)
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
    .on_press(on_press)
    .height(20.0)
    .width(20.0)
}
