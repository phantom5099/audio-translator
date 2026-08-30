use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

use super::{
    AudioAsset, AudioAssetId, AudioAssetOrigin, AudioInputService, AudioInputSource, AudioMetadata,
    CoverImage,
};
use crate::error::CoreError;

/// Registers local media by reference and downloads URL media into managed storage.
pub struct MediaAudioInputService {
    root: PathBuf,
}

impl MediaAudioInputService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for MediaAudioInputService {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("audio-translator-assets"))
    }
}

#[async_trait]
impl AudioInputService for MediaAudioInputService {
    async fn import(&self, source: AudioInputSource) -> Result<AudioAsset, CoreError> {
        let (path, origin, metadata) = match source {
            AudioInputSource::LocalFile(path) => {
                validate_file(&path)?;
                let metadata = probe_media(&path)?;
                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .ok_or_else(|| {
                        CoreError::InvalidInput(format!(
                            "local media path {} has no file name",
                            path.display()
                        ))
                    })?;
                (
                    path.clone(),
                    AudioAssetOrigin::LocalFile { file_name },
                    metadata,
                )
            }
            AudioInputSource::Url(url) => {
                std::fs::create_dir_all(&self.root).map_err(|error| CoreError::Provider {
                    provider: "media-input".to_owned(),
                    message: format!("cannot create asset directory: {error}"),
                })?;
                let response = reqwest::get(&url)
                    .await
                    .map_err(|error| CoreError::Provider {
                        provider: "media-download".to_owned(),
                        message: format!("failed to download {url}: {error}"),
                    })?;
                let response =
                    response
                        .error_for_status()
                        .map_err(|error| CoreError::Provider {
                            provider: "media-download".to_owned(),
                            message: format!(
                                "media download returned an HTTP error for {url}: {error}"
                            ),
                        })?;
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|error| CoreError::Provider {
                        provider: "media-download".to_owned(),
                        message: format!("failed to read {url}: {error}"),
                    })?;
                if bytes.is_empty() {
                    return Err(CoreError::InvalidInput(
                        "downloaded media is empty".to_owned(),
                    ));
                }
                let path = self.root.join(format!("{}.media", Uuid::new_v4()));
                std::fs::write(&path, &bytes).map_err(|error| CoreError::Provider {
                    provider: "media-download".to_owned(),
                    message: format!("cannot persist downloaded media: {error}"),
                })?;
                let mut metadata = probe_media(&path)?;
                metadata.size_bytes = Some(bytes.len() as u64);
                (path, AudioAssetOrigin::Url { url }, metadata)
            }
        };
        let asset = AudioAsset {
            id: AudioAssetId::new(),
            path,
            origin,
            metadata,
        };
        Ok(asset)
    }
}

fn validate_file(path: &PathBuf) -> Result<(), CoreError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        CoreError::InvalidInput(format!("cannot access {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(CoreError::InvalidInput(format!(
            "media file {} is empty or not a regular file",
            path.display()
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
struct ProbeOutput {
    format: ProbeFormat,
    streams: Vec<ProbeStream>,
}
#[derive(Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
    format_name: Option<String>,
    tags: Option<ProbeTags>,
}
#[derive(Deserialize)]
struct ProbeTags {
    title: Option<String>,
}
#[derive(Deserialize)]
struct ProbeStream {
    index: u32,
    codec_name: Option<String>,
    disposition: Option<ProbeDisposition>,
}
#[derive(Deserialize)]
struct ProbeDisposition {
    attached_pic: Option<u8>,
}

fn probe_media(path: &PathBuf) -> Result<AudioMetadata, CoreError> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(|error| CoreError::Provider {
            provider: "ffprobe".to_owned(),
            message: format!("failed to start ffprobe: {error}"),
        })?;
    if !output.status.success() {
        return Err(CoreError::Provider {
            provider: "ffprobe".to_owned(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    let probe: ProbeOutput =
        serde_json::from_slice(&output.stdout).map_err(|error| CoreError::Provider {
            provider: "ffprobe".to_owned(),
            message: format!("invalid ffprobe response: {error}"),
        })?;
    let cover_stream = probe.streams.iter().find(|stream| {
        stream
            .disposition
            .as_ref()
            .and_then(|value| value.attached_pic)
            .unwrap_or(0)
            == 1
    });
    let cover = cover_stream
        .and_then(|stream| extract_cover(path, stream.index, stream.codec_name.as_deref()));
    let duration_ms = probe
        .format
        .duration
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| (value * 1000.0).round() as u64);
    let title = probe.format.tags.and_then(|tags| tags.title).or_else(|| {
        path.file_stem()
            .map(|value| value.to_string_lossy().into_owned())
    });
    Ok(AudioMetadata {
        title,
        duration_ms,
        media_type: probe.format.format_name,
        size_bytes: std::fs::metadata(path).ok().map(|value| value.len()),
        cover,
    })
}

fn extract_cover(path: &PathBuf, index: u32, codec: Option<&str>) -> Option<CoverImage> {
    let bytes = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-map",
            &format!("0:{index}"),
            "-frames:v",
            "1",
            "-f",
            "image2pipe",
            "-",
        ])
        .output()
        .ok()?
        .stdout;
    if bytes.is_empty() {
        return None;
    }
    Some(CoverImage {
        media_type: match codec {
            Some("png") => "image/png",
            Some("mjpeg") | Some("jpeg") => "image/jpeg",
            _ => "application/octet-stream",
        }
        .to_owned(),
        bytes,
    })
}
