use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::{
    AudioAsset, AudioAssetId, AudioInputService, AudioInputSource, AudioMetadata, CoverImage,
};
use crate::error::CoreError;

pub struct MediaAudioInputService {
    root: PathBuf,
    ffprobe_path: PathBuf,
    ffmpeg_path: PathBuf,
    yt_dlp_path: PathBuf,
}

impl MediaAudioInputService {
    pub fn new(
        root: impl Into<PathBuf>,
        ffprobe_path: impl Into<PathBuf>,
        ffmpeg_path: impl Into<PathBuf>,
        yt_dlp_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            root: root.into(),
            ffprobe_path: ffprobe_path.into(),
            ffmpeg_path: ffmpeg_path.into(),
            yt_dlp_path: yt_dlp_path.into(),
        }
    }
}

impl Default for MediaAudioInputService {
    fn default() -> Self {
        Self::new(
            std::env::temp_dir().join("audio-translator-assets"),
            "ffprobe",
            "ffmpeg",
            "yt-dlp",
        )
    }
}

#[async_trait]
impl AudioInputService for MediaAudioInputService {
    async fn import(&self, source: AudioInputSource) -> Result<AudioAsset, CoreError> {
        let (path, file_name, metadata) = match source {
            //本地导入部分
            AudioInputSource::LocalFile(path) => {
                debug!(path = %path.display(), "import: local file source");
                validate_file(&path)?;
                let metadata = probe_media(&self.ffprobe_path, &self.ffmpeg_path, &path)?;
                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .ok_or_else(|| {
                        CoreError::InvalidInput(format!(
                            "local media path {} has no file name",
                            path.display()
                        ))
                    })?;
                (path.clone(), file_name, metadata)
            }
            //url拉取部分
            AudioInputSource::Url(url) => {
                debug!(%url, "import: url source (yt-dlp)");
                std::fs::create_dir_all(&self.root).map_err(|error| CoreError::Provider {
                    provider: "media-input".to_owned(),
                    message: format!("cannot create asset directory: {error}"),
                })?;
                let id = Uuid::new_v4();
                let output_template = self
                    .root
                    .join(format!("{id}-%(title)s.%(ext)s"))
                    .to_string_lossy()
                    .into_owned();
                let output = tokio::process::Command::new(&self.yt_dlp_path)
                    .args([
                        "-x",
                        "-f",
                        "bestaudio/best",
                        "--no-playlist",
                        "--no-progress",
                        "--no-warnings",
                        "--print-json",
                        "--write-thumbnail",
                        "--ffmpeg-location",
                    ])
                    .arg(&self.ffmpeg_path)
                    .args(["-o", &output_template])
                    .arg(&url)
                    .output()
                    .await
                    .map_err(|error| CoreError::Provider {
                        provider: "yt-dlp".to_owned(),
                        message: format!("failed to start yt-dlp: {error}"),
                    })?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!(
                        code = ?output.status.code(),
                        stderr = stderr.trim(),
                        "import: yt-dlp failed"
                    );
                    return Err(CoreError::Provider {
                        provider: "yt-dlp".to_owned(),
                        message: stderr.trim().to_owned(),
                    });
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                let info: YtDlpInfo = serde_json::from_str(&stdout).map_err(|error| {
                    error!(stdout = %stdout, "import: invalid yt-dlp json: {error}");
                    CoreError::Provider {
                        provider: "yt-dlp".to_owned(),
                        message: format!("invalid json response: {error}"),
                    }
                })?;
                let path = if !info.filename.is_empty() {
                    let p = PathBuf::from(&info.filename);
                    if p.is_absolute() && p.exists() {
                        p
                    } else {
                        self.root.join(&info.filename)
                    }
                } else {
                    find_audio_by_prefix(&self.root, &id.to_string())?
                };
                if !path.exists() {
                    return Err(CoreError::Provider {
                        provider: "yt-dlp".to_owned(),
                        message: format!("output audio file not found: {}", path.display()),
                    });
                }
                let byte_len = std::fs::metadata(&path)
                    .map(|value| value.len())
                    .unwrap_or(0);
                let cover = find_thumbnail(&path);
                debug!(
                    %url,
                    byte_len,
                    saved = %path.display(),
                    has_cover = cover.is_some(),
                    "import: yt-dlp saved"
                );
                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .ok_or_else(|| {
                        CoreError::InvalidInput("downloaded media has no file name".to_owned())
                    })?;
                let metadata = AudioMetadata {
                    title: Some(info.title),
                    duration_ms: info.duration.map(|value| (value * 1000.0).round() as u64),
                    media_type: info.ext,
                    size_bytes: Some(byte_len),
                    cover,
                };
                (path, file_name, metadata)
            }
        };
        let asset = AudioAsset {
            id: AudioAssetId::new(),
            path,
            file_name,
            metadata,
        };
        info!(
            asset_id = ?asset.id,
            file_name = %asset.file_name,
            duration_ms = ?asset.metadata.duration_ms,
            size_bytes = ?asset.metadata.size_bytes,
            has_cover = asset.metadata.cover.is_some(),
            "import: completed"
        );
        Ok(asset)
    }
}

