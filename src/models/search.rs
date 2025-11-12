//! 搜索相关的数据模型

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Grep工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepInput {
    /// 正则表达式模式
    #[schemars(description = "正则表达式模式")]
    pub pattern: String,
    /// 要搜索的文件或目录
    #[schemars(description = "要搜索的文件或目录")]
    pub path: Option<String>,
    /// 用于过滤文件的 Glob 模式
    #[schemars(description = "用于过滤文件的 Glob 模式")]
    pub glob: Option<String>,
    /// 要搜索的文件类型
    #[schemars(description = "要搜索的文件类型")]
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    /// 输出模式: "content", "files_with_matches", 或 "count"
    #[schemars(description = "输出模式: \"content\", \"files_with_matches\", 或 \"count\"")]
    pub output_mode: Option<String>,
    
    /// 不区分大小写搜索
    #[schemars(description = "不区分大小写搜索", default_with = "DefaultValue::default")]
    #[serde(rename = "-i")]
    pub case_insensitive: Option<bool>,
    /// 显示行号
    #[schemars(description = "显示行号", default_with = "DefaultValue::default")]
    #[serde(rename = "-n")]
    pub show_line_numbers: Option<bool>,
    /// 每个匹配前显示的行数
    #[schemars(description = "每个匹配前显示的行数")]
    #[serde(rename = "-B")]
    pub before_context: Option<i32>,
    /// 每个匹配后显示的行数
    #[schemars(description = "每个匹配后显示的行数")]
    #[serde(rename = "-A")]
    pub after_context: Option<i32>,
    /// 匹配前后显示的行数
    #[schemars(description = "匹配前后显示的行数")]
    #[serde(rename = "-C")]
    pub context: Option<i32>,
    /// 限制输出到前 N 行/条目
    #[schemars(description = "限制输出到前 N 行/条目")]
    pub head_limit: Option<i32>,
    /// 启用多行模式
    #[schemars(description = "启用多行模式", default_with = "DefaultValue::default")]
    pub multiline: Option<bool>,
}

/// Grep匹配项
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepMatch {
    /// 文件路径
    #[schemars(description = "文件路径")]
    pub file: String,
    /// 行号
    #[schemars(description = "行号")]
    pub line_number: Option<i32>,
    /// 匹配的行内容
    #[schemars(description = "匹配的行内容")]
    pub line: String,
    /// 前文内容
    #[schemars(description = "前文内容")]
    pub before_context: Option<Vec<String>>,
    /// 后文内容
    #[schemars(description = "后文内容")]
    pub after_context: Option<Vec<String>>,
}

/// Grep工具输出结果（content模式）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepContentOutput {
    /// 匹配项列表
    #[schemars(description = "匹配项列表")]
    pub matches: Vec<GrepMatch>,
    /// 总匹配数
    #[schemars(description = "总匹配数")]
    pub total_matches: u64,
}

/// Grep工具输出结果（files_with_matches模式）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepFilesOutput {
    /// 包含匹配的文件
    #[schemars(description = "包含匹配的文件")]
    pub files: Vec<String>,
    /// 有匹配的文件数
    #[schemars(description = "有匹配的文件数")]
    pub count: u64,
}

/// Glob工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobInput {
    /// 用于匹配文件的 Glob 模式
    #[schemars(description = "用于匹配文件的 Glob 模式")]
    pub pattern: String,
    /// 要搜索的目录(默认为 cwd)
    #[schemars(description = "要搜索的目录(默认为 cwd)")]
    pub path: Option<String>,
}

/// Glob工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobOutput {
    /// 匹配的文件路径数组
    #[schemars(description = "匹配的文件路径数组")]
    pub matches: Vec<String>,
    /// 找到的匹配数
    #[schemars(description = "找到的匹配数")]
    pub count: u64,
    /// 使用的搜索目录
    #[schemars(description = "使用的搜索目录")]
    pub search_path: String,
}