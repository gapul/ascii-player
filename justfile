# ASCII Player Development Tasks

# Show all available tasks
default:
    @just --list

# Initialize development environment
init:
    nix develop
    
# Build the project
build:
    cargo build

# Build optimized release version
build-release:
    cargo build --release

# Run the project with a test video
run VIDEO_PATH:
    cargo run -- "{{VIDEO_PATH}}"

# Run with additional options
run-with-options VIDEO_PATH *ARGS:
    cargo run -- "{{VIDEO_PATH}}" {{ARGS}}

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Format code
fmt:
    cargo fmt

# Check code with clippy
clippy:
    cargo clippy -- -D warnings

# Check code formatting
check-fmt:
    cargo fmt --check

# Full quality check (format, clippy, test)
check: check-fmt clippy test

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Generate documentation
docs:
    cargo doc --open

# Install the binary locally
install:
    cargo install --path .

# Build with Nix
nix-build:
    nix build

# Enter development shell
nix-shell:
    nix develop

# Create a sample test video (requires ffmpeg)
create-test-video:
    #!/usr/bin/env bash
    echo "Creating test video..."
    mkdir -p tests/assets
    ffmpeg -f lavfi -i testsrc=duration=5:size=320x240:rate=10 -pix_fmt yuv420p tests/assets/sample.mp4 -y
    echo "Test video created: tests/assets/sample.mp4"

# Run performance benchmark
benchmark:
    cargo build --release
    hyperfine --warmup 3 'target/release/ascii-player tests/assets/sample.mp4'

# Git workflow commands
git-setup:
    git add .
    git commit -m "Initial commit"
    git push -u origin main

# Prepare for release
prepare-release:
    just check
    just build-release
    just test
    echo "✅ Ready for release"