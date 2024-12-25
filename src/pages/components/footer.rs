use iced::{
    widget::{center, container, row, text, tooltip, Container, Text},
    Alignment, Background, Border, Color, Length,
};
use url::Url;

use crate::pages::components::icon_button::{icon_button, ICON};

pub fn footer<'a, Message: Clone + 'a>(
    url: Option<String>,
    on_refresh: Message,
) -> Container<'a, Message> {
    let url_view = if let Some(url) = url {
        let url_cloned = url.clone();
        let url = Url::parse(&url).unwrap();
        let host = url.host_str().unwrap_or_default().to_string();

        container(tooltip(
            Text::new(host).size(13).style(|_| text::Style {
                color: Some(Color::from_rgb(0.2, 0.2, 0.2)),
            }),
            container(
                Text::new(url_cloned)
                    .size(13)
                    .color(Color::from_rgb(0.2, 0.2, 0.2)),
            )
            .padding(4)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 1.0))),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: Color::from_rgb(0.7, 0.7, 0.9),
                },
                ..Default::default()
            }),
            tooltip::Position::Top,
        ))
        .padding([8, 12])
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 1.0))),
            border: Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.8, 0.8, 0.9),
            },
            ..Default::default()
        })
        .center_x(Length::Fill)
    } else {
        container(Text::new(""))
    };
    let refresh_button = center(icon_button(
        ICON::Refresh,
        on_refresh.clone(),
        "Refresh applications".to_string(),
    ))
    .width(32.0)
    .height(32.0);
    let row = row![url_view, refresh_button]
        .width(Length::Fill)
        .spacing(5.0)
        .align_y(Alignment::Center);
    container(row).width(Length::Fill)
}
