//! Shell工具实现（PowerShell相关）

use crate::models::shell::*;
use crate::ServerError;
use rmcp::{tool, tool_router};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Shell工具处理器
#[derive(Clone)]
pub struct ShellTools {
    /// 后台进程管理
    background_shells: Arc<Mutex<HashMap<String, BackgroundShell>>>,
}

impl ShellTools {
    /// 创建新的Shell工具实例
    pub fn new() -> Self {
        Self {
            background_shells: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 生成唯一的Shell ID
    fn generate_shell_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// 清理已完成的后台进程
    async fn cleanup_completed_shells(&self) {
        let mut shells = self.background_shells.lock().unwrap();
        let mut to_remove = Vec::new();
        
        for (id, shell) in shells.iter() {
            if shell.killed {
                to_remove.push(id.clone());
                continue;
            }
            
            // 检查进程状态
            match shell.process.try_wait() {
                Ok(Some(_status)) => {
                    // 进程已结束
                    to_remove.push(id.clone());
                }
                Ok(None) => {
                    // 进程仍在运行
                }
                Err(e) => {
                    warn!("⚠️ 检查进程状态时出错: {}", e);
                    to_remove.push(id.clone());
                }
            }
        }
        
        for id in to_remove {
            shells.remove(&id);
        }
    }

    /// 验证命令安全性
    fn validate_command(command: &str) -> Result<(), ServerError> {
        if command.is_empty() {
            return Err(ServerError::ShellExecution("命令不能为空".to_string()));
        }

        // 基本安全检查 - 可以根据需要扩展
        let dangerous_commands = [
            "format", "del", "rmdir", "rd", "shutdown", "restart", 
            "net user", "net localgroup", "reg add", "reg delete"
        ];
        
        let lower_command = command.to_lowercase();
        for dangerous in &dangerous_commands {
            if lower_command.contains(dangerous) {
                warn!("⚠️ 检测到潜在危险命令: {}", command);
                // 在生产环境中可能需要更严格的检查
            }
        }

        Ok(())
    }

    /// 执行PowerShell命令
    async fn execute_powershell_command(
        command: &str,
        timeout: Option<u64>,
        background: bool,
    ) -> Result<(String, i32, Option<tokio::process::Child>), ServerError> {
        let mut cmd = Command::new("pwsh.exe");
        cmd.arg("-NoProfile")
           .arg("-Command")
           .arg(command);

        if background {
            // 后台执行
            let child = cmd.spawn()
                .map_err(|e| ServerError::ShellExecution(format!("启动PowerShell失败: {}", e)))?;
            
            Ok(("".to_string(), 0, Some(child)))
        } else {
            // 前台执行
            let timeout_duration = timeout.map(Duration::from_millis).unwrap_or(Duration::from_secs(30));
            
            let output = tokio::time::timeout(
                timeout_duration,
                cmd.output()
            ).await
                .map_err(|_| ServerError::ShellExecution("命令执行超时".to_string()))?
                .map_err(|e| ServerError::ShellExecution(format!("命令执行失败: {}", e)))?;
            
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined_output = format!("{}\n{}", stdout, stderr);
            
            Ok((combined_output, exit_code, None))
        }
    }
}

#[tool_router]
impl ShellTools {
    /// 执行PowerShell命令
    #[tool(description = "Execute PowerShell command")]
    async fn bash(&self, input: BashInput) -> Result<BashOutput, ServerError> {
        debug!("💻 Bash工具调用: command={}", input.command);
        
        // 验证命令安全性
        Self::validate_command(&input.command)?;
        
        // 清理已完成的进程
        self.cleanup_completed_shells().await;
        
        // 执行命令
        let (output, exit_code, child_process) = Self::execute_powershell_command(
            &input.command,
            input.timeout,
            input.run_in_background.unwrap_or(false),
        ).await?;
        
        let mut result = BashOutput {
            output,
            exit_code,
            killed: None,
            shell_id: None,
        };
        
        // 如果是后台执行，保存进程信息
        if let Some(mut child) = child_process {
            let shell_id = Self::generate_shell_id();
            
            let background_shell = BackgroundShell {
                id: shell_id.clone(),
                process: child,
                start_time: std::time::Instant::now(),
                description: input.description,
                killed: false,
            };
            
            self.background_shells.lock().unwrap().insert(shell_id.clone(), background_shell);
            
            result.shell_id = Some(shell_id);
            result.output = "命令已在后台启动".to_string();
        }
        
        if exit_code == 0 {
            info!("✅ Bash命令执行成功: exit_code={}", exit_code);
        } else {
            warn!("⚠️ Bash命令执行失败: exit_code={}", exit_code);
        }
        
        Ok(result)
    }

    /// 获取后台命令输出
    #[tool(description = "Get output from background PowerShell process")]
    async fn bash_output(&self, input: BashOutputInput) -> Result<BashOutputResult, ServerError> {
        debug!("💻 BashOutput工具调用: bash_id={}", input.bash_id);
        
        let mut shells = self.background_shells.lock().unwrap();
        
        let shell = shells.get_mut(&input.bash_id)
            .ok_or_else(|| ServerError::ShellExecution(format!("未找到后台进程: {}", input.bash_id)))?;
        
        // 检查进程状态
        let status = match shell.process.try_wait() {
            Ok(Some(exit_status)) => {
                let exit_code = exit_status.code().unwrap_or(-1);
                if exit_code == 0 {
                    "completed".to_string()
                } else {
                    "failed".to_string()
                }
            }
            Ok(None) => {
                "running".to_string()
            }
            Err(e) => {
                warn!("⚠️ 检查进程状态时出错: {}", e);
                "failed".to_string()
            }
        };
        
        // 简单实现 - 返回基本状态信息
        // 在实际实现中，可能需要捕获和缓冲输出
        let output = match status.as_str() {
            "running" => "进程正在运行中...".to_string(),
            "completed" => "进程已成功完成".to_string(),
            "failed" => "进程执行失败".to_string(),
            _ => "未知状态".to_string(),
        };
        
        let exit_code = if status != "running" {
            shell.process.try_wait().ok()
                .and_then(|r| r)
                .and_then(|s| s.code())
        } else {
            None
        };
        
        Ok(BashOutputResult {
            output,
            status,
            exit_code,
        })
    }

    /// 终止后台进程
    #[tool(description = "Terminate background PowerShell process")]
    async fn kill_bash(&self, input: KillBashInput) -> Result<KillBashOutput, ServerError> {
        debug!("💻 KillBash工具调用: shell_id={}", input.shell_id);
        
        let mut shells = self.background_shells.lock().unwrap();
        
        let shell = shells.remove(&input.shell_id)
            .ok_or_else(|| ServerError::ShellExecution(format!("未找到后台进程: {}", input.shell_id)))?;
        
        // 尝试优雅地终止进程
        let killed = match shell.process.kill().await {
            Ok(_) => {
                info!("✅ 后台进程已终止: {}", input.shell_id);
                true
            }
            Err(e) => {
                warn!("⚠️ 终止进程时出错: {}", e);
                false
            }
        };
        
        Ok(KillBashOutput {
            message: if killed {
                format!("成功终止后台进程: {}", input.shell_id)
            } else {
                format!("终止后台进程失败: {}", input.shell_id)
            },
            shell_id: input.shell_id,
        })
    }
}