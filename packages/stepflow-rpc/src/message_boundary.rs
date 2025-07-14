//! 消息边界处理 - 防止TCP粘包问题

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{debug, error, warn};

use crate::error::{RpcError, RpcResult};

/// 消息边界类型
#[derive(Debug, Clone)]
pub enum MessageBoundary {
    /// 换行符分隔（简单模式）
    Newline,
    /// 长度前缀（4字节大端序）
    LengthPrefix,
    /// 自定义分隔符
    CustomDelimiter(String),
}

impl Default for MessageBoundary {
    fn default() -> Self {
        MessageBoundary::Newline
    }
}

/// 消息读取器
pub struct MessageReader {
    reader: BufReader<TcpStream>,
    boundary: MessageBoundary,
    buffer: Vec<u8>,
    max_message_size: usize,
}

impl MessageReader {
    /// 创建新的消息读取器
    pub fn new(stream: TcpStream, boundary: MessageBoundary) -> Self {
        Self {
            reader: BufReader::new(stream),
            boundary,
            buffer: Vec::new(),
            max_message_size: 1024 * 1024, // 1MB 默认最大消息大小
        }
    }

    /// 设置最大消息大小
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// 读取下一条消息
    pub async fn read_message(&mut self) -> RpcResult<Option<String>> {
        match &self.boundary {
            MessageBoundary::Newline => self.read_newline_message().await,
            MessageBoundary::LengthPrefix => self.read_length_prefix_message().await,
            MessageBoundary::CustomDelimiter(delimiter) => self.read_custom_delimiter_message(delimiter).await,
        }
    }

