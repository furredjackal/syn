# SYN - Simulate Your Narrative

A life simulation game built with Rust and Flutter.

## Project Structure

- `rust/` - Rust simulation core
- `flutter/` - Flutter UI frontend
- `tests/` - Cross-language integration tests
- `docs/` - Documentation

## Getting Started

### Prerequisites

- Rust (latest stable)
- Flutter SDK
- Flutter Rust Bridge CLI

### Setup

1. Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

1. Install Flutter Rust Bridge:
```bash
cargo install flutter_rust_bridge_codegen
```

1. Build Rust library:
```bash
cd rust/syn_api
cargo build --release
```

1. Run Flutter app:
```bash
cd flutter
flutter run
```

## Testing

### Rust Tests
```bash
cd rust
cargo test
```

### Flutter Tests
```bash
cd flutter
flutter test
```

### Integration Tests
```bash
cd tests
cargo test
```

## License

TBD
