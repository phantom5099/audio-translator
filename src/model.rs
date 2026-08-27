use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CoreError;

pub type SegmentId = Uuid;
pub type CueId = Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageTag(#[doc = "语言标签的实际字符串值，例如 zh-CN、en-US 或 ja-JP。"] pub String);

impl LanguageTag {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CoreError::InvalidInput(
                "language tag must not be empty".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LanguageTag {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for LanguageTag {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// 时间范围的起始时间，单位为毫秒，包含该时刻。
    pub start_ms: u64,
    /// 时间范围的结束时间，单位为毫秒，通常不包含该时刻。
    pub end_ms: u64,
}

impl TimeRange {
    pub fn new(start_ms: u64, end_ms: u64) -> Result<Self, CoreError> {
        if start_ms >= end_ms {
            return Err(CoreError::InvalidInput(format!(
                "time range must satisfy start_ms < end_ms, got {start_ms}..{end_ms}"
            )));
        }
        Ok(Self { start_ms, end_ms })
    }

    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.start_ms >= self.end_ms {
            return Err(CoreError::InvalidInput(format!(
                "time range must satisfy start_ms < end_ms, got {}..{}",
                self.start_ms, self.end_ms
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speaker {
    /// 实体的稳定唯一标识。
    pub id: String,
    /// 面向用户显示的名称。
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    /// 使用归一化的 32 位浮点采样值，通常范围为 [-1.0, 1.0]。
    F32,
    /// 使用有符号 16 位整数采样值；按裸字节传输时还需要约定端序。
    I16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleLayout {
    /// 多声道采样值按帧交错排列。
    Interleaved,
    /// 每个声道的采样值分别连续存储。
    Planar,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioFormat {
    /// 音频采样率，单位为 Hz。
    pub sample_rate: u32,
    /// 音频声道数量。
    pub channels: u16,
    /// 单个采样值的数值格式。
    pub sample_format: SampleFormat,
    /// 多声道采样值的存储布局。
    pub layout: SampleLayout,
}

impl AudioFormat {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.sample_rate == 0 {
            return Err(CoreError::InvalidInput(
                "audio sample rate must be greater than zero".to_owned(),
            ));
        }
        if self.channels == 0 {
            return Err(CoreError::InvalidInput(
                "audio channel count must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AudioSamples {
    /// 以 32 位浮点数保存采样值。
    F32(Vec<f32>),
    /// 以 16 位有符号整数保存采样值。
    I16(Vec<i16>),
}

impl AudioSamples {
    pub fn sample_count(&self) -> usize {
        match self {
            Self::F32(samples) => samples.len(),
            Self::I16(samples) => samples.len(),
        }
    }

    pub fn format(&self) -> SampleFormat {
        match self {
            Self::F32(_) => SampleFormat::F32,
            Self::I16(_) => SampleFormat::I16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioInfo {
    /// 音频数据使用的格式描述。
    pub format: AudioFormat,
    /// 音频或音频块的时长，单位为毫秒。
    pub duration_ms: Option<u64>,
    /// 输入是否为持续产生数据的实时流。
    pub is_live: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioChunk {
    /// 该音频块在输入时间轴上的起始时间，单位为毫秒。
    pub timestamp_ms: u64,
    /// 音频或音频块的时长，单位为毫秒。
    pub duration_ms: u32,
    /// 该音频块包含的采样帧数量；一帧包含所有声道的一个采样值。
    pub frames: u32,
    /// 该音频块中的实际采样数据。
    pub samples: AudioSamples,
    /// 音频数据使用的格式描述。
    pub format: AudioFormat,
    /// 该音频块是否是当前输入的最后一个块。
    pub is_final: bool,
}

impl AudioChunk {
    pub fn validate(&self) -> Result<(), CoreError> {
        self.format.validate()?;
        if self.frames == 0 {
            return Err(CoreError::InvalidInput(
                "audio chunk must contain at least one frame".to_owned(),
            ));
        }
        if self.duration_ms == 0 {
            return Err(CoreError::InvalidInput(
                "audio chunk duration must be greater than zero".to_owned(),
            ));
        }
        if self.samples.format() != self.format.sample_format {
            return Err(CoreError::InvalidInput(
                "AudioSamples format does not match AudioFormat.sample_format".to_owned(),
            ));
        }
        let expected = self.frames as usize * self.format.channels as usize;
        if self.samples.sample_count() != expected {
            return Err(CoreError::InvalidInput(format!(
                "audio chunk has {} samples but {} are required for {} frames and {} channels",
                self.samples.sample_count(),
                expected,
                self.frames,
                self.format.channels
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioFormatRequirement {
    /// provider 可以接受的采样率列表，单位为 Hz；为空表示不限制。
    pub accepted_sample_rates_hz: Vec<u32>,
    /// provider 优先使用的采样率，单位为 Hz。
    pub preferred_sample_rate_hz: Option<u32>,
    /// provider 可以接受的声道数量列表；为空表示不限制。
    pub accepted_channels: Vec<u16>,
    /// provider 可以接受的采样值格式列表；为空表示不限制。
    pub accepted_sample_formats: Vec<SampleFormat>,
    /// provider 可以接受的采样布局列表；为空表示不限制。
    pub accepted_layouts: Vec<SampleLayout>,
    /// 是否强制要求输入为单声道。
    pub requires_mono: bool,
}

impl AudioFormatRequirement {
    pub fn accepts(&self, format: &AudioFormat) -> bool {
        let rate_ok = self.accepted_sample_rates_hz.is_empty()
            || self.accepted_sample_rates_hz.contains(&format.sample_rate);
        let channels_ok =
            self.accepted_channels.is_empty() || self.accepted_channels.contains(&format.channels);
        let sample_format_ok = self.accepted_sample_formats.is_empty()
            || self.accepted_sample_formats.contains(&format.sample_format);
        let layout_ok =
            self.accepted_layouts.is_empty() || self.accepted_layouts.contains(&format.layout);
        let mono_ok = !self.requires_mono || format.channels == 1;
        rate_ok && channels_ok && sample_format_ok && layout_ok && mono_ok
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transcript {
    /// 识别出的源语言；无法确定时为空。
    pub language: Option<LanguageTag>,
    /// 按时间顺序排列的文本片段列表。
    pub segments: Vec<TranscriptSegment>,
}

impl Transcript {
    pub fn validate(&self) -> Result<(), CoreError> {
        for segment in &self.segments {
            segment.range.validate()?;
            if segment.text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
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
    pub id: SegmentId,
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
    /// 与该实体关联的文本内容。
    pub text: String,
    /// 可选的说话人信息。
    pub speaker: Option<Speaker>,
    /// provider 对识别或对齐结果的置信度；无法提供时为空。
    pub confidence: Option<f32>,
    /// 可选的词级时间戳和置信度信息。
    pub words: Option<Vec<WordTiming>>,
    /// 该结果的修订版本号，用于表示实时结果或人工修改的更新。
    pub revision: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordTiming {
    /// 与该实体关联的文本内容。
    pub text: String,
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
    /// provider 对识别或对齐结果的置信度；无法提供时为空。
    pub confidence: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslatedTranscript {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 按时间顺序排列的文本片段列表。
    pub segments: Vec<TranslatedSegment>,
}

impl TranslatedTranscript {
    pub fn validate_against(&self, source: &Transcript) -> Result<(), CoreError> {
        if self.target_language.0.trim().is_empty() {
            return Err(CoreError::InvalidResult(
                "translated transcript target language is empty".to_owned(),
            ));
        }
        if self.segments.len() != source.segments.len() {
            return Err(CoreError::InvalidResult(format!(
                "translation returned {} segments for {} source segments",
                self.segments.len(),
                source.segments.len()
            )));
        }
        for (source_segment, translated_segment) in source.segments.iter().zip(&self.segments) {
            if translated_segment.source_segment_id != source_segment.id {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment does not match source segment {}",
                    source_segment.id
                )));
            }
            if translated_segment.range != source_segment.range {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment {} changed its time range",
                    source_segment.id
                )));
            }
            if translated_segment.source_text != source_segment.text {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment {} changed its source text",
                    source_segment.id
                )));
            }
            if translated_segment.translated_text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
                    "translated segment {} has empty translated text",
                    source_segment.id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslatedSegment {
    /// 对应源文本片段的稳定唯一标识。
    pub source_segment_id: SegmentId,
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
    /// 翻译前的源文本。
    pub source_text: String,
    /// 翻译后的目标文本。
    pub translated_text: String,
    /// 可选的说话人信息。
    pub speaker: Option<Speaker>,
    /// 与该翻译片段关联的警告列表。
    pub warnings: Vec<TranslationWarning>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TranslationWarning {
    TerminologyMismatch,
    NumberChanged,
    PlaceholderChanged,
    LengthExceeded,
    ProviderWarning(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtitleDocument {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 字幕 cue 列表。
    pub cues: Vec<SubtitleCue>,
    /// 该结果的修订版本号，用于表示实时结果或人工修改的更新。
    pub revision: u64,
}

impl SubtitleDocument {
    pub fn from_translation(
        transcript: &Transcript,
        translated: &TranslatedTranscript,
    ) -> Result<Self, CoreError> {
        translated.validate_against(transcript)?;
        Ok(Self {
            source_language: transcript.language.clone(),
            target_language: translated.target_language.clone(),
            cues: translated
                .segments
                .iter()
                .map(|segment| SubtitleCue {
                    id: Uuid::new_v4(),
                    source_segment_id: segment.source_segment_id,
                    range: segment.range,
                    text: segment.translated_text.clone(),
                    speaker: segment.speaker.clone(),
                })
                .collect(),
            revision: 0,
        })
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        let mut previous_start = None;
        for cue in &self.cues {
            cue.range.validate()?;
            if cue.text.trim().is_empty() {
                return Err(CoreError::InvalidResult(format!(
                    "subtitle cue {} has empty text",
                    cue.id
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
    /// 实体的稳定唯一标识。
    pub id: CueId,
    /// 对应源文本片段的稳定唯一标识。
    pub source_segment_id: SegmentId,
    /// 该文本片段对应的音频时间范围。
    pub range: TimeRange,
    /// 与该实体关联的文本内容。
    pub text: String,
    /// 可选的说话人信息。
    pub speaker: Option<Speaker>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProviderOptions {
    /// provider 名称到 provider 私有配置 JSON 值的映射。
    pub values: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimestampLevel {
    /// 只要求片段级时间戳。
    Segment,
    /// 要求尽可能提供词级时间戳。
    Word,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrVocabulary {
    /// 用于提示 ASR 的单词或术语列表。
    pub terms: Vec<String>,
    /// 用于提示 ASR 的短语列表。
    pub phrases: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrRequest {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 需要生成的时间戳粒度。
    pub timestamp_level: TimestampLevel,
    /// 是否要求识别并返回说话人标签。
    pub enable_speaker_labels: bool,
    /// 可选的 ASR 词汇或热词提示。
    pub vocabulary: Option<AsrVocabulary>,
    /// provider 专属的扩展配置。
    pub options: ProviderOptions,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TranslationContext {
    /// 当前片段之前可供翻译参考的上下文片段。
    pub previous: Vec<ContextSegment>,
    /// 当前片段之后可供翻译参考的上下文片段。
    pub next: Vec<ContextSegment>,
    /// 可选的整篇文档摘要，用于提供长上下文。
    pub document_summary: Option<String>,
    /// 可选的翻译风格或写作风格说明。
    pub style: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSegment {
    /// 上下文或事件关联的文本片段唯一标识。
    pub segment_id: SegmentId,
    /// 与该实体关联的文本内容。
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GlossaryContext {
    /// 当前请求使用的术语词条列表。
    pub entries: Vec<GlossaryEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlossaryEntry {
    /// 词条的源语言表达。
    pub source: String,
    /// 词条对应的目标语言表达；为空表示只提供源词。
    pub target: Option<String>,
    /// 词条可匹配的别名列表。
    pub aliases: Vec<String>,
    /// 关于词条用法、领域或限制的补充说明。
    pub note: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TranslationConstraints {
    /// 是否要求翻译结果保留源文本中的数字。
    pub preserve_numbers: bool,
    /// 是否要求翻译结果保留占位符。
    pub preserve_placeholders: bool,
    /// 是否要求翻译结果保留源文本换行。
    pub preserve_line_breaks: bool,
    /// 字幕单行允许的最大字符数；为空表示不指定。
    pub max_chars_per_line: Option<u32>,
    /// 是否允许 provider 在翻译前重写或修正源文本。
    pub allow_rewrite_source: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// 源文本使用的语言。
    pub source_language: Option<LanguageTag>,
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 按时间顺序排列的文本片段列表。
    pub segments: Vec<TranscriptSegment>,
    /// 传递给翻译 provider 的上下文信息。
    pub context: TranslationContext,
    /// 本次翻译使用的可选术语词典上下文。
    pub glossary: Option<GlossaryContext>,
    /// 本次翻译需要遵守的约束条件。
    pub constraints: TranslationConstraints,
    /// provider 专属的扩展配置。
    pub options: ProviderOptions,
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
    /// 导出时是否把说话人信息写入字幕文本。
    pub include_speaker: bool,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranslationRequestTemplate {
    /// 目标文本使用的语言。
    pub target_language: LanguageTag,
    /// 从转录结果构造翻译上下文的策略。
    pub context_policy: ContextPolicy,
    /// 本次翻译使用的可选术语词典上下文。
    pub glossary: Option<GlossaryContext>,
    /// 本次翻译需要遵守的约束条件。
    pub constraints: TranslationConstraints,
    /// provider 专属的扩展配置。
    pub options: ProviderOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContextPolicy {
    /// 不向翻译 provider 传递额外上下文。
    None,
    /// 使用当前片段附近的前后片段作为上下文。
    NeighboringSegments {
        /// 当前片段之前需要携带的上下文片段数量。
        before: usize,
        /// 当前片段之后需要携带的上下文片段数量。
        after: usize,
    },
    /// 使用整篇转录内容作为文档上下文。
    Document,
}
