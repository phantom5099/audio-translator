use async_trait::async_trait;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use tracing::{debug, error};

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
        let segment_count = request.segments.len();
        debug!(
            python = %self.python,
            source = ?request.source_language,
            target = ?request.target_language,
            segment_count,
            "translation: starting argos worker"
        );
        let payload = serde_json::to_vec(&request).map_err(|error| {
            error!(?error, "translation: cannot encode request");
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
            // Windows 下 Python stdout 默认走系统代码页（cp936/GBK），
            // 会导致 ensure_ascii=False 输出的中文被编码成 GBK 字节，
            // Rust 侧 serde_json 按 UTF-8 解析失败。启用 Python UTF-8 模式强制 UTF-8。
            .env("PYTHONUTF8", "1")
            .spawn()
            .map_err(|error| {
                error!(python = %self.python, "translation: failed to start worker: {error}");
                TranslationError::Provider {
                    provider: "argos-translate".to_owned(),
                    message: format!("failed to start {}: {error}", self.python),
                }
            })?;
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(&payload)
            .map_err(|error| {
                error!(?error, "translation: failed to write stdin");
                TranslationError::Provider {
                    provider: "argos-translate".to_owned(),
                    message: error.to_string(),
                }
            })?;
        let output = child
            .wait_with_output()
            .map_err(|error| {
                error!(?error, "translation: worker wait failed");
                TranslationError::Provider {
                    provider: "argos-translate".to_owned(),
                    message: error.to_string(),
                }
            })?;
        debug!(code = ?output.status.code(), "translation: worker finished");
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!(
                code = ?output.status.code(),
                stderr = stderr.trim(),
                stdout = stdout.trim(),
                "translation: worker failed"
            );
            return Err(TranslationError::Provider {
                provider: "argos-translate".to_owned(),
                message: stderr.trim().to_owned(),
            });
        }
        let translated: TranslatedTranscript =
            serde_json::from_slice(&output.stdout).map_err(|error| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                error!(stdout = stdout.trim(), "translation: invalid worker response: {error}");
                TranslationError::Provider {
                    provider: "argos-translate".to_owned(),
                    message: format!("invalid worker response: {error}"),
                }
            })?;
        debug!(
            out_segment_count = translated.segments.len(),
            "translation: response parsed"
        );
        Ok(translated)
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
    segments.append({"range": item["range"], "source_text": item["text"], "translated_text": translate.translate(item["text"], source, target), "warnings": []})
print(json.dumps({"source_language": request.get("source_language"), "target_language": request["target_language"], "segments": segments}, ensure_ascii=False))
"#;
