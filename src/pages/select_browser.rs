use iced::{
    widget::{
        container,
        image::{self}, Column, Container, Text,
    },
    Background, Color, Length,
};

use crate::storage::BrowserInfo;

use super::components::{browser_list_item, icon_button, scroll_view};

pub fn select_browser<'a, Message: 'a + Clone>(
    browsers: &Vec<BrowserInfo>,
    select_browser: impl Fn(String) -> Message,
    list_profiles: impl Fn(BrowserInfo) -> Message,
) -> Container<'a, Message> {
    let mut content = Column::new().spacing(8).padding(12).width(Length::Fill);
    content = content.push(
        container(Text::new(""))
            .width(Length::Fill)
            .height(1)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                ..Default::default()
            }),
    );

    for browser in browsers {
        let name = browser.name.to_string();
        let path = browser.path.clone();
        let icon = image::viewer(image::Handle::from_bytes(browser.icon_data.clone()))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));

        content = content.push(browser_list_item::browser_list_item(
            Some(icon),
            name,
            select_browser(path.clone()),
            icon_button::ICON::Profile,
            list_profiles(browser.clone()),
            "List profiles".to_string(),
        ));
    }

    container(
        Column::new().push(
            container(scroll_view::scroll_view(content))
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                    ..Default::default()
                })
                .padding(2)
                .width(Length::Fill)
                .height(Length::Fill),
        ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}
