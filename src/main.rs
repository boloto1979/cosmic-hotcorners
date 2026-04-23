// SPDX-License-Identifier: MPL-2.0

mod app;
mod config;
mod i18n;
mod settings_app;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--settings") {
        let settings = cosmic::app::Settings::default();
        cosmic::app::run::<settings_app::SettingsApp>(settings, ())
    } else {
        let settings = cosmic::app::Settings::default()
            .no_main_window(true)
            .transparent(true);
        cosmic::app::run::<app::AppModel>(settings, ())
    }
}
