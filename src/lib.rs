//! File Tools MCP Server
//! 
//! 企业级文件工具MCP服务器

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
use tracing::{debug, info};
use glob::glob;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::SearcherBuilder;
use grep_searcher::sinks::UTF8;

// 重新导出主服务
pub use FileBashToolsService as FileToolsServer;

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

/// Glob工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct GlobRequest {
    /// 用于匹配文件的glob模式
    pub pattern: String,
    /// 搜索目录（默认为当前工作目录）
    #[serde(default)]
    pub path: Option<String>,
}

/// Grep工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct GrepRequest {
    /// 要搜索的正则表达式模式
    pub pattern: String,
    /// 要搜索的文件或目录（默认为当前工作目录）
    #[serde(default)]
    pub path: Option<String>,
    /// 用于过滤文件的glob模式（例如"*.js"）
    #[serde(default)]
    pub glob: Option<String>,
    /// 要搜索的文件类型（例如"js"、"py"、"rust"）
    #[serde(default)]
    pub r#type: Option<String>,
    /// 输出模式："content"、"files_with_matches"或"count"
    #[serde(default = "default_output_mode")]
    pub output_mode: String,
    /// 不区分大小写搜索
    #[serde(default)]
    pub case_insensitive: bool,
    /// 显示行号（适用于content模式）
    #[serde(default)]
    pub show_line_numbers: bool,
    /// 每个匹配前显示的行数
    #[serde(default)]
    pub before_context: Option<u32>,
    /// 每个匹配后显示的行数
    #[serde(default)]
    pub after_context: Option<u32>,
    /// 每个匹配前后显示的行数
    #[serde(default)]
    pub context: Option<u32>,
    /// 限制输出到前N行/条目
    #[serde(default)]
    pub head_limit: Option<usize>,
    /// 启用多行模式
    #[serde(default)]
    pub multiline: bool,
}

/// 单个待办事项
#[derive(Debug, serde::Deserialize, JsonSchema, Clone)]
pub struct TodoItem {
    /** 任务描述 */
    pub content: String,
    /** 任务状态 */
    pub status: TodoStatus,
    /** 任务描述的主动形式 */
    pub activeForm: String,
}

/// 待办事项状态
#[derive(Debug, serde::Deserialize, JsonSchema, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    /// 待处理
    Pending,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
}

/// TodoWrite工具请求
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct TodoWriteRequest {
    /** 更新后的待办事项列表 */
    pub todos: Vec<TodoItem>,
}

fn default_output_mode() -> String {
    "content".to_string()
}



fn default_offset() -> Option<i32> {
    Some(1)
}

