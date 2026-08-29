use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::CoreError;

mod local_file;
pub use local_file::{LocalFileAudioInput, LocalFileAudioInputService};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AudioAssetId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub file_name: Option<String>,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportedAudio {
    pub asset_id: AudioAssetId,
    pub metadata: AudioMetadata,
}

pub enum AudioInputSource {
    /// 本地媒体文件。
    LocalFile(PathBuf),
    /// 网络媒体地址。
    Url(String),
    /// 已建立的流式输入。
    Stream(Box<dyn AudioInput>),
}

/// 音频导入服务接口。
#[async_trait]
pub trait AudioInputService: Send + Sync {
    async fn import(&self, source: AudioInputSource) -> Result<ImportedAudio, CoreError>;

    async fn open(&self, asset_id: AudioAssetId) -> Result<Box<dyn AudioInput>, CoreError>;
}

/// 统一的音频输入接口。
#[async_trait]
pub trait AudioInput: Send {
    /// 读取完整的音频媒体字节。
    async fn read_all(&mut self) -> Result<Vec<u8>, CoreError>;

    /// 关闭输入并释放其持有的文件或网络资源。
    async fn close(&mut self) -> Result<(), CoreError>;
}
