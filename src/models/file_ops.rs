//! 文件操作相关的数据模型

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Write工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteInput {
    /// 要写入的文件的绝对路径
    #[schemars(description = "要写入的文件的绝对路径")]
    pub file_path: String,
    /// 要写入文件的内容
    #[schemars(description = "要写入文件的内容")]
    pub content: String,
}

/// Write工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteOutput {
    /// 成功消息
    #[schemars(description = "成功消息")]
    pub message: String,
    /// 写入的字节数
    #[schemars(description = "写入的字节数")]
    pub bytes_written: u64,
    /// 被写入的文件路径
    #[schemars(description = "被写入的文件路径")]
    pub file_path: String,
}

/// Read工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadInput {
    /// 要读取的文件的绝对路径
    #[schemars(description = "要读取的文件的绝对路径")]
    pub file_path: String,
    /// 开始读取的行号
    #[schemars(description = "开始读取的行号", default_with = "DefaultValue::default")]
    pub offset: Option<i32>,
    /// 要读取的行数
    #[schemars(description = "要读取的行数")]
    pub limit: Option<i32>,
}

/// Read工具输出结果（文本文件）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadTextOutput {
    /// 带行号的文件内容
    #[schemars(description = "带行号的文件内容")]
    pub content: String,
    /// 文件总行数
    #[schemars(description = "文件总行数")]
    pub total_lines: u64,
    /// 实际返回的行数
    #[schemars(description = "实际返回的行数")]
    pub lines_returned: u64,
}

/// Read工具输出结果（图像文件）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadImageOutput {
    /// Base64 编码的图像数据
    #[schemars(description = "Base64 编码的图像数据")]
    pub image: String,
    /// 图像 MIME 类型
    #[schemars(description = "图像 MIME 类型")]
    pub mime_type: String,
    /// 文件大小(字节)
    #[schemars(description = "文件大小(字节)")]
    pub file_size: u64,
}

/// Edit工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditInput {
    /// 要修改的文件的绝对路径
    #[schemars(description = "要修改的文件的绝对路径")]
    pub file_path: String,
    /// 要替换的文本
    #[schemars(description = "要替换的文本")]
    pub old_string: String,
    /// 用于替换的新文本
    #[schemars(description = "用于替换的新文本")]
    pub new_string: String,
    /// 替换所有匹配项(默认 false)
    #[schemars(description = "替换所有匹配项(默认 false)", default_with = "DefaultValue::default")]
    pub replace_all: Option<bool>,
}

/// Edit工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditOutput {
    /// 确认消息
    #[schemars(description = "确认消息")]
    pub message: String,
    /// 进行的替换次数
    #[schemars(description = "进行的替换次数")]
    pub replacements: u64,
    /// 被编辑的文件路径
    #[schemars(description = "被编辑的文件路径")]
    pub file_path: String,
}