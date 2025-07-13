//! Stepflow CLI - Main Entry
//!
//! 这是 Stepflow Tool System 的命令行工具入口。

use anyhow::Result;
use clap::Parser;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "stepflow-cli")]
#[command(about = "Stepflow Tool System CLI Tool")]
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
    info!("Stepflow CLI 启动");
    // TODO: CLI 逻辑
    Ok(())
} 