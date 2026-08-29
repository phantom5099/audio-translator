use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use super::{
    AudioAssetId, AudioInput, AudioInputService, AudioInputSource, AudioMetadata, ImportedAudio,
};
use crate::error::CoreError;

/// Local media storage. The registry keeps asset IDs resolvable across workflow stages.
pub struct LocalFileAudioInputService {
    root: PathBuf,
}

pub struct LocalFileAudioInput {
    path: PathBuf,
}

impl LocalFileAudioInputService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Default for LocalFileAudioInputService {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("audio-translator-assets"))
    }
}

#[async_trait]
impl AudioInputService for LocalFileAudioInputService {
    async fn import(&self, source: AudioInputSource) -> Result<ImportedAudio, CoreError> {
        std::fs::create_dir_all(&self.root).map_err(|error| CoreError::Provider {
            provider: "local-file-audio-input".to_owned(),
            message: format!("cannot create asset directory: {error}"),
        })?;
        let (bytes, file_name) = match source {
            AudioInputSource::LocalFile(path) => {
                let metadata = std::fs::metadata(&path).map_err(|error| {
                    CoreError::InvalidInput(format!(
                        "cannot read media file {}: {error}",
                        path.display()
                    ))
                })?;
                if !metadata.is_file() || metadata.len() == 0 {
                    return Err(CoreError::InvalidInput(format!(
                        "media file {} is empty or not a regular file",
                        path.display()
                    )));
                }
                let bytes = std::fs::read(&path)
                    .map_err(|error| CoreError::InvalidInput(error.to_string()))?;
                let file_name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                (bytes, file_name)
            }
            AudioInputSource::Stream(mut input) => {
                let bytes = input.read_all().await?;
                input.close().await?;
                (bytes, None)
            }
            AudioInputSource::Url(_) => {
                return Err(CoreError::UnsupportedFormat(
                    "URL input is not implemented by the local provider".to_owned(),
                ));
            }
        };
        let asset_id = AudioAssetId(Uuid::new_v4());
        let path = self.root.join(asset_id.0.to_string());
        std::fs::write(&path, &bytes).map_err(|error| CoreError::Provider {
            provider: "local-file-audio-input".to_owned(),
            message: format!("cannot persist imported media: {error}"),
        })?;
        Ok(ImportedAudio {
            asset_id,
            metadata: AudioMetadata {
                file_name,
                size_bytes: bytes.len() as u64,
            },
        })
    }

    async fn open(&self, asset_id: AudioAssetId) -> Result<Box<dyn AudioInput>, CoreError> {
        let path = self.root.join(asset_id.0.to_string());
        let metadata = std::fs::metadata(&path).map_err(|_| {
            CoreError::InvalidInput(format!("audio asset {} was not found", asset_id.0))
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(CoreError::InvalidInput(format!(
                "audio asset {} is empty or not a regular file",
                asset_id.0
            )));
        }
        Ok(Box::new(LocalFileAudioInput { path }))
    }
}

#[async_trait]
impl AudioInput for LocalFileAudioInput {
    async fn read_all(&mut self) -> Result<Vec<u8>, CoreError> {
        std::fs::read(&self.path).map_err(|error| {
            CoreError::InvalidInput(format!(
                "cannot read media file {}: {error}",
                self.path.display()
            ))
        })
    }

    async fn close(&mut self) -> Result<(), CoreError> {
        Ok(())
    }
}
