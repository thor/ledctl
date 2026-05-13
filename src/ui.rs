use crate::hid::{HidSession, Led, LedState};
use eframe::egui;

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LED Control")
            .with_inner_size([440.0, 180.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "LED Control",
        options,
        Box::new(|_cc| Ok(Box::new(LedApp::new()))),
    )
    .expect("Failed to launch LED Control window");
}

struct LedApp {
    session: Option<HidSession>,
    selected: usize, // 0 = "All Devices"; 1..N = keyboards[selected - 1]
    state: Option<LedState>,
    error: Option<String>,
    count: u32,
    count_str: String,
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
            state,
            error: None,
            count: 1,
            count_str: "1".to_string(),
        }
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
}

impl eframe::App for LedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LED Control");
            ui.add_space(6.0);

            // Device row
            ui.horizontal(|ui| {
                ui.label("Device:");
                let selected_label = if self.selected == 0 {
                    "All Devices".to_string()
                } else {
                    self.session
                        .as_ref()
                        .and_then(|s| s.keyboards().get(self.selected - 1))
                        .map(|kb| kb.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string())
                };
                egui::ComboBox::from_id_source("device_picker")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected, 0, "All Devices");
                        if let Some(session) = &self.session {
                            for (i, kb) in session.keyboards().iter().enumerate() {
                                ui.selectable_value(&mut self.selected, i + 1, &kb.name);
                            }
                        }
                    });
            });

            // Count row
            ui.horizontal(|ui| {
                ui.label("Count:");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.count_str).desired_width(60.0),
                );
                if resp.changed() {
                    if let Ok(n) = self.count_str.parse::<u32>() {
                        if (1..=9999).contains(&n) {
                            self.count = n;
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // LED buttons
            ui.horizontal(|ui| {
                let caps_on = self.state.as_ref().map(|s| s.caps).unwrap_or(false);
                let num_on = self.state.as_ref().map(|s| s.num).unwrap_or(false);
                let scroll_on = self.state.as_ref().map(|s| s.scroll).unwrap_or(false);

                if led_button(ui, "Caps Lock", caps_on).clicked() {
                    self.toggle(Led::Caps);
                }
                ui.add_space(4.0);
                if led_button(ui, "Num Lock", num_on).clicked() {
                    self.toggle(Led::Num);
                }
                ui.add_space(4.0);
                if led_button(ui, "Scroll Lock", scroll_on).clicked() {
                    self.toggle(Led::Scroll);
                }
            });

            // Error display
            if let Some(err) = &self.error {
                ui.add_space(6.0);
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), err);
            }
        });
    }
}

fn led_button(ui: &mut egui::Ui, label: &str, is_on: bool) -> egui::Response {
    let fill = if is_on {
        egui::Color32::from_rgb(80, 180, 90)
    } else {
        egui::Color32::from_rgb(55, 55, 60)
    };
    let text = egui::RichText::new(label).strong();
    ui.add(egui::Button::new(text).fill(fill).min_size(egui::vec2(110.0, 36.0)))
}
