use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use ::yt_dlp::Youtube;
use cosmic::Application;
use tempfile::Builder;
use tokio::task::JoinSet;
use yt_dlp::{
    executor::Executor,
    fetcher::{
        Fetcher,
        deps::{Libraries, LibraryInstaller},
        download_manager::DownloadManager,
    },
    utils::find_executable,
};

use crate::applet::Ytdlp;

pub async fn binaries() -> PathBuf {
    // Install and update deps before running applet
    let deps_dir = xdg::BaseDirectories::with_prefix(Ytdlp::APP_ID)
        .expect("Failed to get xdg base dirs")
        .get_data_home();
    let yt_dlp_exists = deps_dir.join("yt-dlp").exists();
    let installer = LibraryInstaller::new(deps_dir.clone());

    // Check if binaries already exist
    let youtube_path = deps_dir.join(find_executable("yt-dlp"));
    let ffmpeg_path = deps_dir.join(find_executable("ffmpeg"));

    let youtube = if youtube_path.exists() {
        youtube_path
    } else {
        installer
            .install_youtube(None)
            .await
            .expect("Failed to download yt-dlp")
    };

    let ffmpeg = if ffmpeg_path.exists() {
        ffmpeg_path
    } else {
        installer
            .install_ffmpeg(None)
            .await
            .expect("Failed to download ffmpeg")
    };
    if !yt_dlp_exists {
        Youtube {
            libraries: Libraries::new(youtube, ffmpeg),
            output_dir: "".into(),
            args: vec![],
            timeout: Duration::from_secs(30),
            cache: None,
            download_cache: None,
            download_manager: Arc::new(DownloadManager::new()),
        }
        .update_downloader()
        .await
        .unwrap();
    }
    deps_dir
}

pub fn with_output_dir(lib_dir: &Path, output_dir: PathBuf) -> Youtube {
    let youtube = lib_dir.join("yt-dlp");
    let ffmpeg = lib_dir.join("ffmpeg");
    let libraries = Libraries::new(youtube, ffmpeg);
    Youtube {
        libraries,
        output_dir,
        args: vec![],
        timeout: Duration::from_secs(30),
        cache: None,
        download_cache: None,
        download_manager: Arc::new(DownloadManager::new()),
    }
    // libs, output_dir)
}

pub async fn manifest(
    url: String,
    output_dir: PathBuf,
    ffmpeg: PathBuf,
    title: &String,
    video_selected: bool,
) -> bool {
    let dir = Builder::new()
        .prefix(".yt-dlp")
        .tempdir_in(&output_dir)
        .expect("Failed to create temp dir");
    let file = output_dir.join(format!("{title}.m3u8"));
    let base_url = Path::new(&url).parent().unwrap();
    if Fetcher::new(&url).fetch_asset(&file).await.is_err() {
        return false;
    }
    let Ok(m3u8) = tokio::fs::read(&file).await else {
        return false;
    };
    let (_, parsed) = m3u8_rs::parse_media_playlist(&m3u8).expect("Failed to parse");
    let mut files = vec![];
    let mut set = JoinSet::new();
    for segment in parsed.segments {
        let file_name = dir
            .path()
            .join(Path::new(&segment.uri).file_name().unwrap());
        let segment_url = base_url.join(segment.uri).to_str().unwrap().to_owned();
        files.push(file_name.to_str().unwrap().to_owned());
        set.spawn(async { Fetcher::new(segment_url).fetch_asset(file_name).await });
    }
    if set.join_all().await.iter().any(Result::is_err) {
        return false;
    }
    let concat = format!("concat:{}", files.join("|"));
    let file_name = output_dir.join(format!(
        "{title}.{}",
        if video_selected { "mp4" } else { "m4a" }
    ));
    let mut args = vec!["-i", &concat];
    args.extend_from_slice(&["-hwaccel", "auto", "-c:a", "copy", "-acodec", "copy"]);
    if video_selected {
        args.extend_from_slice(&["-c:v", "copy", "-vcodec", "copy"]);
    }
    args.push(file_name.to_str().unwrap());

    let executor = Executor {
        executable_path: ffmpeg,
        timeout: Duration::default(),
        args: yt_dlp::utils::to_owned(args),
    };
    executor.execute().await.is_ok()
}