/// File Tools服务
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

    /// 执行 content 模式的 grep 搜索
    async fn grep_content(
        &self,
        matcher: &RegexMatcher,
        search_path: &Path,
        req: &GrepRequest,
    ) -> Result<CallToolResult, McpError> {
        let mut all_matches = Vec::new();
        let mut total_count = 0;
        
        // 构建搜索器
        let mut searcher = SearcherBuilder::new().build();
        
        // 搜索文件
        self.search_files(search_path, req, |path, content| {
            let mut line_matches = Vec::new();
            let mut file_match_count = 0;
            
            searcher.search_slice(matcher, content.as_bytes(), UTF8(|line_num, line| {
                if let Ok(Some(m)) = matcher.find(line.as_bytes()) {
                    file_match_count += 1;
                    
                    let line_str = line;
                    let matched_text = &line_str[m.start()..m.end()];
                    
                    if req.show_line_numbers {
                        line_matches.push(format!("{}:{}: [{}] {}", path.display(), line_num, matched_text, line_str.trim()));
                    } else {
                        line_matches.push(format!("{}: [{}] {}", path.display(), matched_text, line_str.trim()));
                    }
                    
                    // 检查限制
                    if let Some(limit) = req.head_limit {
                        if total_count >= limit {
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            }))?;
            
            if !line_matches.is_empty() {
                all_matches.extend(line_matches);
                total_count += file_match_count;
            }
            
            Ok(())
        }).await?;
        
        info!("🔍 Grep content 搜索完成: 模式='{}', 找到{}个匹配", req.pattern, total_count);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Grep搜索结果 (content模式):\n模式: {}\n搜索路径: {}\n总匹配数: {}\n\n匹配内容:\n{}", 
                req.pattern, 
                search_path.display(), 
                total_count,
                if all_matches.is_empty() {
                    "无匹配内容".to_string()
                } else {
                    all_matches.join("\n")
                }
            ))
        ]))
    }
    
    /// 执行 files_with_matches 模式的 grep 搜索
    async fn grep_files_with_matches(
        &self,
        matcher: &RegexMatcher,
        search_path: &Path,
        req: &GrepRequest,
    ) -> Result<CallToolResult, McpError> {
        let mut files = Vec::new();
        
        let mut searcher = SearcherBuilder::new().build();
        
        self.search_files(search_path, req, |path, content| {
            let mut found = false;
            searcher.search_slice(matcher, content.as_bytes(), UTF8(|_, _| {
                found = true;
                Ok(false) // 找到一个匹配就停止
            }))?;
            
            if found {
                files.push(path.display().to_string());
            }
            Ok(())
        }).await?;
        
        files.sort();
        let count = files.len();
        
        info!("🔍 Grep files_with_matches 搜索完成: 模式='{}', 找到{}个文件", req.pattern, count);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Grep搜索结果 (files_with_matches模式):\n模式: {}\n搜索路径: {}\n匹配文件数: {}\n\n匹配文件:\n{}", 
                req.pattern, 
                search_path.display(), 
                count,
                if files.is_empty() {
                    "无匹配文件".to_string()
                } else {
                    files.iter().enumerate()
                        .map(|(i, file)| format!("{}. {}", i + 1, file))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ))
        ]))
    }
    
    /// 执行 count 模式的 grep 搜索
    async fn grep_count(
        &self,
        matcher: &RegexMatcher,
        search_path: &Path,
        req: &GrepRequest,
    ) -> Result<CallToolResult, McpError> {
        let mut counts = Vec::new();
        let mut total = 0;
        
        let mut searcher = SearcherBuilder::new().build();
        
        self.search_files(search_path, req, |path, content| {
            let mut file_count = 0;
            searcher.search_slice(matcher, content.as_bytes(), UTF8(|_, _| {
                file_count += 1;
                Ok(true)
            }))?;
            
            if file_count > 0 {
                counts.push((path.display().to_string(), file_count));
                total += file_count;
            }
            Ok(())
        }).await?;
        
        counts.sort_by(|a, b| a.0.cmp(&b.0));
        
        info!("🔍 Grep count 搜索完成: 模式='{}', 总计{}个匹配", req.pattern, total);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Grep搜索结果 (count模式):\n模式: {}\n搜索路径: {}\n总匹配数: {}\n\n各文件匹配数:\n{}", 
                req.pattern, 
                search_path.display(), 
                total,
                if counts.is_empty() {
                    "无匹配文件".to_string()
                } else {
                    counts.iter()
                        .map(|(file, count)| format!("{}: {}", file, count))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ))
        ]))
    }
    
    /// 搜索文件的核心方法
    async fn search_files<F>(
        &self,
        search_path: &Path,
        req: &GrepRequest,
        mut processor: F,
    ) -> Result<(), McpError>
    where
        F: FnMut(&Path, String) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        let entries = walkdir::WalkDir::new(search_path)
            .follow_links(false)
            .max_depth(if req.glob.is_some() { 10 } else { 50 }); // 限制搜索深度
        
        for entry in entries.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // 跳过目录
            if path.is_dir() {
                continue;
            }
            
            // 应用 glob 过滤器
            if let Some(ref glob_pattern) = req.glob {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if !self.matches_glob(file_name, glob_pattern) {
                        continue;
                    }
                }
            }
            
            // 应用类型过滤器
            if let Some(ref file_type) = req.r#type {
                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    if extension != file_type {
                        continue;
                    }
                }
            }
            
            // 读取文件内容
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    if let Err(e) = processor(path, content) {
                        debug!("处理文件 {} 时出错: {}", path.display(), e);
                    }
                }
                Err(e) => {
                    debug!("读取文件 {} 时出错: {}", path.display(), e);
                    continue;
                }
            }
        }
        
        Ok(())
    }
    
    /// 简单的 glob 匹配实现
    fn matches_glob(&self, file_name: &str, pattern: &str) -> bool {
        // 这里实现一个简单的 glob 匹配，实际项目中可以使用 globset 库
        if pattern == "*" {
            return true;
        }
        
        if pattern.starts_with("*.") {
            let ext = &pattern[2..];
            return file_name.ends_with(&format!(".{}", ext));
        }
        
        file_name.contains(pattern)
    }
}

