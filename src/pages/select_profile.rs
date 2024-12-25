use iced::{
    widget::{
        container,
        row, text_input, Column, Container,
    },
    Alignment, Length, Padding,
};

use crate::storage::BrowserProfile;

use super::components::{browser_list_item, icon_button, scroll_view};

pub fn select_profile<'a, Message: 'a + Clone>(
    profiles: &Vec<BrowserProfile>,
    open_browser_with_profile: impl Fn(String) -> Message,
    delete_profile: impl Fn(String) -> Message,
    back: Message,
    type_profile_text: impl Fn(String) -> Message + 'a,
    add_profile: Message,
    profile_text: &String,
) -> Container<'a, Message> {
    let mut content = Column::new()
        .padding(Padding::new(12.0))
        .width(Length::Fill)
        .height(Length::Fill);

    let profile_list = profiles
        .iter()
        .fold(Column::new().spacing(5), |column, profile| {
            let profile_row = browser_list_item::browser_list_item(
                None,
                profile.profile.clone(),
                open_browser_with_profile(profile.profile.clone()),
                icon_button::ICON::Remove,
                delete_profile(profile.profile.clone()),
                "Delete profile".to_string(),
            );
            column.push(profile_row)
        });
    content = content.push(
        container(scroll_view::scroll_view(profile_list))
            .height(Length::Fill)
            .width(Length::Fill),
    );

    let input_row = row![
        icon_button::icon_button(
            icon_button::ICON::Back,
            back,
            "Back to home page".to_string()
        ),
        text_input("New Profile", &profile_text)
            .on_input(type_profile_text)
            .on_submit(add_profile.clone())
            .width(Length::Fill),
        icon_button::icon_button(
            icon_button::ICON::Add,
            add_profile,
            "Add profile".to_string()
        )
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .height(30.0);

    content = content.push(input_row);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
}
