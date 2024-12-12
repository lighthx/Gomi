mod components;
mod config;
mod icons;
mod platform_tools;
mod storage;
use components::icon_button::{icon_button, ICON};
use components::list_item::list_item;
use components::scroll_view::scroll_view;
use config::{LOG_DIR, LOG_FILE, WINDOW_HEIGHT, WINDOW_WIDTH};
use iced::keyboard::Modifiers;
use iced::widget::{text_editor, text_input, tooltip};
use iced::window::{Mode, Position};
use iced::{event, keyboard, Alignment, Event, Font, Subscription};
use iced::{
    widget::{button, center, container, image, row, text, Column, Text},
    window, Background, Border, Color, Element, Length, Shadow, Size, Task, Theme,
};
use platform_tools::get_mouse_position;
use platform_tools::open_url;
use std::mem;
use std::time::{Duration, Instant};
use storage::{BrowserInfo, BrowserProfile, MatchItem, Storage};
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use url::Url;

struct Gomi {
    is_default_browser: bool,
    current_url: Option<String>,
    browser_list: Option<Vec<BrowserInfo>>,
    current_page: Page,
    storage: Storage,
    keyboard: Modifiers,
    launch_time: Instant,
    stacks: Vec<Page>,
}
#[derive(Debug)]
enum Page {
    Home,
    ProfileSelector {
        browser: BrowserInfo,
        profiles: Vec<BrowserProfile>,
        profile_text: String,
    },
    MatchContainEditor {
        match_container_text: text_editor::Content,
        browser_path: String,
        profile: Option<String>,
    },
}

#[derive(Debug, Clone)]
enum ExternalOperation {
    SaveEqual,
    SaveContain,
}

#[derive(Debug, Clone)]
enum Message {
    GoHome(Vec<BrowserInfo>),
    LaunchBrowser(String, Option<String>, Option<ExternalOperation>),
    SetAsDefault,
    ReceiveUrl(String),
    CheckDefaultStatus,
    ListProfiles(BrowserInfo),
    DeleteProfile(String),
    Back,
    AddProfile,
    TypeProfileText(String),
    WindowUnfocused,
    ShowMatchContainEditor(String, Option<String>),
    TypeMatchContainText(text_editor::Action),
    KeyboardModifiersChanged(Modifiers),
}

