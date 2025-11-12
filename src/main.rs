//! File-Bash-Tools MCP Server 主程序

use anyhow::Result;
use file_bash_tools_mcp::FileBashToolsService;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("🚀 启动 File-Bash-Tools MCP Server v0.1.0");
    tracing::info!("📁 支持文件操作: Write, Read, Edit");
    tracing::info!("💻 支持Shell工具: Bash (PowerShell)");

    // 创建服务实例
    let service = FileBashToolsService::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("服务启动错误: {:?}", e);
    })?;

    tracing::info!("✅ MCP服务器已启动，等待连接...");
    
    service.waiting().await?;
    Ok(())
}