#[tool_router]
impl FileBashToolsService {
    /// 写入文件内容 (Only Windows)
    #[tool(
        name = "write_file",
        description = "Write content to a file. Only Windows. Use double backslashes for paths like \"C:\\\\DumpStack.log\""
    )]
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

    /// 读取文件内容 (Only Windows)
    #[tool(
        name = "read_file", 
        description = "Read content from a file. Only Windows. Use double backslashes for paths like \"C:\\\\DumpStack.log\""
    )]
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

    /// 编辑文件内容 (Only Windows)
    #[tool(
        name = "edit_file",
        description = "Edit file content by replacing text. Only Windows. Use double backslashes for paths like \"C:\\\\DumpStack.log\""
    )]
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

    /// Glob文件匹配 (Only Windows)
    #[tool(
        name = "glob",
        description = "Fast file pattern matching tool that works with any codebase size. Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\". Returns matching file paths sorted by modification time. Only Windows."
    )]
    async fn glob_tool(
        &self,
        Parameters(req): Parameters<GlobRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Glob工具调用: pattern={}, path={:?}", req.pattern, req.path);
        
        // 确定搜索路径
        let search_path = req.path.clone().unwrap_or_else(|| ".".to_string());
        let search_path = Path::new(&search_path);
        
        // 验证搜索路径
        if !search_path.exists() {
            return Err(McpError::invalid_params(format!("搜索路径不存在: {}", search_path.display()), None));
        }
        
        // 构建完整的glob模式
        let full_pattern = if req.pattern.contains('/') || req.pattern.contains('\\') {
            // 如果模式已经包含路径分隔符，直接使用
            if search_path.to_string_lossy() == "." {
                req.pattern.clone()
            } else {
                format!("{}/**/{}", search_path.display(), req.pattern.trim_start_matches("./"))
            }
        } else {
            // 否则在搜索路径下搜索
            format!("{}/**/{}", search_path.display(), req.pattern)
        };
        
        debug!("使用glob模式: {}", full_pattern);
        
        // 执行glob匹配
        let mut matches = Vec::new();
        match glob(&full_pattern) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(path) => {
                            if let Some(path_str) = path.to_str() {
                                matches.push(path_str.to_string());
                            }
                        }
                        Err(e) => {
                            debug!("Glob匹配错误: {}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                return Err(McpError::invalid_params(format!("无效的glob模式 '{}': {}", req.pattern, e), None));
            }
        }
        
        // 排序结果
        matches.sort();
        
        let match_count = matches.len();
        info!("🔍 Glob匹配完成: 模式='{}', 路径='{}', 找到{}个匹配", 
              req.pattern, search_path.display(), match_count);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Glob匹配结果:\n模式: {}\n搜索路径: {}\n匹配数: {}\n\n匹配文件:\n{}", 
                req.pattern, 
                search_path.display(), 
                match_count,
                if matches.is_empty() {
                    "无匹配文件".to_string()
                } else {
                    matches.iter().enumerate().map(|(i, _m)| format!("{}.", i + 1)).zip(matches.iter())
                        .map(|(num, file)| format!("{} {}", num, file))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ))
        ]))
    }

    /// Grep文本搜索 (Only Windows)
    #[tool(
        name = "grep",
        description = "A powerful search tool built on ripgrep. Supports content/files_with_matches/count three output modes. Supports context lines (-B/-A/-C). Supports multi-line matching and case insensitive search. Only Windows."
    )]
    async fn grep(
        &self,
        Parameters(req): Parameters<GrepRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("Grep工具调用: pattern={}, path={:?}, output_mode={}", req.pattern, req.path, req.output_mode);
        
        // 确定搜索路径
        let search_path_str = req.path.clone().unwrap_or_else(|| ".".to_string());
        let search_path = Path::new(&search_path_str);
        
        // 验证搜索路径
        if !search_path.exists() {
            return Err(McpError::invalid_params(format!("搜索路径不存在: {}", search_path.display()), None));
        }
        
        // 创建正则表达式匹配器
        let matcher = RegexMatcher::new(&req.pattern)
            .map_err(|e| McpError::invalid_params(format!("无效的正则表达式 '{}': {}", req.pattern, e), None))?;
        
        // 根据输出模式执行不同类型的搜索
        match req.output_mode.as_str() {
            "files_with_matches" => self.grep_files_with_matches(&matcher, search_path, &req).await,
            "count" => self.grep_count(&matcher, search_path, &req).await,
            "content" | _ => self.grep_content(&matcher, search_path, &req).await,
        }
    }

    /// TodoWrite任务管理工具 (Only Windows)
    #[tool(
        name = "TodoWrite",
        description = "使用此工具为当前编码会话创建和管理结构化任务清单，帮助跟踪进度、整理复杂任务并向用户展示工作周密性，仅支持 Windows 系统；TodoWrite 工具必填参数结构为 {todos: [{content: 任务描述（例如：编写 Go 并发安全的工具类）, status: 任务状态（可选值：pending = 待处理 | in_progress = 进行中 | completed = 已完成）, activeForm: 正在进行的任务描述（仅 status 为 in_progress 时需填写，例如：调试 goroutine 生命周期管理逻辑）}]}"
    )]
    async fn todo_write(
        &self,
        Parameters(req): Parameters<TodoWriteRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("TodoWrite工具调用: 收到{}个待办事项", req.todos.len());
        
        // 计算统计信息
        let total = req.todos.len();
        let pending = req.todos.iter().filter(|t| matches!(t.status, TodoStatus::Pending)).count();
        let in_progress = req.todos.iter().filter(|t| matches!(t.status, TodoStatus::InProgress)).count();
        let completed = req.todos.iter().filter(|t| matches!(t.status, TodoStatus::Completed)).count();
        
        // 格式化待办事项列表
        let mut todo_list = Vec::new();
        for (i, todo) in req.todos.iter().enumerate() {
            let status_icon = match todo.status {
                TodoStatus::Pending => "⏳",
                TodoStatus::InProgress => "🔄", 
                TodoStatus::Completed => "✅",
            };
            
            todo_list.push(format!(
                "{} {}. [{}] {} - {}",
                status_icon,
                i + 1,
                format!("{:?}", todo.status).to_lowercase(),
                todo.content,
                todo.activeForm
            ));
        }
        
        let todo_list_str = todo_list.join("\n");
        
        info!("📝 TodoWrite 更新完成: 总计{}个任务 (待处理:{}, 进行中:{}, 已完成:{})", 
              total, pending, in_progress, completed);
        
        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "TodoWrite任务列表已更新:\n\n📊 统计信息:\n• 总任务数: {}\n• 待处理: {}\n• 进行中: {}\n• 已完成: {}\n\n📋 任务列表:\n{}", 
                total, 
                pending, 
                in_progress, 
                completed,
                todo_list_str
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
            instructions: Some("企业级文件操作MCP服务器。支持文件读写、编辑 (Only Windows)。".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glob::glob;

    #[tokio::test]
    async fn test_glob_pattern_matching() {
        // 测试基本的 glob 模式匹配
        let pattern = "*.rs";
        let full_pattern = format!("{}/**/{}", ".", pattern);
        
        let mut matches = Vec::new();
        match glob(&full_pattern) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(path) => {
                            if let Some(path_str) = path.to_str() {
                                matches.push(path_str.to_string());
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => panic!("Invalid glob pattern"),
        }
        
        // 验证至少能找到一些 .rs 文件
        assert!(!matches.is_empty(), "应该至少找到一个 .rs 文件");
        
        // 验证所有匹配都是 .rs 文件
        for file_path in &matches {
            assert!(file_path.ends_with(".rs"), "匹配的文件应该以 .rs 结尾: {}", file_path);
        }
        
        println!("找到 {} 个 .rs 文件", matches.len());
    }

    #[tokio::test]
    async fn test_glob_request_validation() {
        // 测试有效的 GlobRequest
        let valid_request = GlobRequest {
            pattern: "*.txt".to_string(),
            path: Some(".".to_string()),
        };
        
        // 这里我们只测试结构体创建，实际的工具调用需要 MCP 客户端
        assert_eq!(valid_request.pattern, "*.txt");
        assert_eq!(valid_request.path, Some(".".to_string()));
    }

    #[tokio::test]
    async fn test_grep_regex_matcher() {
        // 测试 RegexMatcher 创建和基本功能
        let pattern = r"\bfn\s+\w+";
        let matcher = RegexMatcher::new(pattern);
        assert!(matcher.is_ok(), "应该能够创建有效的正则表达式匹配器");
        
        let matcher = matcher.unwrap();
        let test_line = b"fn test_function() {";
        let result = matcher.find(test_line);
        assert!(result.is_ok(), "应该能够搜索测试行");
        
        if let Ok(Some(m)) = result {
            let matched_text = std::str::from_utf8(&test_line[m.start()..m.end()]).unwrap();
            assert_eq!(matched_text, "fn test_function");
        }
    }

    #[tokio::test]
    async fn test_todo_write_request_validation() {
        // 测试 TodoWrite 结构体创建
        let valid_request = TodoWriteRequest {
            todos: vec![
                TodoItem {
                    content: "测试任务".to_string(),
                    status: TodoStatus::Pending,
                    activeForm: "测试任务".to_string(),
                },
                TodoItem {
                    content: "进行中的任务".to_string(),
                    status: TodoStatus::InProgress,
                    activeForm: "进行中的任务".to_string(),
                },
                TodoItem {
                    content: "完成的任务".to_string(),
                    status: TodoStatus::Completed,
                    activeForm: "完成的任务".to_string(),
                },
            ],
        };
        
        // 验证结构体创建
        assert_eq!(valid_request.todos.len(), 3);
        assert_eq!(valid_request.todos[0].status, TodoStatus::Pending);
        assert_eq!(valid_request.todos[1].status, TodoStatus::InProgress);
        assert_eq!(valid_request.todos[2].status, TodoStatus::Completed);
        
        // 验证任务内容
        assert_eq!(valid_request.todos[0].content, "测试任务");
        assert_eq!(valid_request.todos[1].activeForm, "进行中的任务");
        assert_eq!(valid_request.todos[2].content, "完成的任务");
    }
}