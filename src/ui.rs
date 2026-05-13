use crate::hid::{HidSession, Led, LedState};
use iced::widget::{button, column, pick_list, row, text, text_input};
use iced::{Background, Color, Element};

pub fn run() {
    iced::application(LedApp::new, LedApp::update, LedApp::view)
        .title("LED Control")
        .window(iced::window::Settings {
            size: iced::Size::new(380.0, 160.0),
            resizable: false,
            ..Default::default()
        })
        .run()
        .expect("Failed to launch LED Control window");
}

#[derive(Debug, Clone)]
enum Message {
    DeviceSelected(String),
    CountChanged(String),
    ToggleLed(Led),
}

struct LedApp {
    session: Option<HidSession>,
    selected: usize,
    selected_name: String,
    state: Option<LedState>,
    error: Option<String>,
    count: u32,
    count_str: String,
}

impl Default for LedApp {
    fn default() -> Self {
        Self::new()
    }
}

impl LedApp {
    fn new() -> Self {
        let session = HidSession::new().ok();
        let state = session
            .as_ref()
            .and_then(|s| s.keyboards().first())
            .and_then(|kb| session.as_ref().unwrap().get_led_state(kb).ok());
        LedApp {
            session,
            selected: 0,
            selected_name: "All Devices".to_string(),
            state,
            error: None,
            count: 1,
            count_str: "1".to_string(),
        }
    }

    fn device_names(&self) -> Vec<String> {
        let mut names = vec!["All Devices".to_string()];
        if let Some(session) = &self.session {
            for kb in session.keyboards() {
                names.push(kb.name.clone());
            }
        }
        names
    }

    fn refresh_state(&mut self) {
        let Some(session) = &self.session else { return };
        let idx = if self.selected > 0 { self.selected - 1 } else { 0 };
        self.state = session
            .keyboards()
            .get(idx)
            .and_then(|kb| session.get_led_state(kb).ok());
    }

    fn toggle(&mut self, led: Led) {
        let Some(session) = &self.session else {
            self.error = Some("No HID session available".into());
            return;
        };
        let keyboards = session.keyboards();
        let indices: Vec<usize> = if self.selected == 0 {
            (0..keyboards.len()).collect()
        } else if self.selected <= keyboards.len() {
            vec![self.selected - 1]
        } else {
            vec![]
        };
        self.error = None;
        let count = self.count;
        for idx in indices {
            if let Err(e) = session.toggle_led(&keyboards[idx], led, count) {
                self.error = Some(e.to_string());
            }
        }
        self.refresh_state();
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DeviceSelected(name) => {
                self.selected = if name == "All Devices" {
                    0
                } else {
                    self.session
                        .as_ref()
                        .and_then(|s| {
                            s.keyboards()
                                .iter()
                                .position(|kb| kb.name == name)
                                .map(|i| i + 1)
                        })
                        .unwrap_or(0)
                };
                self.selected_name = name;
                self.refresh_state();
            }
            Message::CountChanged(s) => {
                if let Ok(n) = s.parse::<u32>() && (1..=9999).contains(&n) {
                    self.count = n;
                }
                self.count_str = s;
            }
            Message::ToggleLed(led) => {
                self.toggle(led);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let names = self.device_names();
        let caps_on = self.state.as_ref().map(|s| s.caps).unwrap_or(false);
        let num_on = self.state.as_ref().map(|s| s.num).unwrap_or(false);
        let scroll_on = self.state.as_ref().map(|s| s.scroll).unwrap_or(false);

        let device_row = row![
            text("Device:").width(60),
            pick_list(names, Some(self.selected_name.clone()), Message::DeviceSelected),
        ]
        .spacing(8)
        .align_y(iced::alignment::Vertical::Center);

        let count_row = row![
            text("Count:").width(60),
            text_input("", &self.count_str)
                .on_input(Message::CountChanged)
                .width(60),
        ]
        .spacing(8)
        .align_y(iced::alignment::Vertical::Center);

        let button_row = row![
            led_button("Caps Lock", caps_on, Message::ToggleLed(Led::Caps)),
            led_button("Num Lock", num_on, Message::ToggleLed(Led::Num)),
            led_button("Scroll Lock", scroll_on, Message::ToggleLed(Led::Scroll)),
        ]
        .spacing(4);

        let mut content = column![
            text("LED Control").size(20),
            device_row,
            count_row,
            button_row,
        ]
        .spacing(6)
        .padding(12);

        if let Some(err) = &self.error {
            content = content.push(
                text(err.as_str()).style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(220, 60, 60)),
                }),
            );
        }

        content.into()
    }
}

fn led_button(label: &str, is_on: bool, msg: Message) -> Element<'_, Message> {
    let fill = if is_on {
        Color::from_rgb8(80, 180, 90)
    } else {
        Color::from_rgb8(55, 55, 60)
    };
    button(text(label))
        .on_press(msg)
        .style(move |_theme, _status| button::Style {
            background: Some(Background::Color(fill)),
            text_color: Color::WHITE,
            ..Default::default()
        })
        .width(120)
        .height(36)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> LedApp {
        LedApp {
            session: None,
            selected: 0,
            selected_name: "All Devices".to_string(),
            state: None,
            error: None,
            count: 1,
            count_str: "1".to_string(),
        }
    }

    #[test]
    fn count_changed_valid_input_updates_both() {
        let mut a = app();
        a.update(Message::CountChanged("42".to_string()));
        assert_eq!(a.count, 42);
        assert_eq!(a.count_str, "42");
    }

    #[test]
    fn count_changed_invalid_input_preserves_count_updates_string() {
        let mut a = app();
        a.update(Message::CountChanged("abc".to_string()));
        assert_eq!(a.count, 1);
        assert_eq!(a.count_str, "abc");
    }

    #[test]
    fn count_changed_out_of_range_preserves_count() {
        let mut a = app();
        a.update(Message::CountChanged("10000".to_string()));
        assert_eq!(a.count, 1);
        assert_eq!(a.count_str, "10000");
    }

    #[test]
    fn device_selected_all_devices_sets_index_zero() {
        let mut a = app();
        a.selected = 1;
        a.selected_name = "SomeKeyboard".to_string();
        a.update(Message::DeviceSelected("All Devices".to_string()));
        assert_eq!(a.selected, 0);
        assert_eq!(a.selected_name, "All Devices");
    }

    #[test]
    fn device_names_no_session_returns_all_devices_only() {
        let a = app();
        assert_eq!(a.device_names(), vec!["All Devices".to_string()]);
    }

    #[test]
    fn toggle_led_no_session_sets_error() {
        let mut a = app();
        a.update(Message::ToggleLed(Led::Caps));
        assert!(a.error.is_some());
    }
}
