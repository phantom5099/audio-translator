use async_trait::async_trait;
use serde::Deserialize;
use std::process::Command;
use tracing::{debug, error};

use super::{AsrEngine, AsrRequest, Transcript, TranscriptSegment};
use crate::{
    common::{LanguageTag, TimeRange},
    error::AsrError,
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
        path: std::path::PathBuf,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError> {
        let language = request.source_language.map(|value| language_code(&value));
        debug!(
            python = %self.python,
            model = %self.model,
            audio = %path.display(),
            language = ?language,
            "asr: starting faster-whisper worker"
        );
        let output = Command::new(&self.python)
            .arg("-c")
            .arg(FASTER_WHISPER_WORKER)
            .arg(&self.model)
            .arg(language.as_deref().unwrap_or(""))
            .arg(&path)
            .env("PYTHONUTF8", "1")
            .output()
            .map_err(|error| {
                error!(python = %self.python, "asr: failed to start worker: {error}");
                AsrError::Provider {
                    provider: "faster-whisper".to_owned(),
                    message: format!("failed to start {}: {error}", self.python),
                }
            })?;
        debug!(code = ?output.status.code(), "asr: worker finished");
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!(
                code = ?output.status.code(),
                stderr = stderr.trim(),
                stdout = stdout.trim(),
                "asr: worker failed"
            );
            return Err(AsrError::Provider {
                provider: "faster-whisper".to_owned(),
                message: stderr.trim().to_owned(),
            });
        }
        let response: FasterWhisperResponse =
            serde_json::from_slice(&output.stdout).map_err(|error| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                error!(
                    stdout = stdout.trim(),
                    "asr: invalid worker response: {error}"
                );
                AsrError::Provider {
                    provider: "faster-whisper".to_owned(),
                    message: format!("invalid worker response: {error}"),
                }
            })?;
        debug!(
            language = ?response.language,
            segment_count = response.segments.len(),
            "asr: response parsed"
        );
        let segments = response
            .segments
            .into_iter()
            .map(|segment| {
                Ok(TranscriptSegment {
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
        transcript.validate().map_err(|error| {
            error!(?error, "asr: transcript validation failed");
            AsrError::Core(error)
        })?;
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
