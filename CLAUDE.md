# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Rust MCP SDK 实现的企业级文件和Shell工具MCP服务器，提供高性能的文件操作、搜索和Shell命令执行功能。

**主要特性:**
- 🚀 基于 tokio 异步运行时
- 🛡️ 类型安全的 Rust 实现
- 🔒 企业级安全机制
- 🖼️ 支持图像文件读取（Base64 编码）
- ⚡ 支持后台进程管理

## 常用开发命令

### 构建与运行
```bash
# 开发模式运行
cargo run

# 发布版本构建
cargo build --release

# 运行发布版本
./target/release/file-bash-tools-mcp
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行单元测试（项目内无集成测试）
cargo test --lib

# 查看测试覆盖率（需要先安装）
cargo install cargo-llvm-cov
cargo llvm-cov --lcov --output-path coverage.lcov
```

### 代码质量
```bash
# 代码格式化
cargo fmt

# 静态分析
cargo clippy --all-targets --all-features -- -D warnings

# 生成文档
cargo doc --open

# 检查依赖安全
cargo audit
```

## 代码架构

### 核心目录结构

```
src/
├── main.rs              # MCP服务器入口点，使用stdio传输
├── lib.rs               # 核心服务定义，包含主要工具实现和模型定义
├── models/              # 数据模型定义
│   ├── file_ops.rs      # 文件操作：WriteRequest/ReadRequest/EditRequest
│   ├── search.rs        # 搜索功能：GrepInput/GlobInput 及匹配结果
│   └── shell.rs         # Shell操作：BashInput/BashOutput/KillBashInput
├── tools/               # 工具实现层
│   ├── file_tools.rs    # 文件操作工具：Write/Read/Edit（支持图像文件）
│   ├── search_tools.rs  # 搜索工具：Grep（本地ripgrep库）、Glob（glob库）
│   └── shell_tools.rs   # Shell工具：Bash/BashOutput/KillBash（后台进程管理）
├── utils/               # 底层工具封装
│   ├── ripgrep_utils.rs # ripgrep核心库封装
│   └── fd_utils.rs      # 文件查找封装（带回退机制）
└── handlers/            # 处理器模块
    └── file_handler.rs  # 文件处理逻辑
```

### 关键设计模式

#### 1. **Tool Router 模式** (src/lib.rs:153-863)
```rust
#[tool_router]
impl FileBashToolsService {
    #[tool]
    async fn write(...)      // 写入文件
    #[tool]
    async fn read(...)       // 读取文件（支持图像）
    #[tool]
    async fn edit(...)       // 编辑文件
    #[tool]
    async fn grep(...)       // 文本搜索（ripgrep）
    #[tool]
    async fn glob(...)       // 文件模式匹配
    #[tool]
    async fn todo_write(...) // 任务管理
}
```
- 使用 `#[tool_router]` 宏自动生成MCP工具路由
- 每个方法前用 `#[tool]` 标记为工具

#### 2. **模型验证** (src/lib.rs:166-200)
- `validate_file_path()`: 绝对路径验证，防止路径遍历攻击
- `ensure_directory_exists()`: 自动创建父目录
- 命令安全性检查，防止危险操作

#### 3. **本地库集成** (src/utils/)
- `RipgrepWrapper`: 使用本地 ripgrep 核心库进行文本搜索
- `GlobWrapper`: 基于 glob 库的文件模式匹配
- 支持后台进程管理和输出捕获

### 依赖关系

**核心依赖:**
- `rmcp = { path = "rust-sdk/crates/rmcp" }` - MCP SDK (本地路径)
- `tokio` - 异步运行时
- `serde` + `schemars` - 序列化与JSON Schema生成
- `walkdir` + `ignore` - 文件系统遍历
- `regex` - 正则表达式支持
- `base64` - 图像文件编码
- `uuid` - 后台进程ID生成
- `glob` - 文件模式匹配

**本地ripgrep核心库:**
- `grep = { path = "ripgrep/crates/grep" }` - 核心搜索功能
- `grep-searcher = { path = "ripgrep/crates/searcher" }` - 搜索器实现
- `grep-regex = { path = "ripgrep/crates/regex" }` - 正则表达式支持
- `grep-matcher = { path = "ripgrep/crates/matcher" }` - 匹配器

