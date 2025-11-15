# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an enterprise-grade File Tools MCP (Model Context Protocol) Server implemented in Rust. It provides file operations capabilities through the MCP protocol, specifically designed for Windows environments.

## Key Features

- File operations: Write, Read, Edit
- File pattern matching: Glob
- Text search: Grep (powered by ripgrep)
- Task management: TodoWrite

## Common Development Commands

### Building the Project
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build with specific features if needed
cargo build --features
```

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test modulename::test_name

# Run tests with specific filter
cargo test test_filter_name

# Run single test
cargo test test_name
```

### Running the Server
```bash
# Debug mode
cargo run

# Release mode
cargo run --release

# With environment variables
RUST_LOG=debug cargo run
```

### Code Quality and Formatting
```bash
# Format code
cargo fmt

# Check code formatting without applying changes
cargo fmt -- --check

# Run clippy lints
cargo clippy

# Run clippy with all targets and features
cargo clippy --all-targets --all-features

# Check for unused dependencies
cargo machete
```

### Debugging with MCP Inspector
```bash
# Basic Inspector usage
npx @modelcontextprotocol/inspector cargo run

# Inspector with release build
npx @modelcontextprotocol/inspector cargo run --release

# Inspector with environment variables
RUST_LOG=debug npx @modelcontextprotocol/inspector cargo run
```

## Code Architecture

### Project Structure
```
src/
├── main.rs              # Server initialization and startup
├── lib.rs               # Core service implementation with all tools  
├── handlers/            # Request handlers (if separated)
│   ├── file_handler.rs  # File operation handlers
│   └── mod.rs          # Handler module exports
├── models/              # Data structure definitions
│   ├── file_ops.rs     # File operation request/response models
│   ├── search.rs       # Search-related models (Glob, Grep)
│   └── mod.rs          # Model module exports
├── tools/               # Core tool implementations
│   ├── file_tools.rs   # File operation tools (Write, Read, Edit)
│   ├── search_tools.rs # Search tools (Glob, Grep)
│   └── mod.rs          # Tool module exports
└── utils/               # Utility functions
    ├── fd_utils.rs     # File descriptor utilities
    ├── ripgrep_utils.rs # Ripgrep integration utilities
    └── mod.rs          # Utility module exports
```

### Main Components

1. **Main Entry Point** (`src/main.rs`): Server initialization, logging setup, and startup
2. **Core Service** (`src/lib.rs`): Main MCP service implementation with tool registration and routing
3. **Models** (`src/models/`): Request/response structures for all tools
4. **Tools** (`src/tools/`): Individual tool implementations (file, search operations)
5. **Utilities** (`src/utils/`): Helper functions and integrations (ripgrep, file operations)

### Key Patterns

1. **Tool Registration**: Uses `#[tool_router]` and `#[tool]` macros for automatic tool registration
2. **Parameter Handling**: Uses `Parameters<T>` wrapper for automatic parameter deserialization  
3. **Error Handling**: Uses `McpError` for consistent error responses across all tools
4. **Async Operations**: All tools are async functions using tokio runtime
5. **Modular Design**: Tools are organized into separate modules for maintainability

### Core Dependencies

- **RMCP**: MCP SDK with server, macros, and stdio transport features
- **Tokio**: Async runtime with multi-threading, filesystem, and process support
- **Serde + Schemars**: JSON serialization and schema generation for tool parameters
- **Ripgrep libraries**: High-performance text search (grep, regex, matcher, searcher)
- **Tracing**: Structured logging for debugging and monitoring
- **UUID**: Unique identifier generation for task tracking

### Data Structures

1. **WriteRequest**: File write operation parameters
2. **ReadRequest**: File read operation parameters
3. **EditRequest**: File edit operation parameters
4. **GlobRequest**: File pattern matching parameters
5. **GrepRequest**: Text search parameters
6. **TodoItem**: Task management item structure

## Development Guidelines

### Adding New Tools

1. Define a request structure implementing `serde::Deserialize` and `JsonSchema`
2. Add a new method in the `#[tool_router]` impl block with `#[tool]` attribute
3. Follow the existing patterns for parameter handling and error management

### Testing

1. Unit tests are included in each module under `#[cfg(test)]`
2. Use `cargo test` to run all tests
3. Use `cargo test -- --nocapture` to see test output
4. Use `cargo test test_name` to run specific tests
5. Integration tests can be added in a `tests/` directory
6. Follow existing test patterns for new functionality

### Performance Considerations

1. **File Operations**: Use async I/O and proper error handling
2. **Search Operations**: Ripgrep provides highly optimized text search
3. **Performance Considerations**: 
   - Use async I/O for file operations
   - Ripgrep provides highly optimized text search
   - Stream processing for large files when possible

### Error Handling

1. Use `McpError::invalid_params()` for parameter validation errors
2. Use `McpError::internal_error()` for internal failures
3. Always provide meaningful error messages

### Logging

1. Use `tracing::info!`, `tracing::debug!`, `tracing::warn!`, `tracing::error!` macros
2. Include relevant context in log messages
3. Use appropriate log levels (debug for detailed information, info for key events)

## Windows-Specific Considerations

1. **File Paths**: All file paths must be absolute (use `Path::is_absolute()` validation)
2. **Path Format**: Use double backslashes for Windows paths in documentation/examples  
3. **Security**: Path validation includes security checks for relative path traversal (`..`)

## Development Workflow

1. **Code Changes**: Make changes in appropriate modules
2. **Formatting**: Run `cargo fmt` to ensure consistent code style
3. **Linting**: Run `cargo clippy` to catch potential issues
4. **Testing**: Run `cargo test` to verify functionality
5. **Local Testing**: Use MCP Inspector for interactive testing: `npx @modelcontextprotocol/inspector cargo run`
6. **Build Verification**: Test both debug and release builds

## Environment Variables

- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug cargo run`)
- Default log level is INFO, configured in `src/main.rs`