#[cfg(test)]
mod tests {
    use ledctl::hid::{Led, LedError, LedState};

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
