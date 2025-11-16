# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个使用 Rust 构建的企业级 MCP（Model Context Protocol）服务器，专为 Windows 环境设计，提供强大的文件操作和搜索功能。项目使用 RMCP SDK 构建，通过标准输入输出与 MCP 客户端通信。

## 核心功能

### 📁 文件操作工具
- **写入 (write_file)**: 向文件写入内容，支持自动创建目录，使用原子操作确保数据完整性
- **读取 (read_file)**: 读取文件内容，支持偏移量和行数限制  
- **编辑 (edit_file)**: 替换文件中的文本，支持单次或全部替换

### 🔍 搜索工具
- **文件匹配 (glob)**: 使用 glob 模式快速匹配文件，支持 **/*.js 等复杂模式
- **文本搜索 (grep)**: 基于 ripgrep 核心库的强大文本搜索，支持三种输出模式：
  - `content`: 显示匹配的具体内容
  - `files_with_matches`: 仅显示包含匹配的文件列表
  - `count`: 显示每个文件中的匹配次数

### 📋 任务管理
- **TodoWrite**: 结构化任务列表管理工具，支持待办事项状态跟踪

## 常用开发命令

### 构建和运行
```bash
# 构建项目（开发模式）
cargo build

# 构建项目（发布模式，带优化）
cargo build --release

# 运行服务器
cargo run

# 运行服务器（发布模式）
cargo run --release
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行测试但不执行
cargo test --no-run

# 运行测试并显示详细输出
cargo test -- --nocapture

# 运行单个测试模块
cargo test --lib mod::tests
```

### 调试和分析
```bash
# 检查代码但不构建
cargo check

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 清理构建产物
cargo clean
```

### MCP 客户端集成
```bash
# 使用 MCP Inspector 测试（推荐用于调试）
npx @modelcontextprotocol/inspector cargo run --release

# 作为 stdio 服务器运行（默认）
cargo run --release

# 开发模式运行（用于调试）
cargo run
```

#### 调试技巧
- 使用 `RUST_LOG=debug` 环境变量启用详细日志
- MCP Inspector 提供交互式工具测试界面
- 日志输出到 stderr，包含操作ID和性能信息

## 项目架构

### 核心模块结构
```
src/
├── main.rs              # 服务器入口点，初始化日志和服务 (31行)
├── lib.rs               # 核心服务实现，包含所有 MCP 工具实现 (960行)
├── models/              # 数据结构定义
│   ├── mod.rs           # 模块导出 (10行)
│   ├── file_ops.rs      # 文件操作相关数据结构 (102行)
│   └── search.rs        # 搜索相关数据结构 (119行)
├── handlers/            # 请求处理器
│   ├── mod.rs           # 模块导出 (3行)
│   └── file_handler.rs  # 文件请求处理器 (158行)
├── tools/               # 工具实现模块
│   ├── mod.rs           # 模块导出 (10行)
│   ├── file_tools.rs    # 文件操作工具实现 (249行)
│   └── search_tools.rs  # 搜索工具实现 (169行)
└── utils/               # 通用工具函数
    ├── mod.rs           # 模块导出 (4行)
    ├── ripgrep_utils.rs # ripgrep 相关工具 (6341行)
    └── fd_utils.rs      # 文件描述工具 (7450行)
```

### 关键设计特点

1. **单文件聚合架构**: 核心逻辑集中在 `src/lib.rs` (960行)，包含所有 MCP 工具实现和数据结构定义
2. **模块化辅助设计**: 虽然核心集中在单文件，但保持了清晰的模块结构
3. **并发控制**: 使用 Semaphore 限制同时处理的文件数量（最多10个），防止资源耗尽
4. **性能优化**: 
   - 根据搜索模式动态调整搜索深度（文件匹配模式20层、计数模式30层、默认模式50层）
   - 跳过超过10MB的大文件
   - 智能的glob模式匹配
5. **安全特性**: 
   - 严格的路径验证，防止目录遍历攻击
   - 输入清理和验证
   - Windows路径格式支持（双反斜杠）
   - 原子文件操作，使用 NamedTempFile 确保数据完整性
   - 降级机制：原子操作失败时自动回退到标准文件操作

### MCP 工具实现

所有工具都使用 RMCP 的宏系统实现：

```rust
#[tool(
    name = "tool_name",
    description = "工具描述"
)]
async fn tool_method(
    &self,
    Parameters(req): Parameters<ToolRequest>,
) -> Result<CallToolResult, McpError>
```

### 数据流架构
```
MCP Client → stdio → main.rs → lib.rs → Tool Router → Specific Tool → File System
     ↑                                                          ↓
   JSON Response ← JSON Response ← Result Processing ← Operation Result
```

## 技术栈

### 核心框架
- **RMCP 0.8.5**: MCP SDK，提供协议实现和工具路由，包含 server、macros、transport-io 特性
- **Tokio 1.42**: 异步运行时，支持文件系统和进程操作，包含完整特性集

### 序列化和数据验证
- **Serde 1.0**: 序列化/反序列化，包含 derive 特性
- **Serde JSON 1.0**: JSON 处理
- **Schemars 1.0**: JSON Schema 生成

### 文件系统和搜索
- **Ripgrep 核心库**: 高性能文本搜索（grep 0.4.1, grep-searcher 0.1.16, grep-regex 0.1.14, grep-matcher 0.1.8）
- **Walkdir 2.5**: 文件系统遍历
- **Glob 0.3**: 文件模式匹配
- **Tempfile 3.0**: 原子文件操作
- **Ignore 0.4**: 文件忽略规则

### 错误处理和日志
- **Thiserror 2.0**: 结构化错误定义
- **Anyhow 1.0**: 错误上下文处理
- **Tracing 0.1**: 结构化日志记录
- **Tracing Subscriber 0.3**: 日志输出处理

### 其他关键依赖
- **Regex 1.11**: 正则表达式支持
- **UUID 1.11**: 唯一标识符生成，包含 v4 和 serde 特性
- **Base64 0.22**: Base64 编码支持

## 开发注意事项

### Windows 环境特殊性
- 所有工具都明确标注 "Only Windows"
- 路径处理使用双反斜杠格式：`C:\\\\path\\\\to\\\\file`
- 支持 Windows 路径分隔符和原生文件系统特性

### 错误处理模式
- 内部使用 `anyhow::Result` 进行错误处理
- MCP 响应转换为 `McpError` 
- 提供详细的错误信息和上下文
- 使用降级机制，原子操作失败时自动回退到标准文件操作

### 日志记录
- 使用 `tracing` 进行结构化日志
- 默认输出到 stderr
- 支持环境变量控制日志级别
- 企业级操作日志记录，包含操作ID和耗时信息

### 并发和性能
- 使用 `Arc<Semaphore>` 限制并发文件操作数量（最多10个）
- 动态调整搜索深度以优化性能
- 大文件自动跳过（>10MB）
- 原子文件操作防止数据损坏

### 测试策略
- 单元测试覆盖核心功能（位于 `src/lib.rs` 的 `tests` 模块）
- 集成测试验证工具行为
- 性能测试确保大文件处理能力
- 使用 MCP Inspector 进行交互式测试

#### 内置测试用例
项目包含以下测试用例：
- `test_glob_pattern_matching`: 测试 glob 模式匹配功能
- `test_glob_request_validation`: 测试 GlobRequest 结构体验证
- `test_grep_regex_matcher`: 测试正则表达式匹配器
- `test_todo_write_request_validation`: 测试 TodoWrite 功能验证

### 安全特性
- 严格的绝对路径验证，防止路径遍历攻击
- 输入参数清理和验证
- 最小权限原则，仅请求必要的文件系统访问
- 原子操作确保数据完整性

## MCP 客户端集成示例

### Claude Desktop 配置
```json
{
  "mcpServers": {
    "windows-file-tools": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "/path/to/windows-file-tools-mcp-rust/Cargo.toml"]
    }
  }
}
```

### 工具调用示例
```json
// 文件写入
{
  "tool": "write_file",
  "arguments": {
    "file_path": "C:\\\\Temp\\\\example.txt",
    "content": "Hello from MCP!"
  }
}

// 文件搜索
{
  "tool": "grep",
  "arguments": {
    "pattern": "async fn",
    "path": "src/",
    "output_mode": "content"
  }
}

// 任务管理
{
  "tool": "TodoWrite",
  "arguments": {
    "todos": [
      {
        "content": "实现新功能",
        "status": "in_progress",
        "active_form": "正在实现新功能的核心逻辑"
      }
    ]
  }
}
```

## 重要实现细节

### 原子文件操作
- 使用 `tempfile::NamedTempFile` 在目标文件同目录创建临时文件
- 写入完成后调用 `persist()` 进行原子重命名操作
- 失败时自动降级到标准 `tokio::fs::write` 操作

### 并发控制实现
```rust
// 在 FileBashToolsService 结构体中
file_semaphore: Arc<Semaphore>

// 使用示例
let permit = self.file_semaphore.clone().acquire_owned().await?;
// ... 文件操作 ...
drop(permit); // 释放许可
```

### 搜索性能优化
- 根据输出模式动态调整搜索深度
- 大文件自动跳过 (>10MB)
- 支持上下文行显示和多行匹配
- 内置 glob 和文件类型过滤

## Cursor 和 Copilot 规则

### 项目特点
本项目是一个企业级的 Windows 文件工具 MCP 服务器，具有以下特点：
- 专为 Windows 环境优化
- 使用 Rust + RMCP SDK
- 高性能文件操作和搜索
- 企业级安全性和并发控制
- 原子文件操作确保数据完整性

### 推荐的 Cursor 规则配置
如果使用 Cursor IDE，建议配置以下规则：
- 优先使用 `&str` 而非 `String` 作为函数参数
- 使用 `Result<T, E>` 进行错误处理，避免 `unwrap()`
- 实现常见trait：`Debug`, `Clone`, `PartialEq`
- Windows 路径格式特殊处理（双反斜杠）
- 原子文件操作优先，失败时自动降级

### Copilot 指导原则
- 使用 RMCP 宏系统实现 MCP 工具
- 严格的输入验证和路径安全检查
- 企业级日志记录，包含操作ID和性能信息
- 使用 `Arc<Semaphore>` 进行并发控制
- 遵循 Windows 文件系统最佳实践