    /// 读取换行符分隔的消息
    async fn read_newline_message(&mut self) -> RpcResult<Option<String>> {
        let mut line = String::new();
        
        match self.reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("Connection closed by peer");
                Ok(None)
            }
            Ok(n) => {
                if n > self.max_message_size {
                    return Err(RpcError::internal_error("Message too large"));
                }
                
                line = line.trim_end().to_string();
                if line.is_empty() {
                    // 跳过空行
                    return self.read_newline_message().await;
                }
                
                debug!("Received message: {} bytes", line.len());
                Ok(Some(line))
            }
            Err(e) => {
                error!("Error reading message: {}", e);
                Err(RpcError::internal_error("Read error"))
            }
        }
    }

    /// 读取长度前缀的消息
    async fn read_length_prefix_message(&mut self) -> RpcResult<Option<String>> {
        // 读取长度前缀（4字节大端序）
        let mut length_bytes = [0u8; 4];
        match self.reader.read_exact(&mut length_bytes).await {
            Ok(_) => {
                let message_length = u32::from_be_bytes(length_bytes) as usize;
                
                if message_length > self.max_message_size {
                    return Err(RpcError::internal_error("Message too large"));
                }
                
                if message_length == 0 {
                    return Ok(Some(String::new()));
                }
                
                // 读取消息内容
                let mut message_bytes = vec![0u8; message_length];
                match self.reader.read_exact(&mut message_bytes).await {
                    Ok(_) => {
                        let message = String::from_utf8(message_bytes)
                            .map_err(|_| RpcError::internal_error("Invalid UTF-8"))?;
                        
                        debug!("Received message: {} bytes", message_length);
                        Ok(Some(message))
                    }
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        debug!("Connection closed by peer");
                        Ok(None)
                    }
                    Err(e) => {
                        error!("Error reading message body: {}", e);
                        Err(RpcError::internal_error("Read error"))
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                debug!("Connection closed by peer");
                Ok(None)
            }
            Err(e) => {
                error!("Error reading length prefix: {}", e);
                Err(RpcError::internal_error("Read error"))
            }
        }
    }

    /// 读取自定义分隔符的消息
    async fn read_custom_delimiter_message(&mut self, delimiter: &str) -> RpcResult<Option<String>> {
        let delimiter_bytes = delimiter.as_bytes();
        self.buffer.clear();
        
        loop {
            let mut byte = [0u8; 1];
            match self.reader.read_exact(&mut byte).await {
                Ok(_) => {
                    self.buffer.push(byte[0]);
                    
                    if self.buffer.len() > self.max_message_size {
                        return Err(RpcError::internal_error("Message too large"));
                    }
                    
                    // 检查是否匹配分隔符
                    if self.buffer.len() >= delimiter_bytes.len() {
                        let end_pos = self.buffer.len() - delimiter_bytes.len();
                        if &self.buffer[end_pos..] == delimiter_bytes {
                            // 移除分隔符
                            self.buffer.truncate(end_pos);
                            
                            let message = String::from_utf8(self.buffer.clone())
                                .map_err(|_| RpcError::internal_error("Invalid UTF-8"))?;
                            
                            debug!("Received message: {} bytes", message.len());
                            return Ok(Some(message));
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    debug!("Connection closed by peer");
                    return Ok(None);
                }
                Err(e) => {
                    error!("Error reading message: {}", e);
                    return Err(RpcError::internal_error("Read error"));
                }
            }
        }
    }
}

/// 消息写入器
pub struct MessageWriter {
    writer: TcpStream,
    boundary: MessageBoundary,
    max_message_size: usize,
}

impl MessageWriter {
    /// 创建新的消息写入器
    pub fn new(stream: TcpStream, boundary: MessageBoundary) -> Self {
        Self {
            writer: stream,
            boundary,
            max_message_size: 1024 * 1024, // 1MB 默认最大消息大小
        }
    }

    /// 设置最大消息大小
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// 写入消息
    pub async fn write_message(&mut self, message: &str) -> RpcResult<()> {
        if message.len() > self.max_message_size {
            return Err(RpcError::internal_error("Message too large"));
        }

        match &self.boundary {
            MessageBoundary::Newline => self.write_newline_message(message).await,
            MessageBoundary::LengthPrefix => self.write_length_prefix_message(message).await,
            MessageBoundary::CustomDelimiter(delimiter) => self.write_custom_delimiter_message(message, delimiter).await,
        }
    }

    /// 写入换行符分隔的消息
    async fn write_newline_message(&mut self, message: &str) -> RpcResult<()> {
        let message_with_newline = format!("{}\n", message);
        
        self.writer.write_all(message_with_newline.as_bytes()).await
            .map_err(|_| RpcError::internal_error("Write error"))?;
        
        self.writer.flush().await
            .map_err(|_| RpcError::internal_error("Flush error"))?;
        
        debug!("Sent message: {} bytes", message.len());
        Ok(())
    }

    /// 写入长度前缀的消息
    async fn write_length_prefix_message(&mut self, message: &str) -> RpcResult<()> {
        let message_bytes = message.as_bytes();
        let length = message_bytes.len() as u32;
        let length_bytes = length.to_be_bytes();
        
        // 写入长度前缀
        self.writer.write_all(&length_bytes).await
            .map_err(|_| RpcError::internal_error("Write error"))?;
        
        // 写入消息内容
        self.writer.write_all(message_bytes).await
            .map_err(|_| RpcError::internal_error("Write error"))?;
        
        self.writer.flush().await
            .map_err(|_| RpcError::internal_error("Flush error"))?;
        
        debug!("Sent message: {} bytes", message.len());
        Ok(())
    }

    /// 写入自定义分隔符的消息
    async fn write_custom_delimiter_message(&mut self, message: &str, delimiter: &str) -> RpcResult<()> {
        let message_with_delimiter = format!("{}{}", message, delimiter);
        
        self.writer.write_all(message_with_delimiter.as_bytes()).await
            .map_err(|_| RpcError::internal_error("Write error"))?;
        
        self.writer.flush().await
            .map_err(|_| RpcError::internal_error("Flush error"))?;
        
        debug!("Sent message: {} bytes", message.len());
        Ok(())
    }
}

/// 消息边界处理器
pub struct MessageBoundaryHandler {
    boundary: MessageBoundary,
    max_message_size: usize,
}

impl MessageBoundaryHandler {
    /// 创建新的消息边界处理器
    pub fn new(boundary: MessageBoundary) -> Self {
        Self {
            boundary,
            max_message_size: 1024 * 1024, // 1MB
        }
    }

    /// 设置最大消息大小
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// 创建消息读取器
    pub fn create_reader(&self, stream: TcpStream) -> MessageReader {
        MessageReader::new(stream, self.boundary.clone())
            .with_max_message_size(self.max_message_size)
    }

    /// 创建消息写入器
    pub fn create_writer(&self, stream: TcpStream) -> MessageWriter {
        MessageWriter::new(stream, self.boundary.clone())
            .with_max_message_size(self.max_message_size)
    }

    /// 获取边界类型
    pub fn boundary(&self) -> &MessageBoundary {
        &self.boundary
    }
}

impl Default for MessageBoundaryHandler {
    fn default() -> Self {
        Self::new(MessageBoundary::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_message_boundary_handler() {
        let handler = MessageBoundaryHandler::new(MessageBoundary::Newline);
        assert!(matches!(handler.boundary(), MessageBoundary::Newline));
    }

    #[tokio::test]
    async fn test_newline_boundary() {
        let handler = MessageBoundaryHandler::new(MessageBoundary::Newline);
        assert!(matches!(handler.boundary(), MessageBoundary::Newline));
    }

    #[tokio::test]
    async fn test_length_prefix_boundary() {
        let handler = MessageBoundaryHandler::new(MessageBoundary::LengthPrefix);
        assert!(matches!(handler.boundary(), MessageBoundary::LengthPrefix));
    }

    #[tokio::test]
    async fn test_custom_delimiter_boundary() {
        let handler = MessageBoundaryHandler::new(MessageBoundary::CustomDelimiter("||".to_string()));
        if let MessageBoundary::CustomDelimiter(delimiter) = handler.boundary() {
            assert_eq!(delimiter, "||");
        } else {
            panic!("Expected CustomDelimiter");
        }
    }
} 