mod config;
mod icons;
mod message;
mod pages;
mod platform_tools;
mod storage;
mod subscriptions;

use config::{
    LOG_DIR, LOG_FILE, MENU_WINDOW_HEIGHT, MENU_WINDOW_WIDTH, SETTING_WINDOW_HEIGHT,
    SETTING_WINDOW_WIDTH,
};
use iced::keyboard::Modifiers;
use iced::widget::text_editor;
use iced::window::Position;
use iced::{
    event, keyboard,
    widget::{text, Column},
    window, Element, Event, Font, Size, Subscription, Task, Theme,
};
use message::{ExternalOperation, Message, WindowType};
use pages::components::footer::footer;
use platform_tools::{ensure_default_browser, get_mouse_position};
use platform_tools::{open_url, show_app};
use std::mem;
use std::time::{Duration, Instant};
use storage::{BrowserInfo, BrowserProfile, MatchItem, Storage};
use subscriptions::tray_menu_event_subscription;
use tracing_subscriber::fmt::format::FmtSpan;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

const IS_DEBUG: bool = cfg!(debug_assertions);

struct MenuWindow {
    is_default_browser: bool,
    current_page: MenuWindowPage,
    current_url: Option<String>,
    browser_list: Vec<BrowserInfo>,
    launch_time: Instant,
    stacks: Vec<MenuWindowPage>,
    window_id: window::Id,
}

struct SettingWindow {
    launch_time: Instant,
    match_items: Vec<MatchItem>,
    window_id: window::Id,
}

struct Gomi {
    _tray: TrayIcon,
    storage: Storage,
    keyboard: Modifiers,
    menu_window: Option<MenuWindow>,
    setting_window: Option<SettingWindow>,
}
#[derive(Debug)]
enum MenuWindowPage {
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

impl Gomi {
    fn new() -> (Self, Task<Message>) {
        let storage = Storage::new();
        if IS_DEBUG {
            show_app();
        }
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(
                Menu::with_items(&[
                    &MenuItem::new("rule settings", true, None),
                    &MenuItem::new("Quit", true, None),
                ])
                .unwrap(),
            ))
            .with_tooltip("Gomi")
            .with_title("Gomi")
            .build()
            .unwrap();
        unsafe {
            use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};
            let rl = CFRunLoopGetMain();
            CFRunLoopWakeUp(rl);
        }

