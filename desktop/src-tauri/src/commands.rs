use async_trait::async_trait;
use audio_translator::{
    audio_input::{AudioAssetId, AudioInputService, AudioInputSource},
    speech_translation::{
        SpeechTranslationEngine, SpeechTranslationId, SpeechTranslationInput,
        SpeechTranslationInputContent, SpeechTranslationOutput, SpeechTranslationRequest,
    },
    subtitle::{
        SubtitleCue, SubtitleDocument, SubtitleExportRequest, SubtitleExporter, SubtitleOutput,
    },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

struct FileSpeechInput {
    path: PathBuf,
}

#[async_trait]
impl SpeechTranslationInput for FileSpeechInput {
    async fn content(
        &mut self,
    ) -> Result<SpeechTranslationInputContent, audio_translator::error::CoreError> {
        Ok(SpeechTranslationInputContent::File(self.path.clone()))
    }

    async fn close(&mut self) -> Result<(), audio_translator::error::CoreError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioAssetResponse {
    pub id: AudioAssetId,
    pub origin: audio_translator::audio_input::AudioAssetOrigin,
    pub metadata: audio_translator::audio_input::AudioMetadata,
}

#[tauri::command]
pub async fn import_audio(
    source: AudioInputSource,
    state: State<'_, AppState>,
) -> Result<AudioAssetResponse, String> {
    let asset = state
        .audio_input
        .import(source)
        .await
        .map_err(|error| error.to_string())?;
    state
        .audio_assets
        .lock()
        .map_err(|_| "audio asset state is unavailable".to_owned())?
        .insert(asset.id, asset.path.clone());
    Ok(AudioAssetResponse {
        id: asset.id,
        origin: asset.origin,
        metadata: asset.metadata,
    })
}

#[tauri::command]
pub async fn start_translation(
    asset_id: AudioAssetId,
    request: SpeechTranslationRequest,
    state: State<'_, AppState>,
) -> Result<SpeechTranslationId, String> {
    let path = state
        .audio_assets
        .lock()
        .map_err(|_| "audio asset state is unavailable".to_owned())?
        .get(&asset_id)
        .cloned()
        .ok_or_else(|| format!("audio asset {asset_id:?} was not found"))?;
    let output = state
        .speech_engine
        .translate_audio(Box::new(FileSpeechInput { path }), request)
        .await
        .map_err(|error| error.to_string())?;
    let id = SpeechTranslationId::new();
    state
        .translations
        .lock()
        .map_err(|_| "translation state is unavailable".to_owned())?
        .insert(id, output);
    Ok(id)
}

#[tauri::command]
pub async fn export_subtitle(
    translation_id: SpeechTranslationId,
    request: SubtitleExportRequest,
    state: State<'_, AppState>,
) -> Result<SubtitleOutput, String> {
    let output = state
        .translations
        .lock()
        .map_err(|_| "translation state is unavailable".to_owned())?
        .get(&translation_id)
        .cloned()
        .ok_or_else(|| format!("speech translation {translation_id:?} was not found"))?;
    let document = subtitle_document(output);
    state
        .subtitle_exporter
        .export(&document, request)
        .await
        .map_err(|error| error.to_string())
}

fn subtitle_document(output: SpeechTranslationOutput) -> SubtitleDocument {
    let cues = output
        .segments
        .into_iter()
        .map(|segment| SubtitleCue {
            id: segment.source_segment_id,
            source_segment_id: segment.source_segment_id,
            range: segment.range,
            text: segment.translated_text,
        })
        .collect();
    SubtitleDocument::from_cues(output.source_language, output.target_language, cues)
}
