FROM mcr.microsoft.com/devcontainers/rust:0-1-bullseye

# Update default toolchain to stable.
RUN rustup default stable \
    # Install clippy for rust-analyzer and rustfmt.
    && rustup component add clippy rustfmt