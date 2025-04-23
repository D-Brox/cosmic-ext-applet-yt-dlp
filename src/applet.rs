// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use cosmic::app::{Core, Task};
use cosmic::applet::padded_control;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Limits};
use cosmic::iced_widget::{button, column, pick_list, row};
use cosmic::widget::segmented_button::{Entity, SingleSelectModel};
use cosmic::widget::text::body;
use cosmic::widget::{divider, segmented_control, text_input};
use cosmic::{Action, Application, Apply, Element};

use ashpd::desktop::file_chooser::SelectedFiles;
use notify_rust::Notification;

use crate::formats::{AudioCodec, AudioQuality, VideoCodec, VideoQuality};
use crate::{fetcher, fl, fl_str};

#[derive(Default)]
pub struct Ytdlp {
    core: Core,

    download_type: SingleSelectModel,
    video_entity: Entity,

    video_folder: String,
    audio_folder: String,
    url: String,

    video_quality: VideoQuality,
    audio_quality: AudioQuality,
    video_codec: VideoCodec,
    audio_codec: AudioCodec,

    lib_dir: PathBuf,
    popup: Option<Id>,
    downloading: u32,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    EnterURL(String),
    SelectFolder,
    ProcessSelectFolder(String),
    ChangeType(Entity),
    VideoQuality(VideoQuality),
    AudioQuality(AudioQuality),
    VideoCodec(VideoCodec),
    AudioCodec(AudioCodec),
    Download,
    Finished,
}

impl Application for Ytdlp {
    type Executor = cosmic::executor::Default;

    type Flags = PathBuf;

    type Message = Message;

    const APP_ID: &'static str = "dev.DBrox.CosmicYtdlp";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut download_type = SingleSelectModel::default();
        let video_entity = download_type.insert().text(fl!("video")).id();
        download_type.insert().text(fl!("audio"));
        download_type.activate(video_entity);

        let video_folder = xdg_user::videos()
            .ok()
            .flatten()
            .map_or(String::from("~/Videos"), |path| {
                String::from(path.to_string_lossy())
            });
        let audio_folder = xdg_user::music()
            .ok()
            .flatten()
            .map_or(String::from("~/Music"), |path| {
                String::from(path.to_string_lossy())
            });

