//! Shell操作相关的数据模型

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Bash工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BashInput {
    /// 要执行的命令
    #[schemars(description = "要执行的命令")]
    pub command: String,
    /// 可选的超时时间(毫秒,最大 600000)
    #[schemars(description = "可选的超时时间(毫秒,最大 600000)")]
    pub timeout: Option<u64>,
    /// 清晰简洁的描述(5-10 个字)
    #[schemars(description = "清晰简洁的描述(5-10 个字)")]
    pub description: Option<String>,
    /// 设置为 true 以在后台运行
    #[schemars(description = "设置为 true 以在后台运行", default_with = "DefaultValue::default")]
    pub run_in_background: Option<bool>,
}

/// Bash工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BashOutput {
    /// 合并的 stdout 和 stderr 输出
    #[schemars(description = "合并的 stdout 和 stderr 输出")]
    pub output: String,
    /// 命令的退出码
    #[schemars(description = "命令的退出码")]
    pub exit_code: i32,
    /// 命令是否因超时被终止
    #[schemars(description = "命令是否因超时被终止")]
    pub killed: Option<bool>,
    /// 后台进程的 Shell ID
    #[schemars(description = "后台进程的 Shell ID")]
    pub shell_id: Option<String>,
}

/// BashOutput工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BashOutputInput {
    /// 后台 shell 的 ID
    #[schemars(description = "后台 shell 的 ID")]
    pub bash_id: String,
    /// 可选的正则表达式来过滤输出行
    #[schemars(description = "可选的正则表达式来过滤输出行")]
    pub filter: Option<String>,
}

/// BashOutput工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BashOutputResult {
    /// 自上次检查以来的新输出
    #[schemars(description = "自上次检查以来的新输出")]
    pub output: String,
    /// 当前 shell 状态
    #[schemars(description = "当前 shell 状态")]
    pub status: String,
    /// 完成时的退出码
    #[schemars(description = "完成时的退出码")]
    pub exit_code: Option<i32>,
}

/// KillBash工具输入参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KillBashInput {
    /// 要终止的后台 shell 的 ID
    #[schemars(description = "要终止的后台 shell 的 ID")]
    pub shell_id: String,
}

/// KillBash工具输出结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KillBashOutput {
    /// 成功消息
    #[schemars(description = "成功消息")]
    pub message: String,
    /// 被终止的 shell 的 ID
    #[schemars(description = "被终止的 shell 的 ID")]
    pub shell_id: String,
}

/// Shell进程状态
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ShellStatus {
    /// 运行中
    #[serde(rename = "running")]
    Running,
    /// 已完成
    #[serde(rename = "completed")]
    Completed,
    /// 失败
    #[serde(rename = "failed")]
    Failed,
}

/// 后台Shell进程信息
#[derive(Debug)]
pub struct BackgroundShell {
    /// 唯一标识符
    pub id: String,
    /// PowerShell进程句柄
    pub process: Arc<Mutex<tokio::process::Child>>,
    /// 启动时间
    pub start_time: std::time::Instant,
    /// 命令描述
    pub description: Option<String>,
    /// 是否已被终止
    pub killed: bool,
}