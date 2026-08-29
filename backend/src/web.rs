use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::{audio_input, error, speech_translation, subtitle};

// --------------audio import module----------------
#[async_trait]
pub trait AudioImportService: Send + Sync {
    async fn import(
        &self,
        source: audio_input::AudioInputSource,
    ) -> Result<audio_input::AudioAsset, error::WebError>;
}

pub struct AudioImportApplicationService {
    audio_input: Box<dyn audio_input::AudioInputService>,
}

impl AudioImportApplicationService {
    pub fn new(audio_input: Box<dyn audio_input::AudioInputService>) -> Self {
        Self { audio_input }
    }
}

#[async_trait]
impl AudioImportService for AudioImportApplicationService {
    async fn import(
        &self,
        source: audio_input::AudioInputSource,
    ) -> Result<audio_input::AudioAsset, error::WebError> {
        self.audio_input
            .import(source)
            .await
            .map_err(error::WebError::AudioInput)
    }
}

//-------------speech module----------------
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SpeechTranslationId(pub Uuid);

#[async_trait]
pub trait SpeechTranslationService: Send + Sync {
    async fn translate(
        &self,
        path: PathBuf,
        request: speech_translation::SpeechTranslationRequest,
    ) -> Result<SpeechTranslationId, error::WebError>;
}

pub struct SpeechTranslationApplicationService {
    engine: Box<dyn speech_translation::SpeechTranslationEngine>,
    store: TranslationStore,
}

impl SpeechTranslationApplicationService {
    pub fn new(
        engine: Box<dyn speech_translation::SpeechTranslationEngine>,
        store: TranslationStore,
    ) -> Self {
        Self { engine, store }
    }
}

struct PathSpeechInput {
    path: PathBuf,
}

#[async_trait]
impl speech_translation::SpeechTranslationInput for PathSpeechInput {
    async fn content(
        &mut self,
    ) -> Result<speech_translation::SpeechTranslationInputContent, error::CoreError> {
        Ok(speech_translation::SpeechTranslationInputContent::File(
            self.path.clone(),
        ))
    }

    async fn close(&mut self) -> Result<(), error::CoreError> {
        Ok(())
    }
}

#[async_trait]
impl SpeechTranslationService for SpeechTranslationApplicationService {
    async fn translate(
        &self,
        path: PathBuf,
        request: speech_translation::SpeechTranslationRequest,
    ) -> Result<SpeechTranslationId, error::WebError> {
        let output = self
            .engine
            .translate_audio(Box::new(PathSpeechInput { path }), request)
            .await?;
        let id = SpeechTranslationId(Uuid::new_v4());
        self.store.save(id, &output)?;
        Ok(id)
    }
}

// --------------subtitle export module----------------
#[async_trait]
pub trait SubtitleExportService: Send + Sync {
    async fn export(
        &self,
        translation_id: SpeechTranslationId,
        request: subtitle::SubtitleExportRequest,
    ) -> Result<subtitle::SubtitleOutput, error::WebError>;
}

pub struct SubtitleExportApplicationService {
    exporter: Box<dyn subtitle::SubtitleExporter>,
    store: TranslationStore,
}

impl SubtitleExportApplicationService {
    pub fn new(exporter: Box<dyn subtitle::SubtitleExporter>, store: TranslationStore) -> Self {
        Self { exporter, store }
    }
}

#[async_trait]
impl SubtitleExportService for SubtitleExportApplicationService {
    async fn export(
        &self,
        translation_id: SpeechTranslationId,
        request: subtitle::SubtitleExportRequest,
    ) -> Result<subtitle::SubtitleOutput, error::WebError> {
        let output = self.store.load(translation_id)?;
        let cues = output
            .segments
            .into_iter()
            .map(|segment| subtitle::SubtitleCue {
                id: Uuid::new_v4(),
                source_segment_id: segment.source_segment_id,
                range: segment.range,
                text: segment.translated_text,
            })
            .collect();
        let document = subtitle::SubtitleDocument::from_cues(
            output.source_language,
            output.target_language,
            cues,
        );
        document.validate()?;
        self.exporter
            .export(&document, request)
            .await
            .map_err(error::WebError::from)
    }
}

#[derive(Clone)]
pub struct TranslationStore {
    root: PathBuf,
}

impl TranslationStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn save(
        &self,
        id: SpeechTranslationId,
        output: &speech_translation::SpeechTranslationOutput,
    ) -> Result<(), error::WebError> {
        std::fs::create_dir_all(&self.root).map_err(|error| {
            error::WebError::Core(error::CoreError::Provider {
                provider: "translation-store".to_owned(),
                message: format!("cannot create translation directory: {error}"),
            })
        })?;
        let bytes = serde_json::to_vec(output).map_err(|error| {
            error::WebError::Core(error::CoreError::InvalidResult(format!(
                "cannot encode speech translation: {error}"
            )))
        })?;
        std::fs::write(self.path(id), bytes).map_err(|error| {
            error::WebError::Core(error::CoreError::Provider {
                provider: "translation-store".to_owned(),
                message: format!("cannot persist speech translation: {error}"),
            })
        })?;
        Ok(())
    }

    fn load(
        &self,
        id: SpeechTranslationId,
    ) -> Result<speech_translation::SpeechTranslationOutput, error::WebError> {
        let bytes = std::fs::read(self.path(id)).map_err(|_| {
            error::WebError::Core(error::CoreError::InvalidInput(format!(
                "speech translation {id:?} was not found"
            )))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            error::WebError::Core(error::CoreError::InvalidResult(format!(
                "invalid speech translation {id:?}: {error}"
            )))
        })
    }

    fn path(&self, id: SpeechTranslationId) -> PathBuf {
        self.root.join(format!("{}.json", id.0))
    }
}

impl Default for TranslationStore {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("audio-translator-translations"))
    }
}
