use iced::futures::Stream;
use iced::stream;
use tray_icon::menu::MenuEvent;

use crate::message::{Message, WindowType};

pub fn tray_menu_event_subscription() -> impl Stream<Item = Message> {
    stream::channel(1000, |mut output| async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let event = MenuEvent::receiver().recv().unwrap();
            if event.id == "2" {
                output.try_send(Message::CloseApplication).unwrap();
            }
            if event.id == "1" {
                output
                    .try_send(Message::OpenWindow(WindowType::Setting))
                    .unwrap();
            }
        }
    })
}