impl Gomi {
    fn new() -> (Self, Task<Message>) {
        let storage = Storage::new();
        (
            Self {
                browser_list: None,
                current_url: None,
                is_default_browser: cfg!(debug_assertions)
                    || platform_tools::ensure_default_browser(),
                current_page: Page::Home,
                storage,
                keyboard: Modifiers::default(),
                launch_time: Instant::now(),
                stacks: vec![],
            },
            Task::perform(
                async move {
                    let mut storage = Storage::new();
                    let mut browsers = storage.get_browsers();
                    if browsers.is_empty() {
                        let handlers = platform_tools::get_url_handlers().await;
                        storage.batch_insert_browsers(handlers.clone());
                        browsers = handlers;
                    }
                    browsers
                },
                Message::GoHome,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        info!("message: {:?}", message);
        match message {
            Message::GoHome(browsers) => {
                self.browser_list = Some(browsers);
                Task::none()
            }
            Message::LaunchBrowser(path, profile, external_operation) => {
                if let Some(url) = self.current_url.clone() {
                    open_url(url.clone(), path.clone(), profile.clone());
                    if let Some(external_operation) = external_operation {
                        match external_operation {
                            ExternalOperation::SaveEqual => {
                                self.storage.insert_match(MatchItem {
                                    browser_path: path.clone(),
                                    profile: profile.clone(),
                                    match_type: "Equal".to_string(),
                                    match_value: url.clone(),
                                });
                            }
                            ExternalOperation::SaveContain => {
                                if let Page::MatchContainEditor {
                                    match_container_text,
                                    ..
                                } = &self.current_page
                                {
                                    self.storage.insert_match(MatchItem {
                                        browser_path: path.clone(),
                                        profile: profile.clone(),
                                        match_type: "Contain".to_string(),
                                        match_value: match_container_text.text().trim().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    return window::get_latest()
                        .and_then(move |w| window::change_mode(w, Mode::Hidden));
                }
                Task::none()
            }

            Message::ReceiveUrl(url) => {
                let equal_matched = storage::Storage::new().find_equal_matches_by_url(url.clone());
                if let Some(match_item) = equal_matched {
                    open_url(url.clone(), match_item.browser_path, match_item.profile);
                    return Task::none();
                }
                let contain_matched =
                    storage::Storage::new().find_contain_matches_by_url(url.clone());
                if let Some(match_item) = contain_matched {
                    open_url(url.clone(), match_item.browser_path, match_item.profile);
                    return Task::none();
                }
                self.current_url = Some(url);
                Task::batch(vec![
                    window::get_latest()
                        .and_then(move |w| window::move_to(w, get_mouse_position())),
                    window::get_latest().and_then(move |w| window::change_mode(w, Mode::Windowed)),
                ])
            }
            Message::SetAsDefault => {
                platform_tools::set_as_default_browser();
                Task::none()
            }
            Message::CheckDefaultStatus => {
                if !self.is_default_browser {
                    self.is_default_browser = platform_tools::ensure_default_browser();
                }

                Task::none()
            }
            Message::ListProfiles(browser) => {
                let profiles = self.storage.get_browser_profiles(browser.path.clone());
                let new_page = Page::ProfileSelector {
                    profiles,
                    browser,
                    profile_text: String::new(),
                };
                self.stacks
                    .push(mem::replace(&mut self.current_page, new_page));
                Task::none()
            }
            Message::DeleteProfile(profile_name) => {
                if let Page::ProfileSelector {
                    browser,
                    profile_text: text,
                    ..
                } = &self.current_page
                {
                    self.storage
                        .delete_browser_profile(browser.path.clone(), profile_name.clone());
                    self.storage.delete_match_by_profile_and_browser_path(
                        browser.path.clone(),
                        profile_name.clone(),
                    );
                    let profiles = self.storage.get_browser_profiles(browser.path.clone());
                    self.current_page = Page::ProfileSelector {
                        profiles,
                        browser: browser.clone(),
                        profile_text: text.clone(),
                    };
                }
                Task::none()
            }
            Message::Back => {
                if let Some(page) = self.stacks.pop() {
                    self.current_page = page;
                }
                Task::none()
            }
            Message::AddProfile => {
                if let Page::ProfileSelector {
                    browser,
                    profile_text,
                    ..
                } = &self.current_page
                {
                    if profile_text.is_empty() {
                        return Task::none();
                    }
                    self.storage.insert_browser_profile(BrowserProfile {
                        browser_path: browser.path.clone(),
                        profile: profile_text.clone(),
                        description: None,
                    });
                    let profiles = self.storage.get_browser_profiles(browser.path.clone());
                    self.current_page = Page::ProfileSelector {
                        profiles,
                        browser: browser.clone(),
                        profile_text: String::new(),
                    };
                }
                Task::none()
            }
            Message::TypeProfileText(text) => {
                if let Page::ProfileSelector { profile_text, .. } = &mut self.current_page {
                    *profile_text = text;
                }
                Task::none()
            }
            Message::WindowUnfocused => {
                if cfg!(debug_assertions) {
                    Task::none()
                } else if self.launch_time.elapsed() < Duration::from_secs(1) {
                    Task::none()
                } else {
                    self.current_page = Page::Home;
                    self.stacks.clear();
                    window::get_latest().and_then(move |w| window::change_mode(w, Mode::Hidden))
                }
            }
            Message::ShowMatchContainEditor(browser_path, profile) => {
                if let Some(url) = self.current_url.clone() {
                    let new_page = Page::MatchContainEditor {
                        match_container_text: text_editor::Content::with_text(&url),
                        browser_path,
                        profile,
                    };
                    self.stacks
                        .push(mem::replace(&mut self.current_page, new_page));
                }
                Task::none()
            }
            Message::TypeMatchContainText(action) => {
                if let Page::MatchContainEditor {
                    match_container_text,
                    ..
                } = &mut self.current_page
                {
                    match_container_text.perform(action);
                }
                Task::none()
            }
            Message::KeyboardModifiersChanged(modifiers) => {
                self.keyboard = modifiers;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.current_page {
            Page::Home => {
                let browsers = self.browser_list.clone().unwrap_or_default();
                let current_url = self.current_url.clone();
                let is_default_browser = self.is_default_browser;
                if !is_default_browser {
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
                                    background: Some(Background::Color(Color::from_rgb(
                                        0.2, 0.5, 1.0,
                                    ))),
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
                                .on_press(Message::SetAsDefault),
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
                } else {
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
                        let icon =
                            image::viewer(image::Handle::from_bytes(browser.icon_data.clone()))
                                .width(Length::Fixed(16.0))
                                .height(Length::Fixed(16.0));

                        content = content.push(list_item(
                            Some(icon),
                            name,
                            if self.keyboard.shift() {
                                Message::ShowMatchContainEditor(browser.path.clone(), None)
                            } else {
                                Message::LaunchBrowser(
                                    path,
                                    None,
                                    if self.keyboard.logo() {
                                        Some(ExternalOperation::SaveEqual)
                                    } else {
                                        None
                                    },
                                )
                            },
                            ICON::Profile,
                            Message::ListProfiles(browser.clone()),
                        ));
                    }

                    let footer = if let Some(url) = current_url {
                        let url_cloned = url.clone();
                        let url = Url::parse(&url).unwrap();
                        let host = url.host_str().unwrap_or_default().to_string();

                        container(tooltip(
                            Text::new(host).size(13).style(|_| text::Style {
                                color: Some(Color::from_rgb(0.2, 0.2, 0.2)),
                            }),
                            container(Text::new(url_cloned).size(13))
                                .padding(4)
                                .style(|_| container::Style {
                                    background: Some(Background::Color(Color::from_rgb(
                                        0.9, 0.9, 1.0,
                                    ))),
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

                    container(
                        Column::new()
                            .push(
                                container(scroll_view(content))
                                    .style(|_| container::Style {
                                        background: Some(Background::Color(Color::from_rgb(
                                            0.98, 0.98, 0.98,
                                        ))),
                                        ..Default::default()
                                    })
                                    .padding(2)
                                    .width(Length::Fill)
                                    .height(Length::Fill),
                            )
                            .push(footer),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                }
            }
            Page::ProfileSelector {
                profiles,
                browser,
                profile_text,
            } => {
                let mut content = Column::new().spacing(20).padding(20).width(Length::Fill);

                let profile_list =
                    profiles
                        .iter()
                        .fold(Column::new().spacing(5), |column, profile| {
                            let profile_row = list_item(
                                None,
                                profile.profile.clone(),
                                if self.keyboard.shift() {
                                    Message::ShowMatchContainEditor(
                                        browser.path.clone(),
                                        Some(profile.profile.clone()),
                                    )
                                } else {
                                    Message::LaunchBrowser(
                                        browser.path.clone(),
                                        Some(profile.profile.clone()),
                                        if self.keyboard.logo() {
                                            Some(ExternalOperation::SaveEqual)
                                        } else {
                                            None
                                        },
                                    )
                                },
                                ICON::Remove,
                                Message::DeleteProfile(profile.profile.clone()),
                            );
                            column.push(profile_row)
                        });
                content = content.push(scroll_view(profile_list).height(200.0));

                let add_row = row![
                    icon_button(ICON::Back, Message::Back),
                    text_input("New Profile", &profile_text)
                        .on_input(Message::TypeProfileText)
                        .on_submit(Message::AddProfile)
                        .width(Length::Fill),
                    icon_button(ICON::Add, Message::AddProfile)
                ]
                .spacing(12)
                .align_y(Alignment::Center);

                content = content.push(add_row);

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
            }
            Page::MatchContainEditor {
                match_container_text,
                browser_path,
                profile,
            } => {
                let mut content = Column::new()
                    .spacing(20)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill);
                let footer = row![
                    icon_button(ICON::Back, Message::Back),
                    button(Text::new("Save And Open").size(14).style(|_| text::Style {
                        color: Some(Color::from_rgb(1.0, 1.0, 1.0)),
                    }))
                    .style(|_, _| button::Style {
                        background: Some(Background::Color(Color::from_rgb(0.2, 0.5, 1.0))),
                        ..Default::default()
                    })
                    .padding([6, 16])
                    .on_press(Message::LaunchBrowser(
                        browser_path.clone(),
                        profile.clone(),
                        Some(ExternalOperation::SaveContain),
                    ))
                ]
                .align_y(Alignment::Center)
                .spacing(12);
                content = content
                    .push(
                        text_editor(&match_container_text).on_action(Message::TypeMatchContainText),
                    )
                    .push(footer);
                container(content)
            }
        };
        content.into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_url().map(Message::ReceiveUrl),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::CheckDefaultStatus),
            event::listen_with(|event, _status, _window| -> Option<Message> {
                match event {
                    Event::Window(window::Event::Unfocused) => Some(Message::WindowUnfocused),
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        Some(Message::KeyboardModifiersChanged(modifiers))
                    }

                    _ => None,
                }
            }),
        ])
    }
}

fn main() -> iced::Result {
    let file_appender = tracing_appender::rolling::daily(LOG_DIR, LOG_FILE);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .init();
    iced::application("Gomi", Gomi::update, Gomi::view)
        .window(window::Settings {
            position: Position::Specific(get_mouse_position()),
            decorations: false,
            transparent: true,
            resizable: false,
            ..Default::default()
        })
        .font(include_bytes!("../fonts/Microns.ttf").as_slice())
        .window_size(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .default_font(Font::MONOSPACE)
        .antialiasing(true)
        .theme(Gomi::theme)
        .subscription(Gomi::subscription)
        .run_with(Gomi::new)
}
