use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{asr, common, error, translation};

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
    async fn read_all(&mut self) -> Result<Vec<u8>, error::CoreError>;

    async fn close(&mut self) -> Result<(), error::CoreError>;
}

struct AsrInputAdapter {
    inner: Box<dyn SpeechTranslationInput>,
}

#[async_trait]
impl asr::AsrInput for AsrInputAdapter {
    async fn read_all(&mut self) -> Result<Vec<u8>, error::CoreError> {
        self.inner.read_all().await
    }

    async fn close(&mut self) -> Result<(), error::CoreError> {
        self.inner.close().await
    }
}

/// 基于 ASR 和文本翻译的语音翻译实现。
pub struct AsrThenTranslationEngine {
    asr: Box<dyn asr::AsrEngine>,
    translator: Box<dyn translation::Translator>,
}

impl AsrThenTranslationEngine {
    pub fn new(asr: Box<dyn asr::AsrEngine>, translator: Box<dyn translation::Translator>) -> Self {
        Self { asr, translator }
    }
}

#[async_trait]
impl SpeechTranslationEngine for AsrThenTranslationEngine {
    async fn translate_audio(
        &self,
        input: Box<dyn SpeechTranslationInput>,
        request: SpeechTranslationRequest,
    ) -> Result<SpeechTranslationOutput, error::SpeechTranslationError> {
        let asr_request = asr::AsrRequest {
            source_language: request.source_language.clone(),
            options: request.options.clone(),
        };
        let transcript = self
            .asr
            .transcribe(Box::new(AsrInputAdapter { inner: input }), asr_request)
            .await?;
        transcript.validate()?;

        let source_language = transcript
            .language
            .clone()
            .or(request.source_language.clone());
        let translation_segments = transcript
            .segments
            .iter()
            .map(|segment| translation::TranslationSegment {
                id: segment.id,
                range: segment.range,
                text: segment.text.clone(),
            })
            .collect::<Vec<_>>();
        let translation_request = translation::TranslationRequest {
            source_language: source_language.clone(),
            target_language: request.target_language.clone(),
            segments: translation_segments.clone(),
            constraints: request.constraints.into_translation_constraints(),
            options: request.options,
        };
        let translated = self.translator.translate(translation_request).await?;
        translated.validate_against(&translation_segments)?;

        Ok(SpeechTranslationOutput {
            source_language: translated.source_language.or(source_language),
            target_language: translated.target_language,
            segments: translated
                .segments
                .into_iter()
                .map(|segment| SpeechTranslationSegment {
                    source_segment_id: segment.source_segment_id,
                    range: segment.range,
                    source_text: Some(segment.source_text),
                    translated_text: segment.translated_text,
                })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeechTranslationRequest {
    pub source_language: Option<common::LanguageTag>,
    pub target_language: common::LanguageTag,
    pub constraints: SpeechTranslationConstraints,
    /// Provider 专属配置由具体实现解释，避免将 ASR 或文本翻译细节暴露给 web 层。
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

impl SpeechTranslationConstraints {
    fn into_translation_constraints(self) -> translation::TranslationConstraints {
        translation::TranslationConstraints {
            preserve_numbers: self.preserve_numbers,
            preserve_placeholders: self.preserve_placeholders,
            preserve_line_breaks: self.preserve_line_breaks,
            max_chars_per_line: self.max_chars_per_line,
            allow_rewrite_source: self.allow_rewrite_source,
        }
    }
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
