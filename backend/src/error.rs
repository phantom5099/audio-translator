use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unsupported audio format: {0}")]
    UnsupportedFormat(String),
    #[error("audio input reached end of stream")]
    EndOfStream,
    #[error("operation was cancelled")]
    Cancelled,
    #[error("provider `{provider}` failed: {message}")]
    Provider { provider: String, message: String },
    #[error("invalid provider result: {0}")]
    InvalidResult(String),
}

#[derive(Debug, Error)]
pub enum AsrError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("ASR provider `{provider}` failed: {message}")]
    Provider { provider: String, message: String },
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("translation provider `{provider}` failed: {message}")]
    Provider { provider: String, message: String },
}

#[derive(Debug, Error)]
pub enum SpeechTranslationError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("ASR stage failed: {0}")]
    Asr(#[source] AsrError),
    #[error("translation stage failed: {0}")]
    Translation(#[source] TranslationError),
    #[error("speech translation provider `{provider}` failed: {message}")]
    Provider { provider: String, message: String },
}

impl From<AsrError> for SpeechTranslationError {
    fn from(value: AsrError) -> Self {
        Self::Asr(value)
    }
}

impl From<TranslationError> for SpeechTranslationError {
    fn from(value: TranslationError) -> Self {
        Self::Translation(value)
    }
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("subtitle format `{0}` is not supported")]
    UnsupportedFormat(String),
}

#[derive(Debug, Error)]
pub enum WebError {
    #[error("audio input stage failed: {0}")]
    AudioInput(#[source] CoreError),
    #[error("speech translation stage failed: {0}")]
    SpeechTranslation(#[source] SpeechTranslationError),
    #[error("subtitle export stage failed: {0}")]
    SubtitleExport(#[source] ExportError),
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl From<SpeechTranslationError> for WebError {
    fn from(value: SpeechTranslationError) -> Self {
        Self::SpeechTranslation(value)
    }
}

impl From<ExportError> for WebError {
    fn from(value: ExportError) -> Self {
        Self::SubtitleExport(value)
    }
}