**外部工具依赖:**
- `pwsh.exe` - PowerShell执行（内置于现代Windows）

## 工具使用指南

### 文件操作工具
- **Write**: 安全文件写入，支持自动目录创建
- **Read**: 智能文件读取，自动检测图像文件（jpg/png/gif/bmp/webp/svg）
- **Edit**: 精确文本替换，支持单次或全部替换

### 搜索工具
- **Grep**: 基于本地 ripgrep 核心库的强大文本搜索
  - 支持 content/files_with_matches/count 三种输出模式
  - 支持上下文行（-B/-A/-C）
  - 支持多行匹配和大小写忽略
- **Glob**: 基于 glob 库的高效文件模式匹配
  - 支持 glob 模式：`*.rs`, `src/**/*.ts`
  - 内置文件过滤和类型检测

### 任务管理工具
- **TodoWrite**: 任务列表管理
  - 支持待办事项的创建、更新和状态跟踪
  - 三种状态：Pending/InProgress/Completed
  - 任务描述和主动形式支持

### Shell工具（已移除）
- 项目曾支持Shell工具，但在最新版本中已移除以专注于文件操作和搜索功能

### 任务管理工具
- **TodoWrite**: 企业级任务管理系统
  - 使用 `Vec<TodoItem>` 存储任务列表
  - 支持任务状态实时更新
  - 提供任务统计信息展示

## 安全特性

1. **路径验证** (src/lib.rs:166-185)
   - 强制绝对路径
   - 阻止 `..` 路径遍历
   - 文件存在性检查

2. **模型验证** (src/lib.rs)
   - 使用 `JsonSchema` 进行参数验证
   - 详细的错误信息和日志记录

3. **错误处理** (src/lib.rs)
   - 使用 `anyhow::Result` 和 `rmcp::ErrorData`
   - 结构化错误信息
   - 完整的日志追踪

## 错误处理与日志

**日志系统**:
- 使用 `tracing` + `tracing-subscriber`
- 不同级别: `info`, `debug`, `warn`, `error`
- 输出到 stderr，无颜色编码

**错误类型**:
- `McpError::invalid_params` - 参数验证失败
- `McpError::internal_error` - 内部错误（文件操作失败等）

## 测试策略

当前项目使用单元测试验证核心功能：

**单元测试**:
- 模型验证测试：确保 JSON Schema 生成的正确性
- 文件操作测试：Write → Read → Edit 完整流程验证
- 搜索功能测试：基于本地 ripgrep 库的搜索验证
- 错误处理测试：各种异常情况的处理验证

**测试位置**:
- `src/lib.rs` 中包含内联测试（`#[cfg(test)]`）
- 使用 `cargo test --lib` 运行所有单元测试

**注意**: 项目当前专注于核心文件操作功能，暂无集成测试。

## 性能特性

1. **本地库集成**: 直接使用 ripgrep 核心库，避免外部进程调用开销
2. **异步处理**: 基于 tokio 的并发文件操作和搜索
3. **内存优化**: 流式处理大文件，避免一次性加载
4. **模式匹配**: 高效的 glob 模式匹配和文件过滤

## 配置与扩展

### 添加新工具
1. 在 `models/` 中定义输入/输出结构（使用 `JsonSchema`）
2. 在对应的 `tools/*.rs` 中实现业务逻辑
3. 在 `lib.rs` 中使用 `#[tool]` 标记方法
4. 使用 `#[tool_router]` 自动暴露工具

### 自定义验证
- 扩展 `validate_file_path()` 添加更多安全检查
- 在模型结构中添加自定义验证逻辑

### 本地库配置
- ripgrep 选项调整：`src/utils/ripgrep_utils.rs`
- glob 模式配置：`src/tools/search_tools.rs`

## 开发注意事项

1. **文件路径**: 所有文件操作要求绝对路径（Windows格式: `C:\path\to\file`）
2. **本地依赖**: ripgrep 核心库作为本地路径依赖，无需外部安装
3. **图像处理**: Read工具自动检测图像文件并返回Base64编码
4. **任务管理**: TodoWrite工具提供完整的任务生命周期管理
5. **日志系统**: 使用 tracing 进行结构化日志记录

## 许可证

MIT License - 详见项目根目录 LICENSE 文件
