use std::fmt::Display;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum VideoQuality {
    #[default]
    Highest,
    FHD,
    HD,
    SD,
    Lowest,
}

impl Display for VideoQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoQuality::Highest => write!(f, "Highest"),
            VideoQuality::FHD => write!(f, "1080p"),
            VideoQuality::HD => write!(f, "720p"),
            VideoQuality::SD => write!(f, "480p"),
            VideoQuality::Lowest => write!(f, "Lowest"),
        }
    }
}

impl From<VideoQuality> for yt_dlp::model::VideoQuality {
    fn from(val: VideoQuality) -> Self {
        match val {
            VideoQuality::Highest => yt_dlp::model::VideoQuality::Best,
            VideoQuality::FHD => yt_dlp::model::VideoQuality::High,
            VideoQuality::HD => yt_dlp::model::VideoQuality::Medium,
            VideoQuality::SD => yt_dlp::model::VideoQuality::Low,
            VideoQuality::Lowest => yt_dlp::model::VideoQuality::Worst,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum VideoCodec {
    AV1,
    AVC1,
    VP9,
    #[default]
    Any,
}

impl Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoCodec::AV1 => write!(f, "AV1"),
            VideoCodec::AVC1 => write!(f, "AVC1"),
            VideoCodec::VP9 => write!(f, "VP9"),
            VideoCodec::Any => write!(f, "Any"),
        }
    }
}

impl From<VideoCodec> for yt_dlp::model::VideoCodecPreference {
    fn from(val: VideoCodec) -> Self {
        match val {
            VideoCodec::AV1 => yt_dlp::model::VideoCodecPreference::AV1,
            VideoCodec::AVC1 => yt_dlp::model::VideoCodecPreference::AVC1,
            VideoCodec::VP9 => yt_dlp::model::VideoCodecPreference::VP9,
            VideoCodec::Any => yt_dlp::model::VideoCodecPreference::Any,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum AudioQuality {
    #[default]
    Best,
    High,
    Medium,
    Low,
    Worst,
}

impl Display for AudioQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioQuality::Best => write!(f, "Highest"),
            AudioQuality::High => write!(f, "192kbps"),
            AudioQuality::Medium => write!(f, "128kbps"),
            AudioQuality::Low => write!(f, "96kbps"),
            AudioQuality::Worst => write!(f, "Lowest"),
        }
    }
}

impl From<AudioQuality> for yt_dlp::model::AudioQuality {
    fn from(val: AudioQuality) -> Self {
        match val {
            AudioQuality::Best => yt_dlp::model::AudioQuality::Best,
            AudioQuality::High => yt_dlp::model::AudioQuality::High,
            AudioQuality::Medium => yt_dlp::model::AudioQuality::Medium,
            AudioQuality::Low => yt_dlp::model::AudioQuality::Low,
            AudioQuality::Worst => yt_dlp::model::AudioQuality::Worst,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum AudioCodec {
    Opus,
    ACC,
    MP3,
    #[default]
    Any,
}

impl Display for AudioCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCodec::Opus => write!(f, "Opus"),
            AudioCodec::ACC => write!(f, "ACC"),
            AudioCodec::MP3 => write!(f, "MP3"),
            AudioCodec::Any => write!(f, "Any"),
        }
    }
}

impl From<AudioCodec> for yt_dlp::model::AudioCodecPreference {
    fn from(val: AudioCodec) -> Self {
        match val {
            AudioCodec::Opus => yt_dlp::model::AudioCodecPreference::Opus,
            AudioCodec::ACC => yt_dlp::model::AudioCodecPreference::AAC,
            AudioCodec::MP3 => yt_dlp::model::AudioCodecPreference::MP3,
            AudioCodec::Any => yt_dlp::model::AudioCodecPreference::Any,
        }
    }
}