        (
            Self {
                storage,
                keyboard: Modifiers::default(),
                menu_window: None,
                setting_window: None,
                _tray: tray,
            },
            if ensure_default_browser() {
                Task::done(Message::OpenWindow(WindowType::Menu))
            } else {
                Task::none()
            },
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        println!("update: {:?}", message);
        match message {
            Message::LaunchBrowser(path, profile, external_operation) => {
                if let Some(menu_window) = &self.menu_window {
                    if let Some(url) = menu_window.current_url.clone() {
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
                                    if let MenuWindowPage::MatchContainEditor {
                                        match_container_text,
                                        ..
                                    } = &menu_window.current_page
                                    {
                                        self.storage.insert_match(MatchItem {
                                            browser_path: path.clone(),
                                            profile: profile.clone(),
                                            match_type: "Contain".to_string(),
                                            match_value: match_container_text
                                                .text()
                                                .trim()
                                                .to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        return Task::done(Message::CloseWindow(WindowType::Menu));
                    }
                }
                Task::none()
            }

            Message::ReceiveUrl(url) => {
                if let Some(menu_window) = &mut self.menu_window {
                    menu_window.current_url = Some(url.clone());
                    let equal_matched = self.storage.find_equal_matches_by_url(url.clone());
                    if let Some(match_item) = equal_matched {
                        return Task::done(Message::LaunchBrowser(
                            match_item.browser_path,
                            match_item.profile,
                            None,
                        ));
                    }
                    let contain_matched = self.storage.find_contain_matches_by_url(url.clone());
                    if let Some(match_item) = contain_matched {
                        return Task::done(Message::LaunchBrowser(
                            match_item.browser_path,
                            match_item.profile,
                            None,
                        ));
                    }
                    if let Some(MenuWindow { window_id, .. }) = self.menu_window {
                        return Task::done(Message::MoveWindow(window_id));
                    }
                }
                Task::done(Message::OpenWindow(WindowType::Menu))
            }
            Message::MoveWindow(window_id) => window::move_to(window_id, get_mouse_position()),
            Message::SetAsDefault => {
                platform_tools::set_as_default_browser();
                Task::none()
            }
            Message::CheckDefaultStatus => {
                if let Some(menu_window) = &mut self.menu_window {
                    if !menu_window.is_default_browser {
                        let is_default_browser = platform_tools::ensure_default_browser();
                        menu_window.is_default_browser = is_default_browser;
                        if is_default_browser {
                            return Task::done(Message::CloseWindow(WindowType::Menu));
                        }
                    }
                }
                Task::none()
            }
            Message::ListProfiles(browser) => {
                if let Some(menu_window) = &mut self.menu_window {
                    let profiles = self.storage.get_browser_profiles(browser.path.clone());
                    let new_page = MenuWindowPage::ProfileSelector {
                        profiles,
                        browser,
                        profile_text: String::new(),
                    };
                    menu_window
                        .stacks
                        .push(mem::replace(&mut menu_window.current_page, new_page));
                }
                Task::none()
            }
            Message::DeleteProfile(profile_name) => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let MenuWindowPage::ProfileSelector {
                        browser,
                        profile_text: text,
                        ..
                    } = &menu_window.current_page
                    {
                        self.storage
                            .delete_browser_profile(browser.path.clone(), profile_name.clone());
                        self.storage.delete_match_by_profile_and_browser_path(
                            browser.path.clone(),
                            profile_name.clone(),
                        );
                        let profiles = self.storage.get_browser_profiles(browser.path.clone());
                        menu_window.current_page = MenuWindowPage::ProfileSelector {
                            profiles,
                            browser: browser.clone(),
                            profile_text: text.clone(),
                        };
                    }
                }
                Task::none()
            }
            Message::Back => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let Some(page) = menu_window.stacks.pop() {
                        menu_window.current_page = page;
                    }
                }
                Task::none()
            }
            Message::AddProfile => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let MenuWindowPage::ProfileSelector {
                        browser,
                        profile_text,
                        ..
                    } = &menu_window.current_page
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
                        menu_window.current_page = MenuWindowPage::ProfileSelector {
                            profiles,
                            browser: browser.clone(),
                            profile_text: String::new(),
                        };
                    }
                }
                Task::none()
            }
            Message::TypeProfileText(text) => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let MenuWindowPage::ProfileSelector { profile_text, .. } =
                        &mut menu_window.current_page
                    {
                        *profile_text = text;
                    }
                }
                Task::none()
            }
            Message::ShowMatchContainEditor(browser_path, profile) => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let Some(url) = menu_window.current_url.clone() {
                        let new_page = MenuWindowPage::MatchContainEditor {
                            match_container_text: text_editor::Content::with_text(&url),
                            browser_path,
                            profile,
                        };
                        menu_window
                            .stacks
                            .push(mem::replace(&mut menu_window.current_page, new_page));
                    }
                }
                Task::none()
            }
            Message::TypeMatchContainText(action) => {
                if let Some(menu_window) = &mut self.menu_window {
                    if let MenuWindowPage::MatchContainEditor {
                        match_container_text,
                        ..
                    } = &mut menu_window.current_page
                    {
                        match_container_text.perform(action);
                    }
                }
                Task::none()
            }
            Message::KeyboardModifiersChanged(modifiers) => {
                self.keyboard = modifiers;
                Task::none()
            }
            Message::OpenWindow(window_type) => {
                if self.menu_window.is_some() && window_type == WindowType::Menu {
                    return Task::none();
                }
                if self.setting_window.is_some() && window_type == WindowType::Setting {
                    return Task::none();
                }
                let position = get_mouse_position();
                let open = if window_type == WindowType::Menu {
                    let (id, open) = window::open(window::Settings {
                        position: Position::Specific(position),
                        size: Size::new(MENU_WINDOW_WIDTH, MENU_WINDOW_HEIGHT),
                        ..Default::default()
                    });
                    let mut browser_list = self.storage.get_browsers();
                    if browser_list.is_empty() {
                        let handlers = platform_tools::get_url_handlers();
                        self.storage.batch_insert_browsers(handlers.clone());
                        browser_list = handlers;
                    }
                    self.menu_window = Some(MenuWindow {
                        is_default_browser: IS_DEBUG || platform_tools::ensure_default_browser(),
                        current_page: MenuWindowPage::Home,
                        current_url: None,
                        browser_list: browser_list,
                        launch_time: Instant::now(),
                        stacks: vec![],
                        window_id: id,
                    });
                    open
                } else {
                    let (id, open) = window::open(window::Settings {
                        position: Position::Centered,
                        size: Size::new(SETTING_WINDOW_WIDTH, SETTING_WINDOW_HEIGHT),
                        ..Default::default()
                    });
                    let match_items = self.storage.find_all_match_items();
                    self.setting_window = Some(SettingWindow {
                        launch_time: Instant::now(),
                        match_items,
                        window_id: id,
                    });
                    open
                };
                open.then(|_| Task::none())
            }
            Message::CloseWindow(window_type) => match window_type {
                WindowType::Menu => {
                    if let Some(MenuWindow { window_id, .. }) = self.menu_window {
                        return window::close(window_id);
                    } else {
                        return Task::none();
                    }
                }
                WindowType::Setting => {
                    if let Some(SettingWindow { window_id, .. }) = self.setting_window {
                        return window::close(window_id);
                    } else {
                        return Task::none();
                    }
                }
            },
            Message::WindowClosed(window) => {
                if let Some(MenuWindow { window_id, .. }) = self.menu_window {
                    if window == window_id {
                        self.menu_window = None;
                    }
                }
                if let Some(SettingWindow { window_id, .. }) = self.setting_window {
                    if window == window_id {
                        self.setting_window = None;
                    }
                }
                Task::none()
            }
            Message::WindowUnfocused(window) => {
                if let Some(MenuWindow {
                    window_id,
                    launch_time,
                    ..
                }) = self.menu_window
                {
                    if launch_time.elapsed().as_secs() > 2 && window_id == window {
                        return Task::done(Message::CloseWindow(WindowType::Menu));
                    } else {
                        return Task::none();
                    }
                }
                if let Some(SettingWindow {
                    window_id,
                    launch_time,
                    ..
                }) = self.setting_window
                {
                    if launch_time.elapsed().as_secs() > 2 && window_id == window {
                        return Task::done(Message::CloseWindow(WindowType::Setting));
                    } else {
                        return Task::none();
                    }
                }
                Task::none()
            }

            Message::RefreshBrowserList => {
                self.storage.delete_all_browsers();
                let browsers = platform_tools::get_url_handlers();
                self.storage.batch_insert_browsers(browsers.clone());
                if let Some(menu_window) = &mut self.menu_window {
                    menu_window.browser_list = browsers;
                }
                Task::none()
            }
            Message::CloseApplication => iced::exit(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        println!("view window: {:?}", window_id);
        if self.menu_window.is_some() && self.menu_window.as_ref().unwrap().window_id == window_id {
            let MenuWindow {
                browser_list,
                is_default_browser,
                current_page,
                current_url,
                ..
            } = self.menu_window.as_ref().unwrap();
            let content = match current_page {
                MenuWindowPage::Home => {
                    let is_default_browser = is_default_browser;
                    if !is_default_browser {
                        pages::set_default_browser::set_default_browser(Message::SetAsDefault)
                    } else {
                        let shift_key_pressed = self.keyboard.shift();
                        let logo_key_pressed = self.keyboard.logo();
                        pages::select_browser::select_browser(
                            browser_list,
                            |path| {
                                if shift_key_pressed {
                                    Message::ShowMatchContainEditor(path.clone(), None)
                                } else {
                                    Message::LaunchBrowser(
                                        path,
                                        None,
                                        if logo_key_pressed {
                                            Some(ExternalOperation::SaveEqual)
                                        } else {
                                            None
                                        },
                                    )
                                }
                            },
                            |browser| Message::ListProfiles(browser),
                        )
                    }
                }
                MenuWindowPage::ProfileSelector {
                    profiles,
                    browser,
                    profile_text,
                } => {
                    let shift_key_pressed = self.keyboard.shift();
                    let logo_key_pressed = self.keyboard.logo();
                    let path = browser.path.clone();
                    pages::select_profile::select_profile(
                        profiles,
                        |profile| {
                            if shift_key_pressed {
                                Message::ShowMatchContainEditor(path.clone(), Some(profile.clone()))
                            } else {
                                Message::LaunchBrowser(
                                    path.clone(),
                                    Some(profile.clone()),
                                    if logo_key_pressed {
                                        Some(ExternalOperation::SaveEqual)
                                    } else {
                                        None
                                    },
                                )
                            }
                        },
                        |profile| Message::DeleteProfile(profile),
                        Message::Back,
                        |text| Message::TypeProfileText(text),
                        Message::AddProfile,
                        profile_text,
                    )
                }
                MenuWindowPage::MatchContainEditor {
                    match_container_text,
                    browser_path,
                    profile,
                } => pages::edit_match_value::edit_match_value(
                    Message::Back,
                    Message::LaunchBrowser(
                        browser_path.clone(),
                        profile.clone(),
                        Some(ExternalOperation::SaveContain),
                    ),
                    Message::TypeMatchContainText,
                    match_container_text,
                ),
            };
            let footer = footer(current_url.clone(), Message::RefreshBrowserList);
            Column::new().push(content).push(footer).into()
        } else if self.setting_window.is_some()
            && window_id == self.setting_window.as_ref().unwrap().window_id
        {
            let w = self.setting_window.as_ref().unwrap();
            let a = w.match_items;
            let content = Column::new().push(text("Setting"));
            Column::new().push(content).into()
        } else {
            Column::new().push(text("No window")).into()
        }
    }

    fn theme(&self, _: window::Id) -> Theme {
        Theme::default()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_url().map(Message::ReceiveUrl),
            if IS_DEBUG {
                Subscription::none()
            } else {
                iced::time::every(Duration::from_secs(1)).map(|_| Message::CheckDefaultStatus)
            },
            event::listen_with(|event, _status, window| -> Option<Message> {
                match event {
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        Some(Message::KeyboardModifiersChanged(modifiers))
                    }
                    Event::Window(window::Event::Unfocused) => {
                        if IS_DEBUG {
                            None
                        } else {
                            Some(Message::WindowUnfocused(window))
                        }
                    }
                    Event::Window(window::Event::Closed) => Some(Message::WindowClosed(window)),
                    _ => None,
                }
            }),
            Subscription::run(tray_menu_event_subscription),
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
    iced::daemon("Gomi", Gomi::update, Gomi::view)
        .font(include_bytes!("../fonts/Microns.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .theme(Gomi::theme)
        .subscription(Gomi::subscription)
        .run_with(Gomi::new)
}
