use async_trait::async_trait;

use super::{
    AsrEngine, AsrRequest, AudioInput, ContextPolicy, ContextSegment, SubtitleDocument,
    SubtitleExportRequest, SubtitleOutput, Transcript, TranslatedTranscript, TranslationContext,
    TranslationRequest, TranslationRequestTemplate, TranslationWorkflowError, Translator,
};

#[async_trait]
pub trait AudioTranslationService: Send + Sync {
    async fn translate(
        &self,
        request: AudioTranslationRequest,
    ) -> Result<AudioTranslationResult, TranslationWorkflowError>;
}

pub struct AudioTranslationRequest {
    /// 待处理的统一音频输入。
    pub input: Box<dyn AudioInput>,
    /// ASR 阶段的请求参数。
    pub asr: AsrRequest,
    /// 翻译阶段的请求模板。
    pub translation: TranslationRequestTemplate,
    /// 字幕导出阶段的请求参数或导出结果。
    pub output: SubtitleExportRequest,
}

pub struct AudioTranslationResult {
    /// ASR 生成的源语言转录结果。
    pub transcript: Transcript,
    /// 翻译 provider 生成的目标语言结果。
    pub translated: TranslatedTranscript,
    /// 由转录和译文组合生成的内部字幕文档。
    pub subtitle: SubtitleDocument,
    /// 字幕导出阶段的请求参数或导出结果。
    pub output: SubtitleOutput,
}

pub struct CoreAudioTranslationService<A, T, E> {
    /// 实际使用的 ASR provider 实现。
    pub asr: A,
    /// 实际使用的翻译 provider 实现。
    pub translator: T,
    /// 负责字幕格式导出的实现。
    pub exporter: E,
}

impl<A, T, E> CoreAudioTranslationService<A, T, E> {
    pub fn new(asr: A, translator: T, exporter: E) -> Self {
        Self {
            asr,
            translator,
            exporter,
        }
    }
}

#[async_trait]
impl<A, T, E> AudioTranslationService for CoreAudioTranslationService<A, T, E>
where
    A: AsrEngine,
    T: Translator,
    E: super::SubtitleExporter,
{
    async fn translate(
        &self,
        request: AudioTranslationRequest,
    ) -> Result<AudioTranslationResult, TranslationWorkflowError> {
        let transcript = self
            .asr
            .transcribe(request.input, request.asr)
            .await
            .map_err(TranslationWorkflowError::Asr)?;
        transcript.validate()?;

        let translation_request = TranslationRequest {
            source_language: transcript.language.clone(),
            target_language: request.translation.target_language,
            segments: transcript.segments.clone(),
            context: build_context(&transcript, &request.translation.context_policy),
            glossary: request.translation.glossary,
            constraints: request.translation.constraints,
            options: request.translation.options,
        };

        let translated = self
            .translator
            .translate(translation_request)
            .await
            .map_err(TranslationWorkflowError::Translation)?;
        translated.validate_against(&transcript)?;

        let subtitle = SubtitleDocument::from_translation(&transcript, &translated)?;
        let output = self
            .exporter
            .export(&subtitle, request.output)
            .await
            .map_err(TranslationWorkflowError::Export)?;

        Ok(AudioTranslationResult {
            transcript,
            translated,
            subtitle,
            output,
        })
    }
}

