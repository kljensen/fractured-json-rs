# Development tasks for fractured-json-rs

# Default recipe
default:
    @just --list

# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Run clippy
lint:
    cargo clippy

# Build the dev container image
devbuild:
    docker build -f Dockerfile.dev -t fractured-json-dev .

# Run an interactive dev shell
devshell: devbuild
    docker run -it --rm \
        -v "$(pwd):/workspace" \
        -v "$HOME/.ssh:/home/dev/.ssh:ro" \
        -v "$HOME/.gitconfig:/home/dev/.gitconfig:ro" \
        -v "$HOME/.config/gh:/home/dev/.config/gh:ro" \
        -e ANTHROPIC_API_KEY \
        -e GITHUB_TOKEN \
        -e TAVILY_API_KEY \
        -e OPENROUTER_API_KEY \
        fractured-json-dev
