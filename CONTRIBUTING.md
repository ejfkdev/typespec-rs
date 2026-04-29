# Contributing to typespec-rs

Thank you for your interest in contributing to typespec-rs! This document provides guidelines for contributing.

## How to Contribute

1. **Fork** the repository
2. **Create a branch** for your feature or bugfix
3. **Make your changes** with clear, well-documented code
4. **Test your changes** — ensure all existing tests pass and add new tests for new functionality
5. **Submit a pull request** with a clear description of the changes

## Development Setup

```bash
# Build the project
cargo build

# Run all tests
cargo test --lib

# Run specific test modules
cargo test --lib checker::model_tests

# Run clippy
cargo clippy

# Format code
cargo fmt

# Generate documentation
cargo doc --open
```

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- Resolve all clippy warnings (`cargo clippy`)
- Add `///` doc comments to public APIs
- Use `expect()` instead of `unwrap()` in production code with descriptive messages

## Commit Messages

- Use clear, descriptive commit messages
- Prefix with module area when applicable (e.g., `fix(checker): ...`, `feat(parser): ...`)

## Reporting Issues

- Use GitHub Issues to report bugs or request features
- Include minimal reproduction steps for bug reports

## Release Process

1. Update version in `Cargo.toml` (workspace) and `crates/tspc/Cargo.toml`
2. Update `CHANGELOG.md` with the new version entry
3. Commit and tag: `git tag v0.x.0`
4. Push the tag: `git push origin v0.x.0`
5. The `release.yml` workflow will build binaries with cargo-dist and create a GitHub Release
6. Publish to crates.io via the `publish.yml` workflow (manual dispatch)

### Pre-release Checklist

- [ ] `cargo test --lib` passes
- [ ] `cargo test -p tspc` passes
- [ ] `cargo test --examples` passes
- [ ] `cargo clippy` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] Version bumped in both `Cargo.toml` files
- [ ] `CHANGELOG.md` updated
- [ ] MSRV still supported (`cargo check --lib` with rust 1.85.0)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
