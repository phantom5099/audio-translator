use async_trait::async_trait;

use crate::{
    asr::{AsrRequest, Transcript},
    audio_input::AudioInput,
    error::TranslationWorkflowError,
    subtitle::{SubtitleDocument, SubtitleExportRequest, SubtitleOutput},
    translation::{TranslatedTranscript, TranslationRequestTemplate},
};

/// 音频翻译总流程接口。
#[async_trait]
pub trait AudioTranslationService: Send + Sync {
    /// 执行一次完整的音频翻译任务。
    async fn translate(
        &self,
        request: AudioTranslationRequest,
    ) -> Result<AudioTranslationResult, TranslationWorkflowError>;
}

/// 一次完整音频翻译任务的输入参数。
pub struct AudioTranslationRequest {
    /// 待处理的统一音频输入；使用 trait object 允许运行时切换文件和网络来源。
    pub input: Box<dyn AudioInput>,
    /// ASR 阶段的请求参数；单独保留语言和 provider 专属配置。
    pub asr: AsrRequest,
    /// 翻译阶段的请求模板；流程会根据 ASR 结果补齐源文本和时间轴。
    pub translation: TranslationRequestTemplate,
    /// 字幕导出阶段的请求参数；与内部字幕模型分离以支持多种输出格式。
    pub output: SubtitleExportRequest,
}

/// 一次完整音频翻译任务的结果。
///
/// 同时返回中间结果和最终导出结果，避免调用方为了预览、审校或重新导出而重复执行 ASR、
/// 翻译流程；各阶段失败则通过 `TranslationWorkflowError` 标记具体阶段。
pub struct AudioTranslationResult {
    /// ASR 生成的源语言转录结果，包含原文片段和时间轴。
    pub transcript: Transcript,
    /// 翻译 provider 生成的目标语言结果，并保留与源片段的关联。
    pub translated: TranslatedTranscript,
    /// 由转录和译文组合生成的内部字幕文档，是格式无关的中间表示。
    pub subtitle: SubtitleDocument,
    /// 字幕导出的最终字节和建议文件名。
    pub output: SubtitleOutput,
}
