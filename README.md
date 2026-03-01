# picrs

A high-performance gRPC-based image processing service built with Rust.

## Features

- **gRPC API** - Protocol buffer-based service interface
- **Multiple Format Support** - JPEG, PNG, WebP
- **Image Resizing** - Lanczos3 filter for high-quality scaling
- **Configurable Quality** - Adjustable JPEG compression quality
- **Async Runtime** - Built on Tokio for concurrent processing

## Project Structure

```
picrs/
├── proto/
│   └── service.proto      # Protocol buffer definitions
├── src/
│   ├── main.rs            # gRPC service implementation
│   ├── errors.rs          # Error types and handling
│   └── processor/
│       └── mod.rs         # Image processing logic
├── build.rs               # Protocol buffer compilation
├── Cargo.toml             # Rust dependencies
└── README.md
```

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/TheBorMachine/picrs.git
cd picrs

# Build the project
cargo build

# Run tests
cargo test

# Run the server
cargo run
```

## Usage

The service starts a gRPC server on `[::1]:50051` by default.

### Proto Definition

```protobuf
service ImageProcessor {
  rpc ProcessImage (ProcessRequest) returns (ProcessResponse);
}

message ProcessRequest {
  bytes image_data = 1;
  string format = 2;
  uint32 quality = 3;
  repeated uint32 widths = 4;
  repeated uint32 heights = 5;
}

message ProcessResponse {
  string original_path = 1;
  repeated string processed_paths = 2;
}
```

### Example Client

```rust
// Connect to the service and send image processing requests
// See proto/service.proto for full API specification
```

## Configuration

The `ProcessorConfig` struct allows customization:

```rust
pub struct ProcessorConfig {
    pub base_dir: PathBuf,      // Base directory for output files
    pub default_quality: u8,    // Default JPEG quality (0-100)
}
```

## Error Handling

The service uses typed errors for better error handling:

- `InvalidFormat` - Unsupported image format
- `ImageLoadError` - Failed to decode image
- `ImageSaveError` - Failed to save image
- `InvalidParameters` - Invalid processing parameters
- `InternalError` - Internal service errors

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Dependencies

- [`tonic`](https://crates.io/crates/tonic) - gRPC implementation
- [`image`](https://crates.io/crates/image) - Image processing
- [`tokio`](https://crates.io/crates/tokio) - Async runtime
- [`prost`](https://crates.io/crates/prost) - Protocol buffers
- [`thiserror`](https://crates.io/crates/thiserror) - Error handling
