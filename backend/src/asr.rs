use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audio_input::AudioInput,
    common::{LanguageTag, ProviderOptions},
    error::AsrError,
};

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
        input: Box<dyn AudioInput>,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub language: Option<LanguageTag>,
    pub segments: Vec<TranscriptSegment>,
}

impl Transcript {
    pub fn validate(&self) -> Result<(), crate::error::CoreError> {
        for segment in &self.segments {
            segment.range.validate()?;
            if segment.text.trim().is_empty() {
                return Err(crate::error::CoreError::InvalidResult(format!(
                    "transcript segment {} has empty text",
                    segment.id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: crate::common::SegmentId,
    pub range: crate::common::TimeRange,
    pub text: String,
}
