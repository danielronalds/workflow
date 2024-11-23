@default:
    just --list

# Runs the project with cargo run
dev:
    cargo run

# Formats the project
fmt:
    cargo fmt

# Runs clippy
check:
    cargo clippy