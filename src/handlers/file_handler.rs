//! 文件处理器模块

use crate::ServerError;
use std::path::Path;
use tracing::debug;

/// 文件处理器
pub struct FileHandler;

impl FileHandler {
    /// 验证路径安全性
    pub fn validate_path_security(path: &str) -> Result<(), ServerError> {
        let path = Path::new(path);
        
        // 检查路径遍历攻击
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(ServerError::FileSystem("路径包含不安全的父目录引用'..'", path.to_string_lossy().to_string()));
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// 获取文件的MIME类型
    pub fn get_mime_type(file_path: &str) -> String {
        let path = Path::new(file_path);
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "txt" => "text/plain".to_string(),
                "md" => "text/markdown".to_string(),
                "json" => "application/json".to_string(),
                "xml" => "application/xml".to_string(),
                "yaml" | "yml" => "application/x-yaml".to_string(),
                "csv" => "text/csv".to_string(),
                "html" => "text/html".to_string(),
                "css" => "text/css".to_string(),
                "js" => "application/javascript".to_string(),
                "ts" => "application/typescript".to_string(),
                "py" => "text/x-python".to_string(),
                "rs" => "text/x-rust".to_string(),
                "java" => "text/x-java".to_string(),
                "cpp" | "cxx" | "cc" => "text/x-c++".to_string(),
                "c" => "text/x-c".to_string(),
                "go" => "text/x-go".to_string(),
                "php" => "application/x-php".to_string(),
                "rb" => "text/x-ruby".to_string(),
                "sql" => "application/sql".to_string(),
                "sh" => "application/x-sh".to_string(),
                "ps1" => "application/x-powershell".to_string(),
                "bat" => "application/x-bat".to_string(),
                "cmd" => "application/x-cmd".to_string(),
                "jpg" | "jpeg" => "image/jpeg".to_string(),
                "png" => "image/png".to_string(),
                "gif" => "image/gif".to_string(),
                "bmp" => "image/bmp".to_string(),
                "webp" => "image/webp".to_string(),
                "svg" => "image/svg+xml".to_string(),
                "ico" => "image/x-icon".to_string(),
                "pdf" => "application/pdf".to_string(),
                "doc" | "docx" => "application/msword".to_string(),
                "xls" | "xlsx" => "application/vnd.ms-excel".to_string(),
                "ppt" | "pptx" => "application/vnd.ms-powerpoint".to_string(),
                "zip" => "application/zip".to_string(),
                "rar" => "application/x-rar-compressed".to_string(),
                "7z" => "application/x-7z-compressed".to_string(),
                "tar" => "application/x-tar".to_string(),
                "gz" => "application/gzip".to_string(),
                "mp3" => "audio/mpeg".to_string(),
                "wav" => "audio/wav".to_string(),
                "mp4" => "video/mp4".to_string(),
                "avi" => "video/x-msvideo".to_string(),
                "mov" => "video/quicktime".to_string(),
                "exe" => "application/x-executable".to_string(),
                "dll" => "application/x-msdownload".to_string(),
                "so" => "application/x-sharedlib".to_string(),
                _ => "application/octet-stream".to_string(),
            }
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// 检查是否为文本文件
    pub fn is_text_file(file_path: &str) -> bool {
        let mime_type = Self::get_mime_type(file_path);
        mime_type.starts_with("text/") || 
        mime_type == "application/json" ||
        mime_type == "application/xml" ||
        mime_type == "application/x-yaml" ||
        mime_type == "application/javascript" ||
        mime_type == "application/typescript" ||
        mime_type == "text/x-python" ||
        mime_type == "text/x-rust" ||
        mime_type == "text/x-java" ||
        mime_type == "text/x-c++" ||
        mime_type == "text/x-c" ||
        mime_type == "text/x-go" ||
        mime_type == "application/x-php" ||
        mime_type == "text/x-ruby" ||
        mime_type == "application/sql" ||
        mime_type == "application/x-sh" ||
        mime_type == "application/x-powershell" ||
        mime_type == "application/x-bat" ||
        mime_type == "application/x-cmd"
    }

    /// 检查是否为图像文件
    pub fn is_image_file(file_path: &str) -> bool {
        let mime_type = Self::get_mime_type(file_path);
        mime_type.starts_with("image/")
    }

    /// 规范化文件路径
    pub fn normalize_path(path: &str) -> Result<String, ServerError> {
        let path = Path::new(path);
        
        // 验证路径安全性
        Self::validate_path_security(path.to_str().unwrap_or(""))?;
        
        // 获取绝对路径
        let abs_path = path.canonicalize()
            .map_err(|e| ServerError::FileSystem(format!("无法规范化路径: {}", e)))?;
        
        Ok(abs_path.to_string_lossy().to_string())
    }

    /// 获取文件大小
    pub fn get_file_size(file_path: &str) -> Result<u64, ServerError> {
        let metadata = std::fs::metadata(file_path)
            .map_err(|e| ServerError::FileSystem(format!("无法获取文件元数据: {}", e)))?;
        
        Ok(metadata.len())
    }

    /// 检查文件是否可读
    pub fn is_readable(file_path: &str) -> bool {
        std::fs::metadata(file_path)
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(false)
    }

    /// 检查文件是否可写
    pub fn is_writable(file_path: &str) -> bool {
        if let Some(parent) = Path::new(file_path).parent() {
            std::fs::metadata(parent)
                .map(|metadata| !metadata.permissions().readonly())
                .unwrap_or(false)
        } else {
            false
        }
    }
}