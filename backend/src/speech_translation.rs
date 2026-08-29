use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{common, error};

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
