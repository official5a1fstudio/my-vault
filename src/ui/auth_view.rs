use crate::core::totp::{self, OtpAuth, TotpError};
use crate::ui::theme;
use eframe::egui;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct AuthenticatorViewState {
    issuer: String,
    account_name: String,
    secret: String,
    status: String,
    last_added: Option<OtpAuth>,
    preview_code: Option<String>,
    preview_remaining: u64,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    desktop_targets: Vec<DesktopTargetChoice>,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    selected_target: Option<usize>,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    region_x: String,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    region_y: String,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    region_w: String,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    region_h: String,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    use_region: bool,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[derive(Clone)]
struct DesktopTargetChoice {
    label: String,
    target: totp::desktop_capture::CaptureTarget,
}

/// Default eframe app state used by the desktop binary scaffold.
#[derive(Default)]
pub struct AppState {
    pub screen: crate::ui::Screen,
    pub auth: AuthenticatorViewState,
    pub settings: crate::ui::settings_view::SettingsViewState,
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme_id = egui::Id::new("mawthooq_theme_applied");
        if !ctx.data(|data| data.get_temp::<bool>(theme_id).unwrap_or(false)) {
            theme::apply(ctx);
            ctx.data_mut(|data| data.insert_temp(theme_id, true));
        }

        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                theme::brand_header(ui);
                ui.add_space(32.0);

                let auth_active = self.screen == crate::ui::Screen::Authenticator;
                let settings_active = self.screen == crate::ui::Screen::Settings;

                let mut auth_label =
                    egui::RichText::new("AUTHENTICATOR")
                        .size(11.0)
                        .color(if auth_active {
                            theme::NEON
                        } else {
                            theme::VOID_MUTED
                        });
                if auth_active {
                    auth_label = auth_label.underline();
                }

                let mut settings_label =
                    egui::RichText::new("SETTINGS")
                        .size(11.0)
                        .color(if settings_active {
                            theme::NEON
                        } else {
                            theme::VOID_MUTED
                        });
                if settings_active {
                    settings_label = settings_label.underline();
                }

                if ui.button(auth_label).clicked() {
                    self.screen = crate::ui::Screen::Authenticator;
                }
                ui.add_space(6.0);
                if ui.button(settings_label).clicked() {
                    self.screen = crate::ui::Screen::Settings;
                }
            });
            ui.add_space(10.0);
            let rect = ui.max_rect();
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.bottom() - 1.0),
                    egui::pos2(rect.right(), rect.bottom() - 1.0),
                ],
                egui::Stroke::new(1.0_f32, theme::VOID_LINE),
            );
        });

        let screen = self.screen;
        egui::CentralPanel::default().show(ctx, |ui| {
            theme::grid_background(ui);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let outer_width = ui.available_width();
                    let column_width = outer_width.min(920.0);
                    let left_margin = ((outer_width - column_width) / 2.0).max(0.0);
                    ui.add_space(28.0);
                    ui.horizontal(|ui| {
                        ui.add_space(left_margin);
                        ui.allocate_ui_with_layout(
                            egui::vec2(column_width, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| match screen {
                                crate::ui::Screen::Authenticator => self.auth.ui(ui),
                                crate::ui::Screen::Settings => self.settings.ui(ui),
                            },
                        );
                    });
                    ui.add_space(48.0);
                });
        });
    }
}

