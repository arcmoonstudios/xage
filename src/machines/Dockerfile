# Base image with Rust and Cargo
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/machines

# Copy the Cargo.toml and src directory into the Docker container
COPY Cargo.toml .
COPY src/machines src/machines

# Build the Rust project
RUN cargo build --release

# Use a minimal base image for running the built application
FROM debian:buster-slim
COPY --from=builder /usr/src/machines/target/release/machines /usr/local/bin/machines

# Run the application
CMD ["machines"]
