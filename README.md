<div align="center">
  <h1>🔧 File-Bash-Tools MCP 服务器</h1>

<p>
    <strong>🏢 专为 Windows 环境打造的下一代企业级文件和命令工具 MCP 服务器</strong>
  </p>

<p>
    <a href="#-快速开始"><img src="https://img.shields.io/badge/状态-生产就绪-green" alt="Production Ready"></a>
    <a href="#-技术栈"><img src="https://img.shields.io/badge/Rust-1.70+-orange" alt="Rust Version"></a>
    <a href="https://github.com/x1t/windows-file-tools-mcp-rust"><img src="https://img.shields.io/badge/许可证-MIT-blue" alt="License"></a>
    <a href="https://github.com/x1t/windows-file-tools-mcp-rust/issues"><img src="https://img.shields.io/badge/问题-欢迎提出-yellow" alt="Issues Welcome"></a>
    <a href="https://img.shields.io/badge/版本-0.1.0--blue" alt="Version"></a>
  </p>
</div>

## 📋 目录

- [🚀 项目概述](#-项目概述)
- [✨ 核心功能](#-核心功能)
- [🏗️ 项目架构](#️-项目架构)
- [🛠️ 技术栈](#️-技术栈)
- [🚀 快速开始](#-快速开始)
- [📖 使用示例](#-使用示例)
- [🛡️ 安全特性](#️-安全特性)
- [📚 文档](#-文档)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)

---

## 🚀 项目概述

<div align="center">

  ![Rust](https://img.shields.io/badge/Rust-性能卓越-000000?style=flat-square&logo=rust)
  ![Windows](https://img.shields.io/badge/Windows-专属优化-0078D7?style=flat-square&logo=windows)
  ![MCP](https://img.shields.io/badge/MCP-协议兼容-9CF?style=flat-square)

</div>

这是一款使用 **Rust** 实现的高性能、安全的 **MCP（Model Context Protocol）** 服务器，专为 **Windows 系统**设计，提供强大的文件操作和搜索功能。

### 🌟 为什么选择我们？

- ⚡ **极致性能**: 基于 Rust 的零成本抽象和 Tokio 异步运行时
- 🔒 **企业级安全**: 多层安全验证，原子文件操作，防止路径遍历和资源滥用
- 🎯 **Windows 原生**: 专为 Windows 环境优化，支持原生路径格式和双反斜杠路径
- 📈 **高并发**: 智能并发控制，最多同时处理 10 个文件操作，使用 Semaphore 防止资源耗尽
- 🔍 **强大搜索**: 集成 ripgrep 核心库，支持复杂正则表达式和多种输出模式
- 🛡️ **原子操作**: 使用 NamedTempFile 确保文件写入的原子性，失败时自动降级
- 🧠 **智能优化**: 根据搜索模式动态调整深度，自动跳过大文件（>10MB）

---

## ✨ 核心功能

### 📁 文件操作工具

| 功能           | 描述           | 特性                                                                                                                                    |
| -------------- | -------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| **写入** | `write_file` | ✅ 自动创建目录 `<br>`✅ **原子写入操作**（NamedTempFile）`<br>`✅ 降级机制 `<br>`✅ 详细错误反馈 `<br>`✅ 完整性验证 |
| **读取** | `read_file`  | ✅ 按行读取 `<br>`✅ 偏移量和行数限制 `<br>`✅ 行号显示 `<br>`✅ 大文件自动跳过 `<br>`✅ 企业级日志记录               |
| **编辑** | `edit_file`  | ✅ **原子编辑操作**`<br>`✅ 单次/全部替换 `<br>`✅ 安全编辑 `<br>`✅ 变更统计 `<br>`✅ 失败自动回滚机制                 |

### 🔍 搜索工具

| 工具           | 功能     | 输出模式                                                                                                                                   |
| -------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| **Glob** | `glob` | 📁 文件模式匹配 `<br>`🎯 **/*.js 等复杂模式 `<br>`⚡ 高性能遍历 `<br>`🧠 **智能深度控制**（20层深度）                              |
| **Grep** | `grep` | 📝**content**: 显示匹配内容 `<br>`📂 **files_with_matches**: 文件列表 `<br>`🔢 **count**: 匹配统计 `<br>`⚡ **动态深度调整**（模式自适应） |

### 📋 任务管理

> **TodoWrite** - 结构化任务列表管理工具，支持状态跟踪和进度可视化

```rust
// 示例任务结构
{
  "content": "实现新的文件搜索功能",
  "status": "in_progress",
  "active_form": "正在实现文件搜索算法"
}
```

---

## 🏗️ 项目架构

<div align="center">

```mermaid
graph TB
    A[MCP 客户端] --> B[stdio 通信]
    B --> C[main.rs - 31行服务器入口]
    C --> D[lib.rs - 960行核心服务]
    D --> E[RMCP工具路由层]
    E --> F[文件操作工具]
    E --> G[搜索工具]
    E --> H[任务管理]
    F --> F1[write_file - 原子写入]
    F --> F2[read_file - 智能读取]
    F --> F3[edit_file - 原子编辑]
    G --> G1[glob - 模式匹配]
    G --> G2[grep - ripgrep搜索]
    H --> H1[TodoWrite - 任务管理]
    I[Arc&lt;Semaphore&gt;] --> F
    I --> G
    I --> H
    J[NamedTempFile] --> F1
    J --> F3
```

</div>

### 📂 目录结构

```
file-bash-tools-mcp/
├── 📄 src/
│   ├── 🚀 main.rs              # 31行 - 服务器入口点，日志和服务初始化
│   ├── ⚙️ lib.rs               # 960行 - 核心服务实现，所有MCP工具和数据结构
│   ├── 📂 models/              # 数据结构定义模块
│   │   ├── 📄 mod.rs           # 10行 - 模块导出
│   │   ├── 📄 file_ops.rs      # 102行 - 文件操作相关数据结构
│   │   └── 📄 search.rs        # 119行 - 搜索功能数据结构
│   ├── 📂 handlers/            # 请求处理器模块
│   │   ├── 📄 mod.rs           # 3行 - 模块导出
│   │   └── 📄 file_handler.rs  # 158行 - 文件请求处理器实现
│   ├── 📂 tools/               # 工具实现模块
│   │   ├── 📄 mod.rs           # 10行 - 模块导出
│   │   ├── 🔧 file_tools.rs    # 249行 - 文件操作工具实现
│   │   └── 🔍 search_tools.rs  # 169行 - 搜索工具实现
│   └── 📂 utils/               # 通用工具函数模块
│       ├── 📄 mod.rs           # 4行 - 模块导出
│       ├── 🔧 ripgrep_utils.rs # 6341行 - ripgrep 核心库工具
│       └── 📋 fd_utils.rs      # 7450行 - 文件描述工具
├── 📄 Cargo.toml               # 64行 - 项目配置和依赖管理
├── 📄 CLAUDE.md                # Claude开发指南，完整的项目文档
├── 📄 README.md                # 项目说明文档
└── 📚 .git/                    # Git版本控制
```

---

## 🛠️ 技术栈

<div align="center">

### 🏗️ 核心框架

| 组件 | 版本 | 用途 |
| --- | --- | --- |
| ![Rust](https://img.shields.io/badge/-Rust-000000?style=flat-square&logo=rust) | 1.70+ | 核心编程语言 |
| ![Tokio](https://img.shields.io/badge/-Tokio-0057B7?style=flat-square) | 1.42 | 异步运行时 |
| ![RMCP](https://img.shields.io/badge/-RMCP-9CF?style=flat-square) | 0.8.5 | MCP SDK |

### 📦 序列化和验证

| 组件 | 版本 | 用途 |
| --- | --- | --- |
| ![Serde](https://img.shields.io/badge/-Serde-DEA584?style=flat-square) | 1.0 | 序列化/反序列化 |
| ![Schemars](https://img.shields.io/badge/-Schemars-8E44AD?style=flat-square) | 1.0 | JSON Schema生成 |
| ![Serde JSON](https://img.shields.io/badge/-Serde%20JSON-292929?style=flat-square) | 1.0 | JSON处理 |

### 🔍 文件系统和搜索

| 组件 | 版本 | 用途 |
| --- | --- | --- |
| ![Ripgrep](https://img.shields.io/badge/-Ripgrep-F14E32?style=flat-square) | Core | 高性能文本搜索 |
| ![Walkdir](https://img.shields.io/badge/-Walkdir-4CAF50?style=flat-square) | 2.5 | 文件系统遍历 |
| ![Glob](https://img.shields.io/badge/-Glob-FF9800?style=flat-square) | 0.3 | 文件模式匹配 |
| ![Tempfile](https://img.shields.io/badge/-Tempfile-2196F3?style=flat-square) | 3.0 | 原子文件操作 |

### ⚡ 性能和工具

| 组件 | 版本 | 用途 |
| --- | --- | --- |
| ![Regex](https://img.shields.io/badge/-Regex-9C27B0?style=flat-square) | 1.11 | 正则表达式 |
| ![UUID](https://img.shields.io/badge/-UUID-607D8B?style=flat-square) | 1.11 | 唯一标识符 |
| ![Base64](https://img.shields.io/badge/-Base64-795548?style=flat-square) | 0.22 | Base64编码 |
| ![Thiserror](https://img.shields.io/badge/-Thiserror-F44336?style=flat-square) | 2.0 | 结构化错误 |

### 📝 日志和错误处理

| 组件 | 版本 | 用途 |
| --- | --- | --- |
| ![Tracing](https://img.shields.io/badge/-Tracing-555555?style=flat-square) | 0.1 | 结构化日志 |
| ![Tracing Subscriber](https://img.shields.io/badge/-Tracing%20Subscriber-37474F?style=flat-square) | 0.3 | 日志输出 |
| ![Anyhow](https://img.shields.io/badge/-Anyhow-FF5722?style=flat-square) | 1.0 | 错误上下文 |

</div>

---

## 🚀 快速开始

### 📋 系统要求

- **🦀 Rust** 1.70+
- **📦 Cargo** 包管理器
- **🖥️ Windows** 操作系统

### 🔧 安装与构建

<details>
<summary>📦 克隆仓库</summary>

```bash
git clone https://github.com/x1t/windows-file-tools-mcp-rust.git
cd windows-file-tools-mcp-rust
```

</details>

<details>
<summary>🔨 构建项目</summary>

```bash
# 开发模式构建
cargo build

# 生产模式构建（推荐）
cargo build --release
```

</details>

<details>
<summary>🚀 运行服务器</summary>

```bash
# 开发模式运行
cargo run

# 生产模式运行（推荐）
cargo run --release
```

</details>

<details>
<summary>🧪 运行测试</summary>

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_glob_pattern_matching

# 显示测试详细输出
cargo test -- --nocapture
```

</details>

---

## 📖 使用示例

### 🔧 文件操作

<details>
<summary>📝 写入文件</summary>

```json
{
  "tool": "write_file",
  "arguments": {
    "file_path": "C:\\\\Temp\\\\example.txt",
    "content": "Hello, Windows MCP Server! 🎉"
  }
}
```

</details>

<details>
<summary>📖 读取文件</summary>

```json
{
  "tool": "read_file",
  "arguments": {
    "file_path": "C:\\\\Temp\\\\example.txt",
    "offset": 1,
    "limit": 10
  }
}
```

</details>

<details>
<summary>✏️ 编辑文件</summary>

```json
{
  "tool": "edit_file",
  "arguments": {
    "file_path": "C:\\\\Temp\\\\example.txt",
    "old_string": "Hello",
    "new_string": "Hi",
    "replace_all": true
  }
}
```

</details>

### 🔍 搜索功能

<details>
<summary>🎯 Glob 文件匹配</summary>

```json
{
  "tool": "glob",
  "arguments": {
    "pattern": "**/*.rs",
    "path": "C:\\\\Projects\\\\my-rust-app"
  }
}
```

</details>

<details>
<summary>🔎 Grep 文本搜索</summary>

```json
{
  "tool": "grep",
  "arguments": {
    "pattern": "async fn.*test",
    "path": "src/",
    "output_mode": "content",
    "case_insensitive": true,
    "show_line_numbers": true,
    "before_context": 2,
    "after_context": 2
  }
}
```

</details>

### 📋 任务管理

<details>
<summary>✅ TodoWrite 任务管理</summary>

```json
{
  "tool": "TodoWrite",
  "arguments": {
    "todos": [
      {
        "content": "实现文件搜索功能",
        "status": "completed",
        "active_form": "已完成文件搜索功能实现"
      },
      {
        "content": "添加并发控制",
        "status": "in_progress", 
        "active_form": "正在实现 Semaphore 并发控制"
      },
      {
        "content": "编写单元测试",
        "status": "pending",
        "active_form": "待编写单元测试"
      }
    ]
  }
}
```

</details>

---

### 🔌 MCP 客户端集成

#### 使用 MCP Inspector

```bash
# 启动交互式 MCP 测试环境
npx @modelcontextprotocol/inspector cargo run --release
```

#### 在 Claude Desktop 中使用

```json
{
  "mcpServers": {
    "file-bash-tools": {
      "command": "cargo",
      "args": ["run", "--release", "--manifest-path", "H:/mcp/windows-file-tools-mcp-rust/Cargo.toml"]
    }
  }
}
```

> **注意**: 请将 `H:/mcp/windows-file-tools-mcp-rust/Cargo.toml` 替换为您的实际项目路径

---

## 🛡️ 安全特性

<div align="center">

| 安全特性               | 描述             | 实现方式               |
| ---------------------- | ---------------- | ---------------------- |
| 🔒**路径验证**   | 防止目录遍历攻击 | 严格的绝对路径检查     |
| 🛡️**输入清理** | 防止恶意输入     | 全面的输入验证和清理   |
| ⚡**资源限制**   | 防止资源滥用     | 并发控制和文件大小限制 |
| 🚫**权限控制**   | 最小权限原则     | 仅必要的文件系统访问   |
| 📝**审计日志**   | 完整的操作记录   | 结构化日志记录         |

</div>

### 🎯 性能优化亮点

- **🔄 智能并发控制**: 使用 `Arc<Semaphore>` 最多同时处理 10 个文件，防止资源耗尽
- **📏 动态深度调整**: 根据搜索模式智能调整遍历深度
  - 文件匹配模式：**20层**深度
  - 计数模式：**30层**深度  
  - 默认模式：**50层**深度
- **💾 大文件跳过**: 自动跳过超过 **10MB** 的大文件
- **⚡ 缓存机制**: 路径验证结果缓存，提升重复操作性能
- **🛡️ 原子操作**: 使用 `NamedTempFile` 确保文件操作的原子性和数据完整性
- **🔄 降级机制**: 原子操作失败时自动回退到标准文件操作
- **🧠 Windows优化**: 专门针对Windows路径格式（双反斜杠）和文件系统特性优化

---

## 📚 文档

<div align="center">

| 文档                 | 描述                 | 链接                                                    |
| -------------------- | -------------------- | ------------------------------------------------------- |
| 📖**用户指南** | 完整的使用说明和示例 | [CLAUDE.md](CLAUDE.md)                                     |
| 🐛**调试指南** | 问题诊断和调试技巧   | [rust-debug_testing_guide.md](rust-debug_testing_guide.md) |
| 🔧**工具指南** | stdio 工具详细说明   | [rust-stdio-tools.md](rust-stdio-tools.md)                 |

</div>

---

## 🤝 贡献

<div align="center">

### 🌟 我们欢迎所有形式的贡献！

</div>

#### 🚀 快速贡献流程

```bash
# 1. Fork 并克隆仓库
git clone https://github.com/YOUR_USERNAME/windows-file-tools-mcp-rust.git
cd windows-file-tools-mcp-rust

# 2. 创建功能分支
git checkout -b feature/amazing-feature

# 3. 提交更改
git commit -m '✨ feat: 添加超棒的新功能'

# 4. 推送分支
git push origin feature/amazing-feature

# 5. 创建 Pull Request
```

#### 🧪 测试要求
- **无警告**: 确保所有测试通过，无任何警告信息
- **原子操作**: 所有文件操作必须支持原子性
- **错误处理**: 实现完整的错误处理和降级机制
- **性能验证**: 大文件处理和并发操作性能测试
- **安全检查**: 路径验证和输入清理测试

#### 📝 代码规范
- **Rust最佳实践**: 使用 `cargo fmt` 和 `cargo clippy`
- **文档完整**: 所有公共API必须有完整的rustdoc注释
- **原子操作优先**: 优先使用原子文件操作，失败时自动降级
- **Windows兼容**: 确保所有路径处理支持Windows双反斜杠格式
- **企业级日志**: 使用 `tracing` 记录操作ID和性能信息

#### 📋 贡献指南

- 🔍 **Bug 报告**: 请使用 Issue 模板提供详细的重现步骤
- 💡 **功能请求**: 描述使用场景和期望的行为
- 📝 **文档改进**: 修正错别字、改进示例或添加新内容
- 🧪 **测试**: 添加测试用例或改进现有测试
- 🎨 **代码风格**: 遵循 Rust 官方代码规范

---

## 🏆 致谢

<div align="center">

感谢所有为这个项目做出贡献的开发者和用户！ 🙏

特别感谢：

- **MCP 协议团队** - 提供优秀的协议规范和RMCP SDK
- **Rust 社区** - 提供强大的生态系统和工具链
- **ripgrep 团队** - 提供高性能搜索核心库
- **Tokio 团队** - 提供出色的异步运行时
- **Tempfile 维护者** - 提供原子文件操作支持

### 🌟 核心技术致谢

本项目的企业级特性得益于以下优秀的开源技术：

- **RMCP 0.8.5**: 让MCP服务器开发变得简单高效
- **ripgrep核心**: 为我们提供世界级的文本搜索能力
- **Tokio异步**: 让高并发文件操作成为可能
- **原子操作**: 确保数据完整性和企业级可靠性

</div>

---

## 📄 许可证

<div align="center">

本项目采用 **MIT 许可证** 开源

![MIT License](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)

</div>

---

## 👥 核心团队

<div align="center">

| 角色                     | 成员          | 联系方式                         |
| ------------------------ | ------------- | -------------------------------- |
| 🏗️**项目架构师** | **x1t** | [📧 x1t@qq.com](mailto:x1t@qq.com)  |
| 💻**核心开发者**   | **x1t** | [🐙 GitHub](https://github.com/x1t) |

</div>

---

## 📞 技术支持

<div align="center">

### 💬 获得帮助

| 渠道                      | 类型         | 响应时间   |
| ------------------------- | ------------ | ---------- |
| 📧**邮箱支持**      | 企业技术支持 | 24-48 小时 |
| 🐛**GitHub Issues** | 公开问题跟踪 | 1-3 天     |
| 📚**文档**          | 自助服务     | 即时       |

</div>

<div align="center">

---

**⭐ 如果这个项目对你有帮助，请给我们一个 Star！**

---

</div>
