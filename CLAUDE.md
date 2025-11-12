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
# 开发模式运行（自动重载）
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

# 运行特定测试
cargo test test_file_operations

# 运行集成测试
cargo test --test integration_tests

# 查看测试覆盖率
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

# 检查依赖
cargo audit
```

## 代码架构

### 核心目录结构

```
src/
├── main.rs              # MCP服务器入口点，使用stdio传输
├── lib.rs               # 核心服务定义，包含主要工具实现（Write/Read/Edit/Bash）
├── models/              # 数据模型定义
│   ├── file_ops.rs      # 文件操作：WriteInput/ReadInput/EditInput 及输出结构
│   ├── search.rs        # 搜索功能：GrepInput/GlobInput 及匹配结果
│   └── shell.rs         # Shell操作：BashInput/BashOutput/KillBashInput
├── tools/               # 工具实现层
│   ├── file_tools.rs    # 文件操作工具：Write/Read/Edit（支持图像文件）
│   ├── search_tools.rs  # 搜索工具：Grep（ripgrep）、Glob（fd）
│   └── shell_tools.rs   # Shell工具：Bash/BashOutput/KillBash（后台进程管理）
├── utils/               # 底层工具封装
│   ├── ripgrep_utils.rs # ripgrep命令封装（JSON输出解析）
│   └── fd_utils.rs      # fd命令封装（带回退机制）
└── handlers/            # 处理器（可能用于文件处理逻辑）
```

### 关键设计模式

#### 1. **Tool Router 模式** (src/lib.rs:124-296)
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
    async fn bash(...)       // 执行PowerShell命令
}
```
- 使用 `#[tool_router]` 宏自动生成MCP工具路由
- 每个方法前用 `#[tool]` 标记为工具

#### 2. **模型验证** (src/lib.rs:89-121)
- `validate_file_path()`: 绝对路径验证，防止路径遍历攻击
- `ensure_directory_exists()`: 自动创建父目录
- 命令安全性检查，防止危险操作

#### 3. **异步工具封装** (src/utils/)
- `RipgrepWrapper`: 将 ripgrep 的JSON输出转换为结构化数据
- `FdWrapper`: 提供 fd 搜索，并带 globset 回退机制
- 支持后台进程管理和输出捕获

### 依赖关系

**核心依赖:**
- `rmcp = { path = "../rust-sdk/crates/rmcp" }` - MCP SDK
- `tokio` - 异步运行时
- `serde` + `schemars` - 序列化与JSON Schema生成
- `walkdir` + `ignore` - 文件系统遍历
- `regex` - 正则表达式支持
- `base64` - 图像文件编码
- `uuid` - 后台进程ID生成

**外部工具依赖:**
- `ripgrep (rg)` - 文本搜索（Windows: `winget install BurntSushi.ripgrep`）
- `fd` - 文件查找（Windows: `winget install sharkdp.fd`）
- `pwsh.exe` - PowerShell执行（内置于现代Windows）

## 工具使用指南

### 文件操作工具
- **Write**: 安全文件写入，支持自动目录创建
- **Read**: 智能文件读取，自动检测图像文件（jpg/png/gif/bmp/webp/svg）
- **Edit**: 精确文本替换，支持单次或全部替换

### 搜索工具
- **Grep**: 基于 ripgrep 的强大文本搜索
  - 支持 content/files_with_matches/count 三种输出模式
  - 支持上下文行（-B/-A/-C）
  - 支持多行匹配和大小写忽略
- **Glob**: 基于 fd 的高效文件模式匹配
  - 支持 glob 模式：`*.rs`, `src/**/*.ts`
  - 带 fd 命令回退机制

### Shell工具
- **Bash**: PowerShell命令执行，支持前台/后台模式
- **BashOutput**: 获取后台进程输出和状态
- **KillBash**: 终止后台进程

**后台进程管理** (src/tools/shell_tools.rs:15-25):
- 使用 `Arc<Mutex<HashMap<String, BackgroundShell>>>` 存储进程
- 每个进程有唯一 UUID 标识
- 支持自动清理已完成的进程

## 安全特性

1. **路径验证** (src/lib.rs:89-108)
   - 强制绝对路径
   - 阻止 `..` 路径遍历
   - 文件存在性检查

2. **命令安全** (src/tools/shell_tools.rs:64-85)
   - 危险命令检测（format/del/rmdir 等）
   - 超时控制（默认30秒，最大10分钟）

3. **错误处理** (src/lib.rs)
   - 使用 `anyhow::Result` 和 `rmcp::ErrorData`
   - 结构化错误信息
   - 详细日志记录

## 错误处理与日志

**日志系统**:
- 使用 `tracing` + `tracing-subscriber`
- 不同级别: `info`, `debug`, `warn`, `error`
- 输出到 stderr，无颜色编码

**错误类型**:
- `McpError::invalid_params` - 参数验证失败
- `McpError::internal_error` - 内部错误（文件操作失败等）

## 测试策略

集成测试位于 `tests/integration_tests.rs`:
- 文件操作测试：Write → Read → Edit 完整流程
- 搜索操作测试：需要系统安装 rg 和 fd
- Bash操作测试：PowerShell命令执行
- 模型Schema测试：验证JSON Schema生成

**注意**: 搜索测试依赖外部工具（rg, fd），在CI环境中需要先安装。

## 性能特性

1. **并发处理**: 使用 tokio 多线程异步运行时
2. **流式处理**: 大文件分块读取
3. **进程管理**: 后台进程轻量级跟踪
4. **工具缓存**: ripgrep/fd 包装器可复用

## 配置与扩展

### 添加新工具
1. 在 `models/` 中定义输入/输出结构
2. 在对应的 `tools/*.rs` 中实现业务逻辑
3. 在 `lib.rs` 中使用 `#[tool]` 标记方法
4. 使用 `#[tool_router]` 暴露工具

### 自定义验证
- 扩展 `validate_file_path()` 添加更多安全检查
- 在 `ShellTools::validate_command()` 中添加危险命令列表

### 外部工具配置
- ripgrep 选项调整：`src/utils/ripgrep_utils.rs:124-160`
- fd 回退机制：`src/utils/fd_utils.rs:80-124`

## 开发注意事项

1. **文件路径**: 所有文件操作要求绝对路径（Windows格式: `C:\path\to\file`）
2. **PowerShell**: 仅支持 `pwsh.exe`，不支持传统 `cmd.exe`
3. **外部工具**: 确保 `rg` 和 `fd` 在系统 PATH 中
4. **图像处理**: Read工具自动检测图像文件并返回Base64编码
5. **后台进程**: Bash工具的 `run_in_background` 选项需要配合 `BashOutput` 使用

## 许可证

MIT License - 详见项目根目录 LICENSE 文件
