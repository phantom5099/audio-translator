use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::CoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub duration_ms: Option<u64>,
    pub media_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub cover: Option<CoverImage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoverImage {
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioAsset {
    pub path: PathBuf,
    pub origin: AudioAssetOrigin,
    pub metadata: AudioMetadata,
}

/// Safe source metadata for API responses; backend storage paths stay private.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AudioAssetOrigin {
    LocalFile { file_name: String },
    Url { url: String },
}

pub enum AudioInputSource {
    LocalFile(PathBuf),
    Url(String),
}

#[async_trait]
pub trait AudioInputService: Send + Sync {
    async fn import(&self, source: AudioInputSource) -> Result<AudioAsset, CoreError>;
}
