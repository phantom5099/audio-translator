use async_trait::async_trait;
use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::{TranslatedTranscript, TranslationRequest, Translator};
use crate::error::{CoreError, TranslationError};

/// Offline Argos Translate adapter. Language packages are deployment dependencies.
pub struct ArgosTranslator {
    pub python: String,
}

impl ArgosTranslator {
    pub fn new() -> Self {
        Self {
            python: "python".to_owned(),
        }
    }
    pub fn with_python(mut self, python: impl Into<String>) -> Self {
        self.python = python.into();
        self
    }
}

impl Default for ArgosTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for ArgosTranslator {
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslatedTranscript, TranslationError> {
        let payload = serde_json::to_vec(&request).map_err(|error| {
            TranslationError::Core(CoreError::InvalidInput(format!(
                "cannot encode translation request: {error}"
            )))
        })?;
        let mut child = Command::new(&self.python)
            .arg("-c")
            .arg(ARGOS_WORKER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| TranslationError::Provider {
                provider: "argos-translate".to_owned(),
                message: format!("failed to start {}: {error}", self.python),
            })?;
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(&payload)
            .map_err(|error| TranslationError::Provider {
                provider: "argos-translate".to_owned(),
                message: error.to_string(),
            })?;
        let output = child
            .wait_with_output()
            .map_err(|error| TranslationError::Provider {
                provider: "argos-translate".to_owned(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(TranslationError::Provider {
                provider: "argos-translate".to_owned(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        serde_json::from_slice(&output.stdout).map_err(|error| TranslationError::Provider {
            provider: "argos-translate".to_owned(),
            message: format!("invalid worker response: {error}"),
        })
    }
}

const ARGOS_WORKER: &str = r#"
import json, sys
from argostranslate import translate
request = json.load(sys.stdin)
source = (request.get("source_language") or "").replace("_", "-").split("-")[0]
target = request["target_language"].replace("_", "-").split("-")[0]
segments = []
for item in request["segments"]:
    segments.append({"source_segment_id": item["id"], "range": item["range"], "source_text": item["text"], "translated_text": translate.translate(item["text"], source, target), "warnings": []})
print(json.dumps({"source_language": request.get("source_language"), "target_language": request["target_language"], "segments": segments}, ensure_ascii=False))
"#;
