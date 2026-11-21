use crate::ui::theme;
use eframe::egui;

#[derive(Default)]
pub struct SettingsViewState {
    #[cfg(target_os = "android")]
    pub bridge_port: String,
    #[cfg(target_os = "android")]
    pub bridge_running: bool,
    #[cfg(target_os = "android")]
    bridge: Option<crate::bridge::KeyboardBridgeHub>,
    pub status: String,
}

impl SettingsViewState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        theme::page_header(
            ui,
            "SOFTWARE  /  PREFERENCES",
            "Settings",
            "Quiet defaults, clear boundaries. Companion features are only enabled where the platform can support them.",
        );
        ui.add_space(26.0);

        #[cfg(target_os = "android")]
        {
            self.android_keyboard_bridge_ui(ui);
            ui.add_space(16.0);
        }

        #[cfg(not(target_os = "android"))]
        {
            theme::card(ui, "01", "KEYBOARD BRIDGE", |ui| {
                ui.label(
                    egui::RichText::new("Keyboard bridge")
                        .color(theme::VOID_INK)
                        .size(16.0),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Companion keyboard-bridge hub is available only on Android builds. \
                         Desktop keyboard-bridge endpoints are disabled/omitted.",
                    )
                    .color(theme::VOID_MUTED),
                );
            });
            ui.add_space(16.0);
        }

        theme::card(ui, "02", "AUTHENTICATOR CAPTURE", |ui| {
            ui.label(
                egui::RichText::new("Authenticator capture")
                    .color(theme::VOID_INK)
                    .size(16.0),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Camera/webcam QR scanning is removed on all platforms.")
                    .color(theme::VOID_MUTED),
            );
            ui.label(
                egui::RichText::new(
                    "Use image-file QR import (all platforms) or desktop window/region capture (Windows/Linux).",
                )
                .color(theme::VOID_MUTED),
            );
        });
    }

    #[cfg(target_os = "android")]
    fn android_keyboard_bridge_ui(&mut self, ui: &mut egui::Ui) {
        theme::card(ui, "01", "KEYBOARD BRIDGE (ANDROID)", |ui| {
            ui.label(
                egui::RichText::new(
                    "Local companion keyboard-bridge hub for mobile keyboard paste integration.",
                )
                .color(theme::VOID_MUTED),
            );
            ui.add_space(12.0);
            if self.bridge_port.is_empty() {
                self.bridge_port = "8765".to_string();
            }
            ui.horizontal(|ui| {
                ui.label("Port");
                ui.text_edit_singleline(&mut self.bridge_port);
            });
            let running = self
                .bridge
                .as_ref()
                .map(|b| b.is_running())
                .unwrap_or(false);
            ui.label(if running {
                "Status: running"
            } else {
                "Status: stopped"
            });
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, "Start hub") {
                    let port = self.bridge_port.parse::<u16>().unwrap_or(8765);
                    let hub = crate::bridge::KeyboardBridgeHub::new(port);
                    match hub.start() {
                        Ok(()) => {
                            self.status =
                                format!("Android keyboard-bridge hub started on 127.0.0.1:{port}.");
                            self.bridge = Some(hub);
                        }
                        Err(e) => {
                            self.status = format!("Failed to start keyboard-bridge hub: {e}");
                        }
                    }
                }
                ui.add_space(8.0);
                if theme::secondary_button(ui, "Stop hub") {
                    if let Some(hub) = self.bridge.take() {
                        hub.stop();
                    }
                    self.status = "Android keyboard-bridge hub stopped.".into();
                }
            });
        });
        let status = self.status.clone();
        if !status.is_empty() {
            ui.add_space(8.0);
            let is_error = status.starts_with("Failed");
            theme::status(ui, &status, is_error);
        }
    }
}
