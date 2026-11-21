pub mod core;

#[cfg(target_os = "android")]
pub mod bridge;

pub mod ui;

#[cfg(target_os = "android")]
pub use bridge::KeyboardBridgeHub;

#[cfg(not(target_os = "android"))]
pub fn start() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Mawthooq — موثوق")
            .with_inner_size([1280.0, 840.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Mawthooq — موثوق",
        options,
        Box::new(|_cc| Ok(Box::new(ui::auth_view::AppState::default()))),
    )
}

/// Android native-activity entry point.
///
/// The APK packages `libmy_vault_rs.so`, so this symbol must live in the library
/// crate rather than in `src/main.rs`.
#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        viewport: eframe::egui::ViewportBuilder::default().with_title("Mawthooq"),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Mawthooq",
        options,
        Box::new(|_cc| Ok(Box::new(ui::auth_view::AppState::default()))),
    ) {
        log::error!("Mawthooq failed to start: {err}");
    }
}
