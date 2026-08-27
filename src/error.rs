use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    /// 调用方提供的输入不符合核心接口约束。
    InvalidInput(String),
    #[error("unsupported audio format: {0}")]
    /// 当前实现不支持请求的音频或媒体格式。
    UnsupportedFormat(String),
    #[error("audio input reached end of stream")]
    /// 音频输入已经到达流末尾。
    EndOfStream,
    #[error("operation was cancelled")]
    /// 操作已被取消。
    Cancelled,
    #[error("provider `{provider}` failed: {message}")]
    /// provider 执行失败。
    Provider {
        /// 发生错误的 provider 名称。
        provider: String,
        /// 错误的详细信息。
        message: String,
    },
    #[error("invalid provider result: {0}")]
    /// provider 返回的结果不符合核心模型约束。
    InvalidResult(String),
}

#[derive(Debug, Error)]
pub enum AsrError {
    #[error(transparent)]
    /// ASR 调用过程中发生核心层错误。
    Core(#[from] CoreError),
    #[error("ASR provider `{provider}` failed: {message}")]
    /// ASR provider 执行失败。
    Provider {
        /// 发生错误的 ASR provider 名称。
        provider: String,
        /// ASR provider 返回的错误信息。
        message: String,
    },
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error(transparent)]
    /// 翻译调用过程中发生核心层错误。
    Core(#[from] CoreError),
    #[error("translation provider `{provider}` failed: {message}")]
    /// 翻译 provider 执行失败。
    Provider {
        /// 发生错误的翻译 provider 名称。
        provider: String,
        /// 翻译 provider 返回的错误信息。
        message: String,
    },
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    /// 字幕导出过程中发生核心层错误。
    Core(#[from] CoreError),
    #[error("subtitle format `{0}` is not supported")]
    /// 字幕导出器不支持请求的格式。
    UnsupportedFormat(String),
}

#[derive(Debug, Error)]
pub enum TranslationWorkflowError {
    #[error("ASR stage failed: {0}")]
    /// 音频翻译流程的 ASR 阶段失败。
    Asr(#[source] AsrError),
    #[error("translation stage failed: {0}")]
    /// 音频翻译流程的翻译阶段失败。
    Translation(#[source] TranslationError),
    #[error("subtitle export stage failed: {0}")]
    /// 音频翻译流程的字幕导出阶段失败。
    Export(#[source] ExportError),
    #[error(transparent)]
    /// 音频翻译流程发生核心层错误。
    Core(#[from] CoreError),
}

impl From<AsrError> for TranslationWorkflowError {
    fn from(value: AsrError) -> Self {
        Self::Asr(value)
    }
}

impl From<TranslationError> for TranslationWorkflowError {
    fn from(value: TranslationError) -> Self {
        Self::Translation(value)
    }
}

impl From<ExportError> for TranslationWorkflowError {
    fn from(value: ExportError) -> Self {
        Self::Export(value)
    }
}
