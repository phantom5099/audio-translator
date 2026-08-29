use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    common::{LanguageTag, TimeRange},
    error::{CoreError, ExportError},
};

mod srt;
pub use srt::SrtSubtitleExporter;

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
    pub source_language: Option<LanguageTag>,
    pub target_language: LanguageTag,
    /// 字幕 cue 列表。
    pub cues: Vec<SubtitleCue>,
}

impl SubtitleDocument {
    pub fn from_cues(
        source_language: Option<LanguageTag>,
        target_language: LanguageTag,
        cues: Vec<SubtitleCue>,
    ) -> Self {
        Self {
            source_language,
            target_language,
            cues,
        }
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        let mut previous_start = None;
        for (index, cue) in self.cues.iter().enumerate() {
            cue.range.validate()?;
            if cue.text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
                    "subtitle cue {index} has empty text",
                )));
            }
            if let Some(previous_start) = previous_start
                && cue.range.start_ms < previous_start
            {
                return Err(CoreError::InvalidResult(
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
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
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