fn build_context(transcript: &Transcript, policy: &ContextPolicy) -> TranslationContext {
    let to_context = |segment: &super::TranscriptSegment| ContextSegment {
        segment_id: segment.id,
        text: segment.text.clone(),
    };

    match policy {
        ContextPolicy::None => TranslationContext::default(),
        ContextPolicy::Document => TranslationContext {
            previous: transcript.segments.iter().map(to_context).collect(),
            next: Vec::new(),
            document_summary: None,
            style: None,
        },
        ContextPolicy::NeighboringSegments { before, after } => TranslationContext {
            previous: transcript
                .segments
                .iter()
                .take(*before)
                .map(to_context)
                .collect(),
            next: transcript
                .segments
                .iter()
                .rev()
                .take(*after)
                .rev()
                .map(to_context)
                .collect(),
            document_summary: None,
            style: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use uuid::Uuid;

    use super::*;
    use crate::core::{
        AsrError, AudioChunk, AudioFormat, AudioSamples, AudioTranslationService, ContextPolicy,
        JsonSubtitleExporter, LanguageTag, LineBreakPolicy, MemoryAudioInput, SampleFormat,
        SampleLayout, SubtitleExportRequest, SubtitleFormat, TextEncoding, TimeRange,
        TimestampLevel, TranscriptSegment, TranslatedSegment, TranslationConstraints,
        TranslationError, TranslationRequestTemplate, TranslationWarning,
    };

    struct TestAsr {
        /// 上下文或事件关联的文本片段唯一标识。
        segment_id: Uuid,
    }

    #[async_trait]
    impl AsrEngine for TestAsr {
        fn audio_requirements(&self) -> super::super::AudioFormatRequirement {
            super::super::AudioFormatRequirement {
                accepted_sample_rates_hz: vec![16_000],
                preferred_sample_rate_hz: Some(16_000),
                accepted_channels: vec![1],
                accepted_sample_formats: vec![SampleFormat::F32],
                accepted_layouts: vec![SampleLayout::Interleaved],
                requires_mono: true,
            }
        }

        async fn transcribe(
            &self,
            mut input: Box<dyn AudioInput>,
            request: AsrRequest,
        ) -> Result<Transcript, AsrError> {
            assert_eq!(request.timestamp_level, TimestampLevel::Segment);
            assert!(input.info().await.is_ok());
            assert!(input.next_chunk().await.unwrap().is_some());
            Ok(Transcript {
                language: Some(LanguageTag::from("en-US")),
                segments: vec![TranscriptSegment {
                    id: self.segment_id,
                    range: TimeRange::new(0, 10).unwrap(),
                    text: "hello".to_owned(),
                    speaker: None,
                    confidence: Some(0.99),
                    words: None,
                    revision: 0,
                }],
            })
        }
    }

    struct TestTranslator;

    #[async_trait]
    impl Translator for TestTranslator {
        async fn translate(
            &self,
            request: TranslationRequest,
        ) -> Result<TranslatedTranscript, TranslationError> {
            assert_eq!(request.target_language, LanguageTag::from("zh-CN"));
            assert_eq!(request.context.previous.len(), 1);
            Ok(TranslatedTranscript {
                source_language: request.source_language,
                target_language: request.target_language,
                segments: request
                    .segments
                    .into_iter()
                    .map(|segment| TranslatedSegment {
                        source_segment_id: segment.id,
                        range: segment.range,
                        source_text: segment.text,
                        translated_text: "你好".to_owned(),
                        speaker: segment.speaker,
                        warnings: vec![TranslationWarning::ProviderWarning("test".to_owned())],
                    })
                    .collect(),
            })
        }
    }

    #[test]
    fn core_service_composes_the_four_contracts() {
        let segment_id = Uuid::new_v4();
        let format = AudioFormat {
            sample_rate: 16_000,
            channels: 1,
            sample_format: SampleFormat::F32,
            layout: SampleLayout::Interleaved,
        };
        let chunk = AudioChunk {
            timestamp_ms: 0,
            duration_ms: 10,
            frames: 160,
            samples: AudioSamples::F32(vec![0.0; 160]),
            format: format.clone(),
            is_final: true,
        };
        let input = MemoryAudioInput::new(format, [chunk], false).unwrap();
        let service = CoreAudioTranslationService::new(
            TestAsr { segment_id },
            TestTranslator,
            JsonSubtitleExporter,
        );
        let request = AudioTranslationRequest {
            input: Box::new(input),
            asr: AsrRequest {
                source_language: Some(LanguageTag::from("en-US")),
                timestamp_level: TimestampLevel::Segment,
                enable_speaker_labels: false,
                vocabulary: None,
                options: Default::default(),
            },
            translation: TranslationRequestTemplate {
                target_language: LanguageTag::from("zh-CN"),
                context_policy: ContextPolicy::Document,
                glossary: None,
                constraints: TranslationConstraints::default(),
                options: Default::default(),
            },
            output: SubtitleExportRequest {
                format: SubtitleFormat::Json,
                encoding: TextEncoding::Utf8,
                line_policy: LineBreakPolicy::Preserve,
                include_speaker: false,
            },
        };

        let result = block_on(service.translate(request)).unwrap();
        assert_eq!(result.transcript.segments.len(), 1);
        assert_eq!(result.subtitle.cues[0].text, "你好");
        assert_eq!(result.output.suggested_name, "subtitles.json");
        assert!(
            std::str::from_utf8(&result.output.bytes)
                .unwrap()
                .contains("你好")
        );
    }
}
