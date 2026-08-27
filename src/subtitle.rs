use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ExportError;

/// 字幕导出接口。
#[async_trait]
pub trait SubtitleExporter: Send + Sync {
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleDocument {
    pub source_language: Option<crate::common::LanguageTag>,
    pub target_language: crate::common::LanguageTag,
    /// 字幕 cue 列表。
    pub cues: Vec<SubtitleCue>,
}

impl SubtitleDocument {
    pub fn from_translation(
        transcript: &crate::asr::Transcript,
        translated: &crate::translation::TranslatedTranscript,
    ) -> Result<Self, crate::error::CoreError> {
        translated.validate_against(transcript)?;
        Ok(Self {
            source_language: transcript.language.clone(),
            target_language: translated.target_language.clone(),
            cues: translated
                .segments
                .iter()
                .map(|segment| SubtitleCue {
                    id: uuid::Uuid::new_v4(),
                    source_segment_id: segment.source_segment_id,
                    range: segment.range,
                    text: segment.translated_text.clone(),
                })
                .collect(),
        })
    }

    pub fn validate(&self) -> Result<(), crate::error::CoreError> {
        let mut previous_start = None;
        for cue in &self.cues {
            cue.range.validate()?;
            if cue.text.trim().is_empty() {
                return Err(crate::error::CoreError::InvalidResult(format!(
                    "subtitle cue {} has empty text",
                    cue.id
                )));
            }
            if let Some(previous_start) = previous_start
                && cue.range.start_ms < previous_start
            {
                return Err(crate::error::CoreError::InvalidResult(
                    "subtitle cues must be sorted by start time".to_owned(),
                ));
            }
            previous_start = Some(cue.range.start_ms);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleCue {
    /// 实体的稳定唯一标识。
    pub id: crate::common::CueId,
    /// 对应源文本片段的稳定唯一标识。
    pub source_segment_id: crate::common::SegmentId,
    /// 该文本片段对应的音频时间范围。
    pub range: crate::common::TimeRange,
    /// 与该实体关联的文本内容。
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubtitleFormat {
    Srt,
    WebVtt,
    Ass,
    Json,
    Ttml,
    Custom(String),
}

impl SubtitleFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Srt => "srt",
            Self::WebVtt => "vtt",
            Self::Ass => "ass",
            Self::Json => "json",
            Self::Ttml => "ttml",
            Self::Custom(extension) => extension.as_str(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TextEncoding {
    Utf8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LineBreakPolicy {
    /// 保留源文本或内部模型中的换行。
    Preserve,
    /// 根据指定的单行字符数自动换行。
    AutoWrap {
        /// 自动换行时每行允许的最大字符数。
        max_chars: u32,
    },
    /// 将每条字幕压缩为单行。
    SingleLine,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleExportRequest {
    /// 要导出的字幕格式。
    pub format: SubtitleFormat,
    /// 输出文本使用的字符编码。
    pub encoding: TextEncoding,
    /// 字幕换行策略。
    pub line_policy: LineBreakPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleOutput {
    /// 实际导出的字幕格式。
    pub format: SubtitleFormat,
    /// 导出文件的完整字节内容。
    pub bytes: Vec<u8>,
    /// 建议使用的输出文件名。
    pub suggested_name: String,
}
