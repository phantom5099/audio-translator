use async_trait::async_trait;

use super::{
    AsrError, AsrRequest, AudioChunk, AudioFormatRequirement, AudioInfo, CoreError, ExportError,
    SegmentId, SubtitleDocument, SubtitleExportRequest, SubtitleOutput, TimeRange, Transcript,
    TranscriptSegment, TranslatedTranscript, TranslationError, TranslationRequest,
};

#[async_trait]
pub trait AudioInput: Send {
    async fn info(&self) -> Result<AudioInfo, CoreError>;
    async fn next_chunk(&mut self) -> Result<Option<AudioChunk>, CoreError>;
    async fn close(&mut self) -> Result<(), CoreError>;
}

#[async_trait]
pub trait AsrEngine: Send + Sync {
    fn audio_requirements(&self) -> AudioFormatRequirement;

    async fn transcribe(
        &self,
        input: Box<dyn AudioInput>,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError>;
}

#[async_trait]
pub trait StreamingAsrEngine: AsrEngine {
    async fn open_session(
        &self,
        request: StreamingAsrRequest,
    ) -> Result<Box<dyn AsrSession>, AsrError>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamingAsrRequest {
    /// 源文本使用的语言。
    pub source_language: Option<super::LanguageTag>,
    /// 需要生成的时间戳粒度。
    pub timestamp_level: super::TimestampLevel,
    /// 是否要求识别并返回说话人标签。
    pub enable_speaker_labels: bool,
    /// 可选的 ASR 词汇或热词提示。
    pub vocabulary: Option<super::AsrVocabulary>,
    /// provider 专属的扩展配置。
    pub options: super::ProviderOptions,
}

#[async_trait]
pub trait AsrSession: Send {
    async fn push(&mut self, chunk: AudioChunk) -> Result<Vec<AsrEvent>, AsrError>;
    async fn finish(&mut self) -> Result<Vec<AsrEvent>, AsrError>;
    async fn close(&mut self) -> Result<(), AsrError>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AsrEvent {
    /// ASR 返回尚未最终确认的临时片段。
    Partial {
        /// 尚未最终确认的转录片段。
        segment: TranscriptSegment,
    },
    /// ASR 确认并提交一个稳定片段。
    Commit {
        /// 已经最终确认并提交的转录片段。
        segment: TranscriptSegment,
    },
    /// ASR 判定一个片段已经结束。
    Endpoint {
        /// 上下文或事件关联的文本片段唯一标识。
        segment_id: SegmentId,
        /// 该文本片段对应的音频时间范围。
        range: TimeRange,
    },
}

#[async_trait]
pub trait Translator: Send + Sync {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslatedTranscript, TranslationError>;
}

#[async_trait]
pub trait SubtitleExporter: Send + Sync {
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError>;
}
