# Viaduct CI

## 👋 Welcome to Viaduct CI - Bridging Development to Deployment! 

👨‍💻 A modern, high-performance take on continuous integration and delivery pipeline server built with Rust and lots of ❤️. This project implements a scalable, efficient CI/CD system with a focus on performance, reliability, and developer experience.

![Viaduct CI](https://img.shields.io/badge/Viaduct-CI-blue)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)

## ✨ Features

- **Asynchronous Pipeline Execution**: Built on top of Actix-web and Tokio for maximum concurrency and performance
- **Efficient Job Management**: Smart job scheduling and execution with status tracking
- **Git Integration**: Native git support for repository cloning and pipeline configuration
- **Artifact Management**: Built-in artifact storage and management system
- **RESTful API**: Clean API design following REST principles
- **Database Performance**: Optimized SQLite schema with proper indexing for fast queries
- **CORS Support**: Built-in CORS configuration for seamless frontend integration
- **Stateless Design**: Modern stateless architecture for horizontal scalability

## 🛠 Technical Stack

- **Framework**: Actix-web 4.0
- **Runtime**: Tokio async runtime
- **Database**: SQLite with optimized indexes
- **Git Integration**: Native git2 bindings
- **Serialization**: Serde with YAML and JSON support
- **Error Handling**: Custom error types with proper propagation
- **DateTime Handling**: Chrono for UTC-aware timestamps
- **ID Generation**: UUID v4 for unique identifiers

## 🚀 Performance Features

### 1. Database Optimization
- Carefully designed schema with proper indexes
- Efficient queries with prepared statements
- Optimized table structure for common access patterns
- Index coverage for frequent query patterns

### 2. Concurrent Execution
- Asynchronous HTTP handlers
- Non-blocking I/O operations
- Parallel job execution
- Lock-free state management where possible

### 3. Resource Management
- Temporary directory cleanup
- Proper file handle management
- Connection pooling
- Memory-efficient artifact handling

## 🌐 API Endpoints

### Pipeline Management
- `POST /api/trigger` - Trigger a new pipeline build
- `GET /api/pipelines/{name}/status` - Get pipeline status

### Target Management
- `POST /api/targets` - Add a new target
- `GET /api/targets` - List all targets
- `GET /api/targets/{name}/pipeline` - Get target pipeline configuration

### Job Management
- Job status updates
- Artifact management
- Log retrieval

## 🏗 Architecture

```
src/
├── models/         # Data structures and types
├── handlers/       # HTTP request handlers
├── db/            # Database operations
└── utils/         # Utility functions
```

## Database Schema

### Pipeline Runs
- Tracks overall pipeline execution
- Stores metadata and progress
- Maintains execution history

### Job Runs
- Individual job execution tracking
- Status and output storage
- Performance metrics

### Artifacts
- Build artifacts storage
- Output preservation
- Asset management

## Modern Standards

### Code Organization
- Clear separation of concerns
- Modular architecture
- Type-safe implementations
- Error handling best practices

### Security
- Input validation
- Safe file operations
- Proper error propagation
- Environment-based configuration

### Performance
- Async/await patterns
- Efficient resource utilization
- Optimized database queries
- Proper connection management

## Configuration

The server can be configured using environment variables:
- `HOST`: Server host (default: "0.0.0.0")
- `PORT`: Server port (default: 8000)
- `WORKER_URL`: Worker service URL (default: "http://localhost:8080")

## 🚦 Getting Started

1. **Prerequisites**
   - Rust toolchain (latest stable)
   - SQLite3
   - Git

2. **Setup**
   ```bash
   # Clone the repository
   git clone <repository-url>

   # Build the project
   cargo build --release

   # Run the server
   cargo run --release
   ```

3. **First Pipeline**
   ```bash
   # Add a target
   curl -X POST http://localhost:8000/api/targets \
     -H "Content-Type: application/json" \
     -d '{"repository":"https://github.com/user/repo","branch":"main"}'

   # Trigger a build
   curl -X POST http://localhost:8000/api/trigger \
     -H "Content-Type: application/json" \
     -d '{"repository":"https://github.com/user/repo","branch":"main"}'
   ```

## Performance Considerations

- **Memory Usage**: Efficient handling of large artifacts and logs
- **CPU Utilization**: Parallel job execution with controlled resource usage
- **I/O Performance**: Async file operations and optimized database queries
- **Network Efficiency**: Proper connection pooling and request handling
- **Scalability**: Stateless design for horizontal scaling
- **Response Time**: Fast API responses with async handlers

## 🤝 Contributing

Contributions are welcome! Please follow these steps:
1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.
