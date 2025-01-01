use iced::futures::Stream;
use iced::stream;
use tray_icon::menu::MenuEvent;

use crate::message::{Message, WindowType};

pub fn tray_menu_event_subscription() -> impl Stream<Item = Message> {
    stream::channel(1000, |mut output| async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let event = MenuEvent::receiver().recv().unwrap();
            match event.id.0.as_str() {
                "3" => output.try_send(Message::CloseApplication).unwrap(),
                "2" => output.try_send(Message::SetAsDefault).unwrap(),
                "1" => output
                    .try_send(Message::OpenWindow(WindowType::Setting))
                    .unwrap(),
                _ => (),
            }
        }
    })
}
