// SPDX-License-Identifier: GPL-3.0-only

mod applet;
mod fetcher;
mod formats;
mod i18n;

#[tokio::main]
async fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    cosmic::applet::run::<applet::Ytdlp>(fetcher::binaries().await)
}
