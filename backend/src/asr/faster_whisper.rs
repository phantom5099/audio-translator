use async_trait::async_trait;
use serde::Deserialize;
use std::{path::PathBuf, process::Command};

use super::{AsrEngine, AsrInput, AsrInputContent, AsrRequest, Transcript, TranscriptSegment};
use crate::{
    common::{LanguageTag, TimeRange},
    error::{AsrError, CoreError},
};

pub struct FasterWhisperAsrEngine {
    pub python: String,
    pub model: String,
}

impl FasterWhisperAsrEngine {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            python: "python".to_owned(),
            model: model.into(),
        }
    }

    pub fn with_python(mut self, python: impl Into<String>) -> Self {
        self.python = python.into();
        self
    }
}

#[async_trait]
impl AsrEngine for FasterWhisperAsrEngine {
    async fn transcribe(
        &self,
        mut input: Box<dyn AsrInput>,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError> {
        let content = input.content().await?;
        input.close().await?;
        let (path, remove_after) = match content {
            AsrInputContent::File(path) => (path, false),
            AsrInputContent::Bytes(bytes) => {
                let path = temporary_media_path();
                std::fs::write(&path, bytes).map_err(|error| {
                    AsrError::Core(CoreError::Provider {
                        provider: "faster-whisper".to_owned(),
                        message: format!("cannot create temporary media file: {error}"),
                    })
                })?;
                (path, true)
            }
        };
        let language = request.source_language.map(|value| language_code(&value));
        let output = Command::new(&self.python)
            .arg("-c")
            .arg(FASTER_WHISPER_WORKER)
            .arg(&self.model)
            .arg(language.as_deref().unwrap_or(""))
            .arg(&path)
            .output()
            .map_err(|error| AsrError::Provider {
                provider: "faster-whisper".to_owned(),
                message: format!("failed to start {}: {error}", self.python),
            });
        if remove_after {
            let _ = std::fs::remove_file(&path);
        }
        let output = output?;
        if !output.status.success() {
            return Err(AsrError::Provider {
                provider: "faster-whisper".to_owned(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        let response: FasterWhisperResponse =
            serde_json::from_slice(&output.stdout).map_err(|error| AsrError::Provider {
                provider: "faster-whisper".to_owned(),
                message: format!("invalid worker response: {error}"),
            })?;
        let segments = response
            .segments
            .into_iter()
            .map(|segment| {
                Ok(TranscriptSegment {
                    id: uuid::Uuid::new_v4(),
                    range: TimeRange::new(seconds_to_ms(segment.start), seconds_to_ms(segment.end))
                        .map_err(AsrError::Core)?,
                    text: segment.text.trim().to_owned(),
                })
            })
            .collect::<Result<Vec<_>, AsrError>>()?;
        let transcript = Transcript {
            language: response.language.map(LanguageTag::from),
            segments,
        };
        transcript.validate().map_err(AsrError::Core)?;
        Ok(transcript)
    }
}

#[derive(Deserialize)]
struct FasterWhisperResponse {
    language: Option<String>,
    segments: Vec<FasterWhisperSegment>,
}

#[derive(Deserialize)]
struct FasterWhisperSegment {
    start: f64,
    end: f64,
    text: String,
}

fn temporary_media_path() -> PathBuf {
    std::env::temp_dir().join(format!("audio-translator-{}.media", uuid::Uuid::new_v4()))
}

fn seconds_to_ms(seconds: f64) -> u64 {
    (seconds.max(0.0) * 1_000.0).round() as u64
}

fn language_code(language: &LanguageTag) -> String {
    language
        .as_str()
        .split(['-', '_'])
        .next()
        .unwrap_or(language.as_str())
        .to_owned()
}

const FASTER_WHISPER_WORKER: &str = r#"
import json, sys
from faster_whisper import WhisperModel
model = WhisperModel(sys.argv[1])
segments, info = model.transcribe(sys.argv[3], language=(sys.argv[2] or None), vad_filter=True)
print(json.dumps({"language": info.language, "segments": [{"start": s.start, "end": s.end, "text": s.text} for s in segments]}, ensure_ascii=False))
"#;
