use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ExportError;

/// 字幕导出接口。
///
/// 负责把内部统一的 `SubtitleDocument` 序列化为 SRT、WebVTT、ASS、TTML 或其他格式。
/// 导出器与内部字幕模型分离，使新增字幕格式时不需要修改 ASR 和翻译流程。
///
/// `document` 使用借用避免导出时复制或消耗完整字幕；`request` 使用值传递，
/// 因为导出器可以取得格式、编码和排版策略的所有权并在异步过程中使用。
#[async_trait]
pub trait SubtitleExporter: Send + Sync {
    /// 将统一字幕文档导出为指定格式的字节结果。
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleDocument {
    /// 源文本使用的语言。
    pub source_language: Option<crate::common::LanguageTag>,
    /// 目标文本使用的语言。
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
    /// SubRip 字幕格式。
    Srt,
    /// WebVTT 字幕格式。
    WebVtt,
    /// ASS/SSA 字幕格式，可表达更丰富的样式。
    Ass,
    /// JSON 结构化字幕格式。
    Json,
    /// TTML 字幕格式。
    Ttml,
    /// 由调用方指定的自定义字幕格式。
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
    /// UTF-8 文本编码。
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
