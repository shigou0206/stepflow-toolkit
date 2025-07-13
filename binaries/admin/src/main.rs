//! Stepflow Admin - Main Entry
//!
//! 这是 Stepflow Tool System 的管理工具入口。

use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "stepflow-admin")]
#[command(about = "Stepflow Tool System Admin Tool")]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    /// 日志级别
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(&cli.log_level)
        .init();
    info!("Stepflow Admin 启动");
    // TODO: 管理工具逻辑
    Ok(())
} 