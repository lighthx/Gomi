use iced::{
    widget::{scrollable, Scrollable}, Background, Border, Color, Element
};

pub fn scroll_view<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    scrollable(content).style(|_, _| scrollable::Style {
        container: Default::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            scroller: scrollable::Scroller {
                color: Color::from_rgb(0.85, 0.85, 0.85),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Default::default(),
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: Default::default(),
    })
}
