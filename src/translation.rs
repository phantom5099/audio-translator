use async_trait::async_trait;

use crate::{TranslatedTranscript, TranslationError, TranslationRequest};

/// 文本翻译 provider 接口。
///
/// 接收带时间轴的源语言片段，并返回保持片段关联关系的目标语言结果；
/// 这样翻译层只负责语言转换，不负责重新切分音频或猜测字幕时间。
///
/// `Send + Sync` 允许翻译引擎实例被多个工作任务复用，适合复用 HTTP 客户端、连接池或本地模型。
#[async_trait]
pub trait Translator: Send + Sync {
    /// 翻译一批源语言片段。
    ///
    /// 批量请求而不是逐句调用，是为了让 provider 利用上下文和批处理能力；
    /// `TranslationRequest` 同时携带源文本、目标语言、上下文、词典和约束，
    /// 使不同翻译策略可以共享同一稳定接口。
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslatedTranscript, TranslationError>;
}
