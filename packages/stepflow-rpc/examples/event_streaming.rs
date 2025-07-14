//! 事件推送和流式传输示例

use std::time::Duration;
use serde_json::json;
use tokio::time::{sleep, interval};
use tracing::{info, Level};

use stepflow_rpc::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // 启动服务器
    let server_handle = tokio::spawn(async move {
        run_server().await
    });
    
    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;
    
    // 启动客户端
    let client_handle = tokio::spawn(async move {
        run_client().await
    });
    
    // 启动流式数据发送器
    let stream_handle = tokio::spawn(async move {
        run_stream_sender().await
    });
    
    // 等待所有任务完成
    let _ = tokio::join!(server_handle, client_handle, stream_handle);
    
    Ok(())
}

async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ServerConfig::default();
    let server = std::sync::Arc::new(RpcServer::new(config));
    
    // 注册一个示例方法
    let hello_handler = FunctionHandler::new(
        "hello".to_string(),
        |params| {
            Box::pin(async move {
                let name = if let Some(params) = params {
                    params.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("World")
                        .to_string()
                } else {
                    "World".to_string()
                };
                
                Ok(json!({
                    "message": format!("Hello, {}!", name),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            })
        }
    );
    server.register_handler(std::sync::Arc::new(hello_handler));
    
    // 启动事件发布器
    let event_publisher = server.event_publisher().clone();
    let publisher_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(2));
        let mut counter = 0;
        
        loop {
            interval.tick().await;
            counter += 1;
            
            // 发布系统事件
            let _ = event_publisher.publish_simple(
                "system.heartbeat",
                json!({
                    "counter": counter,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "server_status": "healthy"
                })
            ).await;
            
            // 发布用户事件
            if counter % 3 == 0 {
                let _ = event_publisher.publish_simple(
                    "user.activity",
                    json!({
                        "user_id": format!("user_{}", counter % 10),
                        "activity": "login",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                ).await;
            }
            
            info!("Published events #{}", counter);
        }
    });
    
    // 启动流管理器
    let stream_manager = StreamManager::new();
    let data_stream = stream_manager.create_stream(
        "data-stream-1".to_string(),
        "data.realtime".to_string()
    ).await;
    
    let stream_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(500));
        let mut seq = 0;
        
        loop {
            interval.tick().await;
            seq += 1;
            
            let _ = data_stream.send(json!({
                "sequence": seq,
                "value": seq * 10,
                "random": rand::random::<f64>(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })).await;
            
            if seq >= 20 {
                let _ = data_stream.close().await;
                break;
            }
        }
    });
    
    // 启动服务器
    tokio::select! {
        _ = server.serve() => {},
        _ = publisher_handle => {},
        _ = stream_handle => {},
    }
    
    Ok(())
}

async fn run_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let client = RpcClient::new(addr, ClientConfig::default());
    client.connect().await?;
    
    // 调用 RPC 方法
    let response = client.send_request("hello", json!({"name": "Alice"})).await?;
    info!("RPC Response: {}", response);
    
    // 服务发现
    let response = client.send_request("rpc.discover", serde_json::Value::Null).await?;
    let methods: Vec<String> = serde_json::from_value(response)?;
    info!("Available methods: {:?}", methods);
    
    // 健康检查
    let response = client.send_request("rpc.ping", serde_json::Value::Null).await?;
    let is_healthy: bool = serde_json::from_value(response)?;
    info!("Server health: {}", is_healthy);
    
    // 模拟客户端事件处理
    let event_handler = SimpleEventHandler::new(
        vec!["system.*".to_string(), "user.*".to_string()],
        |event| {
            let event_name = event.event.clone();
            let event_data = event.data.clone();
            Box::pin(async move {
                info!("Received event: {} - {:?}", event_name, event_data);
            })
        }
    );
    
    // 注意：这里只是示例，实际客户端事件处理需要更复杂的实现
    info!("Event handler created for patterns: {:?}", event_handler.interested_events());
    
    // 等待一段时间来观察事件
    sleep(Duration::from_secs(10)).await;
    
    client.disconnect().await;
    info!("Client disconnected");
    
    Ok(())
}

async fn run_stream_sender() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 等待服务器启动
    sleep(Duration::from_millis(200)).await;
    
    let stream_manager = StreamManager::new();
    
    // 创建多个流
    let sensor_stream = stream_manager.create_stream(
        "sensor-data".to_string(),
        "sensor.temperature".to_string()
    ).await;
    
    let log_stream = stream_manager.create_stream(
        "application-logs".to_string(),
        "log.application".to_string()
    ).await;
    
    // 发送传感器数据
    let sensor_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(1000));
        let mut temp = 20.0;
        
        for i in 1..=15 {
            interval.tick().await;
            temp += (rand::random::<f64>() - 0.5) * 2.0; // 模拟温度变化
            
            let _ = sensor_stream.send(json!({
                "sensor_id": "temp_001",
                "temperature": temp,
                "humidity": 45.0 + (rand::random::<f64>() - 0.5) * 10.0,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "sequence": i
            })).await;
        }
        
        let _ = sensor_stream.close().await;
    });
    
    // 发送日志数据
    let log_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(800));
        let log_levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        
        for i in 1..=12 {
            interval.tick().await;
            let level = log_levels[i % log_levels.len()];
            
            let _ = log_stream.send(json!({
                "level": level,
                "message": format!("Application event #{}", i),
                "module": "example",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "sequence": i
            })).await;
        }
        
        let _ = log_stream.close().await;
    });
    
    // 等待所有流完成
    let _ = tokio::join!(sensor_handle, log_handle);
    
    // 显示统计信息
    let stats = stream_manager.get_stats().await;
    info!("Stream Manager Stats: {:?}", stats);
    
    Ok(())
}

// 简单的随机数生成器模拟
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub fn random<T>() -> T 
    where 
        T: From<f64>
    {
        let mut hasher = DefaultHasher::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        now.as_nanos().hash(&mut hasher);
        
        let hash = hasher.finish();
        let normalized = (hash as f64) / (u64::MAX as f64);
        T::from(normalized)
    }
} 