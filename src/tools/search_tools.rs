//! 搜索工具实现（Grep和Glob）

use crate::models::search::*;
use crate::utils::{ripgrep_utils::RipgrepWrapper, fd_utils::FdWrapper};
use crate::ServerError;
use rmcp::{tool, tool_router};
use std::path::Path;
use tracing::{debug, info};

/// 搜索工具处理器
#[derive(Clone)]
pub struct SearchTools {
    ripgrep: RipgrepWrapper,
    fd: FdWrapper,
}

impl SearchTools {
    /// 创建新的搜索工具实例
    pub fn new() -> Self {
        Self {
            ripgrep: RipgrepWrapper::new(),
            fd: FdWrapper::new(),
        }
    }

    /// 验证搜索路径
    fn validate_search_path(path: &Option<String>) -> Result<String, ServerError> {
        match path {
            Some(p) => {
                if p.is_empty() {
                    return Err(ServerError::FileSystem("搜索路径不能为空".to_string()));
                }
                if !Path::new(p).exists() {
                    return Err(ServerError::FileSystem(format!("搜索路径不存在: {}", p)));
                }
                Ok(p.clone())
            }
            None => {
                // 默认使用当前目录
                Ok(std::env::current_dir()
                    .map_err(|e| ServerError::FileSystem(format!("无法获取当前目录: {}", e)))?
                    .to_string_lossy()
                    .to_string())
            }
        }
    }
}

#[tool_router]
impl SearchTools {
    /// 使用ripgrep进行文本搜索
    #[tool(description = "Search files using ripgrep engine")]
    async fn grep(&self, input: GrepInput) -> Result<serde_json::Value, ServerError> {
        debug!("🔍 Grep工具调用: pattern={}, path={:?}", input.pattern, input.path);
        
        // 验证搜索模式
        if input.pattern.is_empty() {
            return Err(ServerError::FileSystem("搜索模式不能为空".to_string()));
        }
        
        // 验证搜索路径
        let search_path = Self::validate_search_path(&input.path)?;
        
        // 确定输出模式
        let output_mode = input.output_mode.as_deref().unwrap_or("content");
        
        match output_mode {
            "content" => {
                let matches = self.ripgrep.search_content(
                    &input.pattern,
                    &search_path,
                    input.glob.as_deref(),
                    input.type_field.as_deref(),
                    input.case_insensitive.unwrap_or(false),
                    input.show_line_numbers.unwrap_or(true),
                    input.before_context,
                    input.after_context,
                    input.context,
                    input.head_limit,
                    input.multiline.unwrap_or(false),
                ).await?;
                
                let total_matches = matches.len();
                
                info!("✅ Grep搜索完成: pattern={}, total_matches={}", input.pattern, total_matches);
                
                let output = GrepContentOutput {
                    matches,
                    total_matches: total_matches as u64,
                };
                
                Ok(serde_json::to_value(output)?)
            }
            "files_with_matches" => {
                let files = self.ripgrep.search_files(
                    &input.pattern,
                    &search_path,
                    input.glob.as_deref(),
                    input.type_field.as_deref(),
                    input.case_insensitive.unwrap_or(false),
                ).await?;
                
                let count = files.len();
                
                info!("✅ Grep文件搜索完成: pattern={}, files_found={}", input.pattern, count);
                
                let output = GrepFilesOutput {
                    files,
                    count: count as u64,
                };
                
                Ok(serde_json::to_value(output)?)
            }
            "count" => {
                let files = self.ripgrep.search_files(
                    &input.pattern,
                    &search_path,
                    input.glob.as_deref(),
                    input.type_field.as_deref(),
                    input.case_insensitive.unwrap_or(false),
                ).await?;
                
                let count = files.len();
                
                info!("✅ Grep计数完成: pattern={}, count={}", input.pattern, count);
                
                let output = GrepFilesOutput {
                    files: vec![],
                    count: count as u64,
                };
                
                Ok(serde_json::to_value(output)?)
            }
            _ => {
                Err(ServerError::FileSystem(format!("不支持的输出模式: {}", output_mode)))
            }
        }
    }

    /// 使用fd进行文件模式匹配
    #[tool(description = "Find files using glob patterns")]
    async fn glob(&self, input: GlobInput) -> Result<GlobOutput, ServerError> {
        debug!("🔍 Glob工具调用: pattern={}, path={:?}", input.pattern, input.path);
        
        // 验证搜索模式
        if input.pattern.is_empty() {
            return Err(ServerError::FileSystem("Glob模式不能为空".to_string()));
        }
        
        // 验证搜索路径
        let search_path = Self::validate_search_path(&input.path)?;
        
        // 执行文件搜索
        let matches = self.fd.find_files(
            &input.pattern,
            &search_path,
        ).await?;
        
        let count = matches.len();
        
        info!("✅ Glob搜索完成: pattern={}, matches_found={}", input.pattern, count);
        
        Ok(GlobOutput {
            matches,
            count: count as u64,
            search_path,
        })
    }
}