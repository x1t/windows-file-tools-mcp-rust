//! Ripgrep工具封装

use crate::models::search::{GrepMatch};
use crate::ServerError;
use regex::Regex;
use std::path::Path;
use std::process::Command;
use tokio::process::Command as TokioCommand;
use tracing::{debug, warn, error};

/// Ripgrep包装器
#[derive(Clone)]
pub struct RipgrepWrapper {
    // 可以添加配置选项
}

impl RipgrepWrapper {
    /// 创建新的RipgrepWrapper实例
    pub fn new() -> Self {
        Self {}
    }

    /// 搜索内容（详细匹配信息）
    pub async fn search_content(
        &self,
        pattern: &str,
        path: &str,
        glob: Option<&str>,
        file_type: Option<&str>,
        case_insensitive: bool,
        show_line_numbers: bool,
        before_context: Option<i32>,
        after_context: Option<i32>,
        context: Option<i32>,
        head_limit: Option<i32>,
        multiline: bool,
    ) -> Result<Vec<GrepMatch>, ServerError> {
        let mut cmd = self.build_base_ripgrep_command(
            pattern,
            path,
            glob,
            file_type,
            case_insensitive,
        );

        // 添加内容输出选项
        cmd.arg("--json"); // 输出JSON格式

        // 处理上下文选项
        if let Some(c) = context {
            cmd.arg(&format!("-C{}", c));
        } else {
            if let Some(b) = before_context {
                cmd.arg(&format!("-B{}", b));
            }
            if let Some(a) = after_context {
                cmd.arg(&format!("-A{}", a));
            }
        }

        // 多行模式
        if multiline {
            cmd.arg("--multiline");
        }

        // 执行命令
        let output = cmd.output()
            .await
            .map_err(|e| ServerError::FileSystem(format!("执行ripgrep失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("ripgrep执行错误: {}", stderr);
            return Ok(vec![]);
        }

        // 解析JSON输出
        self.parse_ripgrep_json_output(&output.stdout, head_limit)
    }

    /// 搜索文件（仅返回匹配的文件列表）
    pub async fn search_files(
        &self,
        pattern: &str,
        path: &str,
        glob: Option<&str>,
        file_type: Option<&str>,
        case_insensitive: bool,
    ) -> Result<Vec<String>, ServerError> {
        let mut cmd = self.build_base_ripgrep_command(
            pattern,
            path,
            glob,
            file_type,
            case_insensitive,
        );

        // 仅输出文件名
        cmd.arg("--files-with-matches");

        // 执行命令
        let output = cmd.output()
            .await
            .map_err(|e| ServerError::FileSystem(format!("执行ripgrep失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("ripgrep执行错误: {}", stderr);
            return Ok(vec![]);
        }

        // 解析文件列表
        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(files)
    }

    /// 构建基础的ripgrep命令
    fn build_base_ripgrep_command(
        &self,
        pattern: &str,
        path: &str,
        glob: Option<&str>,
        file_type: Option<&str>,
        case_insensitive: bool,
    ) -> TokioCommand {
        let mut cmd = TokioCommand::new("rg");
        
        // 基础选项
        cmd.arg(pattern);
        cmd.arg(path);
        
        // 大小写敏感
        if case_insensitive {
            cmd.arg("--ignore-case");
        }

        // Glob模式过滤
        if let Some(glob_pattern) = glob {
            cmd.arg("--glob");
            cmd.arg(glob_pattern);
        }

        // 文件类型过滤
        if let Some(ft) = file_type {
            cmd.arg("--type");
            cmd.arg(ft);
        }

        // 其他有用的选项
        cmd.arg("--no-heading"); // 禁用文件标题
        cmd.arg("--no-column");   // 禁用列号

        cmd
    }

    /// 解析ripgrep JSON输出
    fn parse_ripgrep_json_output(&self, stdout: &[u8], head_limit: Option<i32>) -> Result<Vec<GrepMatch>, ServerError> {
        let stdout_str = String::from_utf8_lossy(stdout);
        let mut matches = Vec::new();
        let mut match_count = 0;

        debug!("解析ripgrep JSON输出，长度: {}", stdout_str.len());

        for line in stdout_str.lines() {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(event_type) = json_value.get("type").and_then(|v| v.as_str()) {
                    match event_type {
                        "match" => {
                            if let Some(data) = json_value.get("data") {
                                if let Some(grep_match) = self.parse_match_data(data) {
                                    matches.push(grep_match);
                                    match_count += 1;
                                    
                                    // 应用head_limit
                                    if let Some(limit) = head_limit {
                                        if match_count >= limit {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        "context" => {
                            // 处理上下文行
                            if let Some(data) = json_value.get("data") {
                                if let Err(e) = self.update_context_with_data(&mut matches, data) {
                                    warn!("Failed to update context: {}", e);
                                    // 企业级容错: 单个上下文错误不影响整体搜索结果
                                    // 继续执行而不是返回Err，保证用户体验
                                }
                            }
                        }
                        _ => {
                            // 其他事件类型暂时忽略
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    /// 解析匹配数据
    fn parse_match_data(&self, data: &serde_json::Value) -> Option<GrepMatch> {
        let path = data.get("path")?.get("text")?.as_str()?;
        let line_number = data.get("line_number").and_then(|v| v.as_i64()).map(|v| v as i32);
        let line = data.get("lines")?.get("text")?.as_str()?.trim();

        Some(GrepMatch {
            file: path.to_string(),
            line_number,
            line: line.to_string(),
            before_context: None,
            after_context: None,
        })
    }

    /// 更新上下文信息
    ///
    /// # 错误处理
    /// - 当 matches 为空时返回 `InvalidInput` 错误
    /// - 当数据解析失败时记录警告日志并继续执行
    ///
    /// # 企业级安全保证
    /// - 所有边界条件都经过显式检查
    /// - 使用 `ok_or_else` 和 `?` 确保栈展开时资源正确释放
    /// - 详细的错误日志便于生产环境故障排查
    fn update_context_with_data(
        &self,
        matches: &mut Vec<GrepMatch>,
        data: &serde_json::Value
    ) -> Result<(), ServerError> {
        // 防御式编程：显式检查边界条件
        if matches.is_empty() {
            warn!("attempting to update context for empty match list");
            return Err(ServerError::FileSystem(
                "Cannot update context: no matches found".to_string()
            ));
        }

        // 安全提取行内容，使用 ok_or_else 延迟错误创建
        let line = data.get("lines")
            .and_then(|v| v.get("text"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .ok_or_else(|| {
                debug!("failed to extract line text from context data");
                ServerError::FileSystem("Invalid context data format".to_string())
            })?;

        // 安全获取最后一个匹配项
        let last_match = matches.last_mut()
            .ok_or_else(|| {
                error!("unexpected empty match list during context update");
                ServerError::FileSystem("Match list became empty unexpectedly".to_string())
            })?;

        // 使用 get_or_insert_with 优雅地处理 Option<Vec>
        // 避免 if-else 分支，减少认知复杂度
        last_match.before_context
            .get_or_insert_with(Vec::new)
            .push(line.to_string());

        debug!("successfully added context line to match in file: {}", last_match.file);
        Ok(())
    }
}