use async_trait::async_trait;

use crate::{ExportError, SubtitleDocument, SubtitleExportRequest, SubtitleOutput};

/// 字幕导出接口。
///
/// 负责把内部统一的 `SubtitleDocument` 序列化为 SRT、WebVTT、ASS、TTML 或其他格式。
/// 导出器与内部字幕模型分离，使新增字幕格式时不需要修改 ASR 和翻译流程。
///
/// `document` 使用借用避免导出时复制或消耗完整字幕；`request` 使用值传递，
/// 因为导出器可以取得格式、编码和排版策略的所有权并在异步过程中使用。
#[async_trait]
pub trait SubtitleExporter: Send + Sync {
    /// 将统一字幕文档导出为指定格式的字节结果。
    async fn export(
        &self,
        document: &SubtitleDocument,
        request: SubtitleExportRequest,
    ) -> Result<SubtitleOutput, ExportError>;
}
