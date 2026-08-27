//! Small, dependency-light adapters that are useful for wiring and testing the
//! core contracts. Real FFmpeg/GStreamer, network, microphone and provider
//! adapters should live in separate crates or modules.

use async_trait::async_trait;

use crate::{
    audio_input::AudioInput,
    error::{CoreError, ExportError},
    subtitle::{
        LineBreakPolicy, SubtitleDocument, SubtitleExportRequest, SubtitleExporter, SubtitleFormat,
        SubtitleOutput, TextEncoding,
    },
};

/// 用于测试或内存媒体来源的完整音频输入。
pub struct MemoryAudioInput {
    /// 尚未读取的原始媒体字节。
    bytes: Vec<u8>,
    /// 音频输入是否已经关闭。
    closed: bool,
}

impl MemoryAudioInput {
    /// 使用原始媒体字节创建输入。
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            closed: false,
        }
    }
}

#[async_trait]
impl AudioInput for MemoryAudioInput {
    async fn read_all(&mut self) -> Result<Vec<u8>, CoreError> {
        if self.closed {
            return Err(CoreError::InvalidInput(
                "cannot read a closed audio input".to_owned(),
            ));
        }
        Ok(std::mem::take(&mut self.bytes))
    }

    async fn close(&mut self) -> Result<(), CoreError> {
        self.closed = true;
        self.bytes.clear();
        Ok(())
    }
}

/// A built-in JSON exporter. It is deliberately the only exporter in core:
/// SRT/WebVTT/ASS require format-specific policy and renderers and can be
/// implemented as independent adapters without changing the core contracts.
pub struct JsonSubtitleExporter;

#[async_trait]
impl SubtitleExporter for JsonSubtitleExporter {
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError> {
        document.validate().map_err(ExportError::Core)?;
        if !matches!(request.format, SubtitleFormat::Json) {
            return Err(ExportError::UnsupportedFormat(format_name(&request.format)));
        }
        if !matches!(request.encoding, TextEncoding::Utf8) {
            return Err(ExportError::Core(CoreError::InvalidInput(
                "JSON exporter only supports UTF-8".to_owned(),
            )));
        }
        if let LineBreakPolicy::AutoWrap { max_chars } = request.line_policy
            && max_chars == 0
        {
            return Err(ExportError::Core(CoreError::InvalidInput(
                "max_chars must be greater than zero".to_owned(),
            )));
        }

        let bytes = serde_json::to_vec_pretty(document).map_err(|error| {
            ExportError::Core(CoreError::InvalidResult(format!(
                "failed to serialize subtitle document: {error}"
            )))
        })?;
        Ok(SubtitleOutput {
            format: SubtitleFormat::Json,
            bytes,
            suggested_name: "subtitles.json".to_owned(),
        })
    }
}

fn format_name(format: &SubtitleFormat) -> String {
    match format {
        SubtitleFormat::Srt => "srt".to_owned(),
        SubtitleFormat::WebVtt => "webvtt".to_owned(),
        SubtitleFormat::Ass => "ass".to_owned(),
        SubtitleFormat::Json => "json".to_owned(),
        SubtitleFormat::Ttml => "ttml".to_owned(),
        SubtitleFormat::Custom(value) => value.clone(),
    }
}
