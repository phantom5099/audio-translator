use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    common::{LanguageTag, ProviderOptions, TimeRange},
    error::{CoreError, TranslationError},
};

mod argos;
pub use argos::ArgosTranslator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationSegment {
    pub range: TimeRange,
    pub text: String,
}

/// 文本翻译 provider 接口。
#[async_trait]
pub trait Translator: Send + Sync {
    /// 翻译一批源语言片段。
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslatedTranscript, TranslationError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslatedTranscript {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 按时间顺序排列的文本片段列表。
    pub segments: Vec<TranslatedSegment>,
}

impl TranslatedTranscript {
    pub fn validate_against(&self, source: &[TranslationSegment]) -> Result<(), CoreError> {
        if self.target_language.as_str().trim().is_empty() {
            return Err(CoreError::InvalidResult(
                "translated transcript target language is empty".to_owned(),
            ));
        }
        if self.segments.len() != source.len() {
            return Err(CoreError::InvalidResult(format!(
                "translation returned {} segments for {} source segments",
                self.segments.len(),
                source.len()
            )));
        }
        for (index, (source_segment, translated_segment)) in
            source.iter().zip(&self.segments).enumerate()
        {
            if translated_segment.range != source_segment.range {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment at index {index} changed its time range",
                )));
            }
            if translated_segment.source_text != source_segment.text {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment at index {index} changed its source text",
                )));
            }
            if translated_segment.translated_text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment at index {index} has empty translated text",
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslatedSegment {
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
    /// 翻译前的源文本。
    pub source_text: String,
    /// 翻译后的目标文本。
    pub translated_text: String,
    /// 与该翻译片段关联的警告列表。
    pub warnings: Vec<TranslationWarning>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TranslationWarning {
    TerminologyMismatch,
    NumberChanged,
    PlaceholderChanged,
    LengthExceeded,
    ProviderWarning(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TranslationConstraints {
    /// 是否要求翻译结果保留源文本中的数字。
    pub preserve_numbers: bool,
    /// 是否要求翻译结果保留占位符。
    pub preserve_placeholders: bool,
    /// 是否要求翻译结果保留源文本换行。
    pub preserve_line_breaks: bool,
    /// 字幕单行允许的最大字符数；为空表示不指定。
    pub max_chars_per_line: Option<u32>,
    /// 是否允许 provider 在翻译前重写或修正源文本。
    pub allow_rewrite_source: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 按时间顺序排列的文本片段列表。
    pub segments: Vec<TranslationSegment>,
    /// 本次翻译需要遵守的约束条件。
    pub constraints: TranslationConstraints,
    /// provider 专属的扩展配置。
    pub options: ProviderOptions,
}
