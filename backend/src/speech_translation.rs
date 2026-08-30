use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use crate::{common, error};

/// 持久化后的语音翻译结果标识。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SpeechTranslationId(pub Uuid);

/// 语音翻译结果的持久化端口。
#[async_trait]
pub trait SpeechTranslationRepository: Send + Sync {
    async fn save(
        &self,
        output: SpeechTranslationOutput,
    ) -> Result<SpeechTranslationId, error::CoreError>;

    async fn load(
        &self,
        id: SpeechTranslationId,
    ) -> Result<SpeechTranslationOutput, error::CoreError>;
}

/// 基于 JSON 文件的本地语音翻译结果存储。
#[derive(Clone)]
pub struct FileSpeechTranslationRepository {
    root: PathBuf,
}

impl FileSpeechTranslationRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path(&self, id: SpeechTranslationId) -> PathBuf {
        self.root.join(format!("{}.json", id.0))
    }
}

impl Default for FileSpeechTranslationRepository {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("audio-translator-translations"))
    }
}

#[async_trait]
impl SpeechTranslationRepository for FileSpeechTranslationRepository {
    async fn save(
        &self,
        output: SpeechTranslationOutput,
    ) -> Result<SpeechTranslationId, error::CoreError> {
        fs::create_dir_all(&self.root).map_err(|error| error::CoreError::Provider {
            provider: "speech-translation-store".to_owned(),
            message: format!("cannot create translation directory: {error}"),
        })?;
        let id = SpeechTranslationId(Uuid::new_v4());
        let bytes = serde_json::to_vec(&output).map_err(|error| {
            error::CoreError::InvalidResult(format!("cannot encode speech translation: {error}"))
        })?;
        fs::write(self.path(id), bytes).map_err(|error| error::CoreError::Provider {
            provider: "speech-translation-store".to_owned(),
            message: format!("cannot persist speech translation: {error}"),
        })?;
        Ok(id)
    }

    async fn load(
        &self,
        id: SpeechTranslationId,
    ) -> Result<SpeechTranslationOutput, error::CoreError> {
        let bytes = fs::read(self.path(id)).map_err(|_| {
            error::CoreError::InvalidInput(format!("speech translation {id:?} was not found"))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            error::CoreError::InvalidResult(format!("invalid speech translation {id:?}: {error}"))
        })
    }
}

/// 统一的语音翻译能力接口。
#[async_trait]
pub trait SpeechTranslationEngine: Send + Sync {
    async fn translate_audio(
        &self,
        input: Box<dyn SpeechTranslationInput>,
        request: SpeechTranslationRequest,
    ) -> Result<SpeechTranslationOutput, error::SpeechTranslationError>;
}

#[async_trait]
pub trait SpeechTranslationInput: Send {
    async fn content(&mut self) -> Result<SpeechTranslationInputContent, error::CoreError>;

    async fn close(&mut self) -> Result<(), error::CoreError>;
}

pub enum SpeechTranslationInputContent {
    File(std::path::PathBuf),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationRequest {
    pub source_language: Option<common::LanguageTag>,
    pub target_language: common::LanguageTag,
    pub constraints: SpeechTranslationConstraints,
    /// Provider 专属配置由具体实现解释，避免将 ASR 或文本翻译细节暴露给应用层。
    pub options: common::ProviderOptions,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpeechTranslationConstraints {
    pub preserve_numbers: bool,
    pub preserve_placeholders: bool,
    pub preserve_line_breaks: bool,
    pub max_chars_per_line: Option<u32>,
    pub allow_rewrite_source: bool,
}

/// 语音翻译能力对外发布的、与具体 ASR/翻译实现无关的结果。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationOutput {
    pub source_language: Option<common::LanguageTag>,
    pub target_language: common::LanguageTag,
    pub segments: Vec<SpeechTranslationSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationSegment {
    pub source_segment_id: common::SegmentId,
    pub range: common::TimeRange,
    pub source_text: Option<String>,
    pub translated_text: String,
}