fn validate_file(path: &PathBuf) -> Result<(), CoreError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        error!(path = %path.display(), "validate_file: cannot access: {error}");
        CoreError::InvalidInput(format!("cannot access {}: {error}", path.display()))
    })?;
    debug!(
        path = %path.display(),
        size = metadata.len(),
        "validate_file: file accessible"
    );
    if !metadata.is_file() || metadata.len() == 0 {
        error!(
            path = %path.display(),
            is_file = metadata.is_file(),
            size = metadata.len(),
            "validate_file: empty or not a regular file"
        );
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

fn probe_media(
    ffprobe_path: &PathBuf,
    ffmpeg_path: &PathBuf,
    path: &PathBuf,
) -> Result<AudioMetadata, CoreError> {
    debug!(
        ffprobe = %ffprobe_path.display(),
        target = %path.display(),
        "probe_media: running ffprobe"
    );
    let output = std::process::Command::new(ffprobe_path)
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
        .map_err(|error| {
            error!(
                ffprobe = %ffprobe_path.display(),
                "probe_media: failed to start ffprobe: {error}"
            );
            CoreError::Provider {
                provider: "ffprobe".to_owned(),
                message: format!("failed to start ffprobe: {error}"),
            }
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            code = ?output.status.code(),
            stderr = stderr.trim(),
            "probe_media: ffprobe failed"
        );
        return Err(CoreError::Provider {
            provider: "ffprobe".to_owned(),
            message: stderr.trim().to_owned(),
        });
    }
    let probe: ProbeOutput = serde_json::from_slice(&output.stdout).map_err(|error| {
        error!(
            stdout = %String::from_utf8_lossy(&output.stdout),
            "probe_media: invalid ffprobe response: {error}"
        );
        CoreError::Provider {
            provider: "ffprobe".to_owned(),
            message: format!("invalid ffprobe response: {error}"),
        }
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
        .and_then(|stream| {
            extract_cover(
                ffmpeg_path,
                path,
                stream.index,
                stream.codec_name.as_deref(),
            )
        })
        .or_else(|| extract_video_thumbnail(ffmpeg_path, path));
    let duration_ms = probe
        .format
        .duration
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| (value * 1000.0).round() as u64);
    let title = probe.format.tags.and_then(|tags| tags.title).or_else(|| {
        path.file_stem()
            .map(|value| value.to_string_lossy().into_owned())
    });
    debug!(
        duration_ms,
        format = ?probe.format.format_name,
        has_cover = cover.is_some(),
        "probe_media: parsed"
    );
    Ok(AudioMetadata {
        title,
        duration_ms,
        media_type: probe.format.format_name,
        size_bytes: std::fs::metadata(path).ok().map(|value| value.len()),
        cover,
    })
}

fn extract_cover(
    ffmpeg_path: &PathBuf,
    path: &PathBuf,
    index: u32,
    codec: Option<&str>,
) -> Option<CoverImage> {
    debug!(
        ffmpeg = %ffmpeg_path.display(),
        target = %path.display(),
        index,
        codec,
        "extract_cover: running ffmpeg"
    );
    let bytes = std::process::Command::new(ffmpeg_path)
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
        .ok()?;
    if bytes.stdout.is_empty() {
        debug!("extract_cover: no cover bytes returned");
        return None;
    }
    debug!(
        byte_len = bytes.stdout.len(),
        "extract_cover: cover extracted"
    );
    Some(CoverImage {
        media_type: match codec {
            Some("png") => "image/png",
            Some("mjpeg") | Some("jpeg") => "image/jpeg",
            _ => "application/octet-stream",
        }
        .to_owned(),
        bytes: bytes.stdout,
    })
}

fn extract_video_thumbnail(ffmpeg_path: &PathBuf, path: &PathBuf) -> Option<CoverImage> {
    debug!(
        ffmpeg = %ffmpeg_path.display(),
        target = %path.display(),
        "extract_video_thumbnail: running ffmpeg"
    );
    for seek in ["00:00:05", "00:00:00"] {
        if let Ok(output) = std::process::Command::new(ffmpeg_path)
            .args(["-ss", seek, "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-q:v", "2", "-f", "image2pipe", "-"])
            .output()
        {
            if !output.stdout.is_empty() {
                debug!(
                    byte_len = output.stdout.len(),
                    seek, "extract_video_thumbnail: frame extracted"
                );
                return Some(CoverImage {
                    media_type: "image/jpeg".to_owned(),
                    bytes: output.stdout,
                });
            }
        }
    }
    debug!("extract_video_thumbnail: no frame bytes returned");
    None
}

/// yt-dlp `--print-json` 输出的 info_dict 字段子集。
#[derive(Deserialize)]
struct YtDlpInfo {
    title: String,
    duration: Option<f64>,
    ext: Option<String>,
    filename: String,
}

/// yt-dlp 落盘后扫描指定目录中以 `prefix` 开头的音频文件。
fn find_audio_by_prefix(dir: &PathBuf, prefix: &str) -> Result<PathBuf, CoreError> {
    let entries = std::fs::read_dir(dir).map_err(|error| CoreError::Provider {
        provider: "yt-dlp".to_owned(),
        message: format!("cannot read asset directory: {error}"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
            if name.starts_with(prefix) && is_audio_extension(name) {
                return Ok(path);
            }
        }
    }
    Err(CoreError::Provider {
        provider: "yt-dlp".to_owned(),
        message: format!("no audio file found with prefix {prefix}"),
    })
}

fn is_audio_extension(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.rsplit('.').next(),
        Some("m4a" | "mp3" | "opus" | "webm" | "ogg" | "wav" | "flac" | "aac" | "mp4" | "mkv")
    )
}

fn find_thumbnail(audio_path: &PathBuf) -> Option<CoverImage> {
    let dir = audio_path.parent()?;
    let stem = audio_path.file_stem()?.to_string_lossy().into_owned();
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        let lower = name.to_lowercase();
        if !lower.starts_with(&stem) {
            continue;
        }
        let ext = lower.rsplit('.').next();
        let media_type = match ext {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("webp") => "image/webp",
            _ => continue,
        };
        if let Ok(bytes) = std::fs::read(&path) {
            if !bytes.is_empty() {
                debug!(
                    thumbnail = %path.display(),
                    byte_len = bytes.len(),
                    "find_thumbnail: thumbnail loaded"
                );
                return Some(CoverImage {
                    media_type: media_type.to_owned(),
                    bytes,
                });
            }
        }
    }
    debug!("find_thumbnail: no thumbnail found");
    None
}
