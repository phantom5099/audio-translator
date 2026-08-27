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

pub struct AudioTranslationRequest {
    /// 已由 audio_input 导入服务准备好的统一音频输入。
    pub input: Box<dyn AudioInput>,
    /// ASR 阶段的请求参数
    pub asr: AsrRequest,
    /// 翻译阶段的请求模板
    pub translation: TranslationRequestTemplate,
    /// 字幕导出阶段的请求参数
    pub output: SubtitleExportRequest,
}

/// 一次完整音频翻译任务的结果。
pub struct AudioTranslationResult {
    /// ASR 生成的源语言转录结果
    pub transcript: Transcript,
    /// 翻译 provider 生成的目标语言结果，并保留与源片段的关联
    pub translated: TranslatedTranscript,
    /// 由转录和译文组合生成的内部字幕文档，是格式无关的中间表示
    pub subtitle: SubtitleDocument,
    /// 字幕导出的最终字节和建议文件名
    pub output: SubtitleOutput,
}
