use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use audio_translator::{
    asr::FasterWhisperAsrEngine,
    audio_input::{AudioAssetId, MediaAudioInputService},
    speech_translation::{AsrThenTranslationEngine, SpeechTranslationId, SpeechTranslationOutput},
    subtitle::SrtSubtitleExporter,
    translation::ArgosTranslator,
};

pub struct AppState {
    pub audio_input: MediaAudioInputService,
    pub speech_engine: AsrThenTranslationEngine,
    pub subtitle_exporter: SrtSubtitleExporter,
    pub audio_assets: Mutex<HashMap<AudioAssetId, PathBuf>>,
    pub translations: Mutex<HashMap<SpeechTranslationId, SpeechTranslationOutput>>,
}

impl AppState {
    pub fn new() -> Self {
        let model =
            std::env::var("AUDIO_TRANSLATOR_WHISPER_MODEL").unwrap_or_else(|_| "small".to_owned());
        Self {
            audio_input: MediaAudioInputService::default(),
            speech_engine: AsrThenTranslationEngine::new(
                Box::new(FasterWhisperAsrEngine::new(model)),
                Box::new(ArgosTranslator::default()),
            ),
            subtitle_exporter: SrtSubtitleExporter,
            audio_assets: Mutex::new(HashMap::new()),
            translations: Mutex::new(HashMap::new()),
        }
    }
}