impl AuthenticatorViewState {
    /// Authenticator add flow (mobile + desktop): manual entry and image-file QR import.
    /// Desktop additionally supports window/region capture-and-scan without camera.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        theme::page_header(
            ui,
            "SOFTWARE  /  TOTP",
            "Add an authenticator",
            "Local codes. No cloud. Import a QR image or enter otpauth details manually — everything stays on this device.",
        );
        ui.add_space(26.0);

        theme::card(ui, "01", "QR IMAGE IMPORT", |ui| {
            ui.label(
                egui::RichText::new(
                    "Camera QR scanning is disabled. Import a QR image file instead.",
                )
                .color(theme::VOID_MUTED),
            );
            ui.add_space(14.0);
            if theme::primary_button(ui, "Import QR from image file") {
                self.import_from_image_file();
            }
        });

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            ui.add_space(16.0);
            theme::card(ui, "02", "DESKTOP CAPTURE", |ui| {
                ui.label(
                    egui::RichText::new("Scan a window or screen region — no camera required.")
                        .color(theme::VOID_MUTED),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Refresh monitors/windows") {
                        self.refresh_desktop_targets();
                    }
                    ui.add_space(8.0);
                    if theme::secondary_button(ui, "Scan selected target") {
                        self.scan_selected_desktop_target();
                    }
                });
                ui.add_space(10.0);
                self.desktop_capture_ui(ui);
            });
        }

        ui.add_space(16.0);
        theme::card(ui, "03", "MANUAL ENTRY", |ui| {
            ui.label(
                egui::RichText::new(
                    "Issuer, account and Base32 secret — or a full otpauth:// URI.",
                )
                .color(theme::VOID_MUTED),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label("Issuer");
                ui.text_edit_singleline(&mut self.issuer);
            });
            ui.horizontal(|ui| {
                ui.label("Account");
                ui.text_edit_singleline(&mut self.account_name);
            });
            ui.horizontal(|ui| {
                ui.label("Secret (Base32 or otpauth://)");
                ui.text_edit_singleline(&mut self.secret);
            });
            ui.add_space(12.0);
            if theme::primary_button(ui, "Parse & add") {
                self.parse_and_add_manual();
            }
        });

        let status = self.status.clone();
        let added = self.last_added.clone();
        if !status.is_empty() || added.is_some() {
            ui.add_space(16.0);
            theme::card(ui, "04", "RESULT", |ui| {
                if !status.is_empty() {
                    let is_error = status.starts_with("Failed")
                        || status.starts_with("Invalid")
                        || status.starts_with("Desktop capture");
                    theme::status(ui, &status, is_error);
                }
                if let Some(otp) = added {
                    ui.add_space(12.0);
                    self.refresh_preview(&otp);
                    ui.label(
                        egui::RichText::new(format!(
                            "Added: {} ({})",
                            otp.issuer, otp.account_name
                        ))
                        .color(theme::VOID_INK),
                    );
                    if let Some(code) = &self.preview_code {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("Current code: {}", format_code(code)))
                                .color(theme::NEON)
                                .size(26.0),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Refreshes in: {}s",
                                self.preview_remaining
                            ))
                            .color(theme::VOID_MUTED),
                        );
                        ui.add_space(10.0);
                        if theme::primary_button(ui, "Copy code") {
                            ui.ctx().copy_text(code.clone());
                            self.status = "Code copied to clipboard.".into();
                        }
                    }
                }
            });
        }
    }

    /// Image-file QR import flow for both mobile and desktop.
    pub fn import_from_image_file(&mut self) {
        #[cfg(not(target_os = "android"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
                .pick_file()
            {
                self.import_from_image_path(path);
            }
        }
        #[cfg(target_os = "android")]
        {
            self.status = "Select an image file via the Android file picker and pass its bytes/path to import_from_image_path/bytes.".into();
        }
    }

    pub fn import_from_image_path(&mut self, path: PathBuf) {
        match totp::decode_otpauth_from_image_path(&path) {
            Ok(otp) => self.apply_otp(otp, "Imported TOTP from image file."),
            Err(e) => self.status = format!("Failed to import QR from image: {e}"),
        }
    }

    pub fn import_from_image_bytes(&mut self, bytes: &[u8]) {
        match totp::decode_otpauth_from_image_bytes(bytes) {
            Ok(otp) => self.apply_otp(otp, "Imported TOTP from image data."),
            Err(e) => self.status = format!("Failed to import QR from image: {e}"),
        }
    }

    fn parse_and_add_manual(&mut self) {
        let input = self.secret.trim();
        let result = if input.to_ascii_lowercase().starts_with("otpauth://") {
            totp::parse_otpauth_uri(input)
        } else {
            let issuer = if self.issuer.trim().is_empty() {
                "Account".to_string()
            } else {
                self.issuer.trim().to_string()
            };
            let account = self.account_name.trim().to_string();
            let uri = format!(
                "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
                urlencoding(&issuer),
                urlencoding(if account.is_empty() { "user" } else { &account }),
                urlencoding(input),
                urlencoding(&issuer),
            );
            totp::parse_otpauth_uri(&uri)
        };
        match result {
            Ok(otp) => self.apply_otp(otp, "Authenticator entry parsed successfully."),
            Err(e) => self.status = format!("Invalid TOTP input: {e}"),
        }
    }

    fn apply_otp(&mut self, otp: OtpAuth, msg: &str) {
        self.issuer = otp.issuer.clone();
        self.account_name = otp.account_name.clone();
        self.secret = otp.normalized_secret();
        self.status = msg.to_string();
        self.last_added = Some(otp);
        self.preview_code = None;
    }

    fn refresh_preview(&mut self, otp: &OtpAuth) {
        let now = unix_now();
        self.preview_remaining = totp::seconds_remaining(otp.period, now);
        self.preview_code = totp::generate_totp(otp, now).ok();
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn refresh_desktop_targets(&mut self) {
        self.desktop_targets.clear();
        match totp::desktop_capture::list_monitors() {
            Ok(monitors) => {
                for t in monitors {
                    let label = match &t {
                        totp::desktop_capture::CaptureTarget::Monitor { id, name } => {
                            format!("Monitor {id}: {name}")
                        }
                        _ => "Monitor".into(),
                    };
                    self.desktop_targets
                        .push(DesktopTargetChoice { label, target: t });
                }
            }
            Err(e) => self.status = format!("Failed to list monitors: {e}"),
        }
        match totp::desktop_capture::list_windows() {
            Ok(windows) => {
                for t in windows {
                    let label = match &t {
                        totp::desktop_capture::CaptureTarget::Window {
                            id,
                            title,
                            app_name,
                        } => {
                            let t = if title.is_empty() {
                                "(untitled)"
                            } else {
                                title
                            };
                            format!("Window {id}: {app_name} - {t}")
                        }
                        _ => "Window".into(),
                    };
                    self.desktop_targets
                        .push(DesktopTargetChoice { label, target: t });
                }
            }
            Err(e) => self.status = format!("Failed to list windows: {e}"),
        }
        if self.selected_target.unwrap_or(usize::MAX) >= self.desktop_targets.len() {
            self.selected_target = if self.desktop_targets.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn desktop_capture_ui(&mut self, ui: &mut egui::Ui) {
        if self.desktop_targets.is_empty() {
            ui.label("No capture targets loaded. Click 'Refresh monitors/windows'.");
        } else {
            egui::ComboBox::from_label("Capture target")
                .selected_text(
                    self.selected_target
                        .and_then(|i| self.desktop_targets.get(i))
                        .map(|c| c.label.as_str())
                        .unwrap_or("Select target"),
                )
                .show_ui(ui, |ui| {
                    for (i, choice) in self.desktop_targets.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_target, Some(i), &choice.label);
                    }
                });
        }
        ui.checkbox(
            &mut self.use_region,
            "Scan selected region (x,y,w,h) within target",
        );
        if self.use_region {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::TextEdit::singleline(&mut self.region_x).desired_width(60.0));
                ui.label("y");
                ui.add(egui::TextEdit::singleline(&mut self.region_y).desired_width(60.0));
                ui.label("w");
                ui.add(egui::TextEdit::singleline(&mut self.region_w).desired_width(60.0));
                ui.label("h");
                ui.add(egui::TextEdit::singleline(&mut self.region_h).desired_width(60.0));
            });
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn scan_selected_desktop_target(&mut self) {
        let Some(idx) = self.selected_target else {
            self.status = format!("{}", TotpError::NoCaptureTarget);
            return;
        };
        let Some(choice) = self.desktop_targets.get(idx).cloned() else {
            self.status = format!("{}", TotpError::NoCaptureTarget);
            return;
        };
        let region = if self.use_region {
            match (
                self.region_x.parse(),
                self.region_y.parse(),
                self.region_w.parse(),
                self.region_h.parse(),
            ) {
                (Ok(x), Ok(y), Ok(w), Ok(h)) => Some(totp::desktop_capture::CaptureRegion {
                    x,
                    y,
                    width: w,
                    height: h,
                }),
                _ => {
                    self.status = "Invalid region values (expected integers x,y,w,h).".into();
                    return;
                }
            }
        } else {
            None
        };
        let selection = totp::desktop_capture::CaptureSelection {
            target: choice.target,
            region,
        };
        match totp::desktop_capture::scan_capture_selection(&selection) {
            Ok(otp) => self.apply_otp(otp, "Scanned TOTP QR from selected desktop window/region."),
            Err(e) => self.status = format!("Desktop capture scan failed: {e}"),
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_code(code: &str) -> String {
    if code.len() == 6 {
        format!("{} {}", &code[0..3], &code[3..6])
    } else {
        code.to_string()
    }
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
