use audio_translator::{
    audio_input::{AudioAssetId, AudioInputService, AudioInputSource, AudioMetadata},
    speech_translation::{
        SpeechTranslationEngine, SpeechTranslationId, SpeechTranslationOutput,
        SpeechTranslationRequest,
    },
    subtitle::{
        SubtitleCue, SubtitleDocument, SubtitleExportRequest, SubtitleExporter, SubtitleOutput,
    },
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

use crate::state::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioAssetResponse {
    pub id: AudioAssetId,
    pub file_name: String,
    pub metadata: AudioMetadata,
}

#[tauri::command]
pub async fn import_audio(
    source: AudioInputSource,
    state: State<'_, AppState>,
) -> Result<AudioAssetResponse, String> {
    info!(?source, "command: import_audio");
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
        file_name: asset.file_name,
        metadata: asset.metadata,
    })
}

#[tauri::command]
pub async fn start_translation(
    asset_id: AudioAssetId,
    request: SpeechTranslationRequest,
    state: State<'_, AppState>,
) -> Result<SpeechTranslationId, String> {
    info!(
        ?asset_id,
        source = ?request.source_language,
        target = ?request.target_language,
        "command: start_translation"
    );
    let path = state
        .audio_assets
        .lock()
        .map_err(|_| "audio asset state is unavailable".to_owned())?
        .get(&asset_id)
        .cloned()
        .ok_or_else(|| {
            error!(?asset_id, "command: audio asset not found");
            format!("audio asset {asset_id:?} was not found")
        })?;
    let output = state
        .speech_engine
        .translate_audio(path, request)
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
    info!(
        ?translation_id,
        ?request,
        "command: export_subtitle"
    );
    let output = state
        .translations
        .lock()
        .map_err(|_| "translation state is unavailable".to_owned())?
        .get(&translation_id)
        .cloned()
        .ok_or_else(|| {
            error!(?translation_id, "command: speech translation not found");
            format!("speech translation {translation_id:?} was not found")
        })?;
    let document = subtitle_document(output);
    debug!(cue_count = document.cues.len(), "command: subtitle document built");
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
            range: segment.range,
            text: segment.translated_text,
        })
        .collect();
    SubtitleDocument::from_cues(output.source_language, output.target_language, cues)
}
