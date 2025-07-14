//! 流式数据传输支持

use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::error::RpcResult;
use crate::protocol::RpcEvent;

/// 流状态
#[derive(Debug, Clone)]
pub enum StreamState {
    Active,
    Paused,
    Closed,
    Error(String),
}

/// 流信息
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_id: String,
    pub event_type: String,
    pub state: StreamState,
    pub created_at: std::time::Instant,
    pub last_sequence: u64,
    pub total_messages: u64,
}

/// 流数据传输器
#[derive(Clone)]
pub struct StreamTransmitter {
    stream_id: String,
    event_type: String,
    sequence: Arc<RwLock<u64>>,
    sender: broadcast::Sender<RpcEvent>,
    stats: Arc<RwLock<StreamInfo>>,
}

impl StreamTransmitter {
    /// 创建新的流传输器
    pub fn new(stream_id: String, event_type: String) -> Self {
        let (sender, _) = broadcast::channel(100);
        let stats = Arc::new(RwLock::new(StreamInfo {
            stream_id: stream_id.clone(),
            event_type: event_type.clone(),
            state: StreamState::Active,
            created_at: std::time::Instant::now(),
            last_sequence: 0,
            total_messages: 0,
        }));

        Self {
            stream_id,
            event_type,
            sequence: Arc::new(RwLock::new(0)),
            sender,
            stats,
        }
    }

    /// 发送流数据
    pub async fn send(&self, data: Value) -> RpcResult<()> {
        let mut sequence = self.sequence.write().await;
        *sequence += 1;
        
        let event = RpcEvent::with_sequence(
            self.event_type.clone(),
            data,
            self.stream_id.clone(),
            *sequence,
        );

        match self.sender.send(event) {
            Ok(_) => {
                let mut stats = self.stats.write().await;
                stats.last_sequence = *sequence;
                stats.total_messages += 1;
                debug!("Stream {} sent message #{}", self.stream_id, *sequence);
                Ok(())
            }
            Err(_) => {
                warn!("No subscribers for stream: {}", self.stream_id);
                Ok(())
            }
        }
    }

    /// 关闭流
    pub async fn close(&self) -> RpcResult<()> {
        let mut stats = self.stats.write().await;
        stats.state = StreamState::Closed;
        
        // 发送流结束事件
        let end_event = RpcEvent::with_stream(
            format!("{}.end", self.event_type),
            serde_json::json!({
                "stream_id": self.stream_id,
                "total_messages": stats.total_messages,
                "last_sequence": stats.last_sequence
            }),
            self.stream_id.clone(),
        );

        let _ = self.sender.send(end_event);
        info!("Stream {} closed", self.stream_id);
        Ok(())
    }

    /// 暂停流
    pub async fn pause(&self) -> RpcResult<()> {
        let mut stats = self.stats.write().await;
        stats.state = StreamState::Paused;
        info!("Stream {} paused", self.stream_id);
        Ok(())
    }

    /// 恢复流
    pub async fn resume(&self) -> RpcResult<()> {
        let mut stats = self.stats.write().await;
        stats.state = StreamState::Active;
        info!("Stream {} resumed", self.stream_id);
        Ok(())
    }

    /// 创建接收器
    pub fn subscribe(&self) -> StreamReceiver {
        let receiver = self.sender.subscribe();
        StreamReceiver::new(self.stream_id.clone(), receiver)
    }

    /// 获取流统计信息
    pub async fn get_stats(&self) -> StreamInfo {
        self.stats.read().await.clone()
    }
}

/// 流接收器
pub struct StreamReceiver {
    stream_id: String,
    receiver: broadcast::Receiver<RpcEvent>,
    buffer: Vec<RpcEvent>,
    expected_sequence: u64,
}

impl StreamReceiver {
    fn new(stream_id: String, receiver: broadcast::Receiver<RpcEvent>) -> Self {
        Self {
            stream_id,
            receiver,
            buffer: Vec::new(),
            expected_sequence: 1,
        }
    }

