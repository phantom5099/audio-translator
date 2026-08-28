use async_trait::async_trait;

use crate::{
    audio_input::AudioInput,
    error::TranslationWorkflowError,
    speech_translation::{SpeechTranslationOutput, SpeechTranslationRequest},
    subtitle::{SubtitleDocument, SubtitleExportRequest, SubtitleOutput},
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
    /// 语音翻译阶段的统一请求参数。
    pub speech_translation: SpeechTranslationRequest,
    /// 字幕导出阶段的请求参数
    pub output: SubtitleExportRequest,
}

/// 一次完整音频翻译任务的结果。
pub struct AudioTranslationResult {
    /// 语音翻译能力对外发布的结果。
    pub speech_translation: SpeechTranslationOutput,
    /// 由转录和译文组合生成的内部字幕文档，是格式无关的中间表示
    pub subtitle: SubtitleDocument,
    /// 字幕导出的最终字节和建议文件名
    pub output: SubtitleOutput,
}
