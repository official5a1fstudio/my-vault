use crate::core::totp::{self, OtpAuth, TotpError};
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
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.screen, crate::ui::Screen::Authenticator, "Authenticator");
                ui.selectable_value(&mut self.screen, crate::ui::Screen::Settings, "Settings");
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            crate::ui::Screen::Authenticator => self.auth.ui(ui),
            crate::ui::Screen::Settings => self.settings.ui(ui),
        });
    }
}

impl AuthenticatorViewState {
    /// Authenticator add flow (mobile + desktop): manual entry and image-file QR import.
    /// Desktop additionally supports window/region capture-and-scan without camera.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add Authenticator (TOTP)");
        ui.label("Camera QR scanning is disabled. Import a QR image file or enter details manually.");
        ui.add_space(8.0);

        if ui.button("Import QR from image file").clicked() {
            self.import_from_image_file();
        }

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.label("Desktop: scan window or screen region (no camera)");
                ui.horizontal(|ui| {
                    if ui.button("Refresh monitors/windows").clicked() {
                        self.refresh_desktop_targets();
                    }
                    if ui.button("Scan selected target").clicked() {
                        self.scan_selected_desktop_target();
                    }
                });
                self.desktop_capture_ui(ui);
            });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.label("Manual / parsed otpauth details");
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
        if ui.button("Parse & add").clicked() {
            self.parse_and_add_manual();
        }

        if !self.status.is_empty() {
            ui.add_space(8.0);
            ui.label(&self.status);
        }

        if let Some(otp) = self.last_added.clone() {
            ui.add_space(8.0);
            self.refresh_preview(&otp);
            ui.group(|ui| {
                ui.label(format!("Added: {} ({})", otp.issuer, otp.account_name));
                if let Some(code) = &self.preview_code {
                    ui.label(format!("Current code: {}", format_code(code)));
                    ui.label(format!("Refreshes in: {}s", self.preview_remaining));
                    if ui.button("Copy code").clicked() {
                        ui.ctx().copy_text(code.clone());
                        self.status = "Code copied to clipboard.".into();
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
            let issuer = if self.issuer.trim().is_empty() { "Account".to_string() } else { self.issuer.trim().to_string() };
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
                        totp::desktop_capture::CaptureTarget::Monitor { id, name } => format!("Monitor {id}: {name}"),
                        _ => "Monitor".into(),
                    };
                    self.desktop_targets.push(DesktopTargetChoice { label, target: t });
                }
            }
            Err(e) => self.status = format!("Failed to list monitors: {e}"),
        }
        match totp::desktop_capture::list_windows() {
            Ok(windows) => {
                for t in windows {
                    let label = match &t {
                        totp::desktop_capture::CaptureTarget::Window { id, title, app_name } => {
                            let t = if title.is_empty() { "(untitled)" } else { title };
                            format!("Window {id}: {app_name} - {t}")
                        }
                        _ => "Window".into(),
                    };
                    self.desktop_targets.push(DesktopTargetChoice { label, target: t });
                }
            }
            Err(e) => self.status = format!("Failed to list windows: {e}"),
        }
        if self.selected_target.unwrap_or(usize::MAX) >= self.desktop_targets.len() {
            self.selected_target = if self.desktop_targets.is_empty() { None } else { Some(0) };
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
        ui.checkbox(&mut self.use_region, "Scan selected region (x,y,w,h) within target");
        if self.use_region {
            ui.horizontal(|ui| {
                ui.label("x"); ui.add(egui::TextEdit::singleline(&mut self.region_x).desired_width(60.0));
                ui.label("y"); ui.add(egui::TextEdit::singleline(&mut self.region_y).desired_width(60.0));
                ui.label("w"); ui.add(egui::TextEdit::singleline(&mut self.region_w).desired_width(60.0));
                ui.label("h"); ui.add(egui::TextEdit::singleline(&mut self.region_h).desired_width(60.0));
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
            match (self.region_x.parse(), self.region_y.parse(), self.region_w.parse(), self.region_h.parse()) {
                (Ok(x), Ok(y), Ok(w), Ok(h)) => Some(totp::desktop_capture::CaptureRegion { x, y, width: w, height: h }),
                _ => {
                    self.status = "Invalid region values (expected integers x,y,w,h).".into();
                    return;
                }
            }
        } else {
            None
        };
        let selection = totp::desktop_capture::CaptureSelection { target: choice.target, region };
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
