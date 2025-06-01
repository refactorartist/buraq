---
trigger: always_on
---

# Rust Async Web Development Guide

## Project Setup and Workflow

### Development Commands
- Run tests: `devbox run tests`
- Start application: `devbox run app`
- Check code: `devbox run cargo check`
- Run linter: `devbox run cargo clippy`
- Run linter with fix: `cargo clippy --fix --lib -p buraq`

### Prerequisites
- Rust
- Tokio
- Actix-web
- MongoDB
- Devbox

## Project Structure
```
project_root/
|
|-- src/
|     |
|     |-- main.rs
|     |-- lib.rs
|     |-- routes/
|     |-- models/
|     |-- repositories/
|     |-- services/
|
|-- tests/
|     |-- integration/
|     |-- unit/
|
|-- Cargo.toml
|-- devbox.json
|-- README.md
```

## Testing Workflow

### Run All Tests
```bash
devbox run tests
```

### Run Specific Test Module
```bash
devbox run tests -- --test integration_tests
```

### Run with Verbose Output
```bash
devbox run tests -- --nocapture
```

## Key Development Principles

### Async Programming
- Use `tokio` for async runtime
- Implement async functions with `async fn`
- Leverage `tokio::spawn` for concurrency

### Error Handling
- Use `Result` for fallible operations
- Implement comprehensive error types
- Provide meaningful error messages

### Database Interactions
- Use `mongodb` with async support
- Implement repository pattern
- Use connection pooling
- Leverage `serde` for serialization

## Recommended Ecosystem
- HTTP Server: Actix-web
- Database: MongoDB
- Serialization: Serde
- Logging: Tracing
- Configuration: Config crate

## Performance Optimization
- Minimize async overhead
- Implement connection pooling
- Optimize database queries
- Use efficient serialization

## Security Considerations
- Input validation
- Secure MongoDB connections
- Implement authentication
- Use HTTPS
- Protect against web vulnerabilities

## Best Practices
1. Modular application structure
2. Clear separation of concerns
3. Comprehensive logging
4. Environment-based configuration
5. Thorough testing

## Continuous Integration
Ensure all tests pass:
```bash
devbox run tests
```
cargo test
Start application:
```bash
devbox run app
```

## Keyboard Shortcuts for Productivity
- `F5` - Start debugging
- `Shift+F5` - Stop debugging
- `Ctrl+Space` - Trigger code completion
- `F12` - Go to definition
- `Alt+F12` - Peek definition
- `Shift+Alt+F` - Format document

## Common Commands Reference
```bash
# Build the project
devbox cargo build

# Run the project
devbox run app

# Run tests
devbox run tests

# Check code without building
devbox run cargo check

# Run linter
devbox run cargo clippy

# Format code
devbox run cargo fmt
```
