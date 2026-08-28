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
    #[error("speech translation provider `{provider}` failed: {message}")]
    Provider { provider: String, message: String },
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("subtitle format `{0}` is not supported")]
    UnsupportedFormat(String),
}

#[derive(Debug, Error)]
pub enum TranslationWorkflowError {
    #[error("speech translation stage failed: {0}")]
    SpeechTranslation(#[source] SpeechTranslationError),
    #[error("ASR stage failed: {0}")]
    Asr(#[source] AsrError),
    #[error("translation stage failed: {0}")]
    Translation(#[source] TranslationError),
    #[error("subtitle export stage failed: {0}")]
    Export(#[source] ExportError),
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl From<SpeechTranslationError> for TranslationWorkflowError {
    fn from(value: SpeechTranslationError) -> Self {
        Self::SpeechTranslation(value)
    }
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