        let app = Ytdlp {
            core,
            download_type,
            video_entity,
            video_folder,
            audio_folder,
            lib_dir: flags,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("multimedia-video-player-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let video_selected = self.video_entity == self.download_type.active();
        let pad = self.core.applet.suggested_padding(true);
        let Spacing {
            space_xxs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        let content_list = column![
            text_input(fl!("url"), &self.url)
                .on_input(Message::EnterURL)
                .apply(padded_control)
                .width(Length::Fill),
            segmented_control::horizontal(&self.download_type)
                .on_activate(Message::ChangeType)
                .apply(padded_control)
                .width(Length::Fill),
            if video_selected {
                self.view_video()
            } else {
                self.view_audio()
            },
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            row![
                body(fl!("folder")).width(Length::Fill),
                button(body(fl!("browse"))).on_press(Message::SelectFolder)
            ]
            .align_y(Alignment::Center)
            .spacing(pad)
            .apply(padded_control),
            text_input(
                "",
                if video_selected {
                    self.video_folder.clone()
                } else {
                    self.audio_folder.clone()
                }
            )
            .on_focus(Message::SelectFolder)
            .on_input(Message::ProcessSelectFolder)
            .apply(padded_control),
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            row![
                body(fl!("downloading", total = self.downloading)).width(Length::Fill),
                button(body(fl!("download"))).on_press(Message::Download),
            ]
            .align_y(Alignment::Center)
            .spacing(pad)
            .apply(padded_control)
        ]
        .padding(pad);

        self.core.applet.popup_container(content_list).into()
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(800.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1000.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::EnterURL(url) => self.url = url,
            Message::ChangeType(id) => self.download_type.activate(id),
            Message::VideoQuality(video_resolution) => self.video_quality = video_resolution,
            Message::AudioQuality(audio_quality) => self.audio_quality = audio_quality,
            Message::VideoCodec(video_codec) => self.video_codec = video_codec,
            Message::AudioCodec(audio_codec) => self.audio_codec = audio_codec,
            Message::SelectFolder => {
                let future = async {
                    let request = SelectedFiles::open_file()
                        .title("Download Folder")
                        .accept_label("Select")
                        .directory(true)
                        .multiple(false)
                        .modal(true)
                        .send()
                        .await
                        .ok()?;
                    let folder = request.response().ok()?;
                    let uri = folder.uris().first()?;
                    Some(uri.path().to_string())
                };
                return Task::perform(future, |folder| {
                    if let Some(folder) = folder {
                        return Action::App(Message::ProcessSelectFolder(folder));
                    }
                    Action::App(Message::TogglePopup)
                });
            }
            Message::ProcessSelectFolder(folder) => {
                let video_selected = self.video_entity == self.download_type.active();
                if video_selected {
                    self.video_folder = folder;
                } else {
                    self.audio_folder = folder;
                }
                return Task::done(Action::App(Message::TogglePopup));
            }
            Message::Download => {
                self.downloading += 1;
                let video_selected = self.video_entity == self.download_type.active();

                let url = self.url.clone();
                let lib_dir = self.lib_dir.clone();
                self.url.clear();
                let output_dir = PathBuf::from(if video_selected {
                    &self.video_folder
                } else {
                    &self.audio_folder
                });
                let video_quality = self.video_quality.into();
                let video_codec = self.video_codec.into();
                let audio_quality = self.audio_quality.into();
                let audio_codec = self.audio_codec.into();
                return Task::future(async move {
                    let mut notify = Notification::new()
                        .appname("yt-dlp applet")
                        .icon("multimedia-video-player-symbolic")
                        .finalize();

                    let fetcher = fetcher::with_output_dir(&lib_dir, output_dir);
                    let res = fetcher.fetch_video_infos(url.clone()).await;
                    let Ok(video) = res else {
                        let _ = notify
                            .summary(fl_str!("metadata-failed"))
                            .show_async()
                            .await;
                        return Action::App(Message::Finished);
                    };
                    let title = video.filename.rsplit_once('.').unwrap().0.to_string();

                    let (Some(format), extension) = (if video_selected {
                        (video.select_video_format(video_quality, video_codec), "mp4")
                    } else {
                        (
                            video.select_audio_format(audio_quality, audio_codec),
                            ".m4a",
                        )
                    }) else {
                        let _ = notify.summary(fl_str!("missing-format")).show_async().await;
                        return Action::App(Message::Finished);
                    };
                    let download_failed = {
                        let title = title.clone();
                        async || {
                            let _ = notify
                                .summary(fl_str!("download-failed", title = title))
                                .show_async()
                                .await;
                            Action::App(Message::Finished)
                        }
                    };

                    if format.is_manifest() {
                        if !fetcher::manifest(
                            url,
                            fetcher.output_dir,
                            fetcher.libraries.ffmpeg,
                            &title,
                            video_selected,
                        )
                        .await
                        {
                            return download_failed().await;
                        }
                    } else if fetcher
                        .download_format(format, format!("{title}.{extension}"))
                        .await
                        .is_err()
                    {
                        return download_failed().await;
                    }

                    let _ = notify
                        .summary(fl_str!("finished-download", title = title))
                        .show_async()
                        .await;
                    Action::App(Message::Finished)
                });
            }
            Message::Finished => self.downloading -= 1,
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl Ytdlp {
    fn view_video(&self) -> Element<Message> {
        column![
            row![
                body(fl!("video-quality")).width(Length::FillPortion(1)),
                pick_list(
                    vec![
                        VideoQuality::Highest,
                        VideoQuality::FHD,
                        VideoQuality::HD,
                        VideoQuality::SD,
                        VideoQuality::Lowest,
                    ],
                    Some(self.video_quality),
                    Message::VideoQuality
                )
                .width(Length::FillPortion(1)),
            ]
            .apply(padded_control),
            row![
                body(fl!("video-codec")).width(Length::FillPortion(1)),
                pick_list(
                    vec![
                        VideoCodec::AV1,
                        VideoCodec::AVC1,
                        VideoCodec::VP9,
                        VideoCodec::Any,
                    ],
                    Some(self.video_codec),
                    Message::VideoCodec
                )
                .width(Length::FillPortion(1)),
            ]
            .apply(padded_control),
        ]
        .into()
    }

    fn view_audio(&self) -> Element<Message> {
        column![
            row![
                body(fl!("audio-quality")).width(Length::FillPortion(1)),
                pick_list(
                    vec![
                        AudioQuality::Best,
                        AudioQuality::High,
                        AudioQuality::Medium,
                        AudioQuality::Low,
                        AudioQuality::Worst,
                    ],
                    Some(self.audio_quality),
                    Message::AudioQuality
                )
                .width(Length::FillPortion(1)),
            ]
            .apply(padded_control),
            row![
                body(fl!("audio-codec")).width(Length::FillPortion(1)),
                pick_list(
                    vec![
                        AudioCodec::Opus,
                        AudioCodec::ACC,
                        AudioCodec::MP3,
                        AudioCodec::Any,
                    ],
                    Some(self.audio_codec),
                    Message::AudioCodec
                )
                .width(Length::FillPortion(1)),
            ]
            .apply(padded_control),
        ]
        .into()
    }
}
