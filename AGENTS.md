# Repository Guidelines

## Project Structure & Module Organization

本项目采用标准的 Rust 项目结构：

```
windows-file-tools-mcp-rust/
├── src/                    # 源代码目录
│   ├── main.rs            # 服务器入口点
│   ├── lib.rs             # 核心服务实现
│   ├── handlers/          # 请求处理器模块
│   ├── tools/             # MCP工具实现
│   ├── models/            # 数据结构定义
│   └── utils/             # 工具函数库
├── test_files/            # 测试文件存储目录
├── Cargo.toml            # 项目配置和依赖
└── README.md             # 项目文档
```

**模块职责：**
- `handlers/`：处理 MCP 请求路由和验证
- `tools/`：实现具体的文件操作工具（write_file, read_file, edit_file, glob, grep）
- `models/`：定义请求/响应数据结构
- `utils/`：提供文件系统操作和 ripgrep 集成等辅助功能

## Build, Test, and Development Commands

### 基础构建命令
```bash
cargo build              # 构建项目
cargo build --release    # 发布模式构建
cargo run               # 运行开发版本
cargo run --release     # 运行发布版本
```

### 开发调试命令
```bash
cargo check              # 快速语法检查
cargo fmt               # 格式化代码
cargo clippy            # 代码质量检查
```

### 测试命令
```bash
cargo test              # 运行所有测试
cargo test -- --nocapture  # 测试并显示输出
```

## Coding Style & Naming Conventions

### 代码风格
- 使用 `rustfmt` 进行代码格式化
- 遵循 Rust 官方命名约定：
  - 函数和变量：`snake_case`
  - 类型名和结构体：`PascalCase`
  - 常量：`SCREAMING_SNAKE_CASE`
- 使用 `tracing` 进行日志记录，级别：`info!`, `error!`, `debug!`

### 文件组织
- 每个模块都应有 `mod.rs` 文件进行导出
- 工具实现放在 `tools/` 目录，使用描述性命名如 `file_tools.rs`
- 数据模型放在 `models/` 目录，按功能分组如 `file_ops.rs`

### 代码质量
- 使用 `thiserror` 进行错误处理
- 异步函数使用 `Result<T>` 返回类型
- 所有公共函数必须有完整的文档注释

## Testing Guidelines

### 测试框架
使用 Rust 内置的 `#[cfg(test)]` 测试框架和 `tokio::test` 进行异步测试。

### 测试文件
- 单元测试嵌入在源文件末尾
- 集成测试文件放在 `test_files/` 目录
- 测试数据使用相对路径引用

### 测试命名
```rust
#[tokio::test]
async fn test_write_file_creates_file() {
    // 测试实现
}

#[tokio::test]
async fn test_search_with_regex() {
    // 测试实现
}
```

### 测试覆盖率
- 每个工具函数都应有对应的单元测试
- 关键路径（文件操作、安全验证）需要完整测试覆盖

## Commit & Pull Request Guidelines

### 提交信息格式
```
[类型] 简短描述

详细说明（可选）

- 具体改动点1
- 具体改动点2
```

**提交类型：**
- `[功能]` 新功能实现
- `[优化]` 性能优化或重构
- `[修复]` 错误修复
- `[文档]` 文档更新
- `[测试]` 测试相关

### Pull Request 要求
1. **标题**：清晰描述变更内容
2. **描述**：包含变更原因和实现方法
3. **测试**：确保所有测试通过
4. **代码检查**：通过 `cargo fmt` 和 `cargo clippy`
5. **文档**：更新相关文档和注释

### 分支管理
- 主分支：`main`
- 功能分支：使用描述性名称如 `feature/file-atom-write`
- 提交前先 `rebase` 主分支保持历史干净

## Security & Configuration Tips

### 安全要求
- 所有文件路径必须进行路径遍历检查
- 使用 `tempfile` 实现原子文件操作
- 限制并发操作数量，避免资源耗尽

### 开发配置
- 设置 `RUST_LOG=info` 环境变量查看详细日志
- 使用 `cargo watch` 进行开发时自动重编译
- 推荐使用 VS Code 配合 rust-analyzer 插件

## Agent-Specific Instructions

### MCP 工具开发
- 实现新工具时需在 `tools/mod.rs` 中注册
- 工具函数使用 `#[tool]` 宏标记
- 参数验证使用 `schemars` 进行 JSON Schema 生成

### 性能优化
- 文件操作使用异步 I/O（`tokio::fs`）
- 搜索功能集成 ripgrep 核心库提升性能
- 实现并发控制限制同时操作数量