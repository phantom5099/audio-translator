use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    common::{LanguageTag, ProviderOptions, TimeRange},
    error::{AsrError, CoreError},
};

mod faster_whisper;
pub use faster_whisper::FasterWhisperAsrEngine;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrRequest {
    pub source_language: Option<LanguageTag>,
    pub options: ProviderOptions,
}

/// 非流式 ASR 引擎接口。
#[async_trait]
pub trait AsrEngine: Send + Sync {
    /// 将完整音频输入识别为带时间戳的转录结果。
    async fn transcribe(
        &self,
        path: std::path::PathBuf,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub language: Option<LanguageTag>,
    pub segments: Vec<TranscriptSegment>,
}

impl Transcript {
    pub fn validate(&self) -> Result<(), CoreError> {
        for (index, segment) in self.segments.iter().enumerate() {
            segment.range.validate()?;
            if segment.text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
                    "transcript segment {index} has empty text",
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub range: TimeRange,
    pub text: String,
}
