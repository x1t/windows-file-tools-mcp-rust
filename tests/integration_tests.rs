//! 集成测试

use file_bash_tools_mcp::{FileBashToolsServer, models::*};
use tempfile::TempDir;
use tokio;
use std::fs;

#[tokio::test]
async fn test_file_operations() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let test_file_path = temp_dir.path().join("test.txt");
    let test_file_str = test_file_path.to_string_lossy().to_string();
    
    let server = FileBashToolsServer::new();
    
    // 测试Write功能
    let write_input = WriteInput {
        file_path: test_file_str.clone(),
        content: "Hello, World!\nThis is a test file.".to_string(),
    };
    
    let write_result = server.file_tools.write(write_input).await.unwrap();
    assert_eq!(write_result.bytes_written, 29);
    assert!(test_file_path.exists());
    
    // 测试Read功能
    let read_input = ReadInput {
        file_path: test_file_str.clone(),
        offset: None,
        limit: None,
    };
    
    let read_result = server.file_tools.read(read_input).await.unwrap();
    if let rmcp::tool::Result::Text(value) = read_result {
        let read_output: ReadTextOutput = serde_json::from_value(value).unwrap();
        assert_eq!(read_output.total_lines, 2);
        assert_eq!(read_output.lines_returned, 2);
        assert!(read_output.content.contains("Hello, World!"));
    } else {
        panic!("Expected text result");
    }
    
    // 测试Edit功能
    let edit_input = EditInput {
        file_path: test_file_str.clone(),
        old_string: "World".to_string(),
        new_string: "Rust".to_string(),
        replace_all: Some(false),
    };
    
    let edit_result = server.file_tools.edit(edit_input).await.unwrap();
    assert_eq!(edit_result.replacements, 1);
    
    // 验证修改
    let content = fs::read_to_string(&test_file_path).unwrap();
    assert!(content.contains("Hello, Rust!"));
    assert!(!content.contains("Hello, World!"));
}

#[tokio::test]
async fn test_search_operations() {
    let server = FileBashToolsServer::new();
    
    // 测试Glob功能
    let glob_input = GlobInput {
        pattern: "*.rs".to_string(),
        path: Some("src".to_string()),
    };
    
    // 注意：这个测试需要ripgrep和fd工具在系统PATH中
    // 在CI环境中可能需要安装这些工具
    let glob_result = server.search_tools.glob(glob_input).await;
    if let Ok(result) = glob_result {
        assert!(result.matches.len() > 0);
        assert!(result.search_path.contains("src"));
    }
}

#[tokio::test]
async fn test_bash_operations() {
    let server = FileBashToolsServer::new();
    
    // 测试简单的Bash命令
    let bash_input = BashInput {
        command: "echo 'Hello from PowerShell'".to_string(),
        timeout: Some(5000),
        description: Some("测试echo命令".to_string()),
        run_in_background: Some(false),
    };
    
    let bash_result = server.shell_tools.bash(bash_input).await.unwrap();
    assert_eq!(bash_result.exit_code, 0);
    assert!(bash_result.output.contains("Hello from PowerShell"));
}

#[test]
fn test_models_schema() {
    // 测试所有模型的schema生成
    let _write_schema = WriteInput::schema();
    let _read_schema = ReadInput::schema();
    let _edit_schema = EditInput::schema();
    let _grep_schema = GrepInput::schema();
    let _glob_schema = GlobInput::schema();
    let _bash_schema = BashInput::schema();
    
    // 如果没有panic，说明schema生成成功
    assert!(true);
}