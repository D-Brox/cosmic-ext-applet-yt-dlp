// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use cosmic::Application;
use yt_dlp::Youtube;

mod applet;
mod i18n;
mod ytdlp;

#[tokio::main]
async fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Install and update deps before running applet
    let deps_dir = xdg::BaseDirectories::with_prefix(applet::Ytdlp::APP_ID)
        .expect("Failed to get xdg base dirs")
        .get_data_home();
    let yt_dlp_exists = deps_dir.join("yt-dlp").exists();
    let fetcher = Youtube::with_new_binaries(deps_dir.clone(), PathBuf::new())
        .await
        .unwrap();
    if !yt_dlp_exists {
        fetcher.update_downloader().await.unwrap();
    }
    
    cosmic::applet::run::<applet::Ytdlp>(deps_dir)
}
