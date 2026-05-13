#[cfg(test)]
mod tests {
    use ledctl::hid::{Led, LedError, LedState};
    use ledctl::cli::{parse_args, CliError, Command, LedAction};

    #[test]
    fn parse_list_flag() {
        let args = ["--list".to_string()];
        let parsed = parse_args(&args).unwrap();
        assert!(matches!(parsed.command, Command::List));
    }

    #[test]
    fn parse_caps_toggle() {
        let args = ["caps".to_string(), "toggle".to_string()];
        let parsed = parse_args(&args).unwrap();
        assert!(matches!(parsed.command, Command::Control { led: ledctl::hid::Led::Caps, action: LedAction::Toggle }));
        assert_eq!(parsed.count, 1);
        assert!(parsed.device_filter.is_none());
    }

    #[test]
    fn parse_num_on_with_device_and_count() {
        let args = [
            "--device".to_string(), "Apple".to_string(),
            "num".to_string(), "on".to_string(),
            "--count".to_string(), "5".to_string(),
        ];
        let parsed = parse_args(&args).unwrap();
        assert!(matches!(parsed.command, Command::Control { led: ledctl::hid::Led::Num, action: LedAction::On }));
        assert_eq!(parsed.count, 5);
        assert_eq!(parsed.device_filter.as_deref(), Some("Apple"));
    }

    #[test]
    fn parse_scroll_off() {
        let args = ["scroll".to_string(), "off".to_string()];
        let parsed = parse_args(&args).unwrap();
        assert!(matches!(parsed.command, Command::Control { led: ledctl::hid::Led::Scroll, action: LedAction::Off }));
    }

    #[test]
    fn parse_count_zero_is_error() {
        let args = ["caps".to_string(), "toggle".to_string(), "--count".to_string(), "0".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_count_10000_is_error() {
        let args = ["caps".to_string(), "toggle".to_string(), "--count".to_string(), "10000".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_unknown_led_is_error() {
        let args = ["brightness".to_string(), "on".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_unknown_state_is_error() {
        let args = ["caps".to_string(), "blink".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_missing_device_value_is_error() {
        let args = ["--device".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_no_args_is_error() {
        let args: [String; 0] = [];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn led_usage_caps() {
        assert_eq!(Led::Caps.usage(), 0x02);
    }

    #[test]
    fn led_usage_num() {
        assert_eq!(Led::Num.usage(), 0x01);
    }

    #[test]
    fn led_usage_scroll() {
        assert_eq!(Led::Scroll.usage(), 0x03);
    }

    #[test]
    fn led_state_fields_accessible() {
        let s = LedState { caps: true, num: false, scroll: true };
        assert!(s.caps);
        assert!(!s.num);
        assert!(s.scroll);
    }

    #[test]
    fn led_error_display_device_not_found() {
        let msg = LedError::DeviceNotFound.to_string();
        assert!(msg.contains("keyboard") || msg.contains("device"), "got: {}", msg);
    }

    #[test]
    fn led_error_display_element_not_found() {
        let msg = LedError::ElementNotFound(Led::Caps).to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn led_error_display_iokit_error() {
        let msg = LedError::IoKitError(0xe00002bcu32 as i32).to_string();
        assert!(msg.contains("e00002bc") || msg.contains("IOKit"), "got: {}", msg);
    }

    #[test]
    fn led_error_display_manager_open_failed() {
        let msg = LedError::ManagerOpenFailed(0xe00002bcu32 as i32).to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    #[ignore]
    fn hid_session_opens_and_lists_keyboards() {
        let session = ledctl::hid::HidSession::new().expect("HidSession::new failed");
        println!("Found {} keyboard(s):", session.keyboards().len());
        for kb in session.keyboards() {
            println!("  [{:#010x}] {}", kb.location_id, kb.name);
        }
    }

    #[test]
    #[ignore]
    fn get_led_state_returns_state_for_first_keyboard() {
        let session = ledctl::hid::HidSession::new().expect("HidSession::new failed");
        let kb = session.keyboards().first().expect("no keyboard found");
        let state = session.get_led_state(kb).expect("get_led_state failed");
        println!("caps={} num={} scroll={}", state.caps, state.num, state.scroll);
    }

    #[test]
    #[ignore]
    fn set_led_and_restore() {
        let session = ledctl::hid::HidSession::new().expect("HidSession::new failed");
        let kb = session.keyboards().first().expect("no keyboard found");
        let original = session.get_led_state(kb).expect("get_led_state failed");

        // Turn scroll lock on, then restore.
        session.set_led(kb, ledctl::hid::Led::Scroll, true).expect("set_led on failed");
        let state = session.get_led_state(kb).expect("get_led_state failed");
        assert!(state.scroll, "scroll should be on after set_led(true)");

        session.set_led(kb, ledctl::hid::Led::Scroll, original.scroll).expect("set_led restore failed");
    }

    #[test]
    #[ignore]
    fn toggle_led_count_2_restores_state() {
        let session = ledctl::hid::HidSession::new().expect("HidSession::new failed");
        let kb = session.keyboards().first().expect("no keyboard found");
        let original = session.get_led_state(kb).expect("get_led_state failed");

        session.toggle_led(kb, ledctl::hid::Led::Num, 2).expect("toggle_led failed");

        let after = session.get_led_state(kb).expect("get_led_state failed");
        assert_eq!(original.num, after.num, "even count should restore original state");
    }
}