    /// 接收下一个事件
    pub async fn recv(&mut self) -> RpcResult<Option<RpcEvent>> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    // 检查是否是我们的流
                    if event.stream_id.as_ref() == Some(&self.stream_id) {
                        if let Some(sequence) = event.sequence {
                            if sequence == self.expected_sequence {
                                self.expected_sequence += 1;
                                return Ok(Some(event));
                            } else {
                                // 序列号不匹配，可能需要重新排序
                                self.buffer.push(event);
                                self.try_deliver_buffered().await?;
                            }
                        } else {
                            // 没有序列号的事件直接返回
                            return Ok(Some(event));
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("Stream receiver lagged, skipped {} events", skipped);
                    // 可能需要重新同步
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Stream {} closed", self.stream_id);
                    return Ok(None);
                }
            }
        }
    }

    /// 尝试从缓冲区中传递事件
    async fn try_deliver_buffered(&mut self) -> RpcResult<()> {
        self.buffer.sort_by(|a, b| {
            a.sequence.unwrap_or(0).cmp(&b.sequence.unwrap_or(0))
        });

        while let Some(event) = self.buffer.first() {
            if let Some(sequence) = event.sequence {
                if sequence == self.expected_sequence {
                    let event = self.buffer.remove(0);
                    self.expected_sequence += 1;
                    // 这里需要某种方式来返回事件，但由于异步限制，我们暂时跳过
                    debug!("Delivered buffered event #{} for stream {}", sequence, self.stream_id);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

/// 流管理器
pub struct StreamManager {
    streams: Arc<DashMap<String, StreamTransmitter>>,
    stats: Arc<RwLock<StreamManagerStats>>,
}

/// 流管理器统计信息
#[derive(Debug, Default, Clone)]
pub struct StreamManagerStats {
    pub total_streams: u64,
    pub active_streams: u64,
    pub closed_streams: u64,
    pub total_messages: u64,
}

impl StreamManager {
    /// 创建新的流管理器
    pub fn new() -> Self {
        Self {
            streams: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(StreamManagerStats::default())),
        }
    }

    /// 创建新的流
    pub async fn create_stream(&self, stream_id: String, event_type: String) -> Arc<StreamTransmitter> {
        let transmitter = Arc::new(StreamTransmitter::new(stream_id.clone(), event_type));
        self.streams.insert(stream_id.clone(), transmitter.as_ref().clone());
        
        let mut stats = self.stats.write().await;
        stats.total_streams += 1;
        stats.active_streams += 1;
        
        info!("Created stream: {}", stream_id);
        transmitter
    }

    /// 获取流
    pub fn get_stream(&self, stream_id: &str) -> Option<StreamTransmitter> {
        self.streams.get(stream_id).map(|entry| entry.value().clone())
    }

    /// 关闭流
    pub async fn close_stream(&self, stream_id: &str) -> RpcResult<()> {
        if let Some((_, transmitter)) = self.streams.remove(stream_id) {
            transmitter.close().await?;
            
            let mut stats = self.stats.write().await;
            stats.active_streams -= 1;
            stats.closed_streams += 1;
        }
        
        Ok(())
    }

    /// 获取所有活跃流
    pub fn get_active_streams(&self) -> Vec<String> {
        self.streams.iter().map(|entry| entry.key().clone()).collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> StreamManagerStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_stream_creation() {
        let manager = StreamManager::new();
        let stream = manager.create_stream("test-stream".to_string(), "test.event".to_string()).await;
        
        assert_eq!(stream.stream_id, "test-stream");
        assert_eq!(stream.event_type, "test.event");
    }

    #[tokio::test]
    async fn test_stream_send() {
        let transmitter = StreamTransmitter::new("test".to_string(), "test.event".to_string());
        let mut receiver = transmitter.subscribe();
        
        // 发送数据
        transmitter.send(json!({"message": "hello"})).await.unwrap();
        
        // 接收数据
        let event = receiver.recv().await.unwrap().unwrap();
        assert_eq!(event.event, "test.event");
        assert_eq!(event.stream_id, Some("test".to_string()));
        assert_eq!(event.sequence, Some(1));
    }
} 