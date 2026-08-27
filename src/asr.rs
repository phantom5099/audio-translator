use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audio_input::AudioInput,
    common::{LanguageTag, ProviderOptions},
    error::AsrError,
};

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
    /// 识别出的源语言；无法确定时为空。
    pub language: Option<LanguageTag>,
    /// 按时间顺序排列的文本片段列表。
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
    /// 实体的稳定唯一标识。
    pub id: crate::common::SegmentId,
    /// 该文本片段对应的音频时间范围。
    pub range: crate::common::TimeRange,
    /// 与该实体关联的文本内容。
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrRequest {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// provider 专属的扩展配置。
    pub options: ProviderOptions,
}
