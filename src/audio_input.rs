use async_trait::async_trait;

use crate::error::CoreError;

/// 统一的音频输入接口。
///
/// 该接口屏蔽本地文件、网络资源和其他媒体来源之间的差异，
/// 上层 ASR 只需要取得完整媒体字节，而不需要了解数据的具体来源。
///
/// `Send` 约束用于保证输入对象的所有权可以安全地移动到异步任务或其他线程；
/// 读取状态通过 `&mut self` 保持独占，避免多个读取者同时消费同一个输入。
#[async_trait]
pub trait AudioInput: Send {
    /// 读取完整的音频媒体字节。
    ///
    /// 返回媒体原始字节而不是预先转换的 PCM，使高层 ASR provider 可以自行使用其成熟的
    /// 文件解码流程；调用方不需要构造采样帧、采样值或音频特征。
    async fn read_all(&mut self) -> Result<Vec<u8>, CoreError>;

    /// 关闭输入并释放其持有的文件或网络资源。
    ///
    /// 单独提供关闭操作，是为了让调用方在正常结束、取消或发生错误时都能执行显式清理；
    /// 具体实现可以根据资源类型决定关闭动作，但不应再继续读取已关闭的输入。
    async fn close(&mut self) -> Result<(), CoreError>;
}
