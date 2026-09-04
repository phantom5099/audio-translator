use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::CoreError;
use uuid::Uuid;

mod media;
pub use media::MediaAudioInputService;

/// 导入媒体在当前应用进程中的稳定标识。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AudioAssetId(pub Uuid);

impl AudioAssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

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
    pub id: AudioAssetId,
    pub path: PathBuf,
    pub file_name: String,
    pub metadata: AudioMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AudioInputSource {
    LocalFile(PathBuf),
    Url(String),
}

#[async_trait]
pub trait AudioInputService: Send + Sync {
    async fn import(&self, source: AudioInputSource) -> Result<AudioAsset, CoreError>;
}
