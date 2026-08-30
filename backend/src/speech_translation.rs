use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::{LanguageTag, ProviderOptions, TimeRange},
    error::SpeechTranslationError,
};

/// 当前应用进程中的语音翻译结果标识。
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SpeechTranslationId(pub Uuid);

impl SpeechTranslationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// 统一的语音翻译能力接口。
#[async_trait]
pub trait SpeechTranslationEngine: Send + Sync {
    async fn translate_audio(
        &self,
        path: std::path::PathBuf,
        request: SpeechTranslationRequest,
    ) -> Result<SpeechTranslationOutput, SpeechTranslationError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationRequest {
    pub source_language: Option<LanguageTag>,
    pub target_language: LanguageTag,
    pub constraints: SpeechTranslationConstraints,
    /// Provider 专属配置由具体实现解释，避免将 ASR 或文本翻译细节暴露给应用层。
    pub options: ProviderOptions,
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
    pub source_language: Option<LanguageTag>,
    pub target_language: LanguageTag,
    pub segments: Vec<SpeechTranslationSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationSegment {
    pub range: TimeRange,
    pub source_text: Option<String>,
    pub translated_text: String,
}
