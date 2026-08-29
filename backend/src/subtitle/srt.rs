use async_trait::async_trait;

use super::{
    LineBreakPolicy, SubtitleDocument, SubtitleExportRequest, SubtitleExporter, SubtitleFormat,
    SubtitleOutput, TextEncoding,
};
use crate::error::{CoreError, ExportError};

/// SRT exporter owned by the subtitle module.
pub struct SrtSubtitleExporter;

#[async_trait]
impl SubtitleExporter for SrtSubtitleExporter {
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError> {
        document.validate()?;
        if !matches!(request.format, SubtitleFormat::Srt) {
            return Err(ExportError::UnsupportedFormat(
                request.format.extension().to_owned(),
            ));
        }
        if !matches!(request.encoding, TextEncoding::Utf8) {
            return Err(ExportError::Core(CoreError::UnsupportedFormat(
                "only UTF-8 is supported".to_owned(),
            )));
        }
        let mut content = String::new();
        for (index, cue) in document.cues.iter().enumerate() {
            content.push_str(&(index + 1).to_string());
            content.push('\n');
            content.push_str(&format!(
                "{} --> {}\n",
                srt_time(cue.range.start_ms),
                srt_time(cue.range.end_ms)
            ));
            content.push_str(&apply_line_policy(&cue.text, &request.line_policy));
            content.push_str("\n\n");
        }
        Ok(SubtitleOutput {
            format: SubtitleFormat::Srt,
            bytes: content.into_bytes(),
            suggested_name: "translated-subtitles.srt".to_owned(),
        })
    }
}

fn apply_line_policy(text: &str, policy: &LineBreakPolicy) -> String {
    match policy {
        LineBreakPolicy::Preserve => text.to_owned(),
        LineBreakPolicy::SingleLine => text.lines().collect::<Vec<_>>().join(" "),
        LineBreakPolicy::AutoWrap { max_chars } if *max_chars > 0 => text
            .chars()
            .collect::<Vec<_>>()
            .chunks(*max_chars as usize)
            .map(|chunk| chunk.iter().copied().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
        LineBreakPolicy::AutoWrap { .. } => text.to_owned(),
    }
}

fn srt_time(milliseconds: u64) -> String {
    let hours = milliseconds / 3_600_000;
    let minutes = (milliseconds % 3_600_000) / 60_000;
    let seconds = (milliseconds % 60_000) / 1_000;
    let millis = milliseconds % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}
