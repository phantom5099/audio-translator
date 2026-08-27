use async_trait::async_trait;

use crate::{
    AsrError, AsrRequest, AsrVocabulary, AudioChunk, AudioFormatRequirement, AudioInput,
    LanguageTag, ProviderOptions, SegmentId, TimeRange, TimestampLevel, Transcript,
    TranscriptSegment,
};

/// 非流式 ASR 引擎接口。
///
/// 负责把统一的 `AudioInput` 转换为带时间轴的源语言 `Transcript`。
/// 具体实现可以调用云端 API、本地传统识别器或本地神经网络模型，核心流程不依赖 provider。
///
/// `Send + Sync` 允许同一个引擎实例被异步任务安全地移动和通过共享引用调用；
/// 这样连接池、模型实例或请求客户端可以由上层服务复用。
#[async_trait]
pub trait AsrEngine: Send + Sync {
    /// 声明该 ASR provider 能接受或偏好的音频格式。
    ///
    /// 单独暴露格式要求，是为了让下载、解码和重采样适配发生在进入 provider 之前，
    /// 而不是让每一个 ASR 实现重复处理格式协商逻辑。
    fn audio_requirements(&self) -> AudioFormatRequirement;

    /// 将完整音频输入识别为带时间戳的转录结果。
    ///
    /// `input` 使用所有权传入，因为识别过程通常会持续消耗输入流，并且可能跨越多个
    /// 异步调度点；使用 `Box<dyn AudioInput>` 使调用方可以在运行时选择文件、网络或实时输入。
    /// `request` 单独承载语言、时间戳、说话人和 provider 扩展参数，避免把配置耦合进音频数据。
    async fn transcribe(
        &self,
        input: Box<dyn AudioInput>,
        request: AsrRequest,
    ) -> Result<Transcript, AsrError>;
}

/// 支持实时流式识别的 ASR 引擎接口。
///
/// 它在 `AsrEngine` 的基础上增加会话能力：先创建一个会话，再持续推送音频块，
/// 从而支持实时字幕和增量转录，而不要求等待整个音频输入结束。
#[async_trait]
pub trait StreamingAsrEngine: AsrEngine {
    /// 创建一个新的流式识别会话。
    ///
    /// 请求参数在建会话时固定，后续 `push` 只负责传输音频数据；这样可以避免每个音频块
    /// 重复携带相同配置，也方便 provider 在会话建立阶段完成鉴权、模型选择和格式协商。
    async fn open_session(
        &self,
        request: StreamingAsrRequest,
    ) -> Result<Box<dyn AsrSession>, AsrError>;
}

/// 创建流式 ASR 会话时使用的配置。
///
/// 将会话配置与 `AudioChunk` 分离，可以让同一份配置作用于整个会话，
/// 同时允许每个音频块只携带数据、格式和时间信息。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamingAsrRequest {
    /// 源文本使用的语言；为空时由 provider 自动检测或按其默认策略处理。
    pub source_language: Option<LanguageTag>,
    /// 需要生成的时间戳粒度，用于在识别精度和实时处理成本之间做选择。
    pub timestamp_level: TimestampLevel,
    /// 是否要求识别并返回说话人标签；独立成开关以避免未需要时承担额外成本。
    pub enable_speaker_labels: bool,
    /// 可选的 ASR 词汇或热词提示，用于人名、术语和产品名等领域词汇。
    pub vocabulary: Option<AsrVocabulary>,
    /// provider 专属的扩展配置；保留开放键值结构，避免核心接口被单一 provider 的参数绑死。
    pub options: ProviderOptions,
}

/// 一个已经建立的流式 ASR 会话。
///
/// 会话对象保存 provider 连接、缓冲区和增量识别状态，因此只允许通过可变引用
/// 按顺序推进；返回事件而不是单个文本，能够同时表达临时结果、最终结果和分段边界。
#[async_trait]
pub trait AsrSession: Send {
    /// 推送一个音频块并获取当前可用的增量识别事件。
    ///
    /// `AudioChunk` 而不是裸 PCM 字节作为参数，是为了同时传递时间戳、时长、帧数和格式，
    /// 避免流式 provider 重新猜测音频边界；一次返回多个事件则能覆盖一个块触发多个结果的情况。
    async fn push(&mut self, chunk: AudioChunk) -> Result<Vec<AsrEvent>, AsrError>;

    /// 标记输入结束并冲刷 provider 内部缓冲区中的剩余识别结果。
    ///
    /// 单独的 `finish` 与 `close` 区分“业务输入结束”和“释放会话资源”：前者应尽量返回
    /// 尚未提交的尾部文本，后者用于取消或清理连接。
    async fn finish(&mut self) -> Result<Vec<AsrEvent>, AsrError>;

    /// 关闭流式会话并释放连接、缓冲区和 provider 资源。
    async fn close(&mut self) -> Result<(), AsrError>;
}

/// 流式 ASR 产生的增量事件。
///
/// 使用枚举区分临时文本、稳定提交和时间边界，调用方可以据此更新实时字幕，
/// 而不必通过特殊字符串或可选字段猜测事件含义。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AsrEvent {
    /// ASR 返回尚未最终确认的临时片段，后续事件可能修正其内容。
    Partial {
        /// 尚未最终确认的转录片段。
        segment: TranscriptSegment,
    },
    /// ASR 确认并提交一个稳定片段，调用方可以将其作为最终结果保存。
    Commit {
        /// 已经最终确认并提交的转录片段。
        segment: TranscriptSegment,
    },
    /// ASR 判定一个片段已经结束，用于驱动实时翻译或字幕换行等下游动作。
    Endpoint {
        /// 上下文或事件关联的文本片段唯一标识。
        segment_id: SegmentId,
        /// 该文本片段对应的音频时间范围。
        range: TimeRange,
    },
}
