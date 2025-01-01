use iced::{
    widget::{
        container,
        image::{self},
        text, tooltip, Column, Container, Row, Text,
    },
    Alignment, Background, Border, Color, Length, Padding, Shadow, Theme,
};
use std::collections::HashMap;

use super::components::icon_button::{icon_button, ICON};
use super::components::scroll_view;
use crate::storage::{BrowserInfo, MatchItem};

const SPACING: f32 = 10.0;
const APP_WIDTH: f32 = 80.0;
const MATCH_TYPE_WIDTH: f32 = 80.0;
const PROFILE_WIDTH: f32 = 100.0;
const ACTION_WIDTH: f32 = 80.0;

pub fn rule_manager<'a, Message: 'a + Clone>(
    match_items: Vec<MatchItem>,
    browser_list: Vec<BrowserInfo>,
    delete_match_item: impl Fn(String) -> Message,
) -> Container<'a, Message> {
    let browser_map: HashMap<String, BrowserInfo> = browser_list
        .into_iter()
        .map(|browser| (browser.path.clone(), browser))
        .collect();

    let mut content = Column::new().spacing(5).padding(15).width(Length::Fill);

    // Add header with better styling
    let header = Row::new()
        .spacing(SPACING)
        .height(Length::Fixed(30.0))
        .align_y(Alignment::Center)
        .push(
            container(Text::new("App").size(12).style(header_text_style()))
                .width(Length::Fixed(APP_WIDTH))
                .center_y(Length::Fill),
        )
        .push(
            container(Text::new("Match Value").size(12).style(header_text_style()))
                .width(Length::Fill)
                .center_y(Length::Fill),
        )
        .push(
            container(Text::new("Match Type").size(12).style(header_text_style()))
                .width(Length::Fixed(MATCH_TYPE_WIDTH))
                .center_y(Length::Fill),
        )
        .push(
            container(Text::new("Profile").size(12).style(header_text_style()))
                .width(Length::Fixed(PROFILE_WIDTH))
                .center_y(Length::Fill),
        )
        .push(
            container(Text::new("Action").size(12).style(header_text_style()))
                .width(Length::Fixed(ACTION_WIDTH))
                .center_y(Length::Fill),
        );

    content = content.push(
        container(header)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                border: Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .padding(8),
    );

    // Add items with better styling
    for item in match_items {
        let browser = browser_map.get(&item.browser_path);
        let row =
            Row::new()
                .spacing(SPACING)
                .align_y(Alignment::Center)
                .push(
                    container(
                        image::viewer(image::Handle::from_bytes(
                            browser.unwrap().icon_data.clone(),
                        ))
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0)),
                    )
                    .padding(Padding::new(0.0).left(5.0))
                    .width(Length::Fixed(APP_WIDTH)),
                )
                .push(tooltip(
                    Text::new(truncate_string(&item.match_value.clone(), 80))
                        .size(11)
                        .width(Length::Fill)
                        .style(|_| text::Style {
                            color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
                            ..Default::default()
                        }),
                    container(Text::new(item.match_value.clone()).size(11).style(|_| {
                        text::Style {
                            color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
                            ..Default::default()
                        }
                    }))
                    .style(|_| container::Style {
                        background: Some(Background::Color(Color::from_rgb(1.0, 1.0, 1.0))),
                        border: Border {
                            radius: 4.0.into(),
                            width: 1.0,
                            color: Color::from_rgb(0.9, 0.9, 0.9),
                        },
                        shadow: Shadow {
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 5.0,
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                        },
                        ..Default::default()
                    })
                    .padding(8),
                    tooltip::Position::Top,
                ))
                .push(
                    container(
                        Text::new(item.match_type.to_lowercase())
                            .size(11)
                            .style(|_| text::Style {
                                color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
                                ..Default::default()
                            }),
                    )
                    .style(move |_| container::Style {
                        background: Some(Background::Color(if item.match_type == "Equal" {
                            Color::from_rgb(0.95, 0.97, 1.0)
                        } else {
                            Color::from_rgb(1.0, 0.97, 0.95)
                        })),
                        border: Border {
                            radius: 4.0.into(),
                            width: 1.0,
                            color: if item.match_type == "Equal" {
                                Color::from_rgb(0.8, 0.9, 1.0)
                            } else {
                                Color::from_rgb(1.0, 0.9, 0.8)
                            },
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(MATCH_TYPE_WIDTH))
                    .center_x(MATCH_TYPE_WIDTH)
                    .padding(5),
                )
                .push(
                    container(
                        Text::new(item.profile.unwrap_or_default())
                            .size(11)
                            .style(|_| text::Style {
                                color: Some(Color::from_rgb(0.3, 0.3, 0.3)),
                                ..Default::default()
                            }),
                    )
                    .width(Length::Fixed(PROFILE_WIDTH)),
                )
                .push(
                    container(icon_button(
                        ICON::Remove,
                        delete_match_item(item.match_value.clone()),
                        "Delete rule".to_string(),
                    ))
                    .width(Length::Fixed(ACTION_WIDTH)),
                );

        content = content.push(
            container(row)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(1.0, 1.0, 1.0))),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: Color::from_rgb(0.95, 0.95, 0.95),
                    },
                    ..Default::default()
                })
                .padding(8),
        );
    }

    container(scroll_view::scroll_view(content))
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
}

fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    }
}

fn header_text_style() -> impl Fn(&Theme) -> text::Style {
    |_| text::Style {
        color: Some(Color::from_rgb(0.4, 0.4, 0.4)),
        ..Default::default()
    }
}
