//! File-Bash-Tools MCP Server
//! 
//! 企业级文件和Shell工具MCP服务器

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
};
use std::path::Path;
use anyhow::Result;
use tracing::{debug, info, error};

// 重新导出主服务
pub use FileBashToolsService as FileBashToolsServer;

/// Write工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct WriteRequest {
    /// 要写入的文件的绝对路径
    pub file_path: String,
    /// 要写入文件的内容
    pub content: String,
}

/// Read工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct ReadRequest {
    /// 要读取的文件的绝对路径
    pub file_path: String,
    /// 开始读取的行号
    #[serde(default = "default_offset")]
    pub offset: Option<i32>,
    /// 要读取的行数
    pub limit: Option<i32>,
}

/// Edit工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct EditRequest {
    /// 要修改的文件的绝对路径
    pub file_path: String,
    /// 要替换的文本
    pub old_string: String,
    /// 用于替换的新文本
    pub new_string: String,
    /// 替换所有匹配项
    #[serde(default)]
    pub replace_all: bool,
}

/// Bash工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct BashRequest {
    /// 要执行的命令
    pub command: String,
    /// 可选的超时时间(毫秒)
    pub timeout: Option<u64>,
    /// 清晰简洁的描述
    pub description: Option<String>,
    /// 设置为 true 以在后台运行
    #[serde(default)]
    pub run_in_background: bool,
}

fn default_offset() -> Option<i32> {
    Some(1)
}

/// File-Bash-Tools服务
#[derive(Debug, Clone)]
pub struct FileBashToolsService {
    tool_router: ToolRouter<FileBashToolsService>,
}

impl FileBashToolsService {
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// 验证文件路径
    fn validate_file_path(file_path: &str) -> Result<(), McpError> {
        if file_path.is_empty() {
            return Err(McpError::invalid_params("文件路径不能为空", None));
        }

        let path = Path::new(file_path);
        
        // 检查路径是否为绝对路径
        if !path.is_absolute() {
            return Err(McpError::invalid_params("文件路径必须是绝对路径", None));
        }

        // 基本安全检查
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(McpError::invalid_params("不允许使用相对路径中的'..'", None));
        }

        Ok(())
    }

    /// 确保目录存在
    async fn ensure_directory_exists(file_path: &str) -> Result<(), McpError> {
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| McpError::internal_error(format!("创建目录失败: {}", e), None))?;
            }
        }
        Ok(())
    }
}

#[tool_router]
impl FileBashToolsService {
    /// 写入文件内容
    #[tool(description = "Write content to a file")]
    async fn write(
        &self,
        Parameters(req): Parameters<WriteRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Write工具调用: file_path={}", req.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&req.file_path)?;
        
        // 确保目录存在
        Self::ensure_directory_exists(&req.file_path).await?;
        
        // 写入文件
        tokio::fs::write(&req.file_path, &req.content).await
            .map_err(|e| McpError::internal_error(format!("写入文件失败: {}", e), None))?;
        
        let bytes_written = req.content.len();
        info!("✅ 文件写入成功: {} ({} bytes)", req.file_path, bytes_written);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!("成功写入文件: {}，字节数: {}", req.file_path, bytes_written))
        ]))
    }

    /// 读取文件内容
    #[tool(description = "Read content from a file")]
    async fn read(
        &self,
        Parameters(req): Parameters<ReadRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Read工具调用: file_path={}", req.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&req.file_path)?;
        
        // 检查文件是否存在
        if !Path::new(&req.file_path).exists() {
            return Err(McpError::invalid_params(format!("文件不存在: {}", req.file_path), None));
        }
        
        // 读取文件内容
        let content = tokio::fs::read_to_string(&req.file_path).await
            .map_err(|e| McpError::internal_error(format!("读取文件失败: {}", e), None))?;
        
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // 处理偏移和限制
        let offset = req.offset.unwrap_or(1).max(1) as usize - 1;
        let limit = req.limit.unwrap_or(total_lines as i32) as usize;
        
        let end_index = (offset + limit).min(total_lines);
        let slice = if offset < total_lines {
            &lines[offset..end_index]
        } else {
            &[]
        };
        
        // 格式化输出
        let mut result_content = String::new();
        for (i, line) in slice.iter().enumerate() {
            let line_num = offset + i + 1;
            result_content.push_str(&format!("{}\t{}\n", line_num, line));
        }
        
        info!("📄 文件读取成功: {} ({} lines returned)", req.file_path, slice.len());
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "文件内容:\n{}\n总计行数: {}, 返回行数: {}", 
                result_content, total_lines, slice.len()
            ))
        ]))
    }

    /// 编辑文件内容
    #[tool(description = "Edit file content by replacing text")]
    async fn edit(
        &self,
        Parameters(req): Parameters<EditRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Edit工具调用: file_path={}", req.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&req.file_path)?;
        
        // 检查文件是否存在
        if !Path::new(&req.file_path).exists() {
            return Err(McpError::invalid_params(format!("文件不存在: {}", req.file_path), None));
        }
        
        // 读取原文件内容
        let original_content = tokio::fs::read_to_string(&req.file_path).await
            .map_err(|e| McpError::internal_error(format!("读取文件失败: {}", e), None))?;
        
        // 执行替换操作
        let new_content = if req.replace_all {
            original_content.replace(&req.old_string, &req.new_string)
        } else {
            original_content.replacen(&req.old_string, &req.new_string, 1)
        };
        
        // 检查是否有变化
        if new_content == original_content {
            info!("⚠️ 文件内容无变化: {}", req.file_path);
            return Ok(CallToolResult::success(vec![
                Content::text(format!("文件 '{}' 中未找到要替换的内容", req.file_path))
            ]));
        }
        
        // 写入新内容
        tokio::fs::write(&req.file_path, new_content).await
            .map_err(|e| McpError::internal_error(format!("写入文件失败: {}", e), None))?;
        
        // 计算替换次数
        let replacements = if req.replace_all {
            original_content.matches(&req.old_string).count()
        } else {
            if original_content.contains(&req.old_string) { 1 } else { 0 }
        };
        
        info!("✏️ 文件编辑成功: {} ({} replacements)", req.file_path, replacements);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!("成功编辑文件: {}，替换次数: {}", req.file_path, replacements))
        ]))
    }

    /// 执行PowerShell命令
    #[tool(description = "Execute PowerShell command")]
    async fn bash(
        &self,
        Parameters(req): Parameters<BashRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Bash工具调用: command={}", req.command);
        
        // 验证命令安全性
        if req.command.is_empty() {
            return Err(McpError::invalid_params("命令不能为空", None));
        }
        
        // 构建PowerShell命令
        let mut cmd = tokio::process::Command::new("pwsh.exe");
        cmd.arg("-NoProfile")
           .arg("-Command")
           .arg(&req.command);
        
        // 执行命令
        let output = cmd.output().await
            .map_err(|e| McpError::internal_error(format!("执行PowerShell命令失败: {}", e), None))?;
        
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("stdout:\n{}\nstderr:\n{}", stdout, stderr);
        
        if exit_code == 0 {
            info!("✅ Bash命令执行成功: exit_code={}", exit_code);
        } else {
            error!("⚠️ Bash命令执行失败: exit_code={}", exit_code);
        }
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "命令执行完成:\n退出码: {}\n输出:\n{}", 
                exit_code, combined_output
            ))
        ]))
    }
}

#[tool_handler]
impl ServerHandler for FileBashToolsService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("企业级文件和Shell工具MCP服务器。支持文件读写、编辑和PowerShell命令执行。".to_string()),
        }
    }
}