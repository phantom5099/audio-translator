use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audio_input::AudioInput,
    common::{LanguageTag, ProviderOptions},
    error::SpeechTranslationError,
    translation::{TranslatedTranscript, TranslationConstraints},
};

/// 统一的语音翻译能力接口。
#[async_trait]
pub trait SpeechTranslationEngine: Send + Sync {
    async fn translate_audio(
        &self,
        input: Box<dyn AudioInput>,
        request: SpeechTranslationRequest,
    ) -> Result<TranslatedTranscript, SpeechTranslationError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationRequest {
    pub source_language: Option<LanguageTag>,
    pub target_language: LanguageTag,
    pub constraints: TranslationConstraints,
    /// Provider 专属配置由具体实现解释，避免将 ASR 或文本翻译细节暴露给 pipeline。
    pub options: ProviderOptions,
}
