//! Small, dependency-light adapters that are useful for wiring and testing the
//! core contracts. Real FFmpeg/GStreamer, network, microphone and provider
//! adapters should live in separate crates or modules.

use std::collections::VecDeque;

use async_trait::async_trait;

use crate::{
    AudioChunk, AudioFormat, AudioInfo, AudioInput, CoreError, ExportError, LineBreakPolicy,
    SubtitleDocument, SubtitleExportRequest, SubtitleExporter, SubtitleFormat, SubtitleOutput,
    TextEncoding,
};

/// A finite in-memory audio input. It does not decode or resample audio; the
/// caller must provide chunks already normalized for the selected ASR provider.
pub struct MemoryAudioInput {
    /// 音频输入的元信息。
    info: AudioInfo,
    /// 尚未读取的音频块队列。
    chunks: VecDeque<AudioChunk>,
    /// 音频输入是否已经关闭。
    closed: bool,
}

impl MemoryAudioInput {
    pub fn new(
        format: AudioFormat,
        chunks: impl IntoIterator<Item = AudioChunk>,
        is_live: bool,
    ) -> Result<Self, CoreError> {
        format.validate()?;
        let chunks: VecDeque<_> = chunks.into_iter().collect();
        for chunk in &chunks {
            chunk.validate()?;
            if chunk.format != format {
                return Err(CoreError::InvalidInput(
                    "all memory input chunks must use the input format".to_owned(),
                ));
            }
        }
        let duration_ms = if is_live {
            None
        } else {
            Some(
                chunks
                    .iter()
                    .map(|chunk| u64::from(chunk.duration_ms))
                    .sum(),
            )
        };
        Ok(Self {
            info: AudioInfo {
                format,
                duration_ms,
                is_live,
            },
            chunks,
            closed: false,
        })
    }

    pub fn remaining_chunks(&self) -> usize {
        self.chunks.len()
    }
}

#[async_trait]
impl AudioInput for MemoryAudioInput {
    async fn info(&self) -> Result<AudioInfo, CoreError> {
        Ok(self.info.clone())
    }

    async fn next_chunk(&mut self) -> Result<Option<AudioChunk>, CoreError> {
        if self.closed {
            return Err(CoreError::InvalidInput(
                "cannot read a closed audio input".to_owned(),
            ));
        }
        Ok(self.chunks.pop_front())
    }

    async fn close(&mut self) -> Result<(), CoreError> {
        self.closed = true;
        self.chunks.clear();
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
