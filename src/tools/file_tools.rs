//! 文件操作工具实现

use crate::models::file_ops::*;
use crate::ServerError;
use rmcp::{tool, tool_router};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

/// 文件工具处理器
#[derive(Clone)]
pub struct FileTools {
    // 工具状态可以在这里添加
}

impl FileTools {
    /// 创建新的文件工具实例
    pub fn new() -> Self {
        Self {}
    }

    /// 确保目录存在
    async fn ensure_directory_exists(file_path: &str) -> Result<(), ServerError> {
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                info!("📁 创建目录: {}", parent.display());
            }
        }
        Ok(())
    }

    /// 验证文件路径
    fn validate_file_path(file_path: &str) -> Result<(), ServerError> {
        if file_path.is_empty() {
            return Err(ServerError::FileSystem("文件路径不能为空".to_string()));
        }

        let path = Path::new(file_path);
        
        // 检查路径是否为绝对路径
        if !path.is_absolute() {
            return Err(ServerError::FileSystem("文件路径必须是绝对路径".to_string()));
        }

        // 基本安全检查
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(ServerError::FileSystem("不允许使用相对路径中的'..'".to_string()));
        }

        Ok(())
    }
}

#[tool_router]
impl FileTools {
    /// 写入文件内容
    #[tool(description = "Write content to a file")]
    async fn write(&self, input: WriteInput) -> Result<WriteOutput, ServerError> {
        debug!("📝 Write工具调用: file_path={}", input.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&input.file_path)?;
        
        // 确保目录存在
        Self::ensure_directory_exists(&input.file_path).await?;
        
        // 写入文件
        let bytes_written = fs::write(&input.file_path, &input.content)?;
        
        info!("✅ 文件写入成功: {} ({} bytes)", input.file_path, bytes_written);
        
        Ok(WriteOutput {
            message: format!("成功写入文件: {}", input.file_path),
            bytes_written: bytes_written as u64,
            file_path: input.file_path,
        })
    }

    /// 读取文件内容
    #[tool(description = "Read content from a file")]
    async fn read(&self, input: ReadInput) -> Result<rmcp::tool::Result, ServerError> {
        debug!("📖 Read工具调用: file_path={}", input.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&input.file_path)?;
        
        // 检查文件是否存在
        if !Path::new(&input.file_path).exists() {
            return Err(ServerError::FileSystem(format!("文件不存在: {}", input.file_path)));
        }
        
        // 读取文件内容
        let content = fs::read_to_string(&input.file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // 处理偏移和限制
        let offset = input.offset.unwrap_or(1).max(1) as usize - 1;
        let limit = input.limit.unwrap_or(total_lines as i32) as usize;
        
        let end_index = (offset + limit).min(total_lines);
        let slice = if offset < total_lines {
            &lines[offset..end_index]
        } else {
            &[]
        };
        
        // 检查是否为图像文件
        let path = Path::new(&input.file_path);
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"].contains(&ext.as_str()) {
                // 处理图像文件
                let image_data = fs::read(&input.file_path)?;
                let mime_type = match ext.as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "bmp" => "image/bmp",
                    "webp" => "image/webp",
                    "svg" => "image/svg+xml",
                    _ => "application/octet-stream",
                };
                
                let base64_data = base64::encode(&image_data);
                
                info!("🖼️ 图像文件读取成功: {} ({} bytes)", input.file_path, image_data.len());
                
                let output = ReadImageOutput {
                    image: base64_data,
                    mime_type: mime_type.to_string(),
                    file_size: image_data.len() as u64,
                };
                
                return Ok(rmcp::tool::Result::Image(serde_json::to_value(output)?));
            }
        }
        
        // 处理文本文件
        let mut result_content = String::new();
        for (i, line) in slice.iter().enumerate() {
            let line_num = offset + i + 1;
            result_content.push_str(&format!("{}\t{}\n", line_num, line));
        }
        
        info!("📄 文本文件读取成功: {} ({} lines returned)", input.file_path, slice.len());
        
        let output = ReadTextOutput {
            content: result_content,
            total_lines: total_lines as u64,
            lines_returned: slice.len() as u64,
        };
        
        Ok(rmcp::tool::Result::Text(serde_json::to_value(output)?))
    }

    /// 编辑文件内容
    #[tool(description = "Edit file content by replacing text")]
    async fn edit(&self, input: EditInput) -> Result<EditOutput, ServerError> {
        debug!("✏️ Edit工具调用: file_path={}", input.file_path);
        
        // 验证文件路径
        Self::validate_file_path(&input.file_path)?;
        
        // 检查文件是否存在
        if !Path::new(&input.file_path).exists() {
            return Err(ServerError::FileSystem(format!("文件不存在: {}", input.file_path)));
        }
        
        // 读取原文件内容
        let original_content = fs::read_to_string(&input.file_path)?;
        
        // 执行替换操作
        let replace_all = input.replace_all.unwrap_or(false);
        let new_content = if replace_all {
            original_content.replace(&input.old_string, &input.new_string)
        } else {
            original_content.replacen(&input.old_string, &input.new_string, 1)
        };
        
        // 检查是否有变化
        if new_content == original_content {
            info!("⚠️ 文件内容无变化: {}", input.file_path);
            return Ok(EditOutput {
                message: format!("文件 '{}' 中未找到要替换的内容", input.file_path),
                replacements: 0,
                file_path: input.file_path,
            });
        }
        
        // 写入新内容
        fs::write(&input.file_path, new_content)?;
        
        // 计算替换次数
        let replacements = if replace_all {
            original_content.matches(&input.old_string).count() as u64
        } else {
            if original_content.contains(&input.old_string) { 1 } else { 0 }
        };
        
        info!("✏️ 文件编辑成功: {} ({} replacements)", input.file_path, replacements);
        
        Ok(EditOutput {
            message: format!("成功编辑文件: {}", input.file_path),
            replacements,
            file_path: input.file_path,
        })
    }
}