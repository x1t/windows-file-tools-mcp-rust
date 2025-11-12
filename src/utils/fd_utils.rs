//! fd工具封装

use crate::ServerError;
use std::path::Path;
use tokio::process::Command as TokioCommand;
use tracing::{debug, warn};

/// fd包装器
#[derive(Clone)]
pub struct FdWrapper {
    // 可以添加配置选项
}

impl FdWrapper {
    /// 创建新的FdWrapper实例
    pub fn new() -> Self {
        Self {}
    }

    /// 使用fd查找文件
    pub async fn find_files(
        &self,
        pattern: &str,
        path: &str,
    ) -> Result<Vec<String>, ServerError> {
        let mut cmd = self.build_base_fd_command(pattern, path);

        // 执行命令
        let output = cmd.output()
            .await
            .map_err(|e| ServerError::FileSystem(format!("执行fd失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("fd执行错误: {}", stderr);
            
            // 如果pattern是简单的glob模式，尝试使用标准库方法
            if self.is_simple_glob(pattern) {
                return self.fallback_to_globsearch(pattern, path).await;
            }
            
            return Ok(vec![]);
        }

        // 解析输出
        self.parse_fd_output(&output.stdout)
    }

    /// 构建基础fd命令
    fn build_base_fd_command(&self, pattern: &str, path: &str) -> TokioCommand {
        let mut cmd = TokioCommand::new("fd");
        
        // 搜索模式
        cmd.arg(pattern);
        
        // 搜索路径
        cmd.arg(path);
        
        // 输出选项
        cmd.arg("--absolute-path");  // 输出绝对路径
        cmd.arg("--type");           // 仅搜索文件
        cmd.arg("f");

        cmd
    }

    /// 解析fd输出
    fn parse_fd_output(&self, stdout: &[u8]) -> Result<Vec<String>, ServerError> {
        let stdout_str = String::from_utf8_lossy(stdout);
        let files: Vec<String> = stdout_str
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        debug!("fd找到 {} 个文件", files.len());
        Ok(files)
    }

    /// 检查是否为简单的glob模式
    fn is_simple_glob(&self, pattern: &str) -> bool {
        // 检查是否包含简单的通配符
        pattern.contains('*') || pattern.contains('?') || pattern.contains('[')
    }

    /// 回退到基于标准库的glob搜索
    async fn fallback_to_globsearch(&self, pattern: &str, path: &str) -> Result<Vec<String>, ServerError> {
        debug!("使用回退的glob搜索: pattern={}, path={}", pattern, path);

        // 使用globset进行模式匹配
        let glob_matcher = globset::Glob::new(pattern)
            .map_err(|e| ServerError::FileSystem(format!("无效的glob模式: {}", e)))?
            .compile_matcher();

        let search_path = Path::new(path);
        let mut matches = Vec::new();

        // 使用walkdir进行文件系统遍历
        let mut entries = tokio::task::spawn_blocking(move || {
            let mut results = Vec::new();
            if let Ok(walker) = walkdir::WalkDir::new(search_path).into_iter() {
                for entry in walker.flatten() {
                    if entry.file_type().is_file() {
                        if let Some(path_str) = entry.path().to_str() {
                            let relative_path = path_str.strip_prefix(path).unwrap_or(path_str);
                            if glob_matcher.is_match(relative_path) {
                                if let Ok(abs_path) = std::fs::canonicalize(entry.path()) {
                                    if let Some(abs_path_str) = abs_path.to_str() {
                                        results.push(abs_path_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            results
        }).await.map_err(|e| ServerError::FileSystem(format!("文件搜索任务失败: {}", e)))?;

        matches.append(&mut entries);
        
        debug!("回退搜索找到 {} 个文件", matches.len());
        Ok(matches)
    }

    /// 高级文件搜索（支持更多选项）
    pub async fn find_files_advanced(
        &self,
        pattern: &str,
        path: &str,
        file_extension: Option<&str>,
        max_depth: Option<usize>,
        exclude_patterns: &[&str],
    ) -> Result<Vec<String>, ServerError> {
        let mut cmd = self.build_base_fd_command(pattern, path);

        // 文件扩展名过滤
        if let Some(ext) = file_extension {
            cmd.arg("--extension");
            cmd.arg(ext.trim_start_matches('.'));
        }

        // 搜索深度限制
        if let Some(depth) = max_depth {
            cmd.arg("--max-depth");
            cmd.arg(&depth.to_string());
        }

        // 排除模式
        for exclude_pattern in exclude_patterns {
            cmd.arg("--exclude");
            cmd.arg(exclude_pattern);
        }

        // 执行命令
        let output = cmd.output()
            .await
            .map_err(|e| ServerError::FileSystem(format!("执行fd失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("fd高级搜索错误: {}", stderr);
            return Ok(vec![]);
        }

        self.parse_fd_output(&output.stdout)
    }

    /// 统计文件数量（不返回具体文件列表）
    pub async fn count_files(
        &self,
        pattern: &str,
        path: &str,
    ) -> Result<usize, ServerError> {
        let mut cmd = self.build_base_fd_command(pattern, path);
        
        // 仅计数，不输出文件名
        cmd.arg("--count");

        let output = cmd.output()
            .await
            .map_err(|e| ServerError::FileSystem(format!("执行fd失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("fd计数错误: {}", stderr);
            return Ok(0);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let count_str = stdout.trim();
        
        count_str.parse::<usize>()
            .map_err(|e| ServerError::FileSystem(format!("解析计数结果失败: {}", e)))
    }
}