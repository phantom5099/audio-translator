use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    common::{LanguageTag, ProviderOptions},
    error::AsrError,
};

#[async_trait]
pub trait AsrInput: Send {
    async fn content(&mut self) -> Result<AsrInputContent, crate::error::CoreError>;

    async fn close(&mut self) -> Result<(), crate::error::CoreError>;
}

pub enum AsrInputContent {
    File(std::path::PathBuf),
    Bytes(Vec<u8>),
}

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
        input: Box<dyn AsrInput>,
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
