use async_trait::async_trait;

use crate::{AudioChunk, AudioInfo, CoreError};

/// 统一的音频输入接口。
///
/// 该接口屏蔽本地文件、网络资源、麦克风和其他实时来源之间的差异，
/// 上层 ASR 只需要按照音频块顺序读取数据，而不需要了解数据的具体来源。
///
/// `Send` 约束用于保证输入对象的所有权可以安全地移动到异步任务或其他线程；
/// 读取状态通过 `&mut self` 保持独占，避免多个读取者同时推进同一个输入游标。
#[async_trait]
pub trait AudioInput: Send {
    /// 获取输入的格式、时长和实时性等元数据。
    ///
    /// 使用不可变引用是因为查询元数据不应推进输入位置；返回 `AudioInfo` 而不是
    /// 将格式拆成多个参数，是为了让未来增加声道布局、编码要求等元数据时不改变接口。
    async fn info(&self) -> Result<AudioInfo, CoreError>;

    /// 按输入时间轴读取下一个音频块。
    ///
    /// `&mut self` 用于推进文件游标、网络缓冲区或实时流状态；返回 `Option` 是为了
    /// 同时表达“读取到一个块”和“输入已经结束”，而不是把正常结束当作错误。
    /// 音频块中携带时间戳和格式，便于 ASR、重采样器和字幕时间轴保持一致。
    async fn next_chunk(&mut self) -> Result<Option<AudioChunk>, CoreError>;

    /// 关闭输入并释放其持有的文件、网络或设备资源。
    ///
    /// 单独提供关闭操作，是为了让调用方在正常结束、取消或发生错误时都能执行显式清理；
    /// 具体实现可以根据资源类型决定关闭动作，但不应再继续读取已关闭的输入。
    async fn close(&mut self) -> Result<(), CoreError>;
}
