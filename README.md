# Windows 文件工具 MCP 服务器

🏢 专为 Windows 环境打造的企业级文件和 Shell 工具 MCP 服务器

## 🚀 项目概述

这是一款使用 Rust 实现的高性能、安全的 MCP（模型上下文协议）服务器，专为 Windows 系统设计，提供强大的文件操作和 Shell 命令执行功能。

## 🎯 核心功能

### 📁 文件操作工具
- **写入 (Write)**: 向文件写入内容，支持自动创建目录
- **读取 (Read)**: 读取文件内容，支持偏移量和行数限制
- **编辑 (Edit)**: 替换文件中的文本，支持单次或全部替换

### 🖥️ Shell 工具
- **命令执行 (Bash)**: 执行 PowerShell/CMD 命令，具备安全验证
- **后台执行 (Background Execution)**: 在后台运行长时间任务
- **输出监控 (Output Monitoring)**: 实时获取后台任务输出

### 🔍 搜索工具
- **文件匹配 (Glob)**: 使用 glob 模式快速匹配文件
- **文本搜索 (Grep)**: 基于 ripgrep 的强大文本搜索功能

### 📋 任务管理
- **任务列表 (TodoWrite)**: 结构化任务列表管理工具

## 🏗️ 项目架构

```
src/
├── main.rs          # 服务器入口点
├── lib.rs           # 核心服务实现
├── handlers/        # 请求处理器
├── models/          # 数据结构定义
├── tools/           # 工具实现模块
│   ├── file_tools.rs    # 文件操作工具
│   ├── shell_tools.rs   # Shell 执行工具
│   └── search_tools.rs  # 搜索工具
└── utils/           # 通用工具函数
```

## 📦 核心依赖

- **RMCP**: 模型上下文协议 SDK
- **Tokio**: 异步运行时框架
- **Serde**: 序列化和反序列化框架
- **Ripgrep**: 高性能文本搜索引擎
- **Tracing**: 结构化日志框架

## 🚀 快速开始

### 系统要求
- Rust 1.70+
- Cargo 包管理器

### 编译构建
```bash
cargo build --release
```

### 运行服务器
```bash
cargo run --release
```

### 运行测试
```bash
cargo test
```

## 🔧 使用方法

### 使用 MCP 检查器
```bash
npx @modelcontextprotocol/inspector cargo run
```

### 作为标准输入输出服务器
服务器默认通过标准输入输出进行通信，兼容任何 MCP 客户端。

## 🛡️ 安全特性

- 路径验证：防止目录遍历攻击
- 命令验证：防止危险操作执行
- 输入清理：对所有操作进行输入验证
- 资源限制：防止资源滥用和拒绝服务

## 📚 相关文档

- [调试与测试指南](rust-debug_testing_guide.md)
- [标准输入输出工具指南](rust-stdio-tools.md)

## 🤝 参与贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 开源协议

本项目采用 MIT 协议开源

## 👥 核心团队

- **x1t** - 项目架构师 & 核心开发者

## 📧 技术支持

企业技术支持请联系：x1t@qq.com

如遇问题请通过 GitHub Issues 提交报告。