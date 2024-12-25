use iced::{keyboard::Modifiers, widget::text_editor, window};

use crate::storage::BrowserInfo;

#[derive(Debug, Clone)]
pub enum Message {
    LaunchBrowser(String, Option<String>, Option<ExternalOperation>),
    SetAsDefault,
    ReceiveUrl(String),
    CheckDefaultStatus,
    ListProfiles(BrowserInfo),
    DeleteProfile(String),
    Back,
    AddProfile,
    TypeProfileText(String),
    ShowMatchContainEditor(String, Option<String>),
    TypeMatchContainText(text_editor::Action),
    KeyboardModifiersChanged(Modifiers),
    OpenWindow(WindowType),
    CloseWindow(WindowType),
    MoveWindow(window::Id),
    WindowClosed(window::Id),
    WindowUnfocused(window::Id),
    RefreshBrowserList,
    CloseApplication,
}

#[derive(Debug, Clone)]
pub enum ExternalOperation {
    SaveEqual,
    SaveContain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Menu,
    Setting,
}
