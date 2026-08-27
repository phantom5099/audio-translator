use async_trait::async_trait;

use crate::error::CoreError;

/// 统一的音频输入接口。
#[async_trait]
pub trait AudioInput: Send {
    /// 读取完整的音频媒体字节。
    async fn read_all(&mut self) -> Result<Vec<u8>, CoreError>;

    /// 关闭输入并释放其持有的文件或网络资源。
    async fn close(&mut self) -> Result<(), CoreError>;
}
