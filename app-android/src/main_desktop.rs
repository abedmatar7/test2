fn main() -> Result<(), slint::PlatformError> {
    let ui = medpulse_app::create_and_bind_ui(Some("medpulse_desktop.sqlite3"))?;
    ui.run()
